//! Regression tests for issues 4 (full statuses), 5 (expand panels), 6 (@mentions)

mod fixtures;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::task::TaskStatus;
use clickdown::models::List;
use clickdown::tui::app::{Screen, TuiApp};
use clickdown::tui::input::InputEvent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}

fn status(name: &str, order: u32) -> TaskStatus {
    TaskStatus {
        id: None,
        status: name.to_string(),
        color: None,
        type_field: None,
        orderindex: Some(order),
        status_group: None,
    }
}

// ==================== Issue 4: full status choices ====================

/// The status picker must offer the list's FULL status configuration,
/// not just the subset observed on loaded tasks.
#[test]
fn test_status_picker_shows_full_list_statuses() {
    let rt = Runtime::new().unwrap();

    // List defines 5 statuses; only tasks with 2 of them are loaded
    let mut list = fixtures::test_list();
    list.statuses = vec![
        status("to do", 0),
        status("Open", 1),
        status("in progress", 2),
        status("In Review", 3),
        status("closed", 4),
    ];

    let mut task = fixtures::test_task();
    task.status = Some(status("to do", 0));
    let all_statuses = list.statuses.clone();

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(
            MockClickUpClient::new()
                .with_tasks(vec![task.clone()])
                .with_list(list),
        ))
        .unwrap()
    });
    app.set_screen(Screen::Tasks);
    app.set_current_list_id(Some("test-list-1".to_string()));
    app.set_cached_list_statuses_for_test("test-list-1", all_statuses);
    app.tasks_mut_for_test().push(task);
    app.rebuild_task_list_for_test();
    app.task_list_mut_for_test().select_first();

    // Open status picker with 's'
    let _guard = rt.enter();
    for c in [' ', 't', 's'] { app.update(key(c)); } // SPC t s
    assert!(app.is_status_picker_open(), "picker should open");

    let options = app.status_picker_statuses_for_test();
    assert_eq!(
        options.len(),
        5,
        "picker must show ALL list statuses, got: {:?}",
        options.iter().map(|s| &s.status).collect::<Vec<_>>()
    );
}

/// When the full status config is not cached, the picker must fetch it in
/// the background (get_list) and merge when the response arrives.
#[test]
fn test_status_picker_fetches_full_statuses_when_not_cached() {
    let rt = Runtime::new().unwrap();

    let mut list = fixtures::test_list();
    list.statuses = vec![
        status("to do", 0),
        status("Open", 1),
        status("in progress", 2),
        status("In Review", 3),
        status("closed", 4),
    ];

    let mut task = fixtures::test_task();
    task.status = Some(status("to do", 0));

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(
            MockClickUpClient::new()
                .with_tasks(vec![task.clone()])
                .with_list(list),
        ))
        .unwrap()
    });
    app.set_screen(Screen::Tasks);
    app.set_current_list_id(Some("test-list-1".to_string()));
    app.tasks_mut_for_test().push(task);
    app.rebuild_task_list_for_test();
    app.task_list_mut_for_test().select_first();

    let _guard = rt.enter();
    for c in [' ', 't', 's'] { app.update(key(c)); } // SPC t s
    assert!(app.is_status_picker_open());

    // Picker opens immediately with observed statuses; fetch is in-flight
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(100)).await });
    app.process_async_messages();

    let options = app.status_picker_statuses_for_test();
    assert_eq!(
        options.len(),
        5,
        "after ListStatusesLoaded, picker must show all 5 statuses, got: {:?}",
        options.iter().map(|s| &s.status).collect::<Vec<_>>()
    );
}

// ==================== Issue 5: expand description/comments ====================

#[test]
fn test_v_expands_description_full_screen() {
    let rt = Runtime::new().unwrap();
    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });
    app.set_screen(Screen::TaskDetail);
    app.task_detail().task = Some(fixtures::test_task());

    for c in [' ', 't', 'v'] { app.update(key(c)); } // SPC t v
    assert_eq!(
        app.expanded_panel_for_test(),
        Some(clickdown::tui::app::ExpandedPanel::Description),
        "'v' should expand the description panel"
    );

    // Esc collapses first
    app.update(InputEvent::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
    assert_eq!(
        app.expanded_panel_for_test(),
        None,
        "Esc should restore the split layout"
    );
    // Esc again leaves the task detail screen
    app.update(InputEvent::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
    assert_eq!(app.screen(), Screen::Tasks);
}

#[test]
fn test_v_expands_comments_in_comment_focus() {
    let rt = Runtime::new().unwrap();
    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });
    app.set_screen(Screen::TaskDetail);
    for c in [' ', 'c', 'v'] { app.update(key(c)); } // SPC c v
    app.set_comment_focus(true);

    app.update(key('v'));
    assert_eq!(
        app.expanded_panel_for_test(),
        Some(clickdown::tui::app::ExpandedPanel::Comments),
        "'v' in comment focus should expand the comments panel"
    );
}

// ==================== Issue 6: @mentions ====================

#[test]
fn test_at_opens_mention_picker_in_insert_mode() {
    let rt = Runtime::new().unwrap();
    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });
    app.set_screen(Screen::TaskDetail);
    app.task_detail().task = Some(fixtures::test_task());
    app.set_comment_focus(true);
    app.set_current_list_id(Some("test-list-1".to_string()));

    // New comment enters insert mode
    let _guard = rt.enter();
    app.update(key('n'));
    assert!(app.insert_mode_active_for_test());

    // Typing @ opens the mention picker
    app.update(key('@'));
    assert!(app.mention_picker_open_for_test(), "@ should open picker");
    // The @ is still in the buffer as text
    assert_eq!(app.comment_new_text(), "@");
}

#[test]
fn test_mention_picker_inserts_username() {
    let rt = Runtime::new().unwrap();
    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });
    app.set_screen(Screen::TaskDetail);
    app.task_detail().task = Some(fixtures::test_task());
    app.set_comment_focus(true);

    let _guard = rt.enter();
    app.update(key('n'));
    app.update(key('@'));
    assert!(app.mention_picker_open_for_test());

    // Set members directly (as if MentionMembersLoaded arrived)
    app.set_mention_members_for_test(vec![fixtures::test_user()]);

    // Enter inserts the selected member
    app.update(InputEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)));
    assert!(!app.mention_picker_open_for_test(), "picker closes after insert");
    assert!(
        app.comment_new_text().contains("@testuser"),
        "buffer should contain @testuser, got: {}",
        app.comment_new_text()
    );
    // Back in insert mode for more typing
    assert!(app.insert_mode_active_for_test());
}
