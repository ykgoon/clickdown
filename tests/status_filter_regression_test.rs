//! Regressions: the status overlay's mode, filter toggling in the Assigned
//! view, and @mention resolution when the hierarchy context is gone.

use std::sync::Arc;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::{Task, TaskStatus, User, Workspace};
use clickdown::tui::app::{Screen, StatusPickerMode, TuiApp};
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

fn task_with_status(id: &str, name: &str, st: TaskStatus) -> Task {
    Task {
        id: id.to_string(),
        name: name.to_string(),
        status: Some(st),
        ..Default::default()
    }
}

fn workspace(id: &str, name: &str) -> Workspace {
    serde_json::from_value(serde_json::json!({ "id": id, "name": name }))
        .expect("workspace fixture should deserialize")
}

fn app(rt: &Runtime, client: MockClickUpClient) -> TuiApp {
    let mut a = rt.block_on(async { TuiApp::with_client(Arc::new(client)).unwrap() });
    a.set_current_user_id(Some(42));
    a.set_current_team_id_for_test("team-1".to_string());
    a
}

fn pump(a: &mut TuiApp, rt: &Runtime, f: impl FnOnce(&mut TuiApp)) {
    let _g = rt.enter();
    f(a);
    std::thread::sleep(std::time::Duration::from_millis(30));
    a.process_async_messages();
}

/// Regression: a status CHANGE left `status_picker_task_id` set, so the next
/// filter picker treated Enter as "save this task's status" and the filter
/// toggle did nothing (while also mutating a task behind the user's back).
#[test]
fn filter_picker_after_status_change_does_not_mutate_a_task() {
    let rt = Runtime::new().unwrap();
    let t = task_with_status("task-1", "Original", status("To Do", 0));
    let mock = MockClickUpClient::new()
        .with_tasks(vec![t.clone()])
        .with_team_tasks(vec![t.clone()]);
    let mut a = app(&rt, mock);
    a.set_screen(Screen::Assigned);
    a.set_tasks_for_test(vec![t.clone()]);

    // Open a status-change picker (sets the task id), then abandon it.
    a.open_status_picker_for_test(t.clone());
    assert_eq!(a.status_picker_mode_for_test(), StatusPickerMode::ChangeTask);

    // Now open the FILTER picker: mode and task id must both be reset.
    a.open_status_filter_picker();
    assert_eq!(a.status_picker_mode_for_test(), StatusPickerMode::FilterList);
    assert!(
        a.status_picker_task_id_for_test().is_none(),
        "filter picker must not inherit the change-picker's task id"
    );

    // Enter must apply a filter, never write a task status.
    pump(&mut a, &rt, |a| {
        a.handle_status_picker_input(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Enter,
        ))
    });

    assert!(
        a.status_filter_for_test().is_some(),
        "Enter in filter mode must set the filter"
    );
    assert_eq!(
        a.tasks_for_test()[0].status.as_ref().unwrap().status,
        "To Do",
        "the task's own status must not change"
    );
}

/// Regression: with a filter active, the picker offered only the one status
/// present in the (filtered) result, so re-filtering looked empty.
#[test]
fn filter_picker_offers_full_status_list_while_filtered() {
    let rt = Runtime::new().unwrap();
    let mock = MockClickUpClient::new().with_team_tasks(vec![
        task_with_status("t1", "A", status("In Progress", 1)),
    ]);
    let mut a = app(&rt, mock);
    a.set_screen(Screen::Assigned);
    a.set_current_list_id(None); // Assigned view: no list context

    // What the app has learned about this workspace's statuses.
    a.set_cached_list_statuses_for_test(
        "some-list",
        vec![
            status("To Do", 0),
            status("In Progress", 1),
            status("Review", 2),
            status("Done", 3),
        ],
    );

    pump(&mut a, &rt, |a| a.load_assigned_tasks());

    // Apply "In Progress"
    a.open_status_filter_picker();
    let idx = a
        .status_picker_statuses_for_test()
        .iter()
        .position(|s| s.status == "In Progress")
        .expect("In Progress offered");
    a.set_status_picker_cursor_for_test(idx);
    pump(&mut a, &rt, |a| {
        a.handle_status_picker_input(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Enter,
        ))
    });
    assert_eq!(a.status_filter_for_test().as_deref(), Some("In Progress"));

    // Reopen: the full configured set must still be on offer, not just the
    // single status the filtered result happens to contain.
    a.open_status_filter_picker();
    let names: Vec<String> = a
        .status_picker_statuses_for_test()
        .iter()
        .map(|s| s.status.clone())
        .collect();
    assert!(
        names.len() >= 4,
        "expected the full status set, got {names:?}"
    );
    for expected in ["To Do", "In Progress", "Review", "Done"] {
        assert!(names.iter().any(|n| n == expected), "missing {expected} in {names:?}");
    }
}

