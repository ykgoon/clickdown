//! Integration tests for assigned tasks loading with various hierarchy configurations
//!
//! These tests verify that the assigned tasks feature works correctly with different
//! workspace hierarchy structures and edge cases.

mod fixtures;

use clickdown::api::MockClickUpClient;
use clickdown::models::workspace::List;
use clickdown::tui::app::TuiApp;
use clickdown::tui::widgets::SidebarItem;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Test that assigned tasks load when lists are in folders
#[test]
fn test_assigned_tasks_from_folder_lists() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let lists = vec![List {
            id: "folder-list-1".to_string(),
            name: "Folder List".to_string(),
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
            .with_accessible_lists(lists.clone())
            .with_tasks_with_assignee_response(fixtures::test_tasks_with_assignees());

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        *app.sidebar().items_mut() = vec![SidebarItem::AssignedTasks];
        app.sidebar().select_first();
        app.navigate_into();

        // Wait for tasks to load
        let mut iterations = 0;
        let max_iterations = 30;
        while app.assigned_tasks().tasks().is_empty() && iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        assert!(
            app.assigned_tasks_error().is_none(),
            "Should load tasks from folder lists: {:?}",
            app.assigned_tasks_error()
        );
        assert_eq!(
            app.assigned_tasks().tasks().len(),
            3,
            "Should load all 3 test tasks"
        );
    });
}

/// Test that assigned tasks load when lists are folderless (directly in space)
#[test]
fn test_assigned_tasks_from_folderless_lists() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let lists = vec![List {
            id: "space-list-1".to_string(),
            name: "Space List (Folderless)".to_string(),
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
            .with_accessible_lists(lists.clone())
            .with_tasks_with_assignee_response(fixtures::test_tasks_with_assignees());

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        *app.sidebar().items_mut() = vec![SidebarItem::AssignedTasks];
        app.sidebar().select_first();
        app.navigate_into();

        let mut iterations = 0;
        let max_iterations = 30;
        while app.assigned_tasks().tasks().is_empty() && iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        assert!(
            app.assigned_tasks_error().is_none(),
            "Should load tasks from folderless lists: {:?}",
            app.assigned_tasks_error()
        );
        assert_eq!(
            app.assigned_tasks().tasks().len(),
            3,
            "Should load all 3 test tasks"
        );
    });
}

/// Test that assigned tasks handle empty lists gracefully with helpful error
#[test]
fn test_assigned_tasks_with_no_accessible_lists() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        // Mock client with NO accessible lists configured
        let mock_client = MockClickUpClient::new()
            .with_current_user(fixtures::test_user());
            // Note: NOT calling with_accessible_lists() - simulates empty hierarchy

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        *app.sidebar().items_mut() = vec![SidebarItem::AssignedTasks];
        app.sidebar().select_first();
        app.navigate_into();

        // Wait for error to be loaded
        let mut iterations = 0;
        let max_iterations = 30;
        while app.assigned_tasks_error().is_none() && iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        assert!(
            app.assigned_tasks_error().is_some(),
            "Should show error when no lists are accessible"
        );
        let error = app.assigned_tasks_error().unwrap();
        assert!(
            error.contains("No accessible lists found"),
            "Error should mention accessible lists: {}",
            error
        );
        assert!(
            error.contains("workspaces have no spaces"),
            "Error should provide helpful guidance: {}",
            error
        );
    });
}

/// Test that assigned tasks handle multiple lists correctly
#[test]
fn test_assigned_tasks_from_multiple_lists() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let lists = vec![
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
            List {
                id: "list-3".to_string(),
                name: "DevOps Tasks".to_string(),
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

        // Configure mock to return tasks from each list
        let mock_client = MockClickUpClient::new()
            .with_current_user(fixtures::test_user())
            .with_accessible_lists(lists.clone())
            .with_tasks_with_assignee_response(fixtures::test_tasks_with_assignees());

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        *app.sidebar().items_mut() = vec![SidebarItem::AssignedTasks];
        app.sidebar().select_first();
        app.navigate_into();

        let mut iterations = 0;
        let max_iterations = 30;
        while app.assigned_tasks().tasks().is_empty() && iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        assert!(
            app.assigned_tasks_error().is_none(),
            "Should load tasks from multiple lists: {:?}",
            app.assigned_tasks_error()
        );
        // Should get 3 tasks from each of 3 lists = 9 total
        assert_eq!(
            app.assigned_tasks().tasks().len(),
            9,
            "Should load tasks from all lists (3 tasks × 3 lists)"
        );
    });
}

/// Test that assigned tasks handle API errors gracefully
#[test]
fn test_assigned_tasks_with_api_error() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        use anyhow::anyhow;
        
        // Create mock client that returns an error for accessible lists
        let mut mock_client = MockClickUpClient::new();
        mock_client.accessible_lists_response = Some(Err(anyhow!("API rate limit exceeded")));
        mock_client.current_user_response = Some(Ok(fixtures::test_user()));

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        *app.sidebar().items_mut() = vec![SidebarItem::AssignedTasks];
        app.sidebar().select_first();
        app.navigate_into();

        // Wait for error to be loaded
        let mut iterations = 0;
        let max_iterations = 30;
        while app.assigned_tasks_error().is_none() && iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        assert!(
            app.assigned_tasks_error().is_some(),
            "Should show error when API fails"
        );
        let error = app.assigned_tasks_error().unwrap();
        assert!(
            error.contains("Failed to fetch lists"),
            "Error should indicate fetch failure: {}",
            error
        );
    });
}

/// Test that assigned tasks handle user ID not set
#[test]
fn test_assigned_tasks_without_user_id() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        // Mock client WITHOUT current user - simulates API failure
        use anyhow::anyhow;
        let mut mock_client = MockClickUpClient::new();
        mock_client.current_user_response = Some(Err(anyhow!("User endpoint not available")));

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        // Don't set current_user_id - simulates user navigating to Assigned Tasks
        // without first opening a task

        *app.sidebar().items_mut() = vec![SidebarItem::AssignedTasks];
        app.sidebar().select_first();
        app.navigate_into();

        // Wait for error to be loaded from async user fetch
        let mut iterations = 0;
        let max_iterations = 30;
        while app.assigned_tasks_error().is_none() && iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        // Should show error when user fetch fails
        assert!(
            app.assigned_tasks_error().is_some(),
            "Should show error when user ID cannot be fetched"
        );
        let error = app.assigned_tasks_error().unwrap();
        assert!(
            error.contains("User endpoint") || error.contains("user"),
            "Error should mention user: {}",
            error
        );
    });
}

/// Test that assigned tasks loading state is set correctly
#[test]
fn test_assigned_tasks_loading_state() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
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
            .with_accessible_lists(lists.clone())
            .with_tasks_with_assignee_response(fixtures::test_tasks_with_assignees());

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        *app.sidebar().items_mut() = vec![SidebarItem::AssignedTasks];
        app.sidebar().select_first();

        // Before navigation, loading should be false
        assert!(!app.assigned_tasks_loading(), "Should not be loading before navigation");

        app.navigate_into();

        // Immediately after navigation, loading should be true
        assert!(app.assigned_tasks_loading(), "Should be loading after navigation");

        // Wait for tasks to load
        let mut iterations = 0;
        let max_iterations = 30;
        while app.assigned_tasks().tasks().is_empty() && iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        // Give one more iteration for the final message to be processed
        app.process_async_messages();
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        // After loading completes, loading should be false
        assert!(!app.assigned_tasks_loading(), "Should stop loading after completion");
    });
}
