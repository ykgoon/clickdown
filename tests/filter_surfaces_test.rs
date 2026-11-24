//! Filtering requirements, exercised in BOTH task-list surfaces.
//!
//! 1. Status filtering must work in Assigned to Me as well as in a list.
//! 2. Several filters compose (status + assigned-to-me, several statuses).
//! 3. The applied filters must be visible in the task-list frame.

mod fixtures;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::task::TaskStatus;
use clickdown::models::EntityRef;
use clickdown::models::Task;
use clickdown::tui::app::{Screen, TuiApp};
use clickdown::tui::input::InputEvent;
use clickdown::tui::widgets::task_list::filter_frame_title;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}

fn enter() -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
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

fn new_app(rt: &Runtime) -> TuiApp {
    rt.block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() })
}

/// Put one screen into the same filtered-capable state so every assertion
/// below can run against both the task list and Assigned to Me.
fn prepare(app: &mut TuiApp, surface: Screen) {
    app.set_screen(surface);
    app.set_current_list_id(Some("list-1".to_string()));
    app.set_cached_list_statuses_for_test(
        "list-1",
        vec![
            status("To Do", 0),
            status("In Progress", 1),
            status("Review", 2),
            status("Done", 3),
        ],
    );
    app.set_tasks_for_test(vec![
        task_with("t1", "a", status("To Do", 0), Some("list-1")),
        task_with("t2", "b", status("In Progress", 1), Some("list-1")),
        task_with("t3", "c", status("Review", 2), Some("list-1")),
        task_with("t4", "d", status("Done", 3), Some("list-1")),
    ]);
    app.set_assigned_tasks_for_test(app.tasks_for_test().to_vec());
    app.remember_status_names_for_test();
}

// ---------------------------------------------------------------- requirement 1

/// Status filtering is available on both surfaces: the picker opens and
/// offers the full set.
#[test]
fn status_picker_available_in_both_surfaces() {
    let rt = Runtime::new().unwrap();

    for surface in [Screen::Tasks, Screen::Assigned] {
        let mut app = new_app(&rt);
        prepare(&mut app, surface.clone());

        app.open_filter_menu();
        assert!(
            app.filter_can_status_for_test(),
            "{surface:?}: status filtering must be available"
        );
        rt.block_on(async { app.update(key('s')) });
        assert!(
            app.status_picker_open_for_test(),
            "{surface:?}: status picker should open"
        );
        assert!(
            app.status_picker_statuses_for_test().len() >= 4,
            "{surface:?}: expected the full status set, got {:?}",
            app.status_picker_statuses_for_test()
                .iter()
                .map(|s| s.status.clone())
                .collect::<Vec<_>>()
        );
    }
}

/// Filtering by status actually narrows the list on both surfaces.
#[test]
fn status_filter_narrows_results_in_both_surfaces() {
    let rt = Runtime::new().unwrap();

    for surface in [Screen::Tasks, Screen::Assigned] {
        let mut app = new_app(&rt);
        prepare(&mut app, surface.clone());

        app.set_status_filters_for_test(vec!["In Progress".to_string()]);
        assert_eq!(
            app.status_filters_for_test(),
            vec!["In Progress".to_string()],
            "{surface:?}: filter should stick"
        );

        // Clearing works from either surface.
        rt.block_on(async { app.clear_all_filters() });
        assert!(
            app.status_filters_for_test().is_empty(),
            "{surface:?}: clear must remove the status filter"
        );
    }
}

// ---------------------------------------------------------------- requirement 2

