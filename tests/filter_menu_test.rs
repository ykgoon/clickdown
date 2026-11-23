//! Tests for the redesigned task-list filter menu (`SPC t f`).
//!
//! Reported before the redesign: after filtering by status, pressing the same
//! key again offered a one-status list, so the filter could not be changed or
//! removed. Root cause: the menu derived its options from the loaded rows,
//! which *are* the filtered result.

mod fixtures;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::task::{ListReference, TaskStatus};
use clickdown::models::Task;
use clickdown::tui::app::{Screen, StatusPickerMode, TuiApp};
use clickdown::tui::input::InputEvent;
use clickdown::tui::widgets::FilterMenuEntry;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}

fn enter() -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
}

fn esc() -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
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
    task.list = list_id.map(|id| ListReference {
        id: id.to_string(),
        name: Some("L".to_string()),
        access: None,
    });
    task
}

fn new_app(rt: &Runtime) -> TuiApp {
    rt.block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() })
}

/// `SPC t f` opens the menu, not the assigned-filter toggle.
#[test]
fn spc_t_f_opens_the_filter_menu() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);
    app.set_screen(Screen::Assigned);

    for k in [' ', 't', 'f'] {
        app.update(key(k));
    }

    assert!(app.filter_menu_open_for_test(), "SPC t f must open the menu");
    assert!(
        !app.assigned_filter_active_for_test(),
        "opening the menu must not toggle a filter by itself"
    );
}

/// The menu lists the available criteria plus a clear-all entry.
#[test]
fn filter_menu_lists_all_entries_including_clear() {
    let entries = FilterMenuEntry::ALL;
    let keys: Vec<char> = entries.iter().map(|e| e.key()).collect();
    assert_eq!(keys, vec!['f', 's', 'x'], "unexpected menu keys {keys:?}");

    let labels: Vec<&str> = entries.iter().map(|e| e.label()).collect();
    assert!(labels.iter().any(|l| l.contains("Assigned")));
    assert!(labels.iter().any(|l| l.contains("status")));
    assert!(
        labels.iter().any(|l| l.to_lowercase().contains("clear")),
        "there must be a way to remove all filters, got {labels:?}"
    );
}

/// The reported bug: after applying a status filter, the status list must
/// still offer every status, not only the one currently shown.
#[test]
fn status_options_survive_an_active_filter() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);
    app.set_screen(Screen::Tasks);
    app.set_current_list_id(Some("list-1".to_string()));

    // First load: several statuses, which seeds the registry.
    app.set_tasks_for_test(vec![
        task_with("t1", "a", status("To Do", 0), Some("list-1")),
        task_with("t2", "b", status("In Progress", 1), Some("list-1")),
        task_with("t3", "c", status("Done", 2), Some("list-1")),
    ]);
    app.remember_status_names_for_test();

    // Now the user filters to one status: the visible rows are only that one.
    app.set_status_filter_for_test(Some("In Progress".to_string()));
    app.set_tasks_for_test(vec![task_with(
        "t2",
        "b",
        status("In Progress", 1),
        Some("list-1"),
    )]);

    let options: Vec<String> = app
        .status_filter_options_for_test()
        .iter()
        .map(|s| s.status.clone())
        .collect();

    assert!(
        options.len() >= 3,
        "filtered view must still offer every known status, got {options:?}"
    );
    for expected in ["To Do", "In Progress", "Done"] {
        assert!(
            options.iter().any(|o| o == expected),
            "missing {expected} in {options:?}"
        );
    }
}

/// Selecting "Clear all filters" removes both filters and is the documented
/// way out; it reports when nothing was active.
#[test]
fn clear_all_filters_removes_every_filter() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);
    app.set_screen(Screen::Tasks);
    app.set_current_list_id(Some("list-1".to_string()));
    app.set_status_filter_for_test(Some("Done".to_string()));
    app.set_assigned_filter_active(true);

    assert!(app.has_active_filters_for_test());

    // Reloading spawns a request, so the call needs a runtime context.
    rt.block_on(async { app.clear_all_filters() });

    assert!(!app.has_active_filters_for_test(), "filters must be gone");
    assert_eq!(app.status_filter_for_test(), None);
    assert!(!app.assigned_filter_active_for_test());
    assert!(app.status_for_test().to_lowercase().contains("cleared"));
}

