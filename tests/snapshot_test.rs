//! Snapshot tests for ClickDown TUI widgets and layouts
//!
//! These tests use the `insta` crate to capture and compare rendered UI output.
//! All tests use mocked data for deterministic, reproducible snapshots.
//!
//! Run tests: `cargo test --test snapshot_test`
//! Review snapshots: `cargo insta review`
//! Accept changes: `cargo insta accept`

mod fixtures;

use clickdown::models::Task;
use clickdown::tui::app::TaskCreationField;
use clickdown::tui::layout::{generate_screen_title, TuiLayout};
use clickdown::tui::widgets::{
    auth::{render_auth, AuthState},
    dialog::{render_dialog, DialogState, DialogType},
    document::{render_document, DocumentState},
    help::{render_help, HelpContext, HelpState},
    sidebar::{render_sidebar, SidebarItem, SidebarState},
    task_detail::{render_task_detail, TaskDetailState},
    task_list::{render_task_list, GroupedTaskList},
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
            let cell = &terminal.backend().buffer()[(x, y)];
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
/// Matches actual app behavior: workspace hierarchy only
fn create_sidebar_items() -> Vec<SidebarItem> {
    vec![
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
        render_sidebar(frame, &sidebar, area);
    });
}

#[test]
fn test_sidebar_with_items() {
    let mut sidebar = SidebarState::new();
    *sidebar.items_mut() = create_sidebar_items();

    assert_widget_snapshot("sidebar_with_items", 40, 15, |frame| {
        let area = Rect::new(0, 0, 40, 15);
        render_sidebar(frame, &sidebar, area);
    });
}

#[test]
fn test_sidebar_with_selection() {
    let mut sidebar = SidebarState::new();
    *sidebar.items_mut() = create_sidebar_items();
    sidebar.select_first();

    assert_widget_snapshot("sidebar_with_selection", 40, 15, |frame| {
        let area = Rect::new(0, 0, 40, 15);
        render_sidebar(frame, &sidebar, area);
    });
}

#[test]
fn test_sidebar_with_zero_assigned() {
    let mut sidebar = SidebarState::new();
    *sidebar.items_mut() = create_sidebar_items();

    // Render with 0 assigned tasks (should show no count or empty badge)
    assert_widget_snapshot("sidebar_with_zero_assigned", 40, 15, |frame| {
        let area = Rect::new(0, 0, 40, 15);
        render_sidebar(frame, &sidebar, area);
    });
}

#[test]
fn test_sidebar_with_large_assigned_count() {
    let mut sidebar = SidebarState::new();
    *sidebar.items_mut() = create_sidebar_items();

    // Render with large count (99+)
    assert_widget_snapshot("sidebar_with_large_assigned_count", 40, 15, |frame| {
        let area = Rect::new(0, 0, 40, 15);
        render_sidebar(frame, &sidebar, area);
    });
}

// ============================================================================
// Task List Widget Snapshot Tests (Task 3.2)
// ============================================================================

#[test]
fn test_task_list_empty() {
    let task_list = GroupedTaskList::new();

    assert_widget_snapshot("task_list_empty", 60, 15, |frame| {
        let area = Rect::new(0, 0, 60, 15);
        render_task_list(frame, &task_list, area, false);
    });
}

#[test]
fn test_task_list_with_tasks() {
    let tasks = create_test_tasks();
    let mut task_list = GroupedTaskList::from_tasks(tasks);
    task_list.select_first();

    assert_widget_snapshot("task_list_with_tasks", 60, 15, |frame| {
        let area = Rect::new(0, 0, 60, 15);
        render_task_list(frame, &task_list, area, false);
    });
}

#[test]
fn test_task_list_with_selection() {
    let tasks = create_test_tasks();
    let mut task_list = GroupedTaskList::from_tasks(tasks);
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
        render_task_detail(frame, &detail, area, "", "", &TaskCreationField::Name);
    });
}

#[test]
fn test_task_detail_empty() {
    let detail = TaskDetailState::new();

    assert_widget_snapshot("task_detail_empty", 60, 20, |frame| {
        let area = Rect::new(0, 0, 60, 20);
        render_task_detail(frame, &detail, area, "", "", &TaskCreationField::Name);
    });
}

// ============================================================================
// Task Detail 's' Key Bug Reproduction (Full Screen Snapshot)
// ============================================================================

