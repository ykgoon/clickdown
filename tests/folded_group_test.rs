//! Folded status groups must stay reachable and unfoldable from the keyboard.
//!
//! Report: once a status group is folded, there is no way to focus it again to
//! unfold it. j/k only ever landed on task rows, and a folded group has none, so
//! its header became unreachable and the group was stuck folded forever.

mod fixtures;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::task::TaskStatus;
use clickdown::models::Task;
use clickdown::tui::app::{Screen, TuiApp};
use clickdown::tui::input::InputEvent;
use clickdown::tui::widgets::ListRow;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn key(code: KeyCode) -> InputEvent {
    InputEvent::Key(KeyEvent::new(code, KeyModifiers::NONE))
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

fn task(id: &str, name: &str, st: &str, order: u32, updated_at: i64) -> Task {
    let mut task = fixtures::test_task();
    task.id = id.to_string();
    task.name = name.to_string();
    task.status = Some(status(st, order));
    task.updated_at = Some(updated_at);
    task
}

fn app_with(rt: &Runtime, tasks: Vec<Task>) -> TuiApp {
    let mut app =
        rt.block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });
    app.set_screen(Screen::Tasks);
    app.set_current_list_id(Some("list-1".to_string()));
    app.set_cached_list_statuses_for_test("list-1", vec![status("Review", 2), status("Done", 3)]);
    app.set_assigned_tasks_for_test(tasks);
    app
}

/// Newest task is in "Done", so Done heads the list; "Review" follows it.
fn two_groups() -> Vec<Task> {
    vec![
        task("t1", "Done thing", "Done", 3, 3000),
        task("t2", "Review thing", "Review", 2, 2000),
    ]
}

fn row_map(app: &mut TuiApp) -> Vec<String> {
    app.task_list()
        .rows()
        .iter()
        .map(|r| match r {
            ListRow::Header { group_name, .. } => format!("H:{group_name}"),
            ListRow::Task(t) => format!("T:{}", t.id),
        })
        .collect()
}

fn selected_row(app: &mut TuiApp) -> Option<String> {
    let list = app.task_list();
    let idx = list.state().selected()?;
    match list.rows().get(idx)? {
        ListRow::Header { group_name, .. } => Some(format!("H:{group_name}")),
        ListRow::Task(t) => Some(format!("T:{}", t.id)),
    }
}

/// The reported dead end: fold a group, walk away, and it could never be
/// focused again, so it could never be unfolded.
#[test]
fn folded_header_can_be_refocused_and_unfolded() {
    let rt = Runtime::new().unwrap();
    let mut app = app_with(&rt, two_groups());

    // Fold "Done": Tab focuses its header, Tab folds it.
    app.update(key(KeyCode::Tab));
    app.update(key(KeyCode::Tab));
    assert!(
        app.task_list().folded_groups().contains("Done"),
        "precondition: Done folded, folded: {:?}",
        app.task_list().folded_groups()
    );
    assert_eq!(row_map(&mut app), vec!["H:Done", "H:Review", "T:t2"]);
    assert_eq!(selected_row(&mut app).as_deref(), Some("H:Done"));

    // Walk to the other group's task — the realistic thing to do next.
    app.update(key(KeyCode::Char('j')));
    assert_eq!(
        app.task_list().selected_task().map(|t| t.id.as_str()),
        Some("t2"),
        "j lands on the Review task"
    );

    // Now try to get back to the folded Done header with k. Before the fix this
    // skipped the header (it is not a task row) and wrapped to t2 again.
    app.update(key(KeyCode::Char('k')));
    assert_eq!(
        selected_row(&mut app).as_deref(),
        Some("H:Done"),
        "the folded header must be reachable again with k"
    );

    // And Tab must unfold it.
    app.update(key(KeyCode::Tab));
    assert!(
        !app.task_list().folded_groups().contains("Done"),
        "Tab on the folded header must unfold it, folded: {:?}",
        app.task_list().folded_groups()
    );
    assert_eq!(
        row_map(&mut app),
        vec!["H:Done", "T:t1", "H:Review", "T:t2"]
    );
}

