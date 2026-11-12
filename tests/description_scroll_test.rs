//! Focus + scroll behaviour for the task description panel.
//!
//! The description pane can be longer than its box, so it must be scrollable
//! with the usual vim keys whenever it holds focus — not only while the panel
//! is expanded full-screen.

mod fixtures;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::{Task, TaskDescription};
use clickdown::tui::app::{ExpandedPanel, Screen, TuiApp};
use clickdown::tui::input::InputEvent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}

fn special(code: KeyCode) -> InputEvent {
    InputEvent::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

/// A task whose description overflows the description pane.
fn long_description_task() -> Task {
    let mut task = fixtures::test_task();
    task.description = Some(TaskDescription::Plain(
        (1..=60)
            .map(|i| format!("line {} of a long description", i))
            .collect::<Vec<_>>()
            .join("\n"),
    ));
    task
}

fn app_on_task_detail(task: Task) -> (Runtime, TuiApp) {
    let rt = Runtime::new().unwrap();
    let mut app =
        rt.block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });
    app.set_screen(Screen::TaskDetail);
    app.task_detail().task = Some(task);
    (rt, app)
}

/// j/k must scroll the description while it holds focus in the split layout.
/// They were dead keys unless the panel had been expanded with `SPC t v`.
#[test]
fn test_j_k_scroll_description_when_focused() {
    let (rt, mut app) = app_on_task_detail(long_description_task());
    app.set_comment_focus(false);

    let _guard = rt.enter();
    assert_eq!(app.description_scroll_offset_for_test(), 0);

    app.update(key('j'));
    app.update(key('j'));
    assert_eq!(
        app.description_scroll_offset_for_test(),
        2,
        "j should scroll the focused description down one line at a time"
    );

    app.update(key('k'));
    assert_eq!(
        app.description_scroll_offset_for_test(),
        1,
        "k should scroll the focused description back up"
    );
}

/// Up/Down, PageUp/PageDown and Ctrl+F/Ctrl+U share the same scroll path.
#[test]
fn test_arrow_and_page_keys_scroll_focused_description() {
    let (rt, mut app) = app_on_task_detail(long_description_task());
    app.set_comment_focus(false);

    let _guard = rt.enter();
    app.update(special(KeyCode::Down));
    assert_eq!(app.description_scroll_offset_for_test(), 1);

    app.update(special(KeyCode::PageDown));
    assert_eq!(
        app.description_scroll_offset_for_test(),
        21,
        "PageDown should move a page"
    );

    app.update(special(KeyCode::Up));
    assert_eq!(app.description_scroll_offset_for_test(), 20);

    app.update(special(KeyCode::PageUp));
    assert_eq!(app.description_scroll_offset_for_test(), 0);
}

/// j/k stay with the comment list when a comment holds focus, so navigating
/// comments never disturbs the description scroll position.
#[test]
fn test_j_k_do_not_scroll_description_when_comment_focused() {
    let (rt, mut app) = app_on_task_detail(long_description_task());
    app.set_comments(vec![fixtures::test_comment()]);
    app.set_comment_focus(true);

    let _guard = rt.enter();
    app.update(key('j'));
    assert_eq!(
        app.description_scroll_offset_for_test(),
        0,
        "j on a focused comment must not scroll the description"
    );
}

/// The expanded panel still scrolls, now one line per keypress like the split
/// layout (it previously moved three lines at a time while Up/Down moved one).
#[test]
fn test_expanded_description_scrolls_with_j_k() {
    let (rt, mut app) = app_on_task_detail(long_description_task());

    let _guard = rt.enter();
    for c in [' ', 't', 'v'] {
        app.update(key(c)); // SPC t v
    }
    assert_eq!(
        app.expanded_panel_for_test(),
        Some(ExpandedPanel::Description)
    );

    app.update(key('j'));
    app.update(key('j'));
    assert_eq!(app.description_scroll_offset_for_test(), 2);
    app.update(key('k'));
    assert_eq!(app.description_scroll_offset_for_test(), 1);
}

/// The default render entry point scrolls from the top, so the pane is never
/// drawn at a stale offset.
#[test]
fn test_opening_task_resets_description_scroll() {
    let rt = Runtime::new().unwrap();
    let mut app =
        rt.block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });
    app.set_screen(Screen::Tasks);
    app.tasks_mut_for_test().push(long_description_task());
    app.rebuild_task_list_for_test();
    app.task_list_mut_for_test().select_first();
    app.set_description_scroll_offset_for_test(40);

    let _guard = rt.enter();
    app.update(special(KeyCode::Enter));

    assert_eq!(app.screen(), Screen::TaskDetail);
    assert_eq!(
        app.description_scroll_offset_for_test(),
        0,
        "description scroll should reset when a task is opened"
    );
}

/// Scrolling far past the end must not bank invisible overscroll: the offset
/// is clamped to the real content height on the next render.
#[test]
fn test_description_scroll_is_clamped_to_content() {
    use clickdown::tui::widgets::task_detail::{render_task_detail_expanded, TaskDetailState};
    use ratatui::{backend::TestBackend, layout::Rect, Terminal};

    let mut detail = TaskDetailState::new();
    detail.task = Some(long_description_task());

    let mut terminal = Terminal::new(TestBackend::new(60, 18)).unwrap();
    let mut scroll = 10_000usize;
    terminal
        .draw(|frame| {
            render_task_detail_expanded(
                frame,
                &detail,
                Rect::new(0, 0, 60, 18),
                "",
                "",
                &clickdown::tui::app::TaskCreationField::Name,
                false,
                true,
                &mut scroll,
            );
        })
        .unwrap();

    assert!(
        scroll < 10_000,
        "render must clamp an out-of-range scroll offset to the content height"
    );
    assert!(scroll > 0, "a long description must still be scrollable");
}
