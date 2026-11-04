//! End-to-end tests for URL-driven navigation (g u → paste URL → Enter → task screen).//!
//! These tests verify the full chain the user experiences when pasting a
//! ClickUp task URL captured from the TUI (or from elsewhere) into the URL
//! dialog: parse → API lookup (with custom-ID fallback) → TaskDetail screen
//! showing the correct task.
//!
//! The canonical team/task IDs mirror a real production URL:
//! `https://app.clickup.com/t/26408409/14zrc6q9vr2`

mod fixtures;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::Comment;
use clickdown::tui::app::{Screen, TuiApp};
use clickdown::tui::input::InputEvent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Team ID from a real shared ClickUp URL (`/t/{team}/{task}`).
const TEAM_ID: &str = "26408409";
/// Standard (UUID-ish) task ID segment.
const TASK_ID: &str = "14zrc6q9vr2";
/// Custom task ID (human-readable) that requires team context to resolve.
const CUSTOM_TASK_ID: &str = "PROJ-123";
/// Comment ID embedded in a shared comment URL query string.
const COMMENT_ID: &str = "1301560000056647";

fn task_named(id: &str, name: &str) -> clickdown::models::Task {
    let mut t = fixtures::test_task();
    t.id = id.to_string();
    t.name = name.to_string();
    t
}

fn comment_with_id(id: &str, text: &str) -> Comment {
    let mut c = fixtures::test_comment();
    c.id = id.to_string();
    c.text = text.to_string();
    c
}

/// Drive the app exactly like a user: g → u → type URL → Enter.
/// Returns after submitting the URL (async fetch is in flight).
fn type_url_and_submit(app: &mut TuiApp, url: &str) {
    let g = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    app.update(InputEvent::Key(g));
    let u = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE);
    app.update(InputEvent::Key(u));
    assert!(app.is_url_input_open(), "g u must open the URL dialog");

    for c in url.chars() {
        app.handle_url_input(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }
    app.update(InputEvent::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
}

/// Pump async API completions into app state, mirroring the main loop's
/// process_async_messages cadence with small yields for spawned tasks.
fn pump(app: &mut TuiApp, rounds: usize) {
    for _ in 0..rounds {
        std::thread::sleep(Duration::from_millis(20));
        app.process_async_messages();
    }
}

// ============================================================================
// Happy path: standard task URL → correct task on TaskDetail screen
// ============================================================================

/// THE end-to-end proof: a full real-world task URL typed into the g u dialog
/// resolves through the API mock and lands the user on the TaskDetail screen
/// showing exactly the task the URL identified.
#[test]
fn test_url_to_task_screen_end_to_end() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let target = task_named(TASK_ID, "Ship release notes");
        let other = task_named("other-task-999", "Unrelated task");

        let mock = MockClickUpClient::new()
            .with_workspaces(vec![fixtures::test_workspace()])
            .with_task(target.clone())
            .with_tasks(vec![other.clone(), target.clone()])
            .with_task_comments(vec![]);

        let mut app = TuiApp::with_client(Arc::new(mock)).unwrap();

        // Start somewhere else entirely: task list view.
        app.set_screen(Screen::Tasks);

        let url = format!("https://app.clickup.com/t/{TEAM_ID}/{TASK_ID}");
        type_url_and_submit(&mut app, &url);

        // Dialog closed after submit; async navigation begins.
        assert!(!app.is_url_input_open(), "dialog should close on submit");
        assert!(
            app.screen() != Screen::TaskDetail || app.task_detail().task.is_some(),
            "premature transition without task is a bug"
        );

        pump(&mut app, 10);

        assert_eq!(
            app.screen(),
            Screen::TaskDetail,
            "task URL must land on TaskDetail screen"
        );
        let shown = app.task_detail().task.as_ref().expect("task must be set");
        assert_eq!(shown.id, TASK_ID, "wrong task fetched for URL");
        assert_eq!(shown.name, "Ship release notes");
        // NOTE: status may already show "Loaded N comment(s)" because comment
        // loading follows navigation and overwrites the navigation status —
        // screen + task identity above are the proof, so no status assert here.
    });
}

