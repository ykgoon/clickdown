//! Tests for the root-level "Assigned to Me" section.
//!
//! The section is a navigation-root entry that queries assigned tasks across
//! the whole workspace and lists them with the most recently active task on
//! top.

mod fixtures;

use std::sync::Arc;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::{Task, Workspace};
use clickdown::tui::app::{Screen, TuiApp};
use clickdown::tui::input::InputEvent;
use clickdown::tui::widgets::{ListRow, SidebarItem};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::runtime::Runtime;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}

/// Esc as a real key event (a `Char('\u{1b}')` is not the same thing).
fn esc() -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
}

/// Enter as a real key event (`Char('\r')` maps to a character, not Enter).
fn enter() -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
}

fn workspace(id: &str, name: &str) -> Workspace {
    serde_json::from_value(serde_json::json!({ "id": id, "name": name }))
        .expect("workspace fixture should deserialize")
}

fn task(id: &str, name: &str, updated_at: Option<i64>) -> Task {
    Task {
        id: id.to_string(),
        name: name.to_string(),
        updated_at,
        ..Default::default()
    }
}

/// Build an app at the navigation root with a workspace and a loaded user.
fn root_app(rt: &Runtime, client: MockClickUpClient) -> TuiApp {
    let mut app = rt.block_on(async { TuiApp::with_client(Arc::new(client)).unwrap() });
    let _guard = rt.enter(); // load_workspaces spawns, so a reactor is needed
    app.load_workspaces();
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(30)).await });
    app.process_async_messages();
    app.set_current_user_id(Some(42));
    app
}

/// The entry is pinned to the navigation root, ahead of the workspaces.
#[test]
fn test_assigned_entry_is_at_navigation_root() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_team_tasks(vec![]);

    let mut app = root_app(&rt, client);
    let items = app.sidebar().items().to_vec();

    assert!(
        matches!(items.first(), Some(SidebarItem::Inbox { .. })),
        "the Inbox entry sits above Assigned at the root, got: {:?}",
        items
    );
    assert!(
        matches!(items.get(1), Some(SidebarItem::AssignedToMe)),
        "the Assigned entry must follow the Inbox entry, got: {:?}",
        items
    );
    assert!(
        matches!(items.get(2), Some(SidebarItem::Workspace { .. })),
        "workspaces must follow the Assigned entry"
    );
}

/// Selecting the entry opens the Assigned view rather than descending into a
/// workspace.
#[test]
fn test_entering_assigned_entry_opens_assigned_screen() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_team_tasks(vec![task("t1", "Fix bug", Some(1000))]);

    let mut app = root_app(&rt, client);
    let _guard = rt.enter();
    app.sidebar().select_by_id("assigned-to-me");
    app.navigate_into();

    assert_eq!(app.screen(), Screen::Assigned);
    assert!(
        app.screen_title_for_test().contains("Assigned to Me"),
        "title should name the view, got: {}",
        app.screen_title_for_test()
    );
}

/// Results arrive sorted by most recent activity, regardless of API order.
#[test]
fn test_assigned_tasks_sorted_by_most_recent_activity() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        // Deliberately out of order, including a task with no timestamp.
        .with_team_tasks(vec![
            task("old", "Oldest", Some(1_000)),
            task("none", "No timestamp", None),
            task("newest", "Newest", Some(9_000)),
            task("mid", "Middle", Some(5_000)),
        ]);

    let mut app = root_app(&rt, client);
    let _guard = rt.enter();
    app.sidebar().select_by_id("assigned-to-me");
    app.navigate_into();
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(50)).await });
    app.process_async_messages();

    let names: Vec<String> = app
        .task_list_for_test()
        .rows()
        .iter()
        .filter_map(|row| match row {
            ListRow::Task(t) => Some(t.name.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(
        names,
        vec!["Newest", "Middle", "Oldest", "No timestamp"],
        "most recently active must be first, missing timestamps last"
    );
}

/// The Assigned view is workspace-wide, so it must not be reachable from a
/// deeper level: navigating down replaces the sidebar and drops the entry.
#[test]
fn test_assigned_entry_absent_below_the_root() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_spaces(vec![])
        .with_team_tasks(vec![]);

    let mut app = root_app(&rt, client);
    let _guard = rt.enter();
    assert!(
        app.sidebar().select_by_id("assigned-to-me"),
        "entry exists at the root"
    );

    // Descend into the workspace.
    app.sidebar().select_by_id("ws-1");
    app.navigate_into();
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(30)).await });
    app.process_async_messages();

    assert_eq!(app.screen(), Screen::Spaces);
    assert!(
        !app.sidebar().select_by_id("assigned-to-me"),
        "the Assigned entry must not appear below the navigation root"
    );
}

