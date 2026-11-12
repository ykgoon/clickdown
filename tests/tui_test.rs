//! Integration tests for ClickDown TUI application
//!
//! These tests verify the TUI application functionality.

mod fixtures;

use clickdown::tui::app::TuiApp;

/// Test that the TUI app initializes correctly
#[test]
fn test_tui_app_initialization() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let app = rt.block_on(async { TuiApp::new() });

    // App should initialize successfully
    assert!(app.is_ok(), "TUI App failed to initialize: {:?}", app.err());
}

/// Test that screen titles are generated correctly
#[test]
fn test_screen_title_generation() {
    use clickdown::tui::layout::generate_screen_title;

    let title = generate_screen_title("Authentication");
    assert_eq!(title, "ClickDown - Authentication");

    let title = generate_screen_title("Workspaces");
    assert_eq!(title, "ClickDown - Workspaces");

    let title = generate_screen_title("Tasks: My List");
    assert_eq!(title, "ClickDown - Tasks: My List");
}

/// Test that screen titles are unique for different screens
#[test]
fn test_screen_titles_are_unique() {
    use clickdown::tui::layout::generate_screen_title;

    let auth_title = generate_screen_title("Authentication");
    let workspace_title = generate_screen_title("Workspaces");
    let tasks_title = generate_screen_title("Tasks");

    assert_ne!(
        auth_title, workspace_title,
        "Auth and Workspace titles should be different"
    );
    assert_ne!(
        auth_title, tasks_title,
        "Auth and Tasks titles should be different"
    );
    assert_ne!(
        workspace_title, tasks_title,
        "Workspace and Tasks titles should be different"
    );
}

/// Test that minimum terminal size constants are defined
#[test]
fn test_minimum_terminal_size() {
    use clickdown::tui::layout::{MIN_HEIGHT, MIN_WIDTH};

    assert_eq!(MIN_WIDTH, 80, "Minimum width should be 80");
    assert_eq!(MIN_HEIGHT, 24, "Minimum height should be 24");
}

/// Test that layout is created correctly
#[test]
fn test_layout_creation() {
    use clickdown::tui::layout::TuiLayout;
    use ratatui::prelude::Rect;

    let area = Rect::new(0, 0, 100, 30);
    let layout = TuiLayout::new(area);

    assert!(
        !layout.too_small,
        "Layout should not be too small for 100x30 terminal"
    );
    assert!(layout.title_area.height == 1, "Title area should be 1 row");
    assert!(
        layout.status_area.height == 1,
        "Status area should be 1 row (hint bar removed)"
    );
}

/// Test that layout detects small terminals
#[test]
fn test_layout_small_terminal() {
    use clickdown::tui::layout::TuiLayout;
    use ratatui::prelude::Rect;

    let area = Rect::new(0, 0, 60, 20);
    let layout = TuiLayout::new(area);

    assert!(
        layout.too_small,
        "Layout should be too small for 60x20 terminal"
    );
}

/// Test input event types
#[test]
fn test_input_event_types() {
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent};

    let key_event = KeyEvent::new(KeyCode::Char('a'), crossterm::event::KeyModifiers::NONE);
    let input_event = InputEvent::Key(key_event);

    match input_event {
        InputEvent::Key(_) => (),
        _ => panic!("Expected Key event"),
    }
}

/// Test that plain q and plain space do NOT quit (SPC q sequence only, via dialog)
#[test]
fn test_quit_detection() {
    use clickdown::tui::app::TuiApp;
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut app = TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap();

        // Plain q in normal mode must not open the quit dialog
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
        )));
        assert!(!app.is_dialog_visible(), "plain q should not quit");

        // Plain space alone must not open the quit dialog
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char(' '),
            KeyModifiers::NONE,
        )));
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));
        assert!(!app.is_dialog_visible(), "plain space should not quit");
    });
}

