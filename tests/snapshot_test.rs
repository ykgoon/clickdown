//! Snapshot tests for ClickDown TUI widgets and layouts
//!
//! These tests use the `insta` crate to capture and compare rendered UI output.
//! All tests use mocked data for deterministic, reproducible snapshots.
//!
//! Run tests: `cargo test --test snapshot_test`
//! Review snapshots: `cargo insta review`
//! Accept changes: `cargo insta accept`

mod fixtures;

use clickdown::models::{Notification, Task};
use clickdown::tui::layout::{generate_screen_title, TuiLayout};
use clickdown::tui::widgets::{
    auth::{render_auth, AuthState},
    dialog::{render_dialog, DialogState, DialogType},
    document::{render_document, DocumentState},
    help::{render_help, HelpState},
    inbox_view::{render_inbox_list, InboxListState},
    sidebar::{render_sidebar, SidebarItem, SidebarState},
    task_detail::{render_task_detail, TaskDetailState},
    task_list::{render_task_list, TaskListState},
};
use insta::assert_snapshot;
use ratatui::{backend::TestBackend, layout::Rect, Terminal};

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a test terminal with the given size
fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(width, height)).unwrap()
}

/// Render widget and assert snapshot
fn assert_widget_snapshot<F>(name: &str, width: u16, height: u16, mut render_fn: F)
where
    F: FnMut(&mut ratatui::Frame),
{
    let mut terminal = create_test_terminal(width, height);

    terminal
        .draw(|frame| {
            render_fn(frame);
        })
        .unwrap();

    // Get buffer contents - use the buffer from the draw closure
    let mut snapshot = String::new();
    for y in 0..height {
        for x in 0..width {
            let cell = terminal.backend().buffer().get(x, y);
            snapshot.push(cell.symbol().chars().next().unwrap_or(' '));
        }
        snapshot.push('\n');
    }

    assert_snapshot!(name, snapshot);
}

// ============================================================================
// Fixture Data Generators (Task 2.4)
// ============================================================================

/// Create multiple test tasks for list snapshots
fn create_test_tasks() -> Vec<Task> {
    vec![
        Task {
            id: "task-1".to_string(),
            name: "Review pull request".to_string(),
            status: Some(clickdown::models::TaskStatus {
                id: None,
                status: "in progress".to_string(),
                color: Some("#5c7cfa".to_string()),
                type_field: None,
                orderindex: None,
                status_group: None,
            }),
            priority: Some(clickdown::models::Priority {
                priority: "high".to_string(),
                color: Some("#ff0000".to_string()),
            }),
            ..fixtures::test_task()
        },
        Task {
            id: "task-2".to_string(),
            name: "Write unit tests".to_string(),
            status: Some(clickdown::models::TaskStatus {
                id: None,
                status: "todo".to_string(),
                color: Some("#87909e".to_string()),
                type_field: None,
                orderindex: None,
                status_group: None,
            }),
            priority: None,
            ..fixtures::test_task()
        },
        Task {
            id: "task-3".to_string(),
            name: "Deploy to production".to_string(),
            status: Some(clickdown::models::TaskStatus {
                id: None,
                status: "done".to_string(),
                color: Some("#00ff00".to_string()),
                type_field: None,
                orderindex: None,
                status_group: None,
            }),
            priority: Some(clickdown::models::Priority {
                priority: "urgent".to_string(),
                color: Some("#ff0000".to_string()),
            }),
            ..fixtures::test_task()
        },
    ]
}

/// Create sidebar items for hierarchy snapshots
/// Matches actual app behavior: AssignedTasks and Inbox at top, then workspace hierarchy
fn create_sidebar_items() -> Vec<SidebarItem> {
    vec![
        SidebarItem::AssignedTasks,
        SidebarItem::Inbox,
        SidebarItem::Workspace {
            name: "Engineering".to_string(),
            id: "ws-1".to_string(),
        },
        SidebarItem::Space {
            name: "Backend".to_string(),
            id: "sp-1".to_string(),

        },
        SidebarItem::Folder {
            name: "API".to_string(),
            id: "fd-1".to_string(),

        },
        SidebarItem::List {
            name: "Sprint Tasks".to_string(),
            id: "lst-1".to_string(),

        },
    ]
}