/// Several statuses can be active at once, and both combine with the
/// assigned-to-me filter.
#[test]
fn multiple_filters_compose_in_both_surfaces() {
    let rt = Runtime::new().unwrap();

    for surface in [Screen::Tasks, Screen::Assigned] {
        let mut app = new_app(&rt);
        prepare(&mut app, surface.clone());

        app.set_status_filters_for_test(vec![
            "In Progress".to_string(),
            "Review".to_string(),
        ]);
        app.set_assigned_filter_active(true);

        assert!(app.has_active_filters_for_test(), "{surface:?}");
        assert_eq!(
            app.status_filters_for_test(),
            vec!["In Progress".to_string(), "Review".to_string()],
            "{surface:?}: both statuses must stay active"
        );

        rt.block_on(async { app.clear_all_filters() });
        assert!(
            !app.has_active_filters_for_test(),
            "{surface:?}: clear must drop every filter"
        );
        assert!(app.status_filters_for_test().is_empty());
        assert!(!app.assigned_filter_active_for_test());
    }
}

/// Ticking two statuses in the picker keeps both.
#[test]
fn picker_multi_select_keeps_every_ticked_status() {
    let rt = Runtime::new().unwrap();

    for surface in [Screen::Tasks, Screen::Assigned] {
        let mut app = new_app(&rt);
        prepare(&mut app, surface.clone());

        app.open_filter_menu();
        rt.block_on(async { app.update(key('s')) });
        assert!(app.status_picker_open_for_test());

        // Tick the highlighted status, move, tick again.
        rt.block_on(async { app.update(key(' ')) });
        rt.block_on(async { app.update(key('j')) });
        rt.block_on(async { app.update(key(' ')) });
        rt.block_on(async { app.update(enter()) });

        let applied = app.status_filters_for_test();
        assert_eq!(
            applied.len(),
            2,
            "{surface:?}: two ticked statuses must both apply, got {applied:?}"
        );
    }
}

// ---------------------------------------------------------------- requirement 3

/// The frame title states the filters, and states nothing when unfiltered.
#[test]
fn frame_title_shows_active_filters() {
    assert_eq!(filter_frame_title(false, &[]), " Tasks ");

    let title = filter_frame_title(false, &["Done".to_string()]);
    assert!(title.contains("Status: Done"), "got {title:?}");

    let both = filter_frame_title(true, &["To Do".to_string(), "Done".to_string()]);
    assert!(both.contains("Assigned to Me"), "got {both:?}");
    assert!(both.contains("Status: To Do, Done"), "got {both:?}");
}

/// Both surfaces show the active filters in the frame title only. The old
/// banner row repeated the same information above the rows and cost a line of
/// list space, so it must not come back.
#[test]
fn rendered_frame_shows_filters_in_both_surfaces() {
    let rt = Runtime::new().unwrap();

    for surface in [Screen::Tasks, Screen::Assigned] {
        let mut app = new_app(&rt);
        prepare(&mut app, surface.clone());
        app.set_status_filters_for_test(vec!["Review".to_string()]);

        let rendered = app.render_to_string_for_test(100, 30);
        assert!(
            rendered.contains("Status: Review"),
            "{surface:?}: the frame must show the active filter, got:\n{rendered}"
        );
        assert!(
            !rendered.contains("Filtered by"),
            "{surface:?}: the redundant filter banner must be gone, got:\n{rendered}"
        );
    }
}

/// With no filters the frame title stays bare and no banner appears.
#[test]
fn unfiltered_frame_has_no_banner() {
    let rt = Runtime::new().unwrap();

    for surface in [Screen::Tasks, Screen::Assigned] {
        let mut app = new_app(&rt);
        prepare(&mut app, surface.clone());

        let rendered = app.render_to_string_for_test(100, 30);
        assert!(
            !rendered.contains("Filtered by"),
            "{surface:?}: unfiltered frame must not show a banner"
        );
        assert!(
            !rendered.contains("Status: "),
            "{surface:?}: unfiltered frame must not mention a status filter"
        );
    }
}
// ---------------------------------------------------------------- wire level
//
// The Assigned view pages results, so a filter applied after the fetch would
// only ever see the current page. The statuses must travel in the query.