/// Esc over an active filter leaves it alone; clearing is explicit.
///
/// Esc used to double as "clear the filter", so a key that reads as cancel
/// silently changed the result set. Filter removal now lives in the filter
/// menu (`SPC t f`, then `x`) and is covered by `clear_all_filters_*` tests.
#[test]
fn esc_keeps_active_filter_in_assigned_view() {
    let rt = Runtime::new().unwrap();
    let mock = MockClickUpClient::new()
        .with_team_tasks(vec![task_with_status("t1", "A", status("Done", 2))]);
    let mut a = app(&rt, mock);
    a.set_screen(Screen::Assigned);
    a.set_cached_list_statuses_for_test("l", vec![status("Done", 2)]);
    pump(&mut a, &rt, |a| a.load_assigned_tasks());

    a.open_status_filter_picker();
    pump(&mut a, &rt, |a| {
        a.handle_status_picker_input(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Enter,
        ))
    });
    assert!(a.status_filter_for_test().is_some(), "filter applied");

    a.open_status_filter_picker();
    pump(&mut a, &rt, |a| {
        a.handle_status_picker_input(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Esc,
        ))
    });
    assert!(
        !a.status_picker_open_for_test(),
        "Esc must close the picker"
    );
    assert_eq!(
        a.status_filter_for_test().as_deref(),
        Some("Done"),
        "Esc must not clear the active filter"
    );

    // The explicit path does remove it.
    pump(&mut a, &rt, |a| a.clear_all_filters());
    assert!(
        a.status_filter_for_test().is_none(),
        "clear_all_filters must remove the filter"
    );
}

/// Regression: a task opened from the Assigned view has no `current_list_id`,
/// which used to abort the member fetch and leave the @ picker empty.
#[test]
fn mention_picker_resolves_members_without_list_context() {
    let rt = Runtime::new().unwrap();
    let mut t = task_with_status("task-1", "From assorted view", status("To Do", 0));
    // The task carries its own list reference; that is the fallback source.
    t.list = Some(clickdown::models::EntityRef {
         id: "list-9".to_string(),
         name: Some("Backlog".to_string()),
         ..Default::default()
    });

    let mock = MockClickUpClient::new().with_list_members(vec![User {
        id: 7,
        username: "bob".to_string(),
        color: None,
        email: Some("bob@x.com".to_string()),
        profile_picture: None,
        initials: None,
    }]);

    let mut a = rt.block_on(async { TuiApp::with_client(Arc::new(mock)).unwrap() });
    a.set_screen(Screen::TaskDetail);
    a.set_current_list_id(None); // Assigned view cleared the hierarchy
    a.set_task_detail_task_for_test(t);

    // Open the mention picker the way the '@' key does.
    pump(&mut a, &rt, |a| a.open_mention_picker_for_test());

    let names: Vec<String> = a
        .mention_members_for_test()
        .iter()
        .map(|m| m.username.clone())
        .collect();
    assert_eq!(
        names,
        vec!["bob"],
        "members must load from the task's list when no hierarchy context exists"
    );
}

/// The picker must not fire a members request when nothing is known, but must
/// still report why instead of silently showing an empty menu.
#[test]
fn mention_picker_reports_missing_context() {
    let rt = Runtime::new().unwrap();
    let mock = MockClickUpClient::new();
    let mut a = rt.block_on(async { TuiApp::with_client(Arc::new(mock)).unwrap() });
    a.set_screen(Screen::TaskDetail);
    a.set_current_list_id(None);

    pump(&mut a, &rt, |a| a.open_mention_picker_for_test());

    assert!(a.is_mention_picker_open());
    assert!(
        a.status().contains("@mention"),
        "expected an explanatory status, got: {}",
        a.status()
    );
}

/// Sanity: no workspace is needed for the above, but the helper type must
/// deserialize as the app expects.
#[test]
fn workspace_fixture_is_valid() {
    assert_eq!(workspace("ws-1", "Eng").id, "ws-1");
}

/// Members cached for one list must not be offered for a task in another
/// list: that would let you @mention someone without access to the task.
#[test]
fn mention_picker_refetches_members_for_a_different_list() {
    let rt = Runtime::new().unwrap();
    let mut t = task_with_status("task-2", "Elsewhere", status("To Do", 0));
    t.list = Some(clickdown::models::EntityRef {
         id: "list-B".to_string(),
         name: Some("Other".to_string()),
         ..Default::default()
    });

    let mock = MockClickUpClient::new()
        .with_list_members(vec![User {
            id: 9,
            username: "other_list_user".to_string(),
            color: None,
            email: None,
            profile_picture: None,
            initials: None,
        }]);
    let mut a = rt.block_on(async { TuiApp::with_client(Arc::new(mock)).unwrap() });
    a.set_screen(Screen::TaskDetail);
    a.set_current_list_id(Some("list-A".to_string()));

    // Seed members that belong to list A.
    a.set_mention_members_for_test(vec![User {
        id: 1,
        username: "list_a_user".to_string(),
        color: None,
        email: None,
        profile_picture: None,
        initials: None,
    }]);

    // The task lives in list B; the picker must not reuse A's people.
    a.set_current_list_id(None);
    a.set_task_detail_task_for_test(t);
    pump(&mut a, &rt, |a| a.open_mention_picker_for_test());

    let names: Vec<String> = a
        .mention_members_for_test()
        .iter()
        .map(|m| m.username.clone())
        .collect();
    assert_eq!(
        names,
        vec!["other_list_user"],
        "expected members for the task's own list, got {names:?}"
    );
}