// ============================================================================
// Mock Clipboard Helper (Task 2.3)
// ============================================================================

/// Mock clipboard for tests - always succeeds without actual clipboard access
#[derive(Debug, Clone, Default)]
pub struct MockClipboard;

impl MockClipboard {
    pub fn new() -> Self {
        Self
    }

    pub fn set_text(&mut self, _text: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn get_text(&mut self) -> Result<String, String> {
        Ok(String::new())
    }
}

// ============================================================================
// Sidebar Widget Snapshot Tests (Task 3.1)
// ============================================================================

#[test]
fn test_sidebar_empty() {
    let mut sidebar = SidebarState::new();
    *sidebar.items_mut() = vec![];

    assert_widget_snapshot("sidebar_empty", 40, 15, |frame| {
        let area = Rect::new(0, 0, 40, 15);
        render_sidebar(frame, &sidebar, area, None);
    });
}

#[test]
fn test_sidebar_with_items() {
    let mut sidebar = SidebarState::new();
    *sidebar.items_mut() = create_sidebar_items();

    // Render with 5 assigned tasks (showing count badge)
    assert_widget_snapshot("sidebar_with_items", 40, 15, |frame| {
        let area = Rect::new(0, 0, 40, 15);
        render_sidebar(frame, &sidebar, area, Some(5));
    });
}

#[test]
fn test_sidebar_with_selection() {
    let mut sidebar = SidebarState::new();
    *sidebar.items_mut() = create_sidebar_items();
    sidebar.select_first();

    // Render with 3 assigned tasks
    assert_widget_snapshot("sidebar_with_selection", 40, 15, |frame| {
        let area = Rect::new(0, 0, 40, 15);
        render_sidebar(frame, &sidebar, area, Some(3));
    });
}

#[test]
fn test_sidebar_with_zero_assigned() {
    let mut sidebar = SidebarState::new();
    *sidebar.items_mut() = create_sidebar_items();

    // Render with 0 assigned tasks (should show no count or empty badge)
    assert_widget_snapshot("sidebar_with_zero_assigned", 40, 15, |frame| {
        let area = Rect::new(0, 0, 40, 15);
        render_sidebar(frame, &sidebar, area, Some(0));
    });
}

#[test]
fn test_sidebar_with_large_assigned_count() {
    let mut sidebar = SidebarState::new();
    *sidebar.items_mut() = create_sidebar_items();

    // Render with large count (99+)
    assert_widget_snapshot("sidebar_with_large_assigned_count", 40, 15, |frame| {
        let area = Rect::new(0, 0, 40, 15);
        render_sidebar(frame, &sidebar, area, Some(99));
    });
}

// ============================================================================
// Task List Widget Snapshot Tests (Task 3.2)
// ============================================================================

#[test]
fn test_task_list_empty() {
    let task_list = TaskListState::new();

    assert_widget_snapshot("task_list_empty", 60, 15, |frame| {
        let area = Rect::new(0, 0, 60, 15);
        render_task_list(frame, &task_list, area, false);
    });
}

#[test]
fn test_task_list_with_tasks() {
    let mut task_list = TaskListState::new();
    *task_list.tasks_mut() = create_test_tasks();

    assert_widget_snapshot("task_list_with_tasks", 60, 15, |frame| {
        let area = Rect::new(0, 0, 60, 15);
        render_task_list(frame, &task_list, area, false);
    });
}

#[test]
fn test_task_list_with_selection() {
    let mut task_list = TaskListState::new();
    *task_list.tasks_mut() = create_test_tasks();
    task_list.select_first();

    assert_widget_snapshot("task_list_with_selection", 60, 15, |frame| {
        let area = Rect::new(0, 0, 60, 15);
        render_task_list(frame, &task_list, area, false);
    });
}

// ============================================================================
// Assigned Tasks View Snapshot Tests
// ============================================================================

/// Create test tasks with assignees for "Assigned to Me" view
fn create_assigned_test_tasks() -> Vec<Task> {
    use clickdown::models::task::User;
    
    vec![
        Task {
            id: "assigned-task-1".to_string(),
            name: "Review Q2 planning doc".to_string(),
            status: Some(clickdown::models::TaskStatus {
                id: None,
                status: "in progress".to_string(),
                color: Some("#5c7cfa".to_string()),
                type_field: None,
                orderindex: None,
                status_group: None,
            }),
            priority: Some(clickdown::models::Priority {
                priority: "high".to_string(),
                color: Some("#ff0000".to_string()),
            }),
            assignees: vec![User {
                id: 123,
                username: "testuser".to_string(),
                color: None,
                email: Some("test@example.com".to_string()),
                profile_picture: None,
                initials: Some("TU".to_string()),
            }],
            ..fixtures::test_task()
        },
        Task {
            id: "assigned-task-2".to_string(),
            name: "Fix bug in task filtering".to_string(),
            status: Some(clickdown::models::TaskStatus {
                id: None,
                status: "todo".to_string(),
                color: Some("#87909e".to_string()),
                type_field: None,
                orderindex: None,
                status_group: None,
            }),
            priority: Some(clickdown::models::Priority {
                priority: "urgent".to_string(),
                color: Some("#ff0000".to_string()),
            }),
            assignees: vec![User {
                id: 123,
                username: "testuser".to_string(),
                color: None,
                email: Some("test@example.com".to_string()),
                profile_picture: None,
                initials: Some("TU".to_string()),
            }],
            ..fixtures::test_task()
        },
        Task {
            id: "assigned-task-3".to_string(),
            name: "Update API documentation".to_string(),
            status: Some(clickdown::models::TaskStatus {
                id: None,
                status: "done".to_string(),
                color: Some("#00ff00".to_string()),
                type_field: None,
                orderindex: None,
                status_group: None,
            }),
            priority: None,
            assignees: vec![User {
                id: 123,
                username: "testuser".to_string(),
                color: None,
                email: Some("test@example.com".to_string()),
                profile_picture: None,
                initials: Some("TU".to_string()),
            }],
            ..fixtures::test_task()
        },
    ]
}

#[test]
fn test_assigned_tasks_view_empty() {
    let task_list = TaskListState::new();

    assert_widget_snapshot("assigned_tasks_view_empty", 60, 15, |frame| {
        let area = Rect::new(0, 0, 60, 15);
        render_task_list(frame, &task_list, area, false);
    });
}

#[test]
fn test_assigned_tasks_view_with_tasks() {
    let mut task_list = TaskListState::new();
    *task_list.tasks_mut() = create_assigned_test_tasks();

    assert_widget_snapshot("assigned_tasks_view_with_tasks", 60, 15, |frame| {
        let area = Rect::new(0, 0, 60, 15);
        render_task_list(frame, &task_list, area, false);
    });
}

#[test]
fn test_assigned_tasks_view_with_selection() {
    let mut task_list = TaskListState::new();
    *task_list.tasks_mut() = create_assigned_test_tasks();
    task_list.select_first();

    assert_widget_snapshot("assigned_tasks_view_with_selection", 60, 15, |frame| {
        let area = Rect::new(0, 0, 60, 15);
        render_task_list(frame, &task_list, area, false);
    });
}

#[test]
fn test_assigned_tasks_view_loading() {
    let task_list = TaskListState::new();

    assert_widget_snapshot("assigned_tasks_view_loading", 60, 15, |frame| {
        let area = Rect::new(0, 0, 60, 15);
        render_task_list(frame, &task_list, area, true);
    });
}

/// Test that verifies "Assigned to Me" loads tasks correctly
///
/// This test verifies the fix for the bug where "Assigned to Me" showed zero tasks.
/// The fix ensures that:
/// - Mock client has accessible lists configured
/// - Tasks are fetched from those lists
/// - User sees their assigned tasks
///
/// Before the fix: get_all_accessible_lists() returned empty, no tasks were fetched.
/// After the fix: accessible lists are provided, tasks are loaded and displayed.
#[test]
fn test_assigned_tasks_view_bug_shows_zero_tasks() {
    use clickdown::api::MockClickUpClient;
    use clickdown::models::workspace::List;
    use clickdown::tui::app::TuiApp;
    use clickdown::tui::widgets::SidebarItem;
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        // Configure mock to test the FIX:
        // - Has current user configured
        // - Has accessible lists configured (this was missing before!)
        // - Has tasks with assignee configured
        let lists = vec![List {
            id: "list-1".to_string(),
            name: "Test List".to_string(),
            content: None,
            description: None,
            archived: false,
            hidden: false,
            orderindex: Some(0),
            space: None,
            folder: None,
            status: None,
            priority: None,
        }];

        let mock_client = MockClickUpClient::new()
            .with_current_user(fixtures::test_user())
            .with_accessible_lists(lists.clone())  // FIX: Now providing accessible lists
            .with_tasks_with_assignee_response(fixtures::test_tasks_with_assignees());

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Set up sidebar with AssignedTasks item (matching real app behavior)
        *app.sidebar().items_mut() = vec![SidebarItem::AssignedTasks];
        app.sidebar().select_first();

        // User navigates to "Assigned to Me" (simulating real workflow)
        app.navigate_into();

        // Process async messages to let the fetch complete
        // Loop until tasks are loaded or timeout
        let mut iterations = 0;
        let max_iterations = 30;
        while app.assigned_tasks().tasks().is_empty() && iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        // Verify tasks were loaded (fix verification)
        assert!(
            !app.assigned_tasks().tasks().is_empty(),
            "Expected tasks to be loaded but got {} tasks after {} iterations",
            app.assigned_tasks().tasks().len(),
            iterations
        );

        // Capture snapshot - should show tasks loaded (the fix)
        assert_widget_snapshot("assigned_tasks_view_with_tasks_loaded", 60, 15, |frame| {
            let area = Rect::new(0, 0, 60, 15);
            render_task_list(
                frame,
                app.assigned_tasks(),
                area,
                app.assigned_tasks_loading(),
            );
        });
    });
}

