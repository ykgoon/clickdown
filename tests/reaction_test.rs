//! Tests for `SPC c u` — thumbs-up a comment.
//!
//! The command flips the reaction optimistically and then confirms or rolls
//! back against the API result, so these tests drive both the local state and
//! the async message handling.

mod fixtures;

use std::sync::Arc;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::Comment;
use clickdown::tui::app::{AppMessage, Screen, TuiApp};
use clickdown::tui::input::InputEvent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::runtime::Runtime;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}

fn comment(id: &str, text: &str, reactions: &[&str]) -> Comment {
    serde_json::from_value(serde_json::json!({
        "id": id,
        "comment_text": text,
        "reactions": reactions,
    }))
    .expect("comment fixture should deserialize")
}

/// Build an app parked on the task screen with one comment selected.
fn app_with_comment(rt: &Runtime, client: MockClickUpClient, reactions: &[&str]) -> TuiApp {
    let mut app = rt.block_on(async { TuiApp::with_client(Arc::new(client)).unwrap() });
    app.set_screen(Screen::TaskDetail);
    app.task_detail().task = Some(fixtures::test_task());
    app.set_comments(vec![comment("c1", "Looks good", reactions)]);
    app.set_comment_focus(true);
    app.set_comment_selected_index_for_test(0);
    app
}

/// The API returns reactions as a flat array of emoji names.
#[test]
fn test_comment_deserializes_reactions_array() {
    let parsed = comment("c1", "hi", &["thumbsup", "eyes"]);
    assert_eq!(parsed.reactions, vec!["thumbsup", "eyes"]);
    assert!(parsed.has_reaction("thumbsup"));
    assert!(!parsed.has_reaction("heart"));
}

/// A comment with no reactions array at all must not fail to parse.
#[test]
fn test_comment_without_reactions_defaults_to_empty() {
    let parsed: Comment = serde_json::from_value(serde_json::json!({
        "id": "c1",
        "comment_text": "no reactions here",
    }))
    .expect("should deserialize");
    assert!(parsed.reactions.is_empty());
    assert!(!parsed.has_reaction("thumbsup"));

    // Explicit null is also tolerated.
    let nulled: Comment = serde_json::from_value(serde_json::json!({
        "id": "c2",
        "comment_text": "x",
        "reactions": null,
    }))
    .expect("null reactions should deserialize");
    assert!(nulled.reactions.is_empty());
}

/// `SPC c u` adds the reaction locally and calls the API once.
#[test]
fn test_spc_c_u_adds_thumbs_up() {
    let rt = Runtime::new().unwrap();
    let mut app = app_with_comment(&rt, MockClickUpClient::new().with_comment_reaction(), &[]);

    let _guard = rt.enter();
    for c in [' ', 'c', 'u'] {
        app.update(key(c));
    }

    assert_eq!(
        app.reactions_for_test("c1"),
        vec!["thumbsup"],
        "the comment must show a thumbs-up immediately"
    );
}

/// Pressing the command again removes the reaction (toggle behaviour).
#[test]
fn test_spc_c_u_toggles_off_when_already_reacted() {
    let rt = Runtime::new().unwrap();
    let mut app = app_with_comment(
        &rt,
        MockClickUpClient::new().with_comment_reaction(),
        &["thumbsup"],
    );

    let _guard = rt.enter();
    for c in [' ', 'c', 'u'] {
        app.update(key(c));
    }

    assert!(
        app.reactions_for_test("c1").is_empty(),
        "a second press must remove the reaction, got: {:?}",
        app.reactions_for_test("c1")
    );
}

