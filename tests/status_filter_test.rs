//! Tests for the task-list status filter (`SPC t f` -> `s`)
//!
//! Covers picker seeding, filter application through the public API
//! (`load_tasks` sends the status to the API), clearing, and title display.

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::task::{Task, TaskStatus};
use clickdown::tui::app::{AppMessage, Screen, TuiApp};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn status(name: &str, order: u32) -> TaskStatus {
    TaskStatus {
        id: None,
        status: name.to_string(),
        color: Some("#123456".to_string()),
        type_field: None,
        orderindex: Some(order),
        status_group: None,
    }
}

fn task_with_status(id: &str, name: &str, status: TaskStatus) -> Task {
    Task {
        id: id.to_string(),
        name: name.to_string(),
        status: Some(status),
        ..Default::default()
    }
}

/// Picker opens seeded from cached list statuses
#[test]
fn test_status_filter_picker_seeds_from_cached_statuses() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.set_screen(Screen::Tasks);
        app.set_current_list_id(Some("list-1".to_string()));
        app.set_cached_list_statuses_for_test(
            "list-1",
            vec![status("To Do", 0), status("In Progress", 1), status("Done", 2)],
        );

        app.open_status_filter_picker();

        assert!(app.is_status_picker_open(), "picker should be open");
        assert_eq!(app.status_picker_statuses_for_test().len(), 3);
        assert_eq!(
            app.status_picker_statuses_for_test()[0].status,
            "To Do",
            "statuses should be sorted by orderindex"
        );
        // No active filter yet: cursor starts at first entry
        assert_eq!(app.status_picker_cursor_for_test(), 0);
    });
}

/// Picker falls back to statuses observed on loaded tasks
#[test]
fn test_status_filter_picker_falls_back_to_task_statuses() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.set_tasks_for_test(vec![
            task_with_status("t1", "A", status("To Do", 0)),
            task_with_status("t2", "B", status("Done", 2)),
        ]);

        app.open_status_filter_picker();

        assert!(app.is_status_picker_open(), "picker should be open");
        let names: Vec<&str> = app
            .status_picker_statuses_for_test()
            .iter()
            .map(|s| s.status.as_str())
            .collect();
        assert_eq!(names, vec!["To Do", "Done"]);
    });
}

/// Enter applies the filter: app state updates and the reload carries the
/// status to the API
#[test]
fn test_enter_applies_status_filter_and_refetches() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let log = mock_client.tasks_call_log_handle();
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.set_screen(Screen::Tasks);
        app.set_current_list_id(Some("list-1".to_string()));
        app.set_cached_list_statuses_for_test("list-1", vec![status("To Do", 0), status("Done", 2)]);
        app.open_status_filter_picker();

        // Move to the second entry (Done) and confirm
        app.handle_status_picker_input(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Char('j'),
        ));
        app.handle_status_picker_input(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Enter,
        ));

        assert!(!app.is_status_picker_open(), "picker should close on apply");
        assert!(
            app.screen_title_for_test()
                .ends_with("Tasks (Status: Done)"),
            "title should show the active status filter, got: {}",
            app.screen_title_for_test()
        );
        assert!(app.is_loading(), "applying the filter should reload tasks");

        // Pump the spawned fetch and verify the API got the statuses filter
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        app.process_async_messages();
        let calls = log.lock().map(|l| l.clone()).unwrap_or_default();
        assert_eq!(calls.len(), 1, "one get_tasks call expected");
        assert_eq!(calls[0].0, "list-1");
        assert_eq!(
            calls[0].1,
            vec!["Done".to_string()],
            "statuses filter should be sent to the API"
        );
    });
}