/// With every group folded there are no task rows at all. Navigation must still
/// land on the headers, otherwise the whole list is frozen.
#[test]
fn fully_folded_list_still_navigates_between_headers() {
    let rt = Runtime::new().unwrap();
    let mut app = app_with(&rt, two_groups());

    // Fold Done (on a Done task: Tab focuses, Tab folds).
    app.update(key(KeyCode::Tab));
    app.update(key(KeyCode::Tab));
    assert_eq!(selected_row(&mut app).as_deref(), Some("H:Done"));

    // Fold Review (j to its task, then Tab twice).
    app.update(key(KeyCode::Char('j')));
    app.update(key(KeyCode::Tab));
    app.update(key(KeyCode::Tab));
    assert_eq!(
        app.task_list().folded_groups().len(),
        2,
        "both groups folded, folded: {:?}",
        app.task_list().folded_groups()
    );
    assert_eq!(row_map(&mut app), vec!["H:Done", "H:Review"]);

    // The list has no task rows left; the headers are the whole list and must
    // stay selectable and navigable in both directions.
    let here = selected_row(&mut app).expect("a header must stay selected");
    app.update(key(KeyCode::Char('j')));
    let next = selected_row(&mut app).expect("j must land on the other header");
    assert_ne!(here, next, "j must move between folded headers");
    app.update(key(KeyCode::Char('k')));
    assert_eq!(
        selected_row(&mut app).as_deref(),
        Some(here.as_str()),
        "k must come back to the first folded header"
    );

    // Tab on a focused header unfolds it.
    app.update(key(KeyCode::Tab));
    assert_eq!(
        app.task_list().folded_groups().len(),
        1,
        "Tab must unfold the focused group, folded: {:?}",
        app.task_list().folded_groups()
    );
}

/// Page navigation must not skip a folded header either: it is the only row in
/// its group, so paging past it would strand the group folded too.
#[test]
fn page_navigation_lands_on_folded_headers() {
    let rt = Runtime::new().unwrap();
    let mut app = app_with(&rt, two_groups());
    // A small page keeps the whole list within one hop.
    app.task_list().set_viewport_height(1);

    app.update(key(KeyCode::Tab));
    app.update(key(KeyCode::Tab));
    assert_eq!(selected_row(&mut app).as_deref(), Some("H:Done"));

    app.update(key(KeyCode::PageDown));
    assert!(
        selected_row(&mut app).is_some(),
        "PageDown must land on a real row, not nowhere"
    );

    // From anywhere in the list, paging up repeatedly must reach the folded
    // header rather than stopping short of it.
    for _ in 0..5 {
        app.update(key(KeyCode::PageUp));
    }
    assert_eq!(
        selected_row(&mut app).as_deref(),
        Some("H:Done"),
        "PageUp must be able to reach the folded header"
    );
}

/// Unfolded headers stay transparent to j/k: walking the list lands on tasks,
/// never on a header that still has its tasks underneath.
#[test]
fn unfolded_headers_are_not_navigation_stops() {
    let rt = Runtime::new().unwrap();
    let mut app = app_with(&rt, two_groups());

    assert_eq!(row_map(&mut app), vec!["H:Done", "T:t1", "H:Review", "T:t2"]);

    // From t1, j must reach t2 directly and skip the Review header.
    app.update(key(KeyCode::Char('j')));
    assert_eq!(
        selected_row(&mut app).as_deref(),
        Some("T:t2"),
        "unfolded headers must not become stops"
    );

    // Wrap-around keeps working.
    app.update(key(KeyCode::Char('j')));
    assert_eq!(
        selected_row(&mut app).as_deref(),
        Some("T:t1"),
        "j must wrap back to the first task"
    );
}
