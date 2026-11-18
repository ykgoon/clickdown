//! Tests for the root-level Inbox section.
//!
//! The inbox is driven by the undocumented `GET /team/{id}/inbox`. The API is
//! read-only, so read/dismiss state is tracked locally; these tests cover the
//! badge, dismissal, ordering, and navigation into the task view.

mod fixtures;

use std::sync::Arc;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::{Inbox, InboxLocation, InboxStatus, InboxTask, Task, Workspace};
use clickdown::tui::app::{Screen, TuiApp};
use clickdown::tui::input::InputEvent;
use clickdown::tui::widgets::SidebarItem;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::runtime::Runtime;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}

/// `SPC i d` — dismiss the highlighted inbox item.
///
/// Dismissing is not a bare key: it lives in the contextual Inbox group of the
/// SPACEmenu, which only exists while the inbox screen is open.
fn dismiss_chord() -> Vec<InputEvent> {
    vec![key(' '), key('i'), key('d')]
}

fn press(app: &mut TuiApp, events: Vec<InputEvent>) {
    for event in events {
        app.update(event);
    }
}

fn esc() -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
}

fn enter() -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
}

fn workspace(id: &str, name: &str) -> Workspace {
    serde_json::from_value(serde_json::json!({ "id": id, "name": name }))
        .expect("workspace fixture should deserialize")
}

fn inbox_task(id: &str, name: &str, updated_at: Option<i64>) -> InboxTask {
    InboxTask {
        id: id.to_string(),
        name: name.to_string(),
        updated_at,
        created_at: None,
        comments_count: Some(1),
        assigned_comments_count: Some(0),
        unresolved_comments_count: Some(0),
        subcategory: Some(InboxLocation {
            id: "list-1".to_string(),
            name: "V1.5 Single Login".to_string(),
            color: None,
        }),
        project: Some(InboxLocation {
            id: "folder-1".to_string(),
            name: "Future Improvement".to_string(),
            color: None,
        }),
        status: Some(InboxStatus {
            status: "in progress".to_string(),
            color: None,
            status_type: Some("custom".to_string()),
        }),
    }
}

fn inbox_of(tasks: Vec<InboxTask>) -> Inbox {
    Inbox {
        ids: tasks.iter().map(|task| task.id.clone()).collect(),
        reminders: vec![],
        tasks,
    }
}

/// App at the navigation root with a workspace and a prepared inbox.
fn root_app(rt: &Runtime, client: MockClickUpClient) -> TuiApp {
    // Test cache, not the real user database: inbox read/dismiss state is
    // persisted, and sharing one file across tests makes them order-dependent.
    let mut app =
        rt.block_on(async { TuiApp::with_client_and_test_cache(Arc::new(client)).unwrap() });
    let _guard = rt.enter();
    app.load_workspaces();
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(30)).await });
    app.process_async_messages();
    app.set_current_user_id(Some(42));
    app
}

fn open_inbox(rt: &Runtime, app: &mut TuiApp) {
    let _guard = rt.enter();
    app.sidebar().select_by_id("inbox");
    app.navigate_into();
    for _ in 0..2 {
        rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(40)).await });
        app.process_async_messages();
    }
}

/// Inbox sits above Assigned at the navigation root.
#[test]
fn test_inbox_is_above_assigned_at_root() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_inbox(inbox_of(vec![]));

    let mut app = root_app(&rt, client);
    let items = app.sidebar().items().to_vec();

    assert!(
        matches!(items.first(), Some(SidebarItem::Inbox { .. })),
        "Inbox must be the first root entry, got: {:?}",
        items
    );
    assert!(
        matches!(items.get(1), Some(SidebarItem::AssignedToMe)),
        "Assigned must follow Inbox, got: {:?}",
        items
    );
}

/// Selecting the entry opens the inbox and lists its items.
#[test]
fn test_entering_inbox_lists_items() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_inbox(inbox_of(vec![
            inbox_task("t1", "Button colour", Some(1000)),
            inbox_task("t2", "Email recipient", Some(2000)),
        ]));

    let mut app = root_app(&rt, client);
    open_inbox(&rt, &mut app);

    assert_eq!(app.screen(), Screen::Inbox);
    assert_eq!(app.inbox_len_for_test(), 2);
}

