//! Tests for `SPC r` — refresh the data shown on screen.
//!
//! The app keeps in-memory copies of everything it fetched. Refresh re-runs
//! the request that produced the current screen, so the assertions below seed
//! the app with *stale* data and configure the mock client with *fresh* data:
//! after `SPC r` plus message processing, the screen must show the new values.

mod fixtures;

use std::sync::Arc;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::Task;
use clickdown::tui::app::{Screen, TuiApp};
use clickdown::tui::input::InputEvent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::runtime::Runtime;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}

fn make_task(id: &str, name: &str) -> Task {
    Task {
        id: id.to_string(),
        name: name.to_string(),
        ..Default::default()
    }
}

/// Drive `SPC r` and let the spawned request land in the channel.
fn press_refresh(rt: &Runtime, app: &mut TuiApp) {
    let _guard = rt.enter(); // tokio::spawn needs a reactor
    for c in [' ', 'r'] {
        app.update(key(c));
    }
    // Two rounds: the task refresh can fan out into a follow-up request
    // (task detail also reloads comments).
    for _ in 0..2 {
        rt.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(100)).await });
        app.process_async_messages();
    }
}

/// `SPC` then `r` must not walk into a command group: refresh is a root-level
/// one-key command, and it must close the menu.
#[test]
fn test_spc_r_closes_menu_and_refreshes() {
    let rt = Runtime::new().unwrap();
    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(MockClickUpClient::new().with_tasks(vec![]))).unwrap()
    });

    let _guard = rt.enter(); // tokio::spawn needs a reactor
    app.update(key(' '));
    assert!(app.command_menu_open_for_test(), "SPACE opens the menu");

    app.update(key('r'));
    assert!(
        !app.command_menu_open_for_test(),
        "SPC r closes the menu after running"
    );
}

/// Task list: a renamed task must come back from the API and replace the
/// stale in-memory copy, keeping the selection on the same task.
#[test]
fn test_refresh_task_list_picks_up_renamed_task() {
    let rt = Runtime::new().unwrap();

    let stale = make_task("task-1", "Old Name");
    let renamed = make_task("task-1", "New Name From ClickUp");
    let second = make_task("task-2", "Second");

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(
            MockClickUpClient::new().with_tasks(vec![renamed.clone(), second.clone()]),
        ))
        .unwrap()
    });

    app.set_screen(Screen::Tasks);
    app.set_current_list_id(Some("list-1".to_string()));
    app.tasks_mut_for_test().push(stale.clone());
    app.tasks_mut_for_test().push(second.clone());
    app.rebuild_task_list_for_test();
    app.task_list_mut_for_test().select_first();

    assert_eq!(
        app.task_list_for_test().selected_task().map(|t| t.name.clone()),
        Some("Old Name".to_string())
    );

    press_refresh(&rt, &mut app);

    let names: Vec<String> = app
        .task_list_for_test()
        .rows()
        .iter()
        .filter_map(|row| match row {
            clickdown::tui::widgets::ListRow::Task(task) => Some(task.name.clone()),
            _ => None,
        })
        .collect();

    assert!(
        names.contains(&"New Name From ClickUp".to_string()),
        "refresh must replace the stale task name, got: {:?}",
        names
    );
    assert!(
        !names.contains(&"Old Name".to_string()),
        "stale name must be gone, got: {:?}",
        names
    );
    assert_eq!(
        app.task_list_for_test().selected_task().map(|t| t.id.clone()),
        Some("task-1".to_string()),
        "refresh must keep the user on the same task"
    );
}

/// Task detail: refresh re-fetches the task and then the comments, so a new
/// comment written elsewhere shows up without leaving the screen.
#[test]
fn test_refresh_task_detail_reloads_task_and_comments() {
    let rt = Runtime::new().unwrap();

    let stale = make_task("task-1", "Old Name");
    let fresh = make_task("task-1", "Renamed Task");
    let new_comment: clickdown::models::Comment = serde_json::from_value(serde_json::json!({
        "id": "comment-new",
        "comment_text": "Written on another machine",
    }))
    .expect("comment fixture should deserialize");

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(
            MockClickUpClient::new()
                .with_task(fresh.clone())
                .with_task_comments(vec![new_comment.clone()]),
        ))
        .unwrap()
    });

    app.set_screen(Screen::TaskDetail);
    app.task_detail().task = Some(stale.clone());
    app.set_current_list_id(Some("list-1".to_string()));

    assert!(app.comments_for_test().is_empty(), "no comments yet");

    press_refresh(&rt, &mut app);

    assert_eq!(
        app.task_detail().task.as_ref().map(|t| t.name.clone()),
        Some("Renamed Task".to_string()),
        "task detail must show the re-fetched task"
    );
    let ids: Vec<&str> = app
        .comments_for_test()
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    assert!(
        ids.contains(&"comment-new"),
        "refresh must pull the new comment, got: {:?}",
        ids
    );
}

/// Refresh must fail soft: an API error keeps the old data on screen and
/// reports it in the status line instead of blanking the view.
#[test]
fn test_refresh_failure_keeps_existing_data() {
    let rt = Runtime::new().unwrap();

    let mut client = MockClickUpClient::new();
    client.task_response = Some(Err(anyhow::anyhow!("network down")));

    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(client)).unwrap() });
    app.set_screen(Screen::TaskDetail);
    app.task_detail().task = Some(make_task("task-1", "Still Here"));

    press_refresh(&rt, &mut app);

    assert_eq!(
        app.task_detail().task.as_ref().map(|t| t.name.clone()),
        Some("Still Here".to_string()),
        "a failed refresh must not clear the task being viewed"
    );
    assert!(
        app.status().contains("Refresh failed"),
        "status must report the failure, got: {}",
        app.status()
    );
}
