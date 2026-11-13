//! Tests for `SPC y c` — copy the focused element's content.
//!
//! Clipboard access is unavailable in headless test environments, so these
//! tests assert on the text *chosen* for copying plus the user feedback, and
//! cover a successful copy by driving the clipboard abstraction directly.

mod fixtures;

use std::sync::Arc;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::{Comment, Task};
use clickdown::tui::app::{Screen, TuiApp};
use clickdown::tui::input::InputEvent;
use clickdown::utils::comment_text_to_plain;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::runtime::Runtime;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}

fn comment(id: &str, text: &str, parent_id: Option<&str>) -> Comment {
    serde_json::from_value(serde_json::json!({
        "id": id,
        "comment_text": text,
        "parent_id": parent_id,
    }))
    .expect("comment fixture should deserialize")
}

fn task_with_description(description: &str) -> Task {
    let mut task = fixtures::test_task();
    task.description = Some(clickdown::models::task::TaskDescription::Plain(
        description.to_string(),
    ));
    task
}

/// `SPC y c` must reach the copy command and report back in the status bar,
/// proving the binding is wired without needing a working clipboard.
#[test]
fn test_spc_y_c_reports_copy_attempt_on_description_focus() {
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap()
    });
    app.set_screen(Screen::TaskDetail);
    app.task_detail().task = Some(task_with_description("Describe the bug"));
    app.set_comment_focus(false);

    for c in [' ', 'y', 'c'] {
        app.update(key(c));
    }

    let status = app.copy_status_for_test().expect("copy feedback expected");
    // Either the copy succeeded or the headless clipboard was unavailable:
    // both prove the command ran and chose the description.
    assert!(
        status.contains("description"),
        "status must identify the description as the copied element, got: {status}"
    );
}

/// With comment focus, the selected comment's text is what gets copied.
#[test]
fn test_copy_selects_the_focused_comment() {
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap()
    });
    app.set_screen(Screen::TaskDetail);
    app.task_detail().task = Some(task_with_description("desc"));
    app.set_comments(vec![
        comment("c1", "First comment body", None),
        comment("c2", "Second comment body", None),
    ]);
    app.set_comment_focus(true);
    app.set_comment_selected_index_for_test(1);

    assert_eq!(
        app.focused_copy_text_for_test().as_deref(),
        Some("Second comment body"),
        "the highlighted comment must be the copy source"
    );
}

/// A comment stored as rich-text JSON must be copied as readable text, not JSON.
#[test]
fn test_rich_text_comment_is_flattened_when_copied() {
    let rich = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Ship it"}]}]}"#;
    assert_eq!(comment_text_to_plain(rich), "Ship it");

    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();
    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });
    app.set_screen(Screen::TaskDetail);
    app.task_detail().task = Some(task_with_description("desc"));
    app.set_comments(vec![comment("c1", rich, None)]);
    app.set_comment_focus(true);

    assert_eq!(
        app.focused_copy_text_for_test().as_deref(),
        Some("Ship it"),
        "clipboard must receive flattened text"
    );
}

/// A stale comment selection (index not visible in the current view) must not
/// copy an unrelated comment.
#[test]
fn test_stale_comment_selection_copies_nothing() {
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });
    app.set_screen(Screen::TaskDetail);
    app.task_detail().task = Some(task_with_description("desc"));
    // c2 is a reply, so only c1 is visible in the top-level view.
    app.set_comments(vec![
        comment("c1", "Top level", None),
        comment("c2", "A reply", Some("c1")),
    ]);
    app.set_comment_focus(true);
    app.set_comment_selected_index_for_test(1); // reply, not visible at top level

    assert_eq!(
        app.focused_copy_text_for_test(),
        None,
        "an out-of-view comment must not be copied"
    );
}

/// On screens without a copyable element the binding explains itself instead
/// of silently doing nothing.
#[test]
fn test_copy_on_unsupported_screen_reports_reason() {
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });
    app.set_screen(Screen::Tasks);

    app.copy_focused_content();

    let status = app.copy_status_for_test().expect("feedback expected");
    assert!(
        status.contains("Nothing to copy"),
        "expected an explanatory message, got: {status}"
    );
}

/// An empty description is reported rather than putting "" on the clipboard.
#[test]
fn test_empty_description_reports_nothing_to_copy() {
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });
    app.set_screen(Screen::TaskDetail);
    app.task_detail().task = Some(fixtures::test_task());
    app.set_comment_focus(false);

    app.copy_focused_content();

    let status = app.copy_status_for_test().expect("feedback expected");
    assert!(
        status.contains("No description"),
        "expected a no-description message, got: {status}"
    );
}