/// Items are ordered most-recent-activity first, regardless of API order.
#[test]
fn test_inbox_ordered_by_most_recent_activity() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        // Deliberately unordered, including a timestamp-less entry.
        .with_inbox(inbox_of(vec![
            inbox_task("old", "Oldest", Some(1_000)),
            inbox_task("none", "No timestamp", None),
            inbox_task("new", "Newest", Some(9_000)),
            inbox_task("mid", "Middle", Some(5_000)),
        ]));

    let mut app = root_app(&rt, client);
    open_inbox(&rt, &mut app);

    let names: Vec<String> = app
        .inbox_names_for_test()
        .iter()
        .map(|task| task.name.clone())
        .collect();
    assert_eq!(
        names,
        vec!["Newest", "Middle", "Oldest", "No timestamp"],
        "most recent first, missing timestamps last"
    );
}

/// The sidebar badge shows the number of unread items.
#[test]
fn test_sidebar_badge_shows_unread_count() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_inbox(inbox_of(vec![
            inbox_task("t1", "One", Some(1000)),
            inbox_task("t2", "Two", Some(2000)),
            inbox_task("t3", "Three", Some(3000)),
        ]));

    let mut app = root_app(&rt, client);
    open_inbox(&rt, &mut app);

    // All three start unread.
    let badge = app
        .sidebar()
        .items()
        .iter()
        .find_map(|item| match item {
            SidebarItem::Inbox { unread } => Some(*unread),
            _ => None,
        })
        .expect("inbox entry present");
    assert_eq!(badge, 3, "all three should count as unread initially");
}

/// Moving through the list marks items read, so the badge decreases.
#[test]
fn test_moving_marks_read_and_lowers_badge() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_inbox(inbox_of(vec![
            inbox_task("t1", "One", Some(3000)),
            inbox_task("t2", "Two", Some(2000)),
        ]));

    let mut app = root_app(&rt, client);
    open_inbox(&rt, &mut app);
    assert_eq!(app.inbox_unread_count(), 2);

    app.update(key('j')); // move to the second entry
    assert_eq!(
        app.inbox_unread_count(),
        1,
        "moving onto an item counts as reading it"
    );
}

/// `SPC i d` dismisses the highlighted item and removes it from the list.
#[test]
fn test_dismiss_removes_item() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_inbox(inbox_of(vec![
            inbox_task("t1", "Keep me", Some(3000)),
            inbox_task("t2", "Dismiss me", Some(2000)),
        ]));

    let mut app = root_app(&rt, client);
    open_inbox(&rt, &mut app);
    assert_eq!(app.inbox_len_for_test(), 2);

    app.update(key('j')); // select "Dismiss me"
    press(&mut app, dismiss_chord());

    assert_eq!(app.inbox_len_for_test(), 1, "dismissed item is removed");
    assert_eq!(
        app.inbox_names_for_test()[0].name,
        "Keep me",
        "the other item stays"
    );
}

/// Dismissed items must not come back on a reload — this is what makes
/// "inbox zero" stick, since the API cannot be told about dismissals.
#[test]
fn test_dismissed_items_stay_dismissed_after_reload() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_inbox(inbox_of(vec![
            inbox_task("t1", "One", Some(3000)),
            inbox_task("t2", "Two", Some(2000)),
        ]));

    let mut app = root_app(&rt, client);
    open_inbox(&rt, &mut app);

    press(&mut app, dismiss_chord()); // dismiss the first
    assert_eq!(app.inbox_len_for_test(), 1);

    // Reload returns the same two items from the "server".
    let _guard = rt.enter();
    app.update(key('r'));
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(60)).await });
    app.process_async_messages();

    assert_eq!(
        app.inbox_len_for_test(),
        1,
        "the dismissed item must stay hidden after a reload"
    );
}

/// Enter opens the highlighted entry in the task detail view.
#[test]
fn test_enter_opens_task_detail_for_inbox_item() {
    let rt = Runtime::new().unwrap();
    let opened = Task {
        id: "t1".to_string(),
        name: "Button colour".to_string(),
        ..Default::default()
    };
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_inbox(inbox_of(vec![inbox_task(
            "t1",
            "Button colour",
            Some(1000),
        )]))
        .with_task(opened)
        .with_task_comments(vec![]);

    let mut app = root_app(&rt, client);
    open_inbox(&rt, &mut app);
    assert_eq!(app.inbox_len_for_test(), 1);

    let _guard = rt.enter();
    app.update(enter());
    for _ in 0..2 {
        rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(50)).await });
        app.process_async_messages();
    }

    assert_eq!(
        app.screen(),
        Screen::TaskDetail,
        "Enter must lead to the individual task view"
    );
    assert_eq!(
        app.task_detail().task.as_ref().map(|task| task.id.clone()),
        Some("t1".to_string())
    );
}

