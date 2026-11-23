//! Regression tests for status assignment while a task list is filtered.
//!
//! Reported: with the task list filtered, opening a task and pressing
//! `SPC t s` offered a single status instead of the full set. Reproduced in
//! the Assigned to Me view, which has no navigation context and keeps its
//! rows in `assigned_tasks` rather than `tasks`.

mod fixtures;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::task::TaskStatus;
use clickdown::models::EntityRef;
use clickdown::models::{Task, User};
use clickdown::tui::app::{Screen, StatusPickerMode, TuiApp};
use std::sync::Arc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use clickdown::tui::input::InputEvent;
use tokio::runtime::Runtime;

fn status(name: &str, orderindex: u32) -> TaskStatus {
    TaskStatus {
        id: Some(format!("status-{name}")),
        status: name.to_string(),
        color: Some("#123456".to_string()),
        type_field: None,
        orderindex: Some(orderindex),
        status_group: None,
    }
}

fn task_with(id: &str, name: &str, st: TaskStatus, list_id: Option<&str>) -> Task {
    let mut task = fixtures::test_task();
    task.id = id.to_string();
    task.name = name.to_string();
    task.status = Some(st);
    task.list = list_id.map(|id| EntityRef {
        id: id.to_string(),
        name: Some("L".to_string()),
        ..Default::default()
    });
    task
}

/// The reported bug: status picker opened from a filtered Assigned view.
#[test]
fn change_status_picker_offers_full_status_set_in_assigned_view() {
    let rt = Runtime::new().unwrap();
    let mock = MockClickUpClient::new();
    let mut app = rt.block_on(async { TuiApp::with_client(Arc::new(mock)).unwrap() });

    app.set_screen(Screen::Assigned);

    // A filter is active and the loaded rows contain only that one status,
    // which is exactly the state that produced a one-entry menu.
    app.set_status_filter_for_test(Some("In Progress".to_string()));
    app.set_assigned_tasks_for_test(vec![
        task_with("t1", "a", status("In Progress", 1), Some("list-1")),
        task_with("t2", "b", status("In Progress", 1), Some("list-1")),
    ]);

    // The app has seen the list's full configuration.
    app.set_cached_list_statuses_for_test(
        "list-1",
        vec![
            status("To Do", 0),
            status("In Progress", 1),
            status("Blocked", 2),
            status("Done", 3),
        ],
    );

    let target = task_with("t1", "a", status("In Progress", 1), Some("list-1"));
    app.open_status_picker_for_test(target);

    let offered: Vec<String> = app
        .status_picker_statuses_for_test()
        .iter()
        .map(|s| s.status.clone())
        .collect();

    assert_eq!(
        offered,
        vec!["To Do", "In Progress", "Blocked", "Done"],
        "filtered view must still offer the full status set, got {offered:?}"
    );
    assert_eq!(
        app.status_picker_cursor_for_test(),
        1,
        "cursor should start on the task's current status"
    );
}

/// Same expectation without any cached config: statuses observed across the
/// assigned rows (not just the filtered subset) must widen the menu.
#[test]
fn change_status_picker_widens_from_all_assigned_tasks() {
    let rt = Runtime::new().unwrap();
    let mock = MockClickUpClient::new();
    let mut app = rt.block_on(async { TuiApp::with_client(Arc::new(mock)).unwrap() });

    app.set_screen(Screen::Assigned);
    app.set_status_filter_for_test(Some("In Progress".to_string()));

    // The workspace-wide query returned rows in several statuses; only the
    // filtered ones are rendered, but all of them were loaded.
    app.set_assigned_tasks_for_test(vec![
        task_with("t1", "a", status("In Progress", 1), None),
        task_with("t2", "b", status("To Do", 0), None),
        task_with("t3", "c", status("Done", 3), None),
    ]);
    app.set_tasks_for_test(vec![task_with(
        "t1",
        "a",
        status("In Progress", 1),
        None,
    )]);

    let target = task_with("t1", "a", status("In Progress", 1), None);
    app.open_status_picker_for_test(target);

    let offered: Vec<String> = app
        .status_picker_statuses_for_test()
        .iter()
        .map(|s| s.status.clone())
        .collect();

    assert_eq!(
        offered,
        vec!["To Do", "In Progress", "Done"],
        "menu should union statuses from every loaded assigned task, got {offered:?}"
    );
}

/// A task with no list context must not reduce the menu to its own status.
#[test]
fn change_status_picker_without_any_context_still_offers_known_statuses() {
    let rt = Runtime::new().unwrap();
    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap()
    });

    app.set_screen(Screen::TaskDetail);
    app.set_tasks_for_test(vec![
        task_with("t1", "a", status("In Progress", 1), None),
        task_with("t2", "b", status("Done", 3), None),
    ]);

    let target = task_with("t1", "a", status("In Progress", 1), None);
    app.open_status_picker_for_test(target);

    assert!(
        app.status_picker_statuses_for_test().len() >= 2,
        "expected more than the task's own status, got {:?}",
        app.status_picker_statuses_for_test()
            .iter()
            .map(|s| s.status.clone())
            .collect::<Vec<_>>()
    );
}

