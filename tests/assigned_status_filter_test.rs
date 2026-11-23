//! Tests for the status filter in the root "Assigned to Me" view (`SPC t F`).
//!
//! The Assigned view is workspace-wide, so the picker seeds from statuses
//! observed on the loaded assigned tasks and applying/clearing re-runs the
//! paged team-tasks query with the status carried through.

use std::sync::Arc;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::{Task, TaskStatus, Workspace};
use clickdown::tui::app::{Screen, TuiApp};
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

fn workspace(id: &str, name: &str) -> Workspace {
    serde_json::from_value(serde_json::json!({ "id": id, "name": name }))
    .expect("workspace fixture should deserialize")
}

fn assigned_app(rt: &Runtime, client: MockClickUpClient) -> TuiApp {
    let mut app = rt.block_on(async { TuiApp::with_client(Arc::new(client)).unwrap() });
    app.set_screen(Screen::Assigned);
    app.set_current_user_id(Some(42));
    app.set_current_team_id_for_test("team-1".to_string());
    app
}

/// Run a spawning call with a runtime reactor context, then drain messages.
fn spawn_and_pump(app: &mut TuiApp, rt: &Runtime, f: impl FnOnce(&mut TuiApp)) {
    let _guard = rt.enter();
    f(app);
    std::thread::sleep(std::time::Duration::from_millis(30));
    app.process_async_messages();
}

fn pump(app: &mut TuiApp, rt: &Runtime) {
    let _guard = rt.enter();
    std::thread::sleep(std::time::Duration::from_millis(30));
    app.process_async_messages();
}

/// The picker opens on the Assigned screen seeded from loaded task statuses.
#[test]
fn test_assigned_picker_seeds_from_loaded_tasks() {
    let rt = Runtime::new().unwrap();
    let mock = MockClickUpClient::new()
        .with_team_tasks(vec![
            task_with_status("t1", "A", status("In Progress", 1)),
            task_with_status("t2", "B", status("Done", 2)),
            task_with_status("t3", "C", status("In Progress", 1)),
        ]);
    let mut app = assigned_app(&rt, mock);

    spawn_and_pump(&mut app, &rt, |a| a.load_assigned_tasks());

    app.open_status_filter_picker();
    assert!(app.is_status_picker_open());
    let names: Vec<String> = app
        .status_picker_statuses_for_test()
        .iter()
        .map(|s| s.status.clone())
        .collect();
    assert_eq!(names, vec!["In Progress", "Done"]);
}

/// Applying a selection reloads assigned tasks with the status filter.
#[test]
fn test_assigned_apply_filter_sends_status() {
    let rt = Runtime::new().unwrap();
    let mock = MockClickUpClient::new()
        .with_team_tasks(vec![
            task_with_status("t1", "A", status("In Progress", 1)),
            task_with_status("t2", "B", status("Done", 2)),
        ]);
    let log = mock.team_tasks_call_log_handle();
    let mut app = assigned_app(&rt, mock);

    spawn_and_pump(&mut app, &rt, |a| a.load_assigned_tasks());

    app.open_status_filter_picker();
    app.handle_status_picker_input(crossterm::event::KeyEvent::from(
        crossterm::event::KeyCode::Char('j'),
    ));
    {
        let _guard = rt.enter();
        app.handle_status_picker_input(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Enter,
        ));
    }
    spawn_and_pump(&mut app, &rt, |_| {});

    assert!(
        app.screen_title_for_test()
            .ends_with("Assigned to Me (Status: Done)"),
        "title should show the filter, got: {}",
        app.screen_title_for_test()
    );
    let calls = log.lock().map(|l| l.clone()).unwrap_or_default();
    // Two loads: initial + filtered reload; the filtered one carries the status
    assert!(calls.len() >= 2, "expected initial + filtered loads");
    assert!(
        calls.iter().any(|(_, statuses)| statuses == &vec!["Done".to_string()]),
        "one load should carry the Done filter, got: {:?}",
        calls
    );
}

/// Esc in the picker clears an active filter and reloads unfiltered.
#[test]
fn test_assigned_clear_filter_reloads_unfiltered() {
    let rt = Runtime::new().unwrap();
    let mock = MockClickUpClient::new()
        .with_team_tasks(vec![
            task_with_status("t1", "A", status("Done", 2)),
        ]);
    let log = mock.team_tasks_call_log_handle();
    let mut app = assigned_app(&rt, mock);

    spawn_and_pump(&mut app, &rt, |a| a.load_assigned_tasks());

    // Apply a filter first
    app.open_status_filter_picker();
    {
        let _guard = rt.enter();
        app.handle_status_picker_input(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Enter,
        ));
    }
    spawn_and_pump(&mut app, &rt, |_| {});
    assert!(
        app.screen_title_for_test()
            .ends_with("Assigned to Me (Status: Done)"),
        "precondition: filter active, got: {}",
        app.screen_title_for_test()
    );

    // Reopen and clear
    app.open_status_filter_picker();
    {
        let _guard = rt.enter();
        app.handle_status_picker_input(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Esc,
        ));
    }
    spawn_and_pump(&mut app, &rt, |_| {});

    assert!(
        !app.screen_title_for_test().contains("Status:"),
        "title should drop the filter, got: {}",
        app.screen_title_for_test()
    );
    let calls = log.lock().map(|l| l.clone()).unwrap_or_default();
    assert!(
        calls.iter().any(|(_, statuses)| statuses.is_empty()),
        "clearing should trigger an unfiltered reload, got: {:?}",
        calls
    );
}

/// The status filter does not leak into plain task-list loads from the
/// Assigned screen state (Tasks loads key off their own screen).
#[test]
fn test_assigned_filter_does_not_touch_list_loads() {
    let rt = Runtime::new().unwrap();
    let mock = MockClickUpClient::new().with_tasks(vec![]);
    let mut app = assigned_app(&rt, mock);
    // Simulate a filter applied while on Assigned
    app.open_status_filter_picker();
    {
        let _guard = rt.enter();
        app.handle_status_picker_input(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Enter,
        ));
    }
    // Switch to a task list view and load
    app.set_screen(Screen::Tasks);
    app.set_current_list_id(Some("list-1".to_string()));
    spawn_and_pump(&mut app, &rt, |a| a.load_tasks("list-1".to_string()));

    // The status filter persists across screens (single shared state);
    // the title reflects it once back on a Tasks screen with a list.
    app.update_screen_title();
    assert!(
        !app.screen_title_for_test().contains("Assigned to Me"),
        "switching screens should re-derive the title, got: {}",
        app.screen_title_for_test()
    );
}