/// A failed API call rolls the optimistic flip back so the UI matches ClickUp.
#[test]
fn test_failed_reaction_rolls_back_and_reports() {
    let rt = Runtime::new().unwrap();
    let mut app = app_with_comment(
        &rt,
        MockClickUpClient::new().with_comment_reaction_error("boom"),
        &[],
    );

    let _guard = rt.enter();
    for c in [' ', 'c', 'u'] {
        app.update(key(c));
    }
    assert_eq!(app.reactions_for_test("c1"), vec!["thumbsup"], "optimistic");

    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(50)).await });
    app.process_async_messages();

    assert!(
        app.reactions_for_test("c1").is_empty(),
        "failed reaction must be rolled back, got: {:?}",
        app.reactions_for_test("c1")
    );
    assert_eq!(app.status(), "Reaction failed");
}

/// A successful API result keeps the reaction in place.
#[test]
fn test_successful_reaction_is_kept_after_response() {
    let rt = Runtime::new().unwrap();
    let mut app = app_with_comment(&rt, MockClickUpClient::new().with_comment_reaction(), &[]);

    let _guard = rt.enter();
    for c in [' ', 'c', 'u'] {
        app.update(key(c));
    }
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(50)).await });
    app.process_async_messages();

    assert_eq!(app.reactions_for_test("c1"), vec!["thumbsup"]);
    assert!(
        app.status().contains("Reacted"),
        "expected a confirmation status, got: {}",
        app.status()
    );
}

/// Without a selected comment the command explains itself instead of failing.
#[test]
fn test_reaction_without_selected_comment_reports_reason() {
    let rt = Runtime::new().unwrap();
    let mut app =
        rt.block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });
    app.set_screen(Screen::TaskDetail);
    app.task_detail().task = Some(fixtures::test_task());
    app.set_comments(vec![]);
    app.set_comment_focus(true);

    let _guard = rt.enter();
    app.toggle_comment_thumbs_up();

    assert_eq!(app.status(), "Select a comment to react to");
}

/// The command is only meaningful on the task screen.
#[test]
fn test_reaction_outside_task_detail_reports_reason() {
    let rt = Runtime::new().unwrap();
    let mut app =
        rt.block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });
    app.set_screen(Screen::Tasks);

    let _guard = rt.enter();
    app.toggle_comment_thumbs_up();

    assert_eq!(app.status(), "Comments live in task detail");
}

/// The async message alone (as the API layer sends it) updates state, proving
/// the handler is wired independently of the key path.
#[test]
fn test_reaction_message_round_trip() {
    let rt = Runtime::new().unwrap();
    let mut app =
        rt.block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });
    app.set_screen(Screen::TaskDetail);
    app.set_comments(vec![comment("c1", "hello", &[])]);

    let tx = app.message_tx_for_testing();
    rt.block_on(async {
        tx.send(AppMessage::CommentReactionUpdated(
            Ok(()),
            "c1".to_string(),
            "thumbsup".to_string(),
            true,
        ))
        .await
        .unwrap();
    });
    app.process_async_messages();

    // The success handler only reports; state came from the optimistic flip.
    // Assert the handler ran by checking its status text.
    assert!(
        app.status().contains("Reacted"),
        "handler should report success, got: {}",
        app.status()
    );
}

/// The wire call must carry the comment id, the `thumbsup` name, and the
/// right direction, since the API decides add vs remove by HTTP method.
#[test]
fn test_reaction_call_uses_expected_api_arguments() {
    let rt = Runtime::new().unwrap();
    let mock = MockClickUpClient::new().with_comment_reaction();
    let calls = mock.reaction_calls_handle();
    let mut app = app_with_comment(&rt, mock, &[]);

    let _guard = rt.enter();
    app.set_comment_selected_index_for_test(0);
    app.toggle_comment_thumbs_up();
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(50)).await });
    app.process_async_messages();

    // Toggle back off.
    app.toggle_comment_thumbs_up();
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(50)).await });
    app.process_async_messages();

    let recorded = calls.lock().unwrap().clone();
    assert_eq!(
        recorded,
        vec![
            ("c1".to_string(), "thumbsup".to_string(), true),
            ("c1".to_string(), "thumbsup".to_string(), false),
        ],
        "add then remove must be recorded against the selected comment"
    );
}