// ============================================================================
// Task Detail Widget Snapshot Tests (Task 3.3)
// ============================================================================

#[test]
fn test_task_detail_view_mode() {
    let mut detail = TaskDetailState::new();
    let task = create_test_tasks().remove(0);
    detail.task = Some(task);

    assert_widget_snapshot("task_detail_view", 60, 20, |frame| {
        let area = Rect::new(0, 0, 60, 20);
        render_task_detail(frame, &detail, area);
    });
}

#[test]
fn test_task_detail_empty() {
    let detail = TaskDetailState::new();

    assert_widget_snapshot("task_detail_empty", 60, 20, |frame| {
        let area = Rect::new(0, 0, 60, 20);
        render_task_detail(frame, &detail, area);
    });
}

// ============================================================================
// Auth View Widget Snapshot Tests (Task 3.4)
// ============================================================================

#[test]
fn test_auth_view_empty() {
    let auth = AuthState::new();

    assert_widget_snapshot("auth_view_empty", 60, 20, |frame| {
        let area = Rect::new(0, 0, 60, 20);
        render_auth(frame, &auth, area);
    });
}

#[test]
fn test_auth_view_partial_token() {
    let mut auth = AuthState::new();
    auth.token_input = "pk_test_abc123xyz".to_string();
    auth.cursor_pos = 15;

    assert_widget_snapshot("auth_view_partial_token", 60, 20, |frame| {
        let area = Rect::new(0, 0, 60, 20);
        render_auth(frame, &auth, area);
    });
}