/// Clearing when nothing is active is not an error and says so.
#[test]
fn clear_all_filters_is_harmless_when_nothing_active() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);
    app.set_screen(Screen::Tasks);

    rt.block_on(async { app.clear_all_filters() });

    assert!(app.status_for_test().to_lowercase().contains("no filters"));
}

/// Esc in the status picker must not silently drop an active filter: only the
/// explicit clear action changes the result set.
#[test]
fn esc_in_status_filter_picker_keeps_the_active_filter() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);
    app.set_screen(Screen::Tasks);
    app.set_current_list_id(Some("list-1".to_string()));
    app.set_status_filter_for_test(Some("In Progress".to_string()));
    app.set_tasks_for_test(vec![]);

    rt.block_on(async { app.open_status_filter_picker() });
    assert_eq!(
        app.status_picker_mode_for_test(),
        StatusPickerMode::FilterList
    );

    rt.block_on(async { app.update(esc()) });

    assert!(!app.status_picker_open_for_test(), "picker should close");
    assert_eq!(
        app.status_filter_for_test().as_deref(),
        Some("In Progress"),
        "Esc must not clear the filter"
    );
}

/// The menu's direct letters activate the matching entry.
#[test]
fn menu_letters_activate_entries() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);
    app.set_screen(Screen::Tasks);
    app.set_current_list_id(Some("list-1".to_string()));

    app.open_filter_menu();
    rt.block_on(async { app.update(key('f')) }); // assigned-to-me
    assert!(app.assigned_filter_active_for_test());
    assert!(!app.filter_menu_open_for_test(), "menu should close");

    // Re-open and clear everything with 'x'.
    app.open_filter_menu();
    rt.block_on(async { app.update(key('x')) });
    assert!(!app.assigned_filter_active_for_test());
    assert!(!app.has_active_filters_for_test());
}

/// j/k move the cursor and Enter applies the highlighted entry.
#[test]
fn menu_navigation_and_enter() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);
    app.set_screen(Screen::Tasks);
    app.set_current_list_id(Some("list-1".to_string()));

    app.open_filter_menu();
    assert_eq!(app.filter_menu_cursor_for_test(), 0);

    app.update(key('j'));
    assert_eq!(app.filter_menu_cursor_for_test(), 1, "j moves down");

    app.update(key('k'));
    assert_eq!(app.filter_menu_cursor_for_test(), 0, "k moves up");

    // Enter on entry 0 (assigned to me).
    rt.block_on(async { app.update(enter()) });
    assert!(app.assigned_filter_active_for_test());
}

/// Esc closes the menu without touching filters.
#[test]
fn esc_closes_menu_without_changing_filters() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);
    app.set_screen(Screen::Tasks);
    app.set_status_filter_for_test(Some("Done".to_string()));

    app.open_filter_menu();
    app.update(esc());

    assert!(!app.filter_menu_open_for_test());
    assert_eq!(app.status_filter_for_test().as_deref(), Some("Done"));
}

/// The Assigned view has no list context, so status filtering is not offered
/// as available there, but clear-all still works.
#[test]
fn assigned_view_disables_status_option_but_allows_clearing() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);
    app.set_screen(Screen::Assigned);
    app.set_status_filter_for_test(Some("In Progress".to_string()));

    app.open_filter_menu();
    assert!(
        !app.filter_can_status_for_test(),
        "status filter is unavailable without a list"
    );

    // 's' explains instead of opening a picker.
    app.update(key('s'));
    assert!(!app.status_picker_open_for_test());
    assert!(app.status_for_test().to_lowercase().contains("list"));

    // 'x' still clears the stale filter.
    rt.block_on(async { app.update(key('x')) });
    assert_eq!(app.status_filter_for_test(), None);
}

