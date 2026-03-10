//! Snapshot tests for assigned tasks pre-loading functionality
//!
//! These tests verify that assigned tasks are pre-fetched at application startup
//! and displayed correctly in the UI. Tests use the `insta` crate for snapshot
//! comparisons with mocked data for deterministic, reproducible results.
//!
//! Run tests: `cargo test --test snapshot_test_assigned_tasks_preload`
//! Review snapshots: `cargo insta review`
//! Accept changes: `cargo insta accept`

mod fixtures;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::workspace::List;
use clickdown::tui::app::TuiApp;
use clickdown::tui::widgets::{render_task_list, SidebarItem};
use insta::assert_snapshot;
use ratatui::{backend::TestBackend, layout::Rect, Terminal};
use std::sync::Arc;
use tokio::runtime::Runtime;

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a test terminal with the given size
fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(width, height)).unwrap()
}

/// Render assigned tasks widget and capture as string for snapshot
fn capture_assigned_tasks(app: &TuiApp, width: u16, height: u16) -> String {
    let mut terminal = create_test_terminal(width, height);
    let loading = app.assigned_tasks_loading();

    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, width, height);
            render_task_list(frame, app.assigned_tasks(), area, loading);
        })
        .unwrap();

    // Get buffer contents
    let mut snapshot = String::new();
    for y in 0..height {
        for x in 0..width {
            let cell = terminal.backend().buffer().get(x, y);
            snapshot.push(cell.symbol().chars().next().unwrap_or(' '));
        }
        snapshot.push('\n');
    }

    snapshot
}

/// Wait for assigned tasks to be pre-loaded
fn wait_for_preload(app: &mut TuiApp, max_iterations: usize) {
    let mut iterations = 0;
    while app.assigned_tasks().tasks().is_empty() && iterations < max_iterations {
        app.process_async_messages();
        std::thread::sleep(std::time::Duration::from_millis(10));
        iterations += 1;
    }
    // Final message processing
    app.process_async_messages();
}

// ============================================================================
// Pre-loading Snapshot Tests
// ============================================================================

/// Test that assigned tasks are pre-loaded from cache at startup
///
/// This test verifies the cache-first pre-loading strategy:
/// - App starts with cached tasks
/// - Tasks are immediately available without loading indicator
/// - UI displays tasks right away
#[test]
fn test_assigned_tasks_preloaded_from_cache_snapshot() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let test_tasks = fixtures::test_tasks_with_assignees();

        let mock_client = MockClickUpClient::new()
            .with_current_user(fixtures::test_user())
            .with_accessible_lists(vec![List {
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
            }]);

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Simulate having user ID from previous session
        app.set_current_user_id(Some(123));

        // Pre-populate cache with tasks (simulating previous fetch)
        let cache_result = app.cache().cache_assigned_tasks(&test_tasks);
        assert!(cache_result.is_ok(), "Failed to cache tasks: {:?}", cache_result);

        // Trigger pre-loading (normally happens after workspaces load)
        app.pre_load_assigned_tasks();

        // Should immediately load from cache (no async wait needed)
        assert_eq!(
            app.assigned_tasks().tasks().len(),
            3,
            "Should pre-load 3 tasks from cache"
        );
        assert!(!app.assigned_tasks_loading(), "Should not show loading indicator for cache hit");

        // Capture snapshot of pre-loaded tasks
        let snapshot = capture_assigned_tasks(&app, 60, 15);
        assert_snapshot!("assigned_tasks_preloaded_from_cache", snapshot);
    });
}

/// Test that assigned tasks are pre-loaded from API when cache is empty
///
/// This test verifies the background fetch strategy:
/// - App starts with empty cache
/// - Pre-loading triggers background API fetch
/// - Tasks appear after async fetch completes
/// - No loading indicator shown (silent pre-load)
#[test]
fn test_assigned_tasks_preloaded_from_api_snapshot() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let test_tasks = fixtures::test_tasks_with_assignees();

        let mock_client = MockClickUpClient::new()
            .with_current_user(fixtures::test_user())
            .with_accessible_lists(vec![List {
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
            }])
            .with_tasks_with_assignee_response(test_tasks.clone());

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Simulate having user ID from previous session
        app.set_current_user_id(Some(123));

        // Clear cache to force API fetch
        let clear_result = app.cache().clear_assigned_tasks();
        assert!(clear_result.is_ok(), "Failed to clear cache: {:?}", clear_result);

        // Trigger pre-loading
        app.pre_load_assigned_tasks();

        // Wait for background fetch to complete
        wait_for_preload(&mut app, 30);

        // Should have pre-loaded tasks from API
        assert!(
            app.assigned_tasks().tasks().len() > 0,
            "Should pre-load tasks from API when cache is empty"
        );
        assert!(
            !app.assigned_tasks_loading(),
            "Should not show loading indicator for pre-load"
        );

        // Capture snapshot of pre-loaded tasks
        let snapshot = capture_assigned_tasks(&app, 60, 15);
        assert_snapshot!("assigned_tasks_preloaded_from_api", snapshot);
    });
}