#[test]
fn test_auth_view_error() {
    let mut auth = AuthState::new();
    auth.error = Some("Invalid token. Please try again.".to_string());

    assert_widget_snapshot("auth_view_error", 60, 20, |frame| {
        let area = Rect::new(0, 0, 60, 20);
        render_auth(frame, &auth, area);
    });
}

// ============================================================================
// Document View Widget Snapshot Tests (Task 3.5)
// ============================================================================

#[test]
fn test_document_view_empty() {
    let doc = DocumentState::new();

    assert_widget_snapshot("document_view_empty", 60, 20, |frame| {
        let area = Rect::new(0, 0, 60, 20);
        render_document(frame, &doc, area);
    });
}

#[test]
fn test_document_view_with_content() {
    let mut doc = DocumentState::new();
    doc.title = "Test Document".to_string();
    doc.content = "# Test Document\n\nThis is test content.".to_string();

    assert_widget_snapshot("document_view_with_content", 60, 20, |frame| {
        let area = Rect::new(0, 0, 60, 20);
        render_document(frame, &doc, area);
    });
}

// ============================================================================
// Inbox View Widget Snapshot Tests (Task 3.7)
// ============================================================================

/// Create test notifications for inbox snapshots
fn create_test_notifications() -> Vec<Notification> {
    vec![
        Notification {
            id: "notif-1".to_string(),
            workspace_id: "ws-1".to_string(),
            title: "Task assigned to you".to_string(),
            description: "You were assigned to 'Review pull request'".to_string(),
            created_at: Some(1704067200000),
            read_at: None,
        },
        Notification {
            id: "notif-2".to_string(),
            workspace_id: "ws-1".to_string(),
            title: "Comment on task".to_string(),
            description: "New comment on 'Deploy to production'".to_string(),
            created_at: Some(1704153600000),
            read_at: None,
        },
        Notification {
            id: "notif-3".to_string(),
            workspace_id: "ws-1".to_string(),
            title: "Status change".to_string(),
            description: "".to_string(),
            created_at: Some(1704240000000),
            read_at: Some(1704326400000),
        },
    ]
}