/// Users are still listed as assignable: guards the *assignment* path from
/// being narrowed by the same filtered-view bug.
#[test]
fn assignee_picker_still_offers_all_members_in_assigned_view() {
    let rt = Runtime::new().unwrap();
    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap()
    });

    app.set_screen(Screen::Assigned);
    app.set_status_filter_for_test(Some("In Progress".to_string()));
    app.set_mention_members_for_test(vec![
        User {
            id: 1,
            username: "alice".to_string(),
            color: None,
            email: None,
            profile_picture: None,
            initials: None,
        },
        User {
            id: 2,
            username: "bob".to_string(),
            color: None,
            email: None,
            profile_picture: None,
            initials: None,
        },
    ]);

    assert_eq!(
        app.mention_members_for_test().len(),
        2,
        "assignment people list must not be narrowed by the list filter"
    );
}

/// A task may only be offered statuses from its own list: ClickUp rejects a
/// status that does not belong to the task's list, so widening to every
/// cached list would present options that cannot be applied.
#[test]
fn change_status_picker_does_not_offer_another_lists_statuses() {
    let rt = Runtime::new().unwrap();
    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });

    app.set_screen(Screen::TaskDetail);
    app.set_cached_list_statuses_for_test(
        "list-A",
        vec![status("To Do", 0), status("In Progress", 1)],
    );
    app.set_cached_list_statuses_for_test(
        "list-B",
        vec![status("Queued", 0), status("Shipped", 1)],
    );

    let target = task_with("t1", "a", status("In Progress", 1), Some("list-A"));
    app.open_status_picker_for_test(target);

    let offered: Vec<String> = app
        .status_picker_statuses_for_test()
        .iter()
        .map(|s| s.status.clone())
        .collect();

    assert_eq!(
        offered,
        vec!["To Do", "In Progress"],
        "only the task's own list statuses are valid choices, got {offered:?}"
    );
    assert!(
        !offered.iter().any(|s| s == "Queued" || s == "Shipped"),
        "list-B statuses must not leak into a list-A task, got {offered:?}"
    );
}

/// The task's current status stays selectable even if the cached list config
/// is missing it (stale cache), so the picker always shows where the task is.
#[test]
fn change_status_picker_keeps_current_status_even_if_config_lacks_it() {
    let rt = Runtime::new().unwrap();
    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });

    app.set_screen(Screen::TaskDetail);
    app.set_cached_list_statuses_for_test("list-A", vec![status("To Do", 0)]);

    let target = task_with("t1", "a", status("Archived", 9), Some("list-A"));
    app.open_status_picker_for_test(target);

    let offered: Vec<String> = app
        .status_picker_statuses_for_test()
        .iter()
        .map(|s| s.status.clone())
        .collect();

    assert!(
        offered.iter().any(|s| s == "Archived"),
        "the task's current status must be selectable, got {offered:?}"
    );
}

// ---------------------------------------------------------------- end-to-end

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}

/// Drive the real key path: open a task from the filtered Assigned view and
/// press `SPC t s`. The unit-level seeding is covered above; this proves the
/// actual keyboard route (`update`) reaches the same full menu.
#[test]
fn spc_t_s_from_filtered_assigned_task_detail_offers_full_status_set() {
    let rt = Runtime::new().unwrap();
    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });

    app.set_screen(Screen::Assigned);
    app.set_status_filter_for_test(Some("In Progress".to_string()));
    app.set_cached_list_statuses_for_test(
        "list-1",
        vec![
            status("To Do", 0),
            status("In Progress", 1),
            status("Blocked", 2),
            status("Done", 3),
        ],
    );

    let task = task_with("t1", "a", status("In Progress", 1), Some("list-1"));
    app.set_assigned_tasks_for_test(vec![task.clone()]);
    app.set_task_detail_task_for_test(task);
    app.set_screen(Screen::TaskDetail);

    // SPACE t s
    for k in [' ', 't', 's'] {
        app.update(key(k));
    }

    let offered: Vec<String> = app
        .status_picker_statuses_for_test()
        .iter()
        .map(|s| s.status.clone())
        .collect();

    assert_eq!(
        offered,
        vec!["To Do", "In Progress", "Blocked", "Done"],
        "SPC t s from a filtered Assigned task must offer the full set, got {offered:?}"
    );
    assert_eq!(
        app.status_picker_mode_for_test(),
        StatusPickerMode::ChangeTask,
        "SPC t s must change the task, not filter the list"
    );
}
