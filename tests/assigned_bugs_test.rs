//! Regression tests for two bugs reported from the Assigned to Me view:
//!
//! 1. Opening a task from there and assigning it failed with
//!    "list context not available": the assign flow required
//!    `current_list_id`, which that view deliberately clears.
//! 2. Quitting from there did not restore back to it on next launch.

mod fixtures;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::EntityRef;
use clickdown::models::Task;
use clickdown::tui::app::{Screen, TuiApp};
use clickdown::tui::input::InputEvent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}

fn task_in_list(id: &str, list_id: Option<&str>) -> Task {
    let mut task = fixtures::test_task();
    task.id = id.to_string();
    task.name = "Assigned task".to_string();
    task.list = list_id.map(|id| EntityRef {
        id: id.to_string(),
        name: Some("L".to_string()),
        ..Default::default()
    });
    task
}

fn new_app(rt: &Runtime) -> TuiApp {
    rt.block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() })
}

// ---------------------------------------------------------------- bug 1: assign

/// A task opened from Assigned has no `current_list_id`, but carries its own
/// list reference, so assigning must still find the member list.
#[test]
fn assign_from_assigned_view_uses_the_tasks_own_list() {
    let rt = Runtime::new().unwrap();
    let mock = MockClickUpClient::new().with_list_members(vec![
        clickdown::models::User {
            id: 1,
            username: "alice".to_string(),
            color: None,
            email: None,
            profile_picture: None,
            initials: None,
        },
    ]);
    let mut app = rt.block_on(async { TuiApp::with_client(Arc::new(mock)).unwrap() });

    // A task opened from Assigned lands on the task-detail screen, while the
    // navigation context stays cleared.
    app.set_screen(Screen::TaskDetail);
    app.set_current_list_id(None);
    app.set_task_detail_task_for_test(task_in_list("t1", Some("list-9")));

    rt.block_on(async { app.update(key(' ')) });
    rt.block_on(async { app.update(key('t')) });
    rt.block_on(async { app.update(key('a')) });

    assert!(
        !app.status_for_test().contains("list context not available"),
        "must not refuse to assign, got: {}",
        app.status_for_test()
    );
    assert!(
        !app.status_for_test().contains("list is unknown"),
        "the task's own list should be resolved, got: {}",
        app.status_for_test()
    );
}

/// With no navigation context and no list on the task there is genuinely
/// nothing to query, and the message must say that rather than a vague one.
#[test]
fn assign_without_any_list_reports_the_real_problem() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);

    app.set_screen(Screen::TaskDetail);
    app.set_current_list_id(None);
    app.set_task_detail_task_for_test(task_in_list("t1", None));

    rt.block_on(async { app.update(key(' ')) });
    rt.block_on(async { app.update(key('t')) });
    rt.block_on(async { app.update(key('a')) });

    assert!(
        app.status_for_test().contains("list is unknown"),
        "expected an explanation naming the list, got: {}",
        app.status_for_test()
    );
}

// ---------------------------------------------------------------- bug 2: restore

/// A session saved while in Assigned to Me reopens there.
#[test]
fn saved_assigned_screen_is_restored_on_launch() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);

    // What quitting from the Assigned view records.
    app.restore_saved_screen_for_test("Assigned");

    // Workspaces arrive (supplies the team), then the user profile.
    app.simulate_workspaces_loaded_for_test(vec![clickdown::models::Workspace {
        id: "team-1".to_string(),
        name: "Workspace".to_string(),
        color: None,
        avatar: None,
        member_count: None,
    }]);
    rt.block_on(async { app.process_async_messages() });

    app.set_current_user_id(Some(123));
    app.simulate_user_loaded_for_test(clickdown::models::User {
        id: 123,
        username: "me".to_string(),
        color: None,
        email: None,
        profile_picture: None,
        initials: None,
    });
    rt.block_on(async { app.process_async_messages() });

    assert_eq!(
        app.screen(),
        Screen::Assigned,
        "should land back in Assigned to Me"
    );
    assert!(
        app.status_for_test().contains("Assigned"),
        "restore should say where it landed, got: {}",
        app.status_for_test()
    );
}

/// The user profile can arrive before the workspace list; restore must still
/// happen once both are present, not depend on message order.
#[test]
fn restore_is_independent_of_message_order() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);
    app.restore_saved_screen_for_test("Assigned");

    // Profile first, workspace second - the reverse of the happy path.
    app.set_current_user_id(Some(123));
    app.simulate_user_loaded_for_test(clickdown::models::User {
        id: 123,
        username: "me".to_string(),
        color: None,
        email: None,
        profile_picture: None,
        initials: None,
    });
    rt.block_on(async { app.process_async_messages() });
    assert_ne!(
        app.screen(),
        Screen::Assigned,
        "cannot restore before a workspace is known"
    );

    app.simulate_workspaces_loaded_for_test(vec![clickdown::models::Workspace {
        id: "team-1".to_string(),
        name: "Workspace".to_string(),
        color: None,
        avatar: None,
        member_count: None,
    }]);
    rt.block_on(async { app.process_async_messages() });

    assert_eq!(
        app.screen(),
        Screen::Assigned,
        "should restore once both prerequisites exist"
    );
}

/// A session saved in a list must not be turned into the Assigned view.
#[test]
fn saved_list_session_still_restores_normally() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);
    app.restore_saved_screen_for_test("Tasks");

    app.set_current_user_id(Some(123));
    app.simulate_user_loaded_for_test(clickdown::models::User {
        id: 123,
        username: "me".to_string(),
        color: None,
        email: None,
        profile_picture: None,
        initials: None,
    });
    rt.block_on(async { app.process_async_messages() });

    assert_ne!(
        app.screen(),
        Screen::Assigned,
        "a list session must not become the Assigned view"
    );
}