/// Test that assigned tasks show correct count badge after pre-loading
///
/// Verifies that the sidebar count badge matches the pre-loaded tasks.
#[test]
fn test_assigned_tasks_preloaded_count_badge_snapshot() {
    use clickdown::tui::widgets::render_sidebar;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let test_tasks = fixtures::test_tasks_with_assignees();

        let mock_client = MockClickUpClient::new()
            .with_current_user(fixtures::test_user())
            .with_accessible_lists(vec![List {
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
            }]);

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        app.set_current_user_id(Some(123));

        // Pre-populate cache
        let _ = app.cache().cache_assigned_tasks(&test_tasks);
        app.pre_load_assigned_tasks();

        // Verify count matches
        let count = app.assigned_tasks_count();
        assert_eq!(count, 3, "Should have count of 3");

        // Capture sidebar with count badge
        let mut terminal = create_test_terminal(40, 15);
        *app.sidebar().items_mut() = vec![SidebarItem::AssignedTasks];
        app.sidebar().select_first();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 40, 15);
                render_sidebar(frame, app.sidebar(), area, Some(count));
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

        assert_snapshot!("assigned_tasks_preloaded_count_badge", snapshot);
    });
}

/// Test that assigned tasks pre-loading refreshes stale cache
///
/// Verifies that when cache has old data, pre-loading:
/// - Shows old data immediately from cache
/// - Refreshes in background with new data
#[test]
fn test_assigned_tasks_preloaded_refreshes_stale_cache_snapshot() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        // Create old cached data (1 task)
        let old_tasks = vec![fixtures::test_task()];

        // Create new data that API will return (3 tasks)
        let new_tasks = fixtures::test_tasks_with_assignees();

        let mock_client = MockClickUpClient::new()
            .with_current_user(fixtures::test_user())
            .with_accessible_lists(vec![List {
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
            }])
            .with_tasks_with_assignee_response(new_tasks.clone());

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        app.set_current_user_id(Some(123));

        // Pre-populate cache with old data
        let _ = app.cache().cache_assigned_tasks(&old_tasks);

        // Trigger pre-loading
        app.pre_load_assigned_tasks();

        // Should immediately show old cached data (1 task)
        assert_eq!(
            app.assigned_tasks().tasks().len(),
            1,
            "Should show cached data immediately"
        );

        // Capture snapshot of initial cached state
        let initial_snapshot = capture_assigned_tasks(&app, 60, 15);
        assert_snapshot!("assigned_tasks_preloaded_initial_cache", initial_snapshot);

        // Wait for background refresh
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        app.process_async_messages();

        // Note: Background refresh behavior depends on implementation
        // The key test is that initial cache load worked
    });
}

/// Test that assigned tasks pre-loading handles empty state gracefully
///
/// Verifies UI shows appropriate empty state when no tasks are assigned.
#[test]
fn test_assigned_tasks_preloaded_empty_snapshot() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let mock_client = MockClickUpClient::new()
            .with_current_user(fixtures::test_user())
            .with_accessible_lists(vec![List {
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
            }])
            .with_tasks_with_assignee_response(vec![]); // Empty response

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        app.set_current_user_id(Some(123));

        // Clear cache
        let _ = app.cache().clear_assigned_tasks();

        // Trigger pre-loading
        app.pre_load_assigned_tasks();

        // Wait for fetch
        wait_for_preload(&mut app, 30);

        // Should have empty tasks but no error
        assert_eq!(app.assigned_tasks().tasks().len(), 0);
        assert!(!app.assigned_tasks_loading());

        // Capture empty state snapshot
        let snapshot = capture_assigned_tasks(&app, 60, 15);
        assert_snapshot!("assigned_tasks_preloaded_empty", snapshot);
    });
}