/// Test dialog state
#[test]
fn test_dialog_state() {
    use clickdown::tui::widgets::{DialogState, DialogType};

    let mut dialog = DialogState::new();

    // Dialog should start hidden
    assert!(!dialog.is_visible(), "Dialog should start hidden");

    // Show dialog
    dialog.show(DialogType::ConfirmDelete);
    assert!(dialog.is_visible(), "Dialog should be visible after show");

    // Toggle selection
    assert!(!dialog.confirmed(), "Dialog should start with No selected");
    dialog.toggle();
    assert!(
        dialog.confirmed(),
        "Dialog should have Yes selected after toggle"
    );

    // Hide dialog
    dialog.hide();
    assert!(!dialog.is_visible(), "Dialog should be hidden after hide");
}

/// Test that confirming the delete dialog actually deletes the selected task
#[test]
fn test_delete_task_on_confirm() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::input::InputEvent;
    use clickdown::tui::widgets::DialogType;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let task = fixtures::test_task();

        let mock_client = MockClickUpClient::new()
            .with_tasks(vec![task.clone()])
            .with_delete_task_success();

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Manually populate the task list (simulating tasks loaded from API)
        // Both TuiApp.tasks and task_list's internal list need to be populated
        app.tasks_mut_for_test().push(task.clone());
        app.rebuild_task_list_for_test();
        app.task_list_mut_for_test().select_first();

        // Verify initial state: task is present
        assert_eq!(app.task_count(), 1, "Should have 1 task");
        assert!(
            app.task_list_for_test().selected_task().is_some(),
            "A task should be selected"
        );

        // Show the delete dialog and confirm
        app.dialog_mut_for_test().show(DialogType::ConfirmDelete);
        app.dialog_mut_for_test().toggle(); // Switch from "No" to "Yes"
        assert!(app.is_dialog_confirmed(), "Dialog should be confirmed");

        // Simulate pressing Enter — this should trigger delete_selected_task()
        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        app.update(InputEvent::Key(enter_key));

        // Dialog should be hidden
        assert!(
            !app.is_dialog_visible(),
            "Dialog should be hidden after confirm"
        );

        // Give the async delete task time to complete
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Process the async message (simulates what run_loop does)
        app.process_async_messages();

        // Task should be removed from the local list
        assert_eq!(
            app.task_count(),
            0,
            "Task should be removed after successful deletion"
        );
        assert!(
            app.task_list_for_test().selected_task().is_none(),
            "No task should be selected after deletion"
        );
    });
}

/// Test that canceling the delete dialog does NOT delete the task
#[test]
fn test_delete_task_on_cancel() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::input::InputEvent;
    use clickdown::tui::widgets::DialogType;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let task = fixtures::test_task();

        let mock_client = MockClickUpClient::new()
            .with_tasks(vec![task.clone()])
            .with_delete_task_success();

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Manually populate the task list
        app.tasks_mut_for_test().push(task.clone());
        app.rebuild_task_list_for_test();
        app.task_list_mut_for_test().select_first();

        // Verify initial state
        assert_eq!(app.task_count(), 1, "Should have 1 task");

        // Show the delete dialog and cancel with Esc
        app.dialog_mut_for_test().show(DialogType::ConfirmDelete);
        let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        app.update(InputEvent::Key(esc_key));

        // Dialog should be hidden, task should remain
        assert!(
            !app.is_dialog_visible(),
            "Dialog should be hidden after cancel"
        );
        assert_eq!(
            app.task_count(),
            1,
            "Task should NOT be removed after cancel"
        );
    });
}

/// Test that pressing 'd' with no task selected does NOT show the dialog
#[test]
fn test_delete_no_task_selected() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::Screen;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let mock_client = MockClickUpClient::new();

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Navigate to task list screen (Tasks screen is needed for 'd' to work)
        app.set_screen_for_test(Screen::Tasks);

        // Press 'd' with no task selected
        let d_key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        app.update(InputEvent::Key(d_key));

        // Dialog should NOT be shown
        assert!(
            !app.is_dialog_visible(),
            "Dialog should NOT appear when no task is selected"
        );
    });
}

