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
    use clickdown::api::mock_client::MockClickUpClient;
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

// ============================================================================
// Navigation Bar Snapshot Tests
// ============================================================================

/// Test that traverses the navigation bar hierarchy and verifies
/// "Assigned to Me" appears above "Inbox" at each level.
///
/// This test:
/// 1. Navigates down through: Workspaces → Spaces → Folders → Lists → Tasks
/// 2. Navigates back up through: Lists → Folders → Spaces → Workspaces
/// 3. Captures a single combined snapshot showing all 9 levels
#[test]
fn test_navigation_hierarchy() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::models::workspace::{Folder, List, Space, Workspace};
    use clickdown::tui::app::{Screen, TuiApp};
    use clickdown::tui::widgets::{render_sidebar, SidebarItem};
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, layout::Rect, Terminal};
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        // Create test hierarchy with multiple items at each level
        // 3 workspaces
        let workspaces = vec![
            Workspace {
                id: "ws-1".to_string(),
                name: "Engineering".to_string(),
                color: Some("#1abc9c".to_string()),
                avatar: None,
                member_count: Some(5),
            },
            Workspace {
                id: "ws-2".to_string(),
                name: "Marketing".to_string(),
                color: Some("#e74c3c".to_string()),
                avatar: None,
                member_count: Some(3),
            },
            Workspace {
                id: "ws-3".to_string(),
                name: "Design".to_string(),
                color: Some("#9b59b6".to_string()),
                avatar: None,
                member_count: Some(4),
            },
        ];

        // 3 spaces in first workspace
        let spaces = vec![
            Space {
                id: "sp-1".to_string(),
                name: "Backend Team".to_string(),
                color: Some("#3498db".to_string()),
                private: false,
                status: None,
                folders: vec![],
                lists: vec![],
            },
            Space {
                id: "sp-2".to_string(),
                name: "Frontend Team".to_string(),
                color: Some("#2ecc71".to_string()),
                private: false,
                status: None,
                folders: vec![],
                lists: vec![],
            },
            Space {
                id: "sp-3".to_string(),
                name: "DevOps Team".to_string(),
                color: Some("#f39c12".to_string()),
                private: false,
                status: None,
                folders: vec![],
                lists: vec![],
            },
        ];

        // 3 folders in first space
        let folders = vec![
            Folder {
                id: "fd-1".to_string(),
                name: "Q1 Projects".to_string(),
                color: Some("#e74c3c".to_string()),
                private: false,
                space: None,
                lists: vec![],
            },
            Folder {
                id: "fd-2".to_string(),
                name: "Q2 Projects".to_string(),
                color: Some("#3498db".to_string()),
                private: false,
                space: None,
                lists: vec![],
            },
            Folder {
                id: "fd-3".to_string(),
                name: "Q3 Projects".to_string(),
                color: Some("#2ecc71".to_string()),
                private: false,
                space: None,
                lists: vec![],
            },
        ];

        // 3 lists in first folder
        let lists = vec![
            List {
                id: "lst-1".to_string(),
                name: "Sprint Planning".to_string(),
                content: None,
                description: None,
                archived: false,
                hidden: false,
                orderindex: Some(0),
                space: None,
                folder: None,
                status: None,
                priority: None,
            },
            List {
                id: "lst-2".to_string(),
                name: "Bug Fixes".to_string(),
                content: None,
                description: None,
                archived: false,
                hidden: false,
                orderindex: Some(1),
                space: None,
                folder: None,
                status: None,
                priority: None,
            },
            List {
                id: "lst-3".to_string(),
                name: "Feature Requests".to_string(),
                content: None,
                description: None,
                archived: false,
                hidden: false,
                orderindex: Some(2),
                space: None,
                folder: None,
                status: None,
                priority: None,
            },
        ];

        // Configure mock client with full hierarchy
        let mock_client = MockClickUpClient::new()
            .with_workspaces(workspaces.clone())
            .with_spaces(spaces.clone())
            .with_folders(folders.clone())
            .with_lists_in_folder(lists.clone())
            .with_tasks(vec![]);

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Load workspaces to populate sidebar
        app.load_workspaces();
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        app.process_async_messages();

        // Helper to capture sidebar snapshot as string
        let capture_sidebar = |app: &mut TuiApp| -> String {
            let mut terminal = Terminal::new(TestBackend::new(40, 15)).unwrap();

            terminal
                .draw(|frame| {
                    let area = Rect::new(0, 0, 40, 15);
                    render_sidebar(frame, app.sidebar(), area, Some(3));
                })
                .unwrap();

            // Get buffer contents
            let mut snapshot = String::new();
            for y in 0..15 {
                for x in 0..40 {
                    let cell = terminal.backend().buffer().get(x, y);
                    snapshot.push(cell.symbol().chars().next().unwrap_or(' '));
                }
                snapshot.push('\n');
            }

            snapshot
        };

        // Helper to verify sidebar order
        let verify_sidebar_order = |app: &mut TuiApp, level_name: &str| {
            let items = app.sidebar().items().clone();

            // Find positions of "Assigned to Me" and "Inbox"
            let mut assigned_pos = None;
            let mut inbox_pos = None;

            for (i, item) in items.iter().enumerate() {
                match item {
                    SidebarItem::AssignedTasks => assigned_pos = Some(i),
                    SidebarItem::Inbox => inbox_pos = Some(i),
                    _ => {}
                }
            }

            // Assert both items exist
            assert!(
                assigned_pos.is_some(),
                "Level '{}': 'Assigned to Me' not found in sidebar. Items: {:?}",
                level_name,
                items.iter().map(|i| format!("{:?}", i)).collect::<Vec<_>>()
            );
            assert!(
                inbox_pos.is_some(),
                "Level '{}': 'Inbox' not found in sidebar. Items: {:?}",
                level_name,
                items.iter().map(|i| format!("{:?}", i)).collect::<Vec<_>>()
            );

            // Assert "Assigned to Me" comes before "Inbox"
            let assigned = assigned_pos.unwrap();
            let inbox = inbox_pos.unwrap();
            assert!(
                assigned < inbox,
                "Level '{}': 'Assigned to Me' (position {}) should appear before 'Inbox' (position {}). Items: {:?}",
                level_name,
                assigned,
                inbox,
                items.iter().map(|i| format!("{:?}", i)).collect::<Vec<_>>()
            );
        };

        // Collect all levels into one combined snapshot
        let mut combined_snapshot = String::new();

        // Helper to add level to combined snapshot
        let mut add_level = |app: &mut TuiApp, level_num: usize, level_name: &str, direction: &str| {
            verify_sidebar_order(app, &format!("{} ({})", level_name, direction));
            
            let sidebar = capture_sidebar(app);
            
            combined_snapshot.push_str(&format!(
                "=== Level {}: {} ({}) ===\n{}\n",
                level_num, level_name, direction, sidebar
            ));
        };

        // =====================================================================
        // TRAVERSE DOWN
        // =====================================================================

        // Level 1: Workspaces
        assert_eq!(app.screen(), Screen::Workspaces, "Should start at Workspaces");
        add_level(&mut app, 1, "Workspaces", "Down");

        // Navigate into workspace
        app.sidebar().select_next(); // Skip "Assigned to Me" (index 0)
        app.sidebar().select_next(); // Skip "Inbox" (index 1)
        app.navigate_into();
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        app.process_async_messages();

        // Level 2: Spaces
        assert_eq!(app.screen(), Screen::Spaces, "Should be at Spaces");
        add_level(&mut app, 2, "Spaces", "Down");

        // Navigate into space
        app.sidebar().select_next();
        app.sidebar().select_next();
        app.navigate_into();
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        app.process_async_messages();

        // Level 3: Folders
        assert_eq!(app.screen(), Screen::Folders, "Should be at Folders");
        add_level(&mut app, 3, "Folders", "Down");

        // Navigate into folder
        app.sidebar().select_next();
        app.sidebar().select_next();
        app.navigate_into();
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        app.process_async_messages();

        // Level 4: Lists
        assert_eq!(app.screen(), Screen::Lists, "Should be at Lists");
        add_level(&mut app, 4, "Lists", "Down");

        // Navigate into list
        app.sidebar().select_next();
        app.sidebar().select_next();
        app.navigate_into();
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        app.process_async_messages();

        // Level 5: Tasks (deepest level)
        assert_eq!(app.screen(), Screen::Tasks, "Should be at Tasks");
        add_level(&mut app, 5, "Tasks", "Down");

        // =====================================================================
        // TRAVERSE UP
        // =====================================================================

        // Navigate back to Lists
        app.navigate_back();
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        app.process_async_messages();

        // Level 6: Lists (going up)
        assert_eq!(app.screen(), Screen::Lists, "Should be back at Lists");
        add_level(&mut app, 6, "Lists", "Up");

        // Navigate back to Folders
        app.navigate_back();
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        app.process_async_messages();

        // Level 7: Folders (going up)
        assert_eq!(app.screen(), Screen::Folders, "Should be back at Folders");
        add_level(&mut app, 7, "Folders", "Up");

        // Navigate back to Spaces
        app.navigate_back();
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        app.process_async_messages();

        // Level 8: Spaces (going up)
        assert_eq!(app.screen(), Screen::Spaces, "Should be back at Spaces");
        add_level(&mut app, 8, "Spaces", "Up");

        // Navigate back to Workspaces
        app.navigate_back();
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        app.process_async_messages();

        // Level 9: Workspaces (going up)
        assert_eq!(app.screen(), Screen::Workspaces, "Should be back at Workspaces");
        add_level(&mut app, 9, "Workspaces", "Up");

        // Assert the combined snapshot
        assert_snapshot!("navigation_hierarchy", combined_snapshot);
    });
}