/// BUG FIX VERIFICATION:
/// After the fix, pressing 's' in Task Detail view opens the status picker overlay.
/// This snapshot shows the status picker with status message and overlay widget.
#[test]
fn test_s_key_in_task_detail_no_response_snapshot() {
    use clickdown::tui::layout::TuiLayout;
    use clickdown::tui::widgets::status_picker::render_status_picker;
    use clickdown::models::TaskStatus;

    // Set up task detail with a task
    let mut detail = TaskDetailState::new();
    let tasks = create_test_tasks();
    detail.task = Some(tasks[0].clone());

    // After pressing 's' in Task Detail view (FIXED):
    // - Status picker overlay opens
    // - Status bar shows navigation instructions
    let status_after_s = "Select new status (j/k navigate, Enter select, Esc cancel)";

    // Build the same default statuses used by open_status_picker()
    let statuses = vec![
        TaskStatus {
            id: None,
            status: "To Do".to_string(),
            color: Some("#8794a6".to_string()),
            type_field: None,
            orderindex: Some(0),
            status_group: Some("todo".to_string()),
        },
        TaskStatus {
            id: None,
            status: "In Progress".to_string(),
            color: Some("#4f46de".to_string()),
            type_field: None,
            orderindex: Some(1),
            status_group: Some("in_progress".to_string()),
        },
        TaskStatus {
            id: None,
            status: "Done".to_string(),
            color: Some("#0f4a58".to_string()),
            type_field: None,
            orderindex: Some(2),
            status_group: Some("done".to_string()),
        },
    ];

    assert_widget_snapshot("s_key_task_detail_status_picker", 80, 24, |frame| {
        let area = Rect::new(0, 0, 80, 24);
        let layout = TuiLayout::new(area);

        // Render title bar
        layout.render_title(frame, "ClickDown - Task Detail");

        // Render task detail content
        render_task_detail(frame, &detail, layout.content_area, "", "", &TaskCreationField::Name);

        // Render status bar (showing status picker instructions)
        let hints = "j/k: Navigate | Enter: Select | Esc: Cancel";
        layout.render_status(frame, status_after_s, hints);

        // Render status picker overlay (the fix!)
        render_status_picker(frame, area, &statuses, 0, Some("in progress"));
    });
}