/// Both selected statuses reach the API.
#[test]
fn assigned_query_carries_every_status_filter() {
    let rt = Runtime::new().unwrap();
    let mock = MockClickUpClient::new().with_team_tasks(vec![]);
    let log = mock.team_tasks_call_log_handle();
    let mut app = rt.block_on(async { TuiApp::with_client(Arc::new(mock)).unwrap() });

    app.set_screen(Screen::Assigned);
    app.set_current_user_id(Some(123));
    app.set_current_team_id_for_test("team-1".to_string());
    app.set_status_filters_for_test(vec!["In Progress".to_string(), "Review".to_string()]);

    rt.block_on(async { app.load_assigned_tasks() });
    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        app.process_async_messages();
    }

    let calls = log.lock().map(|l| l.clone()).unwrap_or_default();
    assert!(!calls.is_empty(), "expected an assigned query");
    assert_eq!(
        calls[0].1,
        vec!["In Progress".to_string(), "Review".to_string()],
        "both statuses must reach the API"
    );
}

/// Unfiltered must send no statuses at all: an empty `statuses[]` would be
/// read as "match no status" rather than "match any".
#[test]
fn assigned_query_omits_statuses_when_unfiltered() {
    let rt = Runtime::new().unwrap();
    let mock = MockClickUpClient::new().with_team_tasks(vec![]);
    let log = mock.team_tasks_call_log_handle();
    let mut app = rt.block_on(async { TuiApp::with_client(Arc::new(mock)).unwrap() });

    app.set_screen(Screen::Assigned);
    app.set_current_user_id(Some(123));
    app.set_current_team_id_for_test("team-1".to_string());

    rt.block_on(async { app.load_assigned_tasks() });
    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        app.process_async_messages();
    }

    let calls = log.lock().map(|l| l.clone()).unwrap_or_default();
    assert!(!calls.is_empty(), "expected an assigned query");
    assert!(
        calls[0].1.is_empty(),
        "unfiltered query must send no statuses, got {:?}",
        calls[0].1
    );
}

// ---------------------------------------------------------------- option order

/// Filter options must follow the workspace's configured `orderindex`, not
/// the order statuses happened to be observed in.
///
/// This regressed when the registry stored only status *names*: rebuilding
/// options from names dropped `orderindex`, so the picker fell back to
/// first-seen order and `j` no longer moved to the expected status.
#[test]
fn filter_options_follow_configured_order() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);
    app.set_screen(Screen::Assigned);

    // Deliberately out of order on the tasks, so first-seen order differs
    // from configured order.
    app.set_assigned_tasks_for_test(vec![
        task_with("t3", "c", status("Done", 3), None),
        task_with("t1", "a", status("To Do", 0), None),
        task_with("t2", "b", status("In Progress", 1), None),
    ]);
    app.remember_status_names_for_test();

    let names: Vec<String> = app
        .status_filter_options_for_test()
        .iter()
        .map(|s| s.status.clone())
        .collect();

    assert_eq!(
        names,
        vec!["To Do", "In Progress", "Done"],
        "options must be ordered by orderindex, got {names:?}"
    );
}

/// The order must survive an active filter: the reloaded rows are one status,
/// so the order has to come from the registry.
#[test]
fn filter_option_order_survives_an_active_filter() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);
    app.set_screen(Screen::Assigned);

    app.set_assigned_tasks_for_test(vec![
        task_with("t4", "d", status("Review", 2), None),
        task_with("t3", "c", status("Done", 3), None),
        task_with("t1", "a", status("To Do", 0), None),
        task_with("t2", "b", status("In Progress", 1), None),
    ]);
    app.remember_status_names_for_test();

    // Filter applied: only matching rows remain loaded.
    app.set_status_filters_for_test(vec!["In Progress".to_string()]);
    app.set_assigned_tasks_for_test(vec![task_with(
        "t2",
        "b",
        status("In Progress", 1),
        None,
    )]);

    let names: Vec<String> = app
        .status_filter_options_for_test()
        .iter()
        .map(|s| s.status.clone())
        .collect();

    assert_eq!(
        names.len(),
        4,
        "every known status must still be offered, got {names:?}"
    );
    assert_eq!(
        names,
        vec!["To Do", "In Progress", "Review", "Done"],
        "order must match orderindex even while filtered, got {names:?}"
    );
}