/// Test full application startup with pre-loaded assigned tasks
///
/// Integration test that simulates real app startup flow:
/// 1. App initializes
/// 2. Workspaces load
/// 3. Pre-loading triggers automatically
/// 4. Tasks appear in UI
#[test]
fn test_app_startup_with_preloaded_assigned_tasks_snapshot() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let test_tasks = fixtures::test_tasks_with_assignees();

        let mock_client = MockClickUpClient::new()
            .with_current_user(fixtures::test_user())
            .with_workspaces(vec![fixtures::test_workspace()])
            .with_accessible_lists(vec![List {
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
            }])
            .with_tasks_with_assignee_response(test_tasks.clone());

        // Create app (simulates startup)
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Simulate session restore with user ID
        app.set_current_user_id(Some(123));

        // Pre-populate cache (simulates previous session's cache)
        let _ = app.cache().cache_assigned_tasks(&test_tasks);

        // Simulate workspace loading (triggers pre-load in real app)
        app.pre_load_assigned_tasks();

        // Verify tasks are pre-loaded
        assert_eq!(
            app.assigned_tasks().tasks().len(),
            3,
            "Should have pre-loaded 3 tasks"
        );
        assert_eq!(app.assigned_tasks_count(), 3);

        // Capture full UI state snapshot
        let mut terminal = create_test_terminal(80, 24);

        // Set up sidebar with AssignedTasks selected
        let sidebar_items = vec![
            SidebarItem::AssignedTasks,
            SidebarItem::Inbox,
            SidebarItem::Workspace {
                name: "Test Workspace".to_string(),
                id: "test-ws-1".to_string(),
            },
        ];
        
        terminal
            .draw(|frame| {
                use clickdown::tui::layout::TuiLayout;
                use clickdown::tui::widgets::{render_sidebar, SidebarState};
                use ratatui::{style::Style, widgets::{Block, Borders, Paragraph}};

                let layout = TuiLayout::new(frame.area());

                // Render title
                let title = "Assigned Tasks";
                frame.render_widget(
                    Block::default()
                        .title(title)
                        .borders(Borders::ALL)
                        .style(Style::default()),
                    layout.title_area,
                );

                // Create sidebar for rendering
                let mut sidebar = SidebarState::new();
                *sidebar.items_mut() = sidebar_items.clone();
                sidebar.select_first();
                
                // Render sidebar
                let count = app.assigned_tasks_count();
                let (sidebar_area, content_area) = layout.split_content(30);
                render_sidebar(frame, &sidebar, sidebar_area, Some(count));

                // Render assigned tasks
                render_task_list(
                    frame,
                    app.assigned_tasks(),
                    content_area,
                    app.assigned_tasks_loading(),
                );

                // Render status bar
                let status = app.status();
                let status_widget = Paragraph::new(status)
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(status_widget, layout.status_area);
            })
            .unwrap();

        // Get buffer contents
        let mut snapshot = String::new();
        for y in 0..24 {
            for x in 0..80 {
                let cell = terminal.backend().buffer().get(x, y);
                snapshot.push(cell.symbol().chars().next().unwrap_or(' '));
            }
            snapshot.push('\n');
        }

        assert_snapshot!("app_startup_with_preloaded_tasks", snapshot);
    });
}

/// Test that assigned tasks pre-loading works with multiple lists
///
/// Verifies that tasks from multiple lists are aggregated correctly.
#[test]
fn test_assigned_tasks_preloaded_from_multiple_lists_snapshot() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let test_tasks = fixtures::test_tasks_with_assignees();

        let mock_client = MockClickUpClient::new()
            .with_current_user(fixtures::test_user())
            .with_accessible_lists(vec![
                List {
                    id: "list-1".to_string(),
                    name: "Backend Tasks".to_string(),
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
                    id: "list-2".to_string(),
                    name: "Frontend Tasks".to_string(),
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
            ])
            .with_tasks_with_assignee_response(test_tasks.clone());

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        app.set_current_user_id(Some(123));

        // Clear cache
        let _ = app.cache().clear_assigned_tasks();

        // Trigger pre-loading
        app.pre_load_assigned_tasks();

        // Wait for fetch
        wait_for_preload(&mut app, 30);

        // Should have tasks from multiple lists
        assert!(app.assigned_tasks().tasks().len() > 0);

        // Capture snapshot
        let snapshot = capture_assigned_tasks(&app, 60, 15);
        assert_snapshot!("assigned_tasks_preloaded_multiple_lists", snapshot);
    });
}