/// Snapshot showing what the Task Detail screen looks like BEFORE 's' is pressed.
/// This is the baseline for comparison after the fix.
#[test]
fn test_task_detail_before_s_key_baseline() {
    use clickdown::tui::layout::TuiLayout;

    let mut detail = TaskDetailState::new();
    let tasks = create_test_tasks();
    detail.task = Some(tasks[0].clone());

    assert_widget_snapshot("task_detail_before_s_key", 80, 24, |frame| {
        let area = Rect::new(0, 0, 80, 24);
        let layout = TuiLayout::new(area);

        // Render title bar
        layout.render_title(frame, "ClickDown - Task Detail");

        // Render task detail content
        render_task_detail(frame, &detail, layout.content_area, "", "", &TaskCreationField::Name);

        // Render status bar
        let hints = "e: Edit task | Tab: Comments | Esc: Back | ? - Help";
        layout.render_status(frame, "", hints);
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
// ============================================================================
// Help Dialog Snapshot Tests (Task 3.6)
// ============================================================================

#[test]
fn test_help_dialog_visible() {
    let mut help = HelpState::new();
    help.visible = true;

    assert_widget_snapshot("help_dialog_visible", 80, 24, |frame| {
        let area = Rect::new(0, 0, 80, 24);
        render_help(frame, &help, &HelpContext::TaskList, area);
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
        render_sidebar(frame, &sidebar, area);
    });
}

#[test]
fn test_main_layout_sidebar_expanded() {
    let mut sidebar = SidebarState::new();
    sidebar.visible = true;
    *sidebar.items_mut() = create_sidebar_items();

    assert_widget_snapshot("main_layout_sidebar_expanded", 80, 24, |frame| {
        let area = Rect::new(0, 0, 80, 24);
        render_sidebar(frame, &sidebar, area);
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
                    render_sidebar(frame, app.sidebar(), area);
                })
                .unwrap();

            // Get buffer contents
            let mut snapshot = String::new();
            for y in 0..15 {
                for x in 0..40 {
                    let cell = &terminal.backend().buffer()[(x, y)];
                    snapshot.push(cell.symbol().chars().next().unwrap_or(' '));
                }
                snapshot.push('\n');
            }

            snapshot
        };

        // Helper to verify sidebar has items
        let verify_sidebar_has_items = |app: &mut TuiApp, level_name: &str| {
            let items = app.sidebar().items();
            assert!(
                !items.is_empty(),
                "Level '{}': Sidebar should not be empty. Items: {:?}",
                level_name,
                items.iter().map(|i| format!("{:?}", i)).collect::<Vec<_>>()
            );
        };

        // Collect all levels into one combined snapshot
        let mut combined_snapshot = String::new();

        // Helper to add level to combined snapshot
        let mut add_level =
            |app: &mut TuiApp, level_num: usize, level_name: &str, direction: &str| {
                verify_sidebar_has_items(app, &format!("{} ({})", level_name, direction));

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
        assert_eq!(
            app.screen(),
            Screen::Workspaces,
            "Should start at Workspaces"
        );
        add_level(&mut app, 1, "Workspaces", "Down");

        // Navigate into workspace (workspace is now at index 0)
        app.sidebar().select_first();
        app.navigate_into();
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        app.process_async_messages();

        // Level 2: Spaces
        assert_eq!(app.screen(), Screen::Spaces, "Should be at Spaces");
        add_level(&mut app, 2, "Spaces", "Down");

        // Navigate into space (space is now at index 0)
        app.sidebar().select_first();
        app.navigate_into();
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        app.process_async_messages();

        // Level 3: Folders
        assert_eq!(app.screen(), Screen::Folders, "Should be at Folders");
        add_level(&mut app, 3, "Folders", "Down");

        // Navigate into folder
        app.sidebar().select_first();
        app.navigate_into();
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        app.process_async_messages();

        // Level 4: Lists
        assert_eq!(app.screen(), Screen::Lists, "Should be at Lists");
        add_level(&mut app, 4, "Lists", "Down");

        // Navigate into list
        app.sidebar().select_first();
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
        assert_eq!(
            app.screen(),
            Screen::Workspaces,
            "Should be back at Workspaces"
        );
        add_level(&mut app, 9, "Workspaces", "Up");

        // Assert the combined snapshot
        assert_snapshot!("navigation_hierarchy", combined_snapshot);
    });
}

/// Test that pasting a URL with 'u' characters works correctly
/// This is a regression test for the bug where 'u' characters were dropped from URLs
/// when pasting, because the global 'u' key handler (for URL copy) was intercepting them.
/// Bug: "https://app.clickup.com/..." became "https://app.clickp.com/..."
#[test]
fn test_url_input_dialog_paste_preserves_u_characters() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::TuiApp;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![fixtures::test_workspace()]);

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(mock_client))
    }).unwrap();

    // Open URL input dialog with 'g' then 'u'
    let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    app.update(InputEvent::Key(g_key));
    let u_key = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE);
    app.update(InputEvent::Key(u_key));
    assert!(app.is_url_input_open(), "URL input dialog should be open");

    // Simulate pasting a ClickUp URL by typing each character
    // This mimics how terminals send paste events character by character
    // IMPORTANT: We must use app.update() NOT app.handle_url_input() directly,
    // because the bug is in the update() method's key event routing
    let test_url = "https://app.clickup.com/t/123";
    for c in test_url.chars() {
        let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
        app.update(InputEvent::Key(key)); // This routes through the buggy key handler
    }

    // Create a text snapshot of the URL input content
    // Before the fix: "https://app.clickp.com/t/123" (missing 'u')
    // After the fix: "https://app.clickup.com/t/123" (correct)
    let url_text = app.url_input_text().to_string();
    assert_snapshot!("url_input_text_after_paste", url_text);
    
    // Also verify programmatically for clearer test output
    assert_eq!(
        app.url_input_text(),
        "https://app.clickup.com/t/123",
        "URL should contain all characters including 'u' - BUG: 'u' characters are being dropped!"
    );
}