/// Dirty pasted input (trailing newline/whitespace from chat copy) must not
/// corrupt the lookup — regression for the trailing-newline 404 class.
#[test]
fn test_dirty_pasted_url_still_opens_correct_task() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let target = task_named(TASK_ID, "Ship release notes");
        let mock = MockClickUpClient::new()
            .with_workspaces(vec![fixtures::test_workspace()])
            .with_task(target)
            .with_task_comments(vec![]);

        let mut app = TuiApp::with_client(Arc::new(mock)).unwrap();
        app.set_screen(Screen::Tasks);

        // Realistic paste: surrounding spaces + trailing newline.
        let url = format!("  https://app.clickup.com/t/{TEAM_ID}/{TASK_ID}\n");
        type_url_and_submit(&mut app, &url);
        pump(&mut app, 10);

        assert_eq!(app.screen(), Screen::TaskDetail, "dirty paste must still navigate");
        assert_eq!(
            app.task_detail().task.as_ref().unwrap().id,
            TASK_ID,
            "trailing whitespace must not corrupt the task ID"
        );
    });
}

// ============================================================================
// Custom task IDs: /t/{team}/{ABC-123} requires team context on lookup
// ============================================================================

/// Mock that only resolves custom IDs when the retry path is used, mimicking
/// the real API (plain GET /task/ABC-123 → 404; with
/// custom_task_ids=true&team_id → 200).
struct CustomIdOnlyClient {
    inner: MockClickUpClient,
}

#[async_trait::async_trait]
impl clickdown::api::ClickUpApi for CustomIdOnlyClient {
    // Delegate everything except get_task to the inner mock.
    async fn get_workspaces(&self) -> anyhow::Result<Vec<clickdown::models::Workspace>> {
        self.inner.get_workspaces().await
    }
    async fn get_current_user(&self) -> anyhow::Result<clickdown::models::User> {
        self.inner.get_current_user().await
    }
    async fn get_spaces(
        &self,
        team_id: &str,
    ) -> anyhow::Result<Vec<clickdown::models::ClickUpSpace>> {
        self.inner.get_spaces(team_id).await
    }
    async fn get_space(&self, space_id: &str) -> anyhow::Result<clickdown::models::ClickUpSpace> {
        self.inner.get_space(space_id).await
    }
    async fn get_folders(&self, space_id: &str) -> anyhow::Result<Vec<clickdown::models::Folder>> {
        self.inner.get_folders(space_id).await
    }
    async fn get_lists_in_folder(
        &self,
        folder_id: &str,
        archived: Option<bool>,
    ) -> anyhow::Result<Vec<clickdown::models::List>> {
        self.inner.get_lists_in_folder(folder_id, archived).await
    }
    async fn get_lists_in_space(
        &self,
        space_id: &str,
        archived: Option<bool>,
    ) -> anyhow::Result<Vec<clickdown::models::List>> {
        self.inner.get_lists_in_space(space_id, archived).await
    }
    async fn get_list(&self, list_id: &str) -> anyhow::Result<clickdown::models::List> {
        self.inner.get_list(list_id).await
    }
    async fn get_tasks(
        &self,
        list_id: &str,
        filters: &clickdown::models::TaskFilters,
    ) -> anyhow::Result<Vec<clickdown::models::Task>> {
        self.inner.get_tasks(list_id, filters).await
    }

    async fn get_task(&self, _task_id: &str) -> anyhow::Result<clickdown::models::Task> {
        // Plain lookup always fails for custom IDs (API behaviour).
        Err(anyhow::anyhow!("API error (404): task not found"))
    }

    async fn get_task_with_context(
        &self,
        task_id: &str,
        team_id: Option<&str>,
    ) -> anyhow::Result<clickdown::models::Task> {
        // Mirror production semantics: plain lookup first; retry with team
        // context when available. If the TUI ever drops the team_id parsed
        // from the URL, this retry never fires and the test fails.
        match self.get_task(task_id).await {
            Ok(task) => Ok(task),
            Err(_) => match team_id {
                Some(_team) => self.inner.get_task(task_id).await,
                None => Err(anyhow::anyhow!("API error (404): task not found")),
            },
        }
    }