/// Esc from a task opened in the inbox returns to the inbox, not to a list.
#[test]
fn test_task_from_inbox_returns_to_inbox() {
    let rt = Runtime::new().unwrap();
    let opened = Task {
        id: "t1".to_string(),
        name: "Button colour".to_string(),
        ..Default::default()
    };
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_inbox(inbox_of(vec![inbox_task(
            "t1",
            "Button colour",
            Some(1000),
        )]))
        .with_task(opened)
        .with_task_comments(vec![]);

    let mut app = root_app(&rt, client);
    open_inbox(&rt, &mut app);

    let _guard = rt.enter();
    app.update(enter());
    for _ in 0..2 {
        rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(50)).await });
        app.process_async_messages();
    }
    assert_eq!(app.screen(), Screen::TaskDetail);

    app.update(esc());
    assert_eq!(
        app.screen(),
        Screen::Inbox,
        "Esc must return to the inbox the task came from"
    );
}

/// Esc from the inbox itself returns to the navigation root.
#[test]
fn test_esc_from_inbox_returns_to_root() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_inbox(inbox_of(vec![inbox_task("t1", "One", Some(1000))]));

    let mut app = root_app(&rt, client);
    open_inbox(&rt, &mut app);
    assert_eq!(app.screen(), Screen::Inbox);

    app.update(esc());

    assert_eq!(app.screen(), Screen::Workspaces);
    assert!(
        matches!(
            app.sidebar().selected_item(),
            Some(SidebarItem::Inbox { .. })
        ),
        "the inbox entry stays selected at the root"
    );
}

/// The endpoint being unavailable is reported as such, not as a crash.
#[test]
fn test_inbox_unavailable_is_graceful() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_inbox_error("API error (404): not found");

    let mut app = root_app(&rt, client);
    open_inbox(&rt, &mut app);

    assert_eq!(app.screen(), Screen::Inbox);
    assert_eq!(app.status(), "Inbox unavailable");
    assert_eq!(app.inbox_len_for_test(), 0);
}

/// An empty inbox reads as "inbox zero", not as an error.
#[test]
fn test_empty_inbox_reports_clear() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_inbox(inbox_of(vec![]));

    let mut app = root_app(&rt, client);
    open_inbox(&rt, &mut app);

    assert_eq!(app.status(), "Inbox clear");
    assert_eq!(app.inbox_unread_count(), 0);
}

/// The inbox entry never appears below the navigation root.
#[test]
fn test_inbox_absent_below_root() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_spaces(vec![])
        .with_inbox(inbox_of(vec![]));

    let mut app = root_app(&rt, client);
    let _guard = rt.enter();
    assert!(app.sidebar().select_by_id("inbox"), "present at root");

    app.sidebar().select_by_id("ws-1");
    app.navigate_into();
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(30)).await });
    app.process_async_messages();

    assert_eq!(app.screen(), Screen::Spaces);
    assert!(
        !app.sidebar().select_by_id("inbox"),
        "the inbox entry must not appear below the root"
    );
}

fn space(id: &str, name: &str) -> clickdown::models::ClickUpSpace {
    serde_json::from_value(serde_json::json!({
        "id": id, "name": name, "private": false
    }))
    .expect("space fixture should deserialize")
}

/// Walking into a workspace and back out with Esc must land on the full root
/// sidebar: the pinned Inbox and Assigned entries plus the workspaces.
///
/// Regression: the Spaces -> Workspaces back path rebuilt the sidebar from
/// the workspace list alone, so the two pinned entries silently vanished once
/// the user came back from a workspace.
#[test]
fn test_esc_from_spaces_restores_pinned_root_entries() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Kakitangan")])
        .with_spaces(vec![space("sp-1", "Product")])
        .with_inbox(inbox_of(vec![inbox_task("t1", "One", Some(1000))]));

    let mut app = root_app(&rt, client);

    // Enter the workspace.
    let _guard = rt.enter();
    app.sidebar().select_by_id("ws-1");
    app.navigate_into();
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(60)).await });
    app.process_async_messages();
    assert_eq!(app.screen(), Screen::Spaces);

    // Back to the root.
    app.update(esc());
    assert_eq!(app.screen(), Screen::Workspaces);

    let items = app.sidebar().items().to_vec();
    assert!(
        matches!(items.first(), Some(SidebarItem::Inbox { .. })),
        "Inbox must still be pinned after Esc from Spaces, got: {:?}",
        items
    );
    assert!(
        matches!(items.get(1), Some(SidebarItem::AssignedToMe)),
        "Assigned must still be pinned after Esc from Spaces, got: {:?}",
        items
    );
    assert!(
        items
            .iter()
            .any(|i| matches!(i, SidebarItem::Workspace { id, .. } if id == "ws-1")),
        "the workspace must still be listed, got: {:?}",
        items
    );
}