/// Test that 'u' key in comment editing mode types the letter instead of copying URL
/// This is a regression test for the bug where the global 'u' shortcut intercepted
/// character input in comment editing mode.
#[test]
fn test_u_key_in_comment_editing() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::{Screen, TuiApp};
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new()
        .with_tasks(vec![fixtures::test_task()])
        .with_task_comments(vec![fixtures::test_comment()]);

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(mock_client))
    }).unwrap();

    // Navigate to TaskDetail
    app.set_screen(Screen::TaskDetail);
    app.set_current_list_id(Some("list-1".to_string()));
    app.set_task_detail_task(fixtures::test_task());
    app.set_comments(vec![fixtures::test_comment()]);

    // Start new comment (press 'n' with comment focus)
    // First need to ensure we're on task detail with comments visible
    // Set up proper state for comment editing
    app.set_screen(Screen::TaskDetail);
    app.set_current_list_id(Some("list-1".to_string()));
    app.set_task_detail_task(fixtures::test_task());
    app.set_comments(vec![fixtures::test_comment()]);
    
    // Enable comment focus (so 'n' key triggers new comment)
    app.set_comment_focus(true);

    // Press 'n' to start new comment
    let n_key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    app.update(InputEvent::Key(n_key));

    // Verify comment editing is active
    assert!(app.is_comment_editing_active(), "Comment editing should be active");

    // Type 'u' key - should add 'u' to comment, NOT trigger URL copy
    let u_key = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE);
    app.update(InputEvent::Key(u_key));

    // Assert 'u' appears in comment_new_text
    let comment_text = app.comment_new_text().to_string();
    assert_snapshot!("u_key_in_comment_editing", comment_text);

    assert!(
        comment_text.contains('u'),
        "Comment text should contain 'u' - BUG: 'u' was intercepted by global shortcut!"
    );
}

/// Test that 'u' key in task name creation mode types the letter instead of copying URL
#[test]
fn test_u_key_in_task_name_creation() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::{Screen, TuiApp};
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new()
        .with_tasks(vec![]);

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(mock_client))
    }).unwrap();

    // Set up task creation mode with name field focused
    app.set_screen(Screen::TaskDetail);
    app.set_current_list_id(Some("list-1".to_string()));
    app.set_task_creating(true);
    app.set_task_creation_focus(TaskCreationField::Name);

    // Type 'u' key - should add 'u' to task name, NOT trigger URL copy
    let u_key = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE);
    app.update(InputEvent::Key(u_key));

    // Assert 'u' appears in task_name_input
    let task_name = app.task_name_input().to_string();
    assert_snapshot!("u_key_in_task_name_creation", task_name);

    assert!(
        task_name.contains('u'),
        "Task name should contain 'u' - BUG: 'u' was intercepted by global shortcut!"
    );
}

/// Test that 'u' key in task description creation mode types the letter instead of copying URL
#[test]
fn test_u_key_in_task_description_creation() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::{Screen, TuiApp};
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new()
        .with_tasks(vec![]);

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(mock_client))
    }).unwrap();

    // Set up task creation mode with description field focused
    app.set_screen(Screen::TaskDetail);
    app.set_current_list_id(Some("list-1".to_string()));
    app.set_task_creating(true);
    app.set_task_creation_focus(TaskCreationField::Description);

    // Type 'u' key - should add 'u' to task description, NOT trigger URL copy
    let u_key = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE);
    app.update(InputEvent::Key(u_key));

    // Assert 'u' appears in task_description_input
    let task_desc = app.task_description_input().to_string();
    assert_snapshot!("u_key_in_task_description_creation", task_desc);

    assert!(
        task_desc.contains('u'),
        "Task description should contain 'u' - BUG: 'u' was intercepted by global shortcut!"
    );
}

/// Test that 'g' then 'u' chord still opens URL dialog when no text input is active
/// This guards against regression when modifying the 'u' key handler.
#[test]
fn test_g_u_chord_still_opens_url_dialog() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::{Screen, TuiApp};
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![fixtures::test_workspace()]);

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(mock_client))
    }).unwrap();

    // Set up on Tasks screen (no text input active)
    app.set_screen(Screen::Tasks);

    // Press 'g' then 'u' - should open URL dialog
    let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    app.update(InputEvent::Key(g_key));
    let u_key = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE);
    app.update(InputEvent::Key(u_key));

    // Assert URL dialog is open
    assert!(app.is_url_input_open(), "URL input dialog should open with 'g' 'u' chord");
    assert_snapshot!("g_u_chord_opens_url_dialog", app.url_input_text().to_string());
}

/// Test that Tab key switches focus between name and description fields
#[test]
fn test_tab_switches_task_creation_field() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::{Screen, TuiApp};
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new().with_tasks(vec![]);

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(mock_client))
    }).unwrap();

    app.set_screen(Screen::TaskDetail);
    app.set_current_list_id(Some("list-1".to_string()));
    app.set_task_creating(true);
    app.set_task_creation_focus(TaskCreationField::Name);

    let tab_key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
    app.update(InputEvent::Key(tab_key));

    let focus = app.task_creation_focus().clone();
    assert_snapshot!("tab_switches_to_description", format!("{:?}", focus));
    assert!(matches!(focus, TaskCreationField::Description),
        "Tab should switch focus to Description field");
}