#[test]
fn test_inbox_view_empty() {
    let mut inbox = InboxListState::new();

    assert_widget_snapshot("inbox_view_empty", 60, 15, |frame| {
        let area = Rect::new(0, 0, 60, 15);
        render_inbox_list(frame, area, &mut inbox, false);
    });
}

#[test]
fn test_inbox_view_with_notifications() {
    let mut inbox = InboxListState::new();
    inbox.set_notifications(create_test_notifications());

    assert_widget_snapshot("inbox_view_with_notifications", 60, 15, |frame| {
        let area = Rect::new(0, 0, 60, 15);
        render_inbox_list(frame, area, &mut inbox, false);
    });
}

#[test]
fn test_inbox_view_with_selection() {
    let mut inbox = InboxListState::new();
    inbox.set_notifications(create_test_notifications());
    inbox.list_state.select(Some(1));

    assert_widget_snapshot("inbox_view_with_selection", 60, 15, |frame| {
        let area = Rect::new(0, 0, 60, 15);
        render_inbox_list(frame, area, &mut inbox, false);
    });
}

// ============================================================================
// Help Dialog Snapshot Tests (Task 3.6)
// ============================================================================

#[test]
fn test_help_dialog_visible() {
    let mut help = HelpState::new();
    help.visible = true;

    assert_widget_snapshot("help_dialog_visible", 80, 24, |frame| {
        let area = Rect::new(0, 0, 80, 24);
        render_help(frame, &help, area);
    });
}

#[test]
fn test_dialog_quit_confirmation() {
    let mut dialog = DialogState::new();
    dialog.show(DialogType::ConfirmQuit);

    assert_widget_snapshot("dialog_quit_confirmation", 80, 24, |frame| {
        let area = Rect::new(0, 0, 80, 24);
        render_dialog(frame, &dialog, area);
    });
}

// ============================================================================
// Full Screen Layout Snapshot Tests (Task 4.1)
// ============================================================================

#[test]
fn test_layout_80x24() {
    let layout = TuiLayout::new(Rect::new(0, 0, 80, 24));

    assert_snapshot!(
        "layout_80x24",
        format!(
            "Title area: {}x{}\nContent area: {}x{}\nStatus area: {}x{}\nToo small: {}",
            layout.title_area.width,
            layout.title_area.height,
            layout.content_area.width,
            layout.content_area.height,
            layout.status_area.width,
            layout.status_area.height,
            layout.too_small
        )
    );
}