/// The user-visible form of the reported bug: open the status picker while a
/// filter is active and theme the seeded menu, not the internal option list.
#[test]
fn status_picker_after_filtering_offers_more_than_one_status() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);
    app.set_screen(Screen::Tasks);
    app.set_current_list_id(Some("list-1".to_string()));

    app.set_tasks_for_test(vec![
        task_with("t1", "a", status("To Do", 0), Some("list-1")),
        task_with("t2", "b", status("In Progress", 1), Some("list-1")),
        task_with("t3", "c", status("Done", 2), Some("list-1")),
    ]);
    app.remember_status_names_for_test();

    // Filter applied, and (as in the real app) the loaded rows are only the
    // matching status.
    app.set_status_filter_for_test(Some("In Progress".to_string()));
    app.set_tasks_for_test(vec![task_with(
        "t2",
        "b",
        status("In Progress", 1),
        Some("list-1"),
    )]);

    rt.block_on(async { app.open_status_filter_picker() });

    let offered: Vec<String> = app
        .status_picker_statuses_for_test()
        .iter()
        .map(|s| s.status.clone())
        .collect();
    assert!(
        offered.len() > 1,
        "picker must not collapse to the filtered status, got {offered:?}"
    );
}

// ---------------------------------------------------------------- user flow

/// The exact reported scenario, driven by real keystrokes:
/// filter by status, then open the menu again and remove the filter.
#[test]
fn user_can_filter_then_unfilter_without_leaving_the_screen() {
    let rt = Runtime::new().unwrap();
    let mut app = new_app(&rt);
    app.set_screen(Screen::Tasks);
    app.set_current_list_id(Some("list-1".to_string()));

    app.set_tasks_for_test(vec![
        task_with("t1", "a", status("To Do", 0), Some("list-1")),
        task_with("t2", "b", status("In Progress", 1), Some("list-1")),
        task_with("t3", "c", status("Done", 2), Some("list-1")),
    ]);
    app.remember_status_names_for_test();

    // 1. `SPC t f` opens the menu.
    for k in [' ', 't', 'f'] {
        rt.block_on(async { app.update(key(k)) });
    }
    assert!(app.filter_menu_open_for_test());

    // 2. `s` opens the status picker, which offers every status.
    rt.block_on(async { app.update(key('s')) });
    assert!(app.status_picker_open_for_test());
    let offered: Vec<String> = app
        .status_picker_statuses_for_test()
        .iter()
        .map(|s| s.status.clone())
        .collect();
    assert!(
        offered.len() >= 3,
        "status picker must offer every known status, got {offered:?}"
    );

    // 3. Pick "In Progress": the filter applies.
    let idx = app
        .status_picker_statuses_for_test()
        .iter()
        .position(|s| s.status == "In Progress")
        .expect("In Progress offered");
    app.set_status_picker_cursor_for_test(idx);
    rt.block_on(async { app.update(enter()) });
    assert_eq!(
        app.status_filter_for_test().as_deref(),
        Some("In Progress"),
        "filter should be active"
    );

    // 4. `SPC t f` again: the menu still lists every status, so the filter
    //    can be changed (the reported bug was a one-status list here).
    for k in [' ', 't', 'f'] {
        rt.block_on(async { app.update(key(k)) });
    }
    assert!(app.filter_menu_open_for_test());
    rt.block_on(async { app.update(key('s')) });
    let reopened: Vec<String> = app
        .status_picker_statuses_for_test()
        .iter()
        .map(|s| s.status.clone())
        .collect();
    assert!(
        reopened.len() >= 3,
        "reopening the picker while filtered must not collapse to one status, got {reopened:?}"
    );
    rt.block_on(async { app.update(esc()) });

    // 5. `SPC t f` then `x` removes the filter.
    for k in [' ', 't', 'f', 'x'] {
        rt.block_on(async { app.update(key(k)) });
    }
    assert_eq!(app.status_filter_for_test(), None, "filter must be gone");
    assert!(!app.has_active_filters_for_test());
    assert!(!app.filter_menu_open_for_test());
}