/// Opening the inbox and pressing Esc twice must return to a root sidebar that
/// still shows both pinned entries, with the inbox entry selected.
#[test]
fn test_double_esc_from_inbox_keeps_pinned_entries() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Kakitangan")])
        .with_inbox(inbox_of(vec![inbox_task("t1", "One", Some(1000))]));

    let mut app = root_app(&rt, client);
    open_inbox(&rt, &mut app);

    app.update(esc());
    assert_eq!(app.screen(), Screen::Workspaces);

    let items = app.sidebar().items().to_vec();
    assert!(matches!(items.first(), Some(SidebarItem::Inbox { .. })));
    assert!(matches!(items.get(1), Some(SidebarItem::AssignedToMe)));
}

/// The dismiss action is only reachable through the SPACEmenu, and only on the
/// inbox screen: a contextual Inbox group, not a bare key.
#[test]
fn test_dismiss_lives_in_contextual_inbox_group() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_inbox(inbox_of(vec![inbox_task("t1", "Only item", Some(3000))]));

    let mut app = root_app(&rt, client);
    open_inbox(&rt, &mut app);
    assert_eq!(app.screen_for_test(), Screen::Inbox);

    // `SPC i` opens the Inbox group (exists only here).
    press(&mut app, vec![key(' '), key('i')]);
    assert_eq!(
        app.command_menu_level_for_test(),
        Some(clickdown::tui::app::CommandMenuLevel::Group('i')),
        "SPC i must enter the contextual Inbox group"
    );

    // `d` alone at the group level is what performs the dismissal.
    press(&mut app, vec![key('d')]);
    assert_eq!(
        app.inbox_len_for_test(),
        0,
        "SPC i d must dismiss the highlighted item"
    );
}

/// A bare `d` on the inbox screen must not dismiss anything — dismissal is
/// deliberately behind the menu so a stray keypress cannot hide an item.
#[test]
fn test_bare_d_does_not_dismiss() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_inbox(inbox_of(vec![inbox_task("t1", "Stays", Some(3000))]));

    let mut app = root_app(&rt, client);
    open_inbox(&rt, &mut app);

    app.update(key('d'));

    assert_eq!(
        app.inbox_len_for_test(),
        1,
        "a bare 'd' must not dismiss; dismiss is SPC i d"
    );
    assert!(!app.command_menu_open_for_test(), "'d' is not a menu key");
}

/// Outside the inbox the contextual group must not exist: pressing `SPC i`
/// closes the menu with feedback rather than offering a dead command.
#[test]
fn test_inbox_group_absent_outside_inbox() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new().with_workspaces(vec![workspace("ws-1", "Engineering")]);
    let mut app = root_app(&rt, client);
    assert_eq!(app.screen_for_test(), Screen::Workspaces);

    press(&mut app, vec![key(' '), key('i')]);

    assert!(
        !app.command_menu_open_for_test(),
        "SPC i must not open a group outside the inbox"
    );
    assert_eq!(
        app.command_menu_level_for_test(),
        None,
        "no group should be entered"
    );
}

/// Dismissing must remain persisted (still filtered after reload) when driven
/// through the menu path rather than the old bare key.
#[test]
fn test_menu_dismiss_persists_across_reload() {
    let rt = Runtime::new().unwrap();
    let client = MockClickUpClient::new()
        .with_workspaces(vec![workspace("ws-1", "Engineering")])
        .with_inbox(inbox_of(vec![
            inbox_task("t1", "One", Some(3000)),
            inbox_task("t2", "Two", Some(2000)),
        ]));

    let mut app = root_app(&rt, client);
    open_inbox(&rt, &mut app);

    press(&mut app, dismiss_chord());
    assert_eq!(app.inbox_len_for_test(), 1);

    let _guard = rt.enter();
    app.update(key('r'));
    rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(60)).await });
    app.process_async_messages();

    assert_eq!(
        app.inbox_len_for_test(),
        1,
        "menu-dismissed entry must stay hidden after reload"
    );
}