    async fn create_task(
        &self,
        _list_id: &str,
        _task: &clickdown::models::CreateTaskRequest,
    ) -> anyhow::Result<clickdown::models::Task> {
        self.inner.create_task(_list_id, _task).await
    }
    async fn update_task(
        &self,
        _task_id: &str,
        _task: &clickdown::models::UpdateTaskRequest,
    ) -> anyhow::Result<clickdown::models::Task> {
        self.inner.update_task(_task_id, _task).await
    }
    async fn delete_task(&self, _task_id: &str) -> anyhow::Result<()> {
        self.inner.delete_task(_task_id).await
    }
    async fn get_list_members(&self, list_id: &str) -> anyhow::Result<Vec<clickdown::models::User>> {
        self.inner.get_list_members(list_id).await
    }
    async fn search_docs(
        &self,
        filters: &clickdown::models::DocumentFilters,
    ) -> anyhow::Result<Vec<clickdown::models::Document>> {
        self.inner.search_docs(filters).await
    }
    async fn get_doc_pages(
        &self,
        doc_id: &str,
    ) -> anyhow::Result<Vec<clickdown::models::Page>> {
        self.inner.get_doc_pages(doc_id).await
    }
    async fn get_page(&self, page_id: &str) -> anyhow::Result<clickdown::models::Page> {
        self.inner.get_page(page_id).await
    }
    async fn get_task_comments(&self, task_id: &str) -> anyhow::Result<Vec<Comment>> {
        self.inner.get_task_comments(task_id).await
    }
    async fn get_comment_replies(&self, comment_id: &str) -> anyhow::Result<Vec<Comment>> {
        self.inner.get_comment_replies(comment_id).await
    }
    async fn create_comment(
        &self,
        task_id: &str,
        comment: &clickdown::models::CreateCommentRequest,
    ) -> anyhow::Result<Comment> {
        self.inner.create_comment(task_id, comment).await
    }
    async fn create_comment_reply(
        &self,
        parent_comment_id: &str,
        comment: &clickdown::models::CreateCommentRequest,
    ) -> anyhow::Result<Comment> {
        self.inner.create_comment_reply(parent_comment_id, comment).await
    }
    async fn update_comment(
        &self,
        comment_id: &str,
        comment: &clickdown::models::UpdateCommentRequest,
    ) -> anyhow::Result<Comment> {
        self.inner.update_comment(comment_id, comment).await
    }
    async fn get_tasks_with_assignee(
        &self,
        list_id: &str,
        user_id: i32,
        limit: Option<i32>,
    ) -> anyhow::Result<Vec<clickdown::models::Task>> {
        self.inner.get_tasks_with_assignee(list_id, user_id, limit).await
    }
}

/// Custom ID with team context must resolve to the right task on screen.
/// (The default trait impl would NOT retry — this fails without the
/// get_task_with_context override, proving the wiring matters.)
#[test]
fn test_custom_task_id_url_resolves_with_team_context() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let target = task_named("real-uuid-task-77", "Custom ID target");
        let inner = MockClickUpClient::new()
            .with_workspaces(vec![fixtures::test_workspace()])
            .with_task(target.clone())
            .with_task_comments(vec![]);
        let client = CustomIdOnlyClient { inner };

        let mut app = TuiApp::with_client(Arc::new(client)).unwrap();
        app.set_screen(Screen::Tasks);

        let url = format!("https://app.clickup.com/t/{TEAM_ID}/{CUSTOM_TASK_ID}");
        type_url_and_submit(&mut app, &url);
        pump(&mut app, 10);

        assert_eq!(app.screen(), Screen::TaskDetail);
        assert_eq!(
            app.task_detail().task.as_ref().unwrap().id,
            "real-uuid-task-77",
            "custom ID must resolve through team-context retry"
        );
    });
}

// ============================================================================
// Comment URLs: task screen + correct comment selected
// ============================================================================

/// A shared comment URL must land on the task's detail screen with the
/// referenced comment selected in the comment list.
#[test]
fn test_comment_url_opens_task_and_selects_comment() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let target = task_named(TASK_ID, "Ship release notes");
        let comments = vec![
            comment_with_id("cmt-first", "first comment"),
            comment_with_id(COMMENT_ID, "the referenced comment"),
        ];

        let mock = MockClickUpClient::new()
            .with_workspaces(vec![fixtures::test_workspace()])
            .with_task(target)
            .with_task_comments(comments);

        let mut app = TuiApp::with_client(Arc::new(mock)).unwrap();
        app.set_screen(Screen::Tasks);

        let url = format!("https://app.clickup.com/t/{TEAM_ID}/{TASK_ID}?comment={COMMENT_ID}");
        type_url_and_submit(&mut app, &url);
        // Two async hops: task fetch → comments fetch. Pump generously.
        pump(&mut app, 25);

        assert_eq!(
            app.screen(),
            Screen::TaskDetail,
            "comment URL must open the task detail screen"
        );
        assert_eq!(app.task_detail().task.as_ref().unwrap().id, TASK_ID);
        assert_eq!(
            app.comments_for_test().len(),
            2,
            "comments must be loaded for selection"
        );
        assert_eq!(
            app.comment_selected_index_for_test(),
            1,
            "the referenced comment must be the selected one"
        );
        assert!(
            app.status_message().contains("Navigated to comment"),
            "status should confirm comment navigation, got: {}",
            app.status_message()
        );
    });
}

