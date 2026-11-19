//! Regression tests for the three reported comment issues:
//! 1. A new-task form must not show the previously viewed task's comments.
//! 2. The empty-comments placeholder names the real key (SPACEmenu c n).
//! 3. The last reply in a thread stays visible when the parent comment is
//!    very long: auto-scroll shows the selected comment's whole block.

mod fixtures;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::tui::app::{Screen, TuiApp};
use clickdown::tui::input::InputEvent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}
fn with(code: KeyCode, mods: KeyModifiers) -> InputEvent {
    InputEvent::Key(KeyEvent::new(code, mods))
}
fn type_str(app: &mut TuiApp, text: &str) {
    for c in text.chars() {
        app.update(key(c));
    }
}

/// App on the task detail screen with a focused comment pane.
fn comment_app(rt: &Runtime, client: MockClickUpClient) -> TuiApp {
    let mut app = rt.block_on(async { TuiApp::with_client(Arc::new(client)).unwrap() });
    app.set_screen(Screen::TaskDetail);
    app.set_task_detail_task(fixtures::test_task());
    app.set_comment_focus(true);
    app
}

fn thread_comments() -> Vec<clickdown::models::Comment> {
    vec![
        {
            let mut c = fixtures::test_comment();
            c.id = "c1".into();
            c.parent_id = None;
            c.text = "P ".repeat(300); // long parent: many wrapped rows
            c
        },
        {
            let mut c = fixtures::test_comment_edited();
            c.id = "r1".into();
            c.parent_id = Some("c1".into());
            c.text = "first reply".into();
            c
        },
        {
            let mut c = fixtures::test_comment();
            c.id = "r2".into();
            c.parent_id = Some("c1".into());
            c.text = "second reply".into();
            c
        },
    ]
}

fn rendered_rows(app: &mut TuiApp, width: u16, height: u16) -> Vec<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| app.draw_ui(frame)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .chunks(width as usize)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect()
}

// ------------------------------------------------------------------ issue 1

/// Starting a new task must not carry over the previously viewed task's
/// comments: the pane under the creation form renders `self.comments`.
#[test]
fn new_task_form_does_not_show_previous_task_comments() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new();
    let mut app = comment_app(&rt, client);
    app.set_comments(vec![fixtures::test_comment()]);
    app.set_comment_selected_index_for_test(0);

    app.set_screen(Screen::Tasks);
    app.update(key(' '));
    app.update(key('t'));
    app.update(key('n'));
    assert!(app.is_task_creating(), "creation form should open");

    let body: String = rendered_rows(&mut app, 80, 24).join("\n");
    assert!(
        !body.contains("This is a test comment"),
        "old task's comments leaked into the new-task screen:\n{}",
        body
    );
}

// ------------------------------------------------------------------ issue 2

/// The empty-comments placeholder must not tell the user to press 'n':
/// in that view (comments not focused) 'n' does not start a comment.
#[test]
fn empty_comments_placeholder_mentions_the_real_binding() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new();
    let mut app = comment_app(&rt, client);
    app.set_comments(vec![]);
    app.set_comment_focus(false);

    let body: String = rendered_rows(&mut app, 80, 24).join("\n");
    assert!(
        !body.contains("Press 'n' to add one"),
        "placeholder still claims 'n' adds a comment:\n{}",
        body
    );
    assert!(
        body.contains("c n"),
        "placeholder should name the SPACEmenu c n chord:\n{}",
        body
    );
}

// ------------------------------------------------------------------ issue 3

/// Long parent + two replies, default 24-row terminal: selecting the last
/// reply (j j after Enter) must render its text, not leave it below the fold.
#[test]
fn last_reply_visible_when_parent_is_very_long() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new();
    let mut app = comment_app(&rt, client);
    app.set_comments(thread_comments());
    app.set_comment_selected_index_for_test(0);

    app.update(InputEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)));
    app.update(key('j'));
    app.update(key('j'));

    let body: String = rendered_rows(&mut app, 80, 24).join("\n");
    assert!(
        body.contains("second reply"),
        "last reply invisible while selected:\n{}",
        body
    );
}

/// The selected comment's own tail must stay visible: a multi-line selected
/// reply may not be cut after its first row.
#[test]
fn selected_multiline_comment_shows_its_tail() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new();
    let mut app = comment_app(&rt, client);
    app.set_comments(vec![
        {
            let mut c = fixtures::test_comment();
            c.id = "c1".into();
            c.parent_id = None;
            c.text = "P ".repeat(100);
            c
        },
        {
            let mut c = fixtures::test_comment_edited();
            c.id = "r1".into();
            c.parent_id = Some("c1".into());
            c.text = "first\nsecond\nthird\nfourth\nfifth".into();
            c
        },
    ]);
    app.set_comment_selected_index_for_test(0);

    app.update(InputEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)));
    app.update(key('j'));

    let body: String = rendered_rows(&mut app, 80, 24).join("\n");
    assert!(
        body.contains("fifth"),
        "tail of the selected reply is cut off:\n{}",
        body
    );
}

/// A selected comment taller than the pane shows its tail when scrolled to
/// (auto-scroll pins the tail; manual Down must be able to reach it).
#[test]
fn tall_selected_comment_can_reach_its_tail() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new();
    let mut app = comment_app(&rt, client);
    app.set_comments(vec![
        {
            let mut c = fixtures::test_comment();
            c.id = "c1".into();
            c.parent_id = None;
            c.text = "P ".repeat(100);
            c
        },
        {
            let mut c = fixtures::test_comment_edited();
            c.id = "r1".into();
            c.parent_id = Some("c1".into());
            c.text = "R\n".repeat(30);
            c
        },
    ]);
    app.set_comment_selected_index_for_test(0);

    app.update(InputEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)));
    app.update(key('j'));
    // Walk down with Down; the last press must leave the reply's tail on
    // screen (the pane clamps at max_offset = content end).
    for _ in 0..12 {
        app.update(with(KeyCode::Down, KeyModifiers::NONE));
    }
    let rows = rendered_rows(&mut app, 80, 24);
    let last_content_row = rows
        .iter()
        .rev()
        .find(|r| r.trim().trim_end_matches('│').trim() != "" && r.contains("R") || r.contains("│ R") || r.contains("│R"));
    assert!(
        last_content_row.is_some(),
        "reply tail unreachable by manual scroll:\n{}",
        rows.join("\n")
    );
}