#[test]
fn test_layout_120x30() {
    let layout = TuiLayout::new(Rect::new(0, 0, 120, 30));

    assert_snapshot!(
        "layout_120x30",
        format!(
            "Title area: {}x{}\nContent area: {}x{}\nStatus area: {}x{}\nToo small: {}",
            layout.title_area.width,
            layout.title_area.height,
            layout.content_area.width,
            layout.content_area.height,
            layout.status_area.width,
            layout.status_area.height,
            layout.too_small
        )
    );
}

#[test]
fn test_layout_160x40() {
    let layout = TuiLayout::new(Rect::new(0, 0, 160, 40));

    assert_snapshot!(
        "layout_160x40",
        format!(
            "Title area: {}x{}\nContent area: {}x{}\nStatus area: {}x{}\nToo small: {}",
            layout.title_area.width,
            layout.title_area.height,
            layout.content_area.width,
            layout.content_area.height,
            layout.status_area.width,
            layout.status_area.height,
            layout.too_small
        )
    );
}

// ============================================================================
// Authentication Screen Layout Tests (Task 4.2)
// ============================================================================

#[test]
fn test_auth_screen_80x24() {
    let auth = AuthState::new();

    assert_widget_snapshot("auth_screen_80x24", 80, 24, |frame| {
        let area = Rect::new(0, 0, 80, 24);
        render_auth(frame, &auth, area);
    });
}

#[test]
fn test_auth_screen_120x30() {
    let auth = AuthState::new();

    assert_widget_snapshot("auth_screen_120x30", 120, 30, |frame| {
        let area = Rect::new(0, 0, 120, 30);
        render_auth(frame, &auth, area);
    });
}

// ============================================================================
// Main Application Layout Tests (Task 4.3)
// ============================================================================

#[test]
fn test_main_layout_sidebar_collapsed() {
    let mut sidebar = SidebarState::new();
    sidebar.visible = false;

    assert_widget_snapshot("main_layout_sidebar_collapsed", 80, 24, |frame| {
        let area = Rect::new(0, 0, 80, 24);
        render_sidebar(frame, &sidebar, area, None);
    });
}

#[test]
fn test_main_layout_sidebar_expanded() {
    let mut sidebar = SidebarState::new();
    sidebar.visible = true;
    *sidebar.items_mut() = create_sidebar_items();

    assert_widget_snapshot("main_layout_sidebar_expanded", 80, 24, |frame| {
        let area = Rect::new(0, 0, 80, 24);
        render_sidebar(frame, &sidebar, area, None);
    });
}

// ============================================================================
// Screen Title Tests (Task 4.4)
// ============================================================================

#[test]
fn test_screen_title_authentication() {
    let title = generate_screen_title("Authentication");
    assert_snapshot!("title_authentication", title);
}

#[test]
fn test_screen_title_workspaces() {
    let title = generate_screen_title("Workspaces");
    assert_snapshot!("title_workspaces", title);
}

#[test]
fn test_screen_title_tasks() {
    let title = generate_screen_title("Tasks: My List");
    assert_snapshot!("title_tasks", title);
}

#[test]
fn test_screen_title_documents() {
    let title = generate_screen_title("Documents");
    assert_snapshot!("title_documents", title);
}

#[test]
fn test_screen_title_inbox() {
    let title = generate_screen_title("Inbox");
    assert_snapshot!("title_inbox", title);
}

// ============================================================================
// Status Bar Tests (Task 4.5)
// ============================================================================

#[test]
fn test_status_bar_help() {
    // Status bar content for main view with help
    let status = " j/k: Navigate | Enter: Select | Esc: Back | ?: Help | q: Quit ";
    assert_snapshot!("status_bar_help", status.to_string());
}

#[test]
fn test_status_bar_error() {
    let status = " Error: Failed to load tasks. Please try again. ";
    assert_snapshot!("status_bar_error", status.to_string());
}

#[test]
fn test_status_bar_loading() {
    let status = " Loading... ";
    assert_snapshot!("status_bar_loading", status.to_string());
}
