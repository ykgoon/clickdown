//! Tests for the vim-style modal input model (issue 7), @mentions (issue 6),
//! and description insert mode.

mod fixtures;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::tui::app::{Screen, TuiApp};
use clickdown::tui::input::InputEvent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}
fn ctrl(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL))
}
fn esc() -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
}
fn enter() -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
}

fn setup_app(rt: &Runtime) -> TuiApp {
    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });
    app.set_screen(Screen::TaskDetail);
    let task = fixtures::test_task();
    app.task_detail().task = Some(task);
    app
}

/// In insert mode, normal-mode command keys must be captured as text (issue 7).
#[test]
fn test_insert_mode_captures_command_chars_as_text() {
    let rt = Runtime::new().unwrap();
    let mut app = setup_app(&rt);

    // Enter insert mode on the description ('e' in normal mode)
    app.update(key('e'));
    assert!(app.insert_mode_active_for_test(), "insert mode should be on");

    // Keys that would be commands in normal mode must now be text
    for c in ['d', 's', 'g', 'u', 'n', 'a', 'j', 'k'] {
        app.update(key(c));
    }
    let buf = app.description_buffer_for_test();
    assert_eq!(buf, "dsgunajk", "all command chars must be captured as text");

    // Esc exits insert mode without saving; buffer is discarded
    app.update(esc());
    assert!(!app.insert_mode_active_for_test(), "Esc should leave insert mode");
    assert_eq!(app.description_buffer_for_test(), "");
}

/// SPACE inside insert mode is a literal space, not the command menu.
#[test]
fn test_insert_mode_space_is_literal() {
    let rt = Runtime::new().unwrap();
    let mut app = setup_app(&rt);

    app.update(key('e'));
    app.update(key('h'));
    app.update(InputEvent::Key(KeyEvent::new(
        KeyCode::Char(' '),
        KeyModifiers::NONE,
    )));
    app.update(key('i'));
    assert_eq!(app.description_buffer_for_test(), "h i");
    assert!(
        !app.command_menu_open_for_test(),
        "SPACE in insert mode must not open the command menu"
    );
}

/// SPACE in normal mode opens the command menu, then sub-keys run commands.
#[test]
fn test_space_menu_g_u_opens_url_dialog() {
    let rt = Runtime::new().unwrap();
    let mut app = setup_app(&rt);

    app.update(InputEvent::Key(KeyEvent::new(
        KeyCode::Char(' '),
        KeyModifiers::NONE,
    )));
    assert!(app.command_menu_open_for_test(), "SPACE should open menu");

    app.update(key('g'));
    assert!(app.command_menu_open_for_test(), "group 'g' keeps menu open");

    app.update(key('u'));
    assert!(
        app.url_input_open_for_test(),
        "'g u' from SPACEmenu should open the URL dialog"
    );
    assert!(!app.command_menu_open_for_test(), "menu should close after command");
}

/// Comment editing goes through insert mode: typing 'd' must not delete the task.
#[test]
fn test_comment_insert_mode_captures_text() {
    let rt = Runtime::new().unwrap();
    let mut app = setup_app(&rt);
    app.set_comment_focus(true);

    // 'n' starts a new comment and enters insert mode
    app.update(key('n'));
    assert!(app.insert_mode_active_for_test());

    app.update(key('d'));
    app.update(key('s'));
    // 'd' and 's' must be text, not commands
    assert_eq!(app.comment_new_text(), "ds");

    // Ctrl+S saves (empty-task mock may error, but insert mode exits via create path)
    let _guard = rt.enter();
    app.update(ctrl('s'));
    assert!(!app.insert_mode_active_for_test(), "Ctrl+S should commit and exit");
}

/// Esc in comment insert mode cancels the comment (compat with old behavior).
#[test]
fn test_comment_insert_mode_esc_cancels() {
    let rt = Runtime::new().unwrap();
    let mut app = setup_app(&rt);
    app.set_comment_focus(true);

    app.update(key('n'));
    app.update(key('h'));
    app.update(key('i'));
    app.update(esc());
    assert!(!app.insert_mode_active_for_test());
    assert_eq!(app.comment_new_text(), "", "Esc should cancel comment");
}