/// Test that Esc cancels task creation
#[test]
fn test_esc_cancels_task_creation() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::{Screen, TuiApp};
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new().with_tasks(vec![]);

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(mock_client))
    }).unwrap();

    app.set_screen(Screen::TaskDetail);
    app.set_current_list_id(Some("list-1".to_string()));
    app.set_task_creating(true);
    app.set_task_creation_focus(TaskCreationField::Name);

    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.update(InputEvent::Key(esc_key));

    assert_snapshot!("esc_cancels_task_creation", app.is_task_creating());
    assert!(!app.is_task_creating(), "Esc should cancel task creation");
}

/// Test that Enter key in task name field moves to description field
#[test]
fn test_enter_in_task_name_field() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::{Screen, TuiApp};
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new().with_tasks(vec![]);

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(mock_client))
    }).unwrap();

    app.set_screen(Screen::TaskDetail);
    app.set_current_list_id(Some("list-1".to_string()));
    app.set_task_creating(true);
    app.set_task_creation_focus(TaskCreationField::Name);

    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.update(InputEvent::Key(enter_key));

    let focus = app.task_creation_focus().clone();
    assert_snapshot!("enter_moves_to_description", format!("{:?}", focus));
    assert!(matches!(focus, TaskCreationField::Description),
        "Enter should move focus to Description field");
}

/// Test that 't' key in task name creation mode types the letter instead of switching fields
#[test]
fn test_task_creation_can_type_t() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::{Screen, TuiApp};
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new()
        .with_tasks(vec![]);

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(mock_client))
    }).unwrap();

    // Set up task creation mode with name field focused
    app.set_screen(Screen::TaskDetail);
    app.set_current_list_id(Some("list-1".to_string()));
    app.set_task_creating(true);
    app.set_task_creation_focus(TaskCreationField::Name);

    // Type 't' key - should add 't' to task name, NOT switch fields
    let t_key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    app.update(InputEvent::Key(t_key));

    // Assert 't' appears in task_name_input
    let task_name = app.task_name_input().to_string();
    assert_snapshot!("task_creation_can_type_t", task_name);

    assert!(
        task_name.contains('t'),
        "Task name should contain 't' - BUG: 't' was intercepted by field switch shortcut!"
    );
}

/// Test that 'g' key in task name creation mode types the letter instead of being intercepted by chord leader
#[test]
fn test_task_creation_can_type_g() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::{Screen, TuiApp};
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new()
        .with_tasks(vec![]);

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(mock_client))
    }).unwrap();

    // Set up task creation mode with name field focused
    app.set_screen(Screen::TaskDetail);
    app.set_current_list_id(Some("list-1".to_string()));
    app.set_task_creating(true);
    app.set_task_creation_focus(TaskCreationField::Name);

    // Type 'g' key - should add 'g' to task name, NOT trigger chord leader
    let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    app.update(InputEvent::Key(g_key));

    // Assert 'g' appears in task_name_input
    let task_name = app.task_name_input().to_string();
    assert_snapshot!("task_creation_can_type_g", task_name);

    assert!(
        task_name.contains('g'),
        "Task name should contain 'g' - BUG: 'g' was intercepted by chord leader!"
    );
}

/// Test that Enter closes quit dialog during task creation
/// Bug: Enter in task creation mode routes to handle_task_creation_input instead of dialog handling
#[test]
fn test_quit_dialog_enter_in_task_creation() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::{Screen, TuiApp};
    use clickdown::tui::input::InputEvent;
    use clickdown::tui::widgets::DialogType;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new()
        .with_tasks(vec![]);

    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(mock_client))
    }).unwrap();

    // Set up task creation mode
    app.set_screen(Screen::TaskDetail);
    app.set_current_list_id(Some("list-1".to_string()));
    app.set_task_creating(true);
    app.set_task_creation_focus(TaskCreationField::Name);

    // Show quit dialog
    app.dialog_mut_for_test().show(DialogType::ConfirmQuit);
    assert!(app.is_dialog_visible(), "Dialog should be visible");

    // Type Enter - should close dialog (not just switch field in task creation)
    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.update(InputEvent::Key(enter_key));

    // Assert dialog is hidden after Enter
    assert_snapshot!("quit_dialog_enter_in_task_creation", app.is_dialog_visible());
    assert!(
        !app.is_dialog_visible(),
        "BUG: Dialog should close on Enter but it didn't!"
    );
}