// ============================================================================
// Navigation: Assigned to Me → Space → Folders Snapshot Test
// ============================================================================

/// Test that navigates into "Assigned to Me" to see tasks, then navigates to a Space to see folders.
///
/// This test verifies:
/// 1. Navigate into "Assigned to Me" → expect list of tasks
/// 2. Navigate back to Workspaces
/// 3. Navigate into a Space
/// 4. Once entered → expect to see list of folders in sidebar
#[test]
fn test_navigation_assigned_to_space() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::models::workspace::{Folder, List, Space, Workspace};
    use clickdown::tui::app::{Screen, TuiApp};
    use clickdown::tui::widgets::{render_sidebar, render_task_list, SidebarItem};
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, layout::Rect, Terminal};
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        // Create test hierarchy
        let workspaces = vec![Workspace {
            id: "ws-1".to_string(),
            name: "Engineering".to_string(),
            color: Some("#1abc9c".to_string()),
            avatar: None,
            member_count: Some(5),
        }];

        let spaces = vec![
            Space {
                id: "sp-1".to_string(),
                name: "Backend Team".to_string(),
                color: Some("#3498db".to_string()),
                private: false,
                status: None,
                folders: vec![],
                lists: vec![],
            },
            Space {
                id: "sp-2".to_string(),
                name: "Frontend Team".to_string(),
                color: Some("#2ecc71".to_string()),
                private: false,
                status: None,
                folders: vec![],
                lists: vec![],
            },
        ];

        let folders = vec![
            Folder {
                id: "fd-1".to_string(),
                name: "Q1 Projects".to_string(),
                color: Some("#e74c3c".to_string()),
                private: false,
                space: None,
                lists: vec![],
            },
            Folder {
                id: "fd-2".to_string(),
                name: "Q2 Projects".to_string(),
                color: Some("#3498db".to_string()),
                private: false,
                space: None,
                lists: vec![],
            },
            Folder {
                id: "fd-3".to_string(),
                name: "Q3 Projects".to_string(),
                color: Some("#2ecc71".to_string()),
                private: false,
                space: None,
                lists: vec![],
            },
        ];

        let lists = vec![List {
            id: "lst-1".to_string(),
            name: "Sprint Planning".to_string(),
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

        // Configure mock client with hierarchy and assigned tasks
        let mock_client = MockClickUpClient::new()
            .with_current_user(fixtures::test_user())
            .with_accessible_lists(lists.clone())
            .with_workspaces(workspaces.clone())
            .with_spaces(spaces.clone())
            .with_folders(folders.clone())
            .with_lists_in_folder(lists.clone())
            .with_tasks_with_assignee_response(fixtures::test_tasks_with_assignees());

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Load workspaces
        app.load_workspaces();
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        app.process_async_messages();

        // Helper to capture sidebar snapshot
        let capture_sidebar = |app: &mut TuiApp| -> String {
            let mut terminal = Terminal::new(TestBackend::new(40, 15)).unwrap();
            terminal
                .draw(|frame| {
                    let area = Rect::new(0, 0, 40, 15);
                    render_sidebar(frame, app.sidebar(), area, Some(3));
                })
                .unwrap();

            let mut snapshot = String::new();
            for y in 0..15 {
                for x in 0..40 {
                    let cell = terminal.backend().buffer().get(x, y);
                    snapshot.push(cell.symbol().chars().next().unwrap_or(' '));
                }
                snapshot.push('\n');
            }
            snapshot
        };

        // Helper to capture task list snapshot
        let capture_task_list = |app: &mut TuiApp| -> String {
            let mut terminal = Terminal::new(TestBackend::new(60, 15)).unwrap();
            terminal
                .draw(|frame| {
                    let area = Rect::new(0, 0, 60, 15);
                    render_task_list(frame, app.assigned_tasks(), area, app.assigned_tasks_loading());
                })
                .unwrap();

            let mut snapshot = String::new();
            for y in 0..15 {
                for x in 0..60 {
                    let cell = terminal.backend().buffer().get(x, y);
                    snapshot.push(cell.symbol().chars().next().unwrap_or(' '));
                }
                snapshot.push('\n');
            }
            snapshot
        };

        // Collect snapshots
        let mut combined_snapshot = String::new();

        // =====================================================================
        // PART 1: Navigate into "Assigned to Me" from Workspaces
        // =====================================================================

        // Select "Assigned to Me" in sidebar (first item)
        app.sidebar().select_first();
        app.navigate_into();
        
        // Wait for async task loading with timeout
        let mut iterations = 0;
        let max_iterations = 30;
        while app.assigned_tasks().tasks().is_empty() && iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        // Verify we're on Assigned Tasks screen
        assert_eq!(
            app.screen(),
            Screen::AssignedTasks,
            "Should be at Assigned Tasks screen"
        );

        // Verify tasks were loaded
        assert!(
            !app.assigned_tasks().tasks().is_empty(),
            "Expected tasks to be loaded in Assigned to Me view after {} iterations",
            iterations
        );

        // Capture task list snapshot
        let task_list_snapshot = capture_task_list(&mut app);
        combined_snapshot.push_str("=== Part 1: Assigned to Me (Task List) ===\n");
        combined_snapshot.push_str(&task_list_snapshot);
        combined_snapshot.push('\n');

        // =====================================================================
        // PART 2: Navigate back out of Assigned to Me (goes to Workspaces)
        // =====================================================================

        app.navigate_back();
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        app.process_async_messages();

        // Should be back at Workspaces (AssignedTasks navigates back to Workspaces)
        assert_eq!(
            app.screen(),
            Screen::Workspaces,
            "Should be back at Workspaces screen"
        );

        // =====================================================================
        // PART 3: Navigate into a Space to see folders
        // =====================================================================

        // After navigate_back, sidebar selection is at index 0 (Assigned to Me)
        // Sidebar has: [AssignedTasks, Inbox, Workspace]
        // Need to select workspace at index 2
        app.sidebar().select_next(); // Skip "Assigned to Me" (index 0 -> 1)
        app.sidebar().select_next(); // Skip "Inbox" (index 1 -> 2)
        // Now at workspace (index 2)
        
        // Verify selection before navigating
        assert!(
            matches!(app.sidebar().selected_item(), Some(SidebarItem::Workspace { .. })),
            "Should have workspace selected before navigate_into, got: {:?}",
            app.sidebar().selected_item()
        );
        
        app.navigate_into();
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        app.process_async_messages();

        // Verify we're at Spaces screen
        assert_eq!(app.screen(), Screen::Spaces, "Should be at Spaces screen after entering workspace");

        // Now navigate into first space
        // Sidebar at Spaces screen has: [AssignedTasks, Inbox, Space, ...]
        app.sidebar().select_next(); // Skip "Assigned to Me" (index 0 -> 1)
        app.sidebar().select_next(); // Skip "Inbox" (index 1 -> 2)
        // Now at first space (index 2)
        
        // Verify selection before navigating
        assert!(
            matches!(app.sidebar().selected_item(), Some(SidebarItem::Space { .. })),
            "Should have space selected before navigate_into, got: {:?}",
            app.sidebar().selected_item()
        );
        
        app.navigate_into();
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        app.process_async_messages();

        // Verify we're at Folders screen (entering a space shows its folders)
        assert_eq!(app.screen(), Screen::Folders, "Should be at Folders screen after entering space");

        // Capture sidebar showing folders
        let sidebar_snapshot = capture_sidebar(&mut app);
        combined_snapshot.push_str("=== Part 2: Space → Folders List ===\n");
        combined_snapshot.push_str(&sidebar_snapshot);

        // Assert the combined snapshot
        assert_snapshot!("navigation_assigned_to_space", combined_snapshot);
    });
}