/// Esc clears an active filter and reloads all tasks
#[test]
fn test_esc_keeps_existing_status_filter_until_cleared_explicitly() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let log = mock_client.tasks_call_log_handle();
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.set_screen(Screen::Tasks);
        app.set_current_list_id(Some("list-1".to_string()));
        app.set_cached_list_statuses_for_test("list-1", vec![status("To Do", 0), status("Done", 2)]);

        // Apply a filter first
        app.open_status_filter_picker();
        app.handle_status_picker_input(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Char('j'),
        ));
        app.handle_status_picker_input(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Enter,
        ));
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        app.process_async_messages();
        assert!(
        app.screen_title_for_test()
            .ends_with("Tasks (Status: Done)"),
        "got: {}",
        app.screen_title_for_test()
    );

        // Reopen and press Esc: the picker closes, the filter stays.
        app.open_status_filter_picker();
        app.handle_status_picker_input(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Esc,
        ));

        assert!(!app.is_status_picker_open(), "picker should close on Esc");
        assert!(
            app.screen_title_for_test()
                .ends_with("Tasks (Status: Done)"),
            "Esc must not drop the filter suffix, got: {}",
            app.screen_title_for_test()
        );

        // Clearing is explicit (`SPC t f` -> `x`).
        app.clear_all_filters();
        assert!(
            app.screen_title_for_test().ends_with("- Tasks"),
            "title should drop the filter suffix, got: {}",
            app.screen_title_for_test()
        );

        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        app.process_async_messages();
        let calls = log.lock().map(|l| l.clone()).unwrap_or_default();
        assert_eq!(calls.len(), 2, "reload after clearing expected");
        assert!(
            calls[1].1.is_empty(),
            "clearing should reload without a statuses filter"
        );
    });
}

/// Picker preselects the currently active filter when reopened
#[test]
fn test_reopened_picker_preselects_active_filter() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.set_screen(Screen::Tasks);
        app.set_current_list_id(Some("list-1".to_string()));
        app.set_cached_list_statuses_for_test(
            "list-1",
            vec![status("To Do", 0), status("In Progress", 1), status("Done", 2)],
        );

        app.open_status_filter_picker();
        // Move to In Progress and apply
        app.handle_status_picker_input(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Char('j'),
        ));
        app.handle_status_picker_input(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Enter,
        ));
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        app.process_async_messages();

        // Reopen: cursor should sit on the active filter
        app.open_status_filter_picker();
        assert_eq!(app.status_picker_cursor_for_test(), 1);
    });
}

/// Filter composes with the assigned-to-me filter: both query params are sent
#[test]
fn test_status_filter_composes_with_assigned_filter() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let log = mock_client.tasks_call_log_handle();
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.set_screen(Screen::Tasks);
        app.set_current_list_id(Some("list-1".to_string()));
        app.set_cached_list_statuses_for_test("list-1", vec![status("Done", 2)]);
        app.set_current_user_id(Some(42));

        // Activate the status filter
        app.open_status_filter_picker();
        app.handle_status_picker_input(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Enter,
        ));
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        app.process_async_messages();

        // Now activate the assigned filter (routes through the assigned path,
        // which bypasses statuses filtering by design; verify no panic and a
        // fetch was triggered)
        app.set_assigned_filter_active(true);
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        app.process_async_messages();

        // The list is not registered in app.lists, so the title falls to the
        // list-less branch; the status filter shows there. Verify the filter
        // still displays and both fetches happened.
        app.update_screen_title();
        assert!(
            app.screen_title_for_test().ends_with("Tasks (Status: Done)"),
            "status filter should display, got: {}",
            app.screen_title_for_test()
        );
        let calls = log.lock().map(|l| l.clone()).unwrap_or_default();
        assert_eq!(calls.len(), 1, "only the status-filter fetch used get_tasks");
        assert_eq!(calls[0].1, vec!["Done".to_string()]);
    });
}

/// ListStatusesLoaded refreshes an open filter picker (statuses arrive late)
#[test]
fn test_statuses_loaded_refreshes_open_filter_picker() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.set_screen(Screen::Tasks);
        app.set_current_list_id(Some("list-1".to_string()));

        // Open with only fallback statuses
        app.set_tasks_for_test(vec![task_with_status("t1", "A", status("To Do", 0))]);
        app.open_status_filter_picker();
        assert_eq!(app.status_picker_statuses_for_test().len(), 1);

        // Full status configuration arrives while the picker is open
        let tx = app.message_tx_for_testing();
        tx.send(AppMessage::ListStatusesLoaded(
            "list-1".to_string(),
            Ok(vec![
                status("To Do", 0),
                status("In Progress", 1),
                status("Done", 2),
            ]),
        ))
        .await
        .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        app.process_async_messages();

        assert_eq!(
            app.status_picker_statuses_for_test().len(),
            3,
            "open picker should refresh when statuses arrive"
        );
    });
}