// ============================================================================
// Error paths: 404 restores the previous screen with a message
// ============================================================================

/// Unknown task ID: user stays where they were (TaskDetail must NOT show a
/// stale/wrong task) and gets a clear status message.
#[test]
fn test_unknown_task_url_restores_previous_screen() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mock = MockClickUpClient::new()
            .with_workspaces(vec![fixtures::test_workspace()])
            // No task_response configured → "Task not found".
            .with_task_comments(vec![]);

        let mut app = TuiApp::with_client(Arc::new(mock)).unwrap();
        app.set_screen(Screen::Tasks);

        let url = format!("https://app.clickup.com/t/{TEAM_ID}/doesnotexist99");
        type_url_and_submit(&mut app, &url);
        pump(&mut app, 10);

        assert_eq!(
            app.screen(),
            Screen::Tasks,
            "failed navigation must restore the previous screen"
        );
        assert!(
            app.task_detail().task.is_none(),
            "a failed lookup must not leave a task on screen"
        );
        assert!(
            app.status_message().contains("Resource not found"),
            "status should explain the failure, got: {}",
            app.status_message()
        );
    });
}

/// Garbage input must NOT navigate and must surface a parse error.
#[test]
fn test_garbage_url_stays_put_with_error() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mock = MockClickUpClient::new()
            .with_workspaces(vec![fixtures::test_workspace()])
            .with_task(task_named(TASK_ID, "Should not be shown"))
            .with_task_comments(vec![]);

        let mut app = TuiApp::with_client(Arc::new(mock)).unwrap();
        app.set_screen(Screen::Tasks);

        type_url_and_submit(&mut app, "not-a-clickup-url");
        pump(&mut app, 5);

        assert!(
            app.is_url_input_open(),
            "dialog must stay open showing the parse error"
        );
        assert!(
            app.url_input_error().is_some(),
            "error message must be displayed"
        );
        assert_eq!(app.screen(), Screen::Tasks, "must not navigate on garbage");
        assert!(app.task_detail().task.is_none(), "no task may be attached");
    });
}

// ============================================================================
// Snapshot: the TaskDetail screen the user lands on
// ============================================================================

/// Snapshot of the exact screen the end user sees after URL navigation —
/// guards the "right task screen" half of the guarantee at the render level.
#[test]
fn test_snapshot_task_detail_after_url_navigation() {
    use clickdown::tui::widgets::task_detail::{render_task_detail, TaskDetailState};
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, layout::Rect, Terminal};

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let target = task_named(TASK_ID, "Ship release notes");
        let mock = MockClickUpClient::new()
            .with_workspaces(vec![fixtures::test_workspace()])
            .with_task(target)
            .with_task_comments(vec![]);

        let mut app = TuiApp::with_client(Arc::new(mock)).unwrap();
        app.set_screen(Screen::Tasks);

        let url = format!("https://app.clickup.com/t/{TEAM_ID}/{TASK_ID}");
        type_url_and_submit(&mut app, &url);
        pump(&mut app, 10);

        assert_eq!(app.screen(), Screen::TaskDetail);

        // Render the widget exactly as the app state holds it post-navigation.
        let mut detail = TaskDetailState::new();
        detail.task = app.task_detail().task.clone();
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        let mut snapshot = String::new();
        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 100, 30);
                render_task_detail(
                    frame,
                    &detail,
                    area,
                    "",
                    "",
                    &clickdown::tui::app::TaskCreationField::Name,
                );
            })
            .unwrap();
        for y in 0..30 {
            for x in 0..100 {
                let cell = &terminal.backend().buffer()[(x, y)];
                snapshot.push(cell.symbol().chars().next().unwrap_or(' '));
            }
            snapshot.push('\n');
        }

        assert_snapshot!("task_detail_after_url_navigation", snapshot);
    });
}