/// Esc from the Assigned view returns to the root with the entry selected.
#[test]
fn test_esc_from_assigned_returns_to_root() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_team_tasks(vec![]);

    let mut app = root_app(&rt, client);
    let _guard = rt.enter();
    app.sidebar().select_by_id("assigned-to-me");
    app.navigate_into();
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(30)).await });
    app.process_async_messages();
    assert_eq!(app.screen(), Screen::Assigned);

    app.update(esc());

    assert_eq!(app.screen(), Screen::Workspaces);
    assert!(
        matches!(
            app.sidebar().selected_item(),
            Some(SidebarItem::AssignedToMe)
        ),
        "the entry should still be selected after going back"
    );
}

/// Opening a task from the Assigned view returns there on Esc, not to a list.
#[test]
fn test_task_detail_from_assigned_returns_to_assigned() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_team_tasks(vec![task("t1", "Fix bug", Some(1000))]);

    let mut app = root_app(&rt, client);
    let _guard = rt.enter();
    app.sidebar().select_by_id("assigned-to-me");
    app.navigate_into();
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(50)).await });
    app.process_async_messages();

    app.update(enter()); // Enter opens the selected task
    assert_eq!(app.screen(), Screen::TaskDetail);

    app.update(esc()); // goes back
    assert_eq!(
        app.screen(),
        Screen::Assigned,
        "Esc must return to the workspace-wide view the task came from"
    );
}

/// A small workload costs exactly one request: no follow-up page.
#[test]
fn test_short_page_does_not_request_another_page() {
    let rt = Runtime::new().unwrap();
    let mock = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_team_tasks(vec![task("t1", "One", Some(1))]);
    let pages = mock.assigned_page_log_handle();

    let mut app = root_app(&rt, mock);
    let _guard = rt.enter();
    app.sidebar().select_by_id("assigned-to-me");
    app.navigate_into();
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(50)).await });
    app.process_async_messages();

    assert_eq!(
        pages.lock().unwrap().clone(),
        vec![0],
        "a short page must end the walk"
    );
}

/// A full page triggers the next page, and multiple pages are merged.
#[test]
fn test_full_page_fetches_next_page_and_merges() {
    let rt = Runtime::new().unwrap();
    let first_page: Vec<Task> = (0..100)
        .map(|i| {
            task(
                &format!("p0-{}", i),
                &format!("Page0 Task {}", i),
                Some(1000 + i),
            )
        })
        .collect();
    let second_page = vec![task("p1-0", "Page1 Task", Some(999_999))];

    let mock = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_team_tasks_page(0, first_page)
        .with_team_tasks_page(1, second_page);
    let pages = mock.assigned_page_log_handle();

    let mut app = root_app(&rt, mock);
    app.sidebar().select_by_id("assigned-to-me");

    let _guard = rt.enter();
    app.navigate_into();
    // Round 2 fetches page 1, round 3 settles.
    for _ in 0..3 {
        rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(60)).await });
        app.process_async_messages();
    }

    assert_eq!(
        pages.lock().unwrap().clone(),
        vec![0, 1],
        "a full page must be followed by the next page, then stop"
    );
    assert_eq!(
        app.task_list_for_test()
            .rows()
            .iter()
            .filter(|r| matches!(r, ListRow::Task(_)))
            .count(),
        101,
        "both pages must be merged into one list"
    );
    // The newest task from page 1 sorts to the top.
    let first_name = app
        .task_list_for_test()
        .rows()
        .iter()
        .find_map(|r| match r {
            ListRow::Task(t) => Some(t.name.clone()),
            _ => None,
        });
    assert_eq!(first_name.as_deref(), Some("Page1 Task"));
}

/// A failing request surfaces an error instead of an empty list.
#[test]
fn test_assigned_load_failure_is_reported() {
    let rt = Runtime::new().unwrap();
    let mut client =
        MockClickUpClient::new().with_workspaces(vec![workspace("ws-1", "Engineering")]);
    client.team_tasks_response = Some(Err(anyhow::anyhow!("network down")));

    let mut app = root_app(&rt, client);
    let _guard = rt.enter();
    app.sidebar().select_by_id("assigned-to-me");
    app.navigate_into();
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(50)).await });
    app.process_async_messages();

    assert_eq!(app.status(), "Failed to load assigned tasks");
}

/// Without a user profile the view explains itself rather than querying with
/// a placeholder id.
#[test]
fn test_assigned_without_user_id_waits() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_team_tasks(vec![]);

    let mut app = rt.block_on(async { TuiApp::with_client(Arc::new(client)).unwrap() });
    let _guard = rt.enter();
    app.load_workspaces();
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(30)).await });
    app.process_async_messages();
    // No set_current_user_id: the profile has not arrived.

    app.sidebar().select_by_id("assigned-to-me");
    app.navigate_into();

    assert_eq!(
        app.status(),
        "Waiting for your user profile...",
        "must not query with a missing user id"
    );
}