/// Test that an empty API response body on delete causes a parse error
/// and the task remains in the UI — reproduces "failed to parse response" bug
#[test]
fn test_delete_task_empty_response_fails_to_parse() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::input::InputEvent;
    use clickdown::tui::widgets::DialogType;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let task = fixtures::test_task();

        // Simulate ClickUp returning an empty body for DELETE
        let mock_client = MockClickUpClient::new()
            .with_tasks(vec![task.clone()])
            .with_delete_task_json("");

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Manually populate the task list
        app.tasks_mut_for_test().push(task.clone());
        app.rebuild_task_list_for_test();
        app.task_list_mut_for_test().select_first();

        // Verify initial state: task is present
        assert_eq!(app.task_count(), 1, "Should have 1 task");

        // Show the delete dialog and confirm
        app.dialog_mut_for_test().show(DialogType::ConfirmDelete);
        app.dialog_mut_for_test().toggle();
        assert!(app.is_dialog_confirmed(), "Dialog should be confirmed");

        // Simulate pressing Enter
        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        app.update(InputEvent::Key(enter_key));

        // Give the async delete task time to complete
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Process the async message
        app.process_async_messages();

        // BUG REPRODUCTION: Task should still be in the list because empty response
        // causes "failed to parse response" error, treated as deletion failure
        assert_eq!(
            app.task_count(),
            1,
            "Task should remain in list because empty response causes parse error"
        );
    });
}

/// SPACEmenu minibuffer: Esc at a group level walks back to the root menu
/// (Doom-style hierarchy), instead of closing the whole menu.
#[test]
fn test_spc_menu_esc_at_group_returns_to_root() {
    use clickdown::tui::app::{CommandMenuLevel, TuiApp};
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut app = TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap();

        // SPC opens root, 't' enters the Task group
        for k in [' ', 't'] {
            app.update(InputEvent::Key(KeyEvent::new(
                KeyCode::Char(k),
                KeyModifiers::NONE,
            )));
        }
        assert!(app.command_menu_open_for_test(), "menu open at group level");
        assert_eq!(
            app.command_menu_level_for_test(),
            Some(CommandMenuLevel::Group('t')),
            "'t' should enter the Task group"
        );

        // Esc returns to the root level, keeping the menu open
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));
        assert!(app.command_menu_open_for_test(), "menu stays open at root");
        assert_eq!(
            app.command_menu_level_for_test(),
            Some(CommandMenuLevel::Root),
            "Esc at group level should return to root"
        );

        // Esc at root closes the menu entirely
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));
        assert!(!app.command_menu_open_for_test(), "Esc at root closes the menu");
    });
}

/// SPACEmenu minibuffer: every root group key has a single-key display,
/// and unknown group keys close the menu with feedback (one key per level).
#[test]
fn test_spc_menu_root_keys_enter_groups() {
    use clickdown::tui::app::{CommandMenuLevel, TuiApp};
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut app = TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap();

        for (open_key, expected) in [
            ('g', CommandMenuLevel::Group('g')),
            ('t', CommandMenuLevel::Group('t')),
            ('c', CommandMenuLevel::Group('c')),
            ('y', CommandMenuLevel::Group('y')),
            ('w', CommandMenuLevel::Group('w')),
        ] {
            app.update(InputEvent::Key(KeyEvent::new(
                KeyCode::Char(' '),
                KeyModifiers::NONE,
            )));
            app.update(InputEvent::Key(KeyEvent::new(
                KeyCode::Char(open_key),
                KeyModifiers::NONE,
            )));
            assert_eq!(
                app.command_menu_level_for_test(),
                Some(expected),
                "SPC {} should enter group level with a single key",
                open_key
            );
            // close via Esc-at-root for the next iteration
            app.update(InputEvent::Key(KeyEvent::new(
                KeyCode::Esc,
                KeyModifiers::NONE,
            )));
            app.update(InputEvent::Key(KeyEvent::new(
                KeyCode::Esc,
                KeyModifiers::NONE,
            )));
            assert!(!app.command_menu_open_for_test());
        }
    });
}
