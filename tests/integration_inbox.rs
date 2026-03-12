//! Integration tests for inbox/activity feed feature
//!
//! These tests verify that the inbox feature works correctly with mock API data
//! and handles various scenarios like loading, errors, and user interactions.

mod fixtures;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::workspace::Workspace;
use clickdown::tui::app::TuiApp;
use clickdown::tui::input::InputEvent;
use clickdown::tui::widgets::SidebarItem;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Create a key event for testing
fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Test that inbox loads activities from mock API
#[test]
fn test_inbox_loads_activities() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let test_activities = fixtures::test_inbox_activities();
        let test_workspace = fixtures::test_workspace();
        let mock_client = MockClickUpClient::new()
            .with_workspaces(vec![test_workspace.clone()])
            .with_inbox_activities(test_activities.clone());

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Set up sidebar with AssignedTasks, Inbox, and Workspace
        *app.sidebar().items_mut() = vec![
            SidebarItem::AssignedTasks,
            SidebarItem::Inbox,
            SidebarItem::Workspace {
                id: test_workspace.id.clone(),
                name: test_workspace.name.clone(),
            },
        ];

        // Select workspace (index 2)
        app.sidebar().select(Some(2));
        app.navigate_into(); // Enter workspace - sets current_workspace_id

        // Navigate to inbox (index 1)
        app.sidebar().select(Some(1));
        app.navigate_into();

        // Wait for activities to load
        let mut iterations = 0;
        let max_iterations = 30;
        while app.inbox_list().activities().is_empty() && iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        assert!(
            app.inbox_error().is_none(),
            "Should load activities without error: {:?}",
            app.inbox_error()
        );
        assert_eq!(
            app.inbox_list().activities().len(),
            4,
            "Should load all 4 test activities"
        );
    });
}

/// Test that inbox shows empty state when no activities
#[test]
fn test_inbox_empty_when_no_activities() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let test_workspace = fixtures::test_workspace();
        let mock_client = MockClickUpClient::new()
            .with_workspaces(vec![test_workspace.clone()])
            .with_inbox_activities(vec![]);

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Set up sidebar
        *app.sidebar().items_mut() = vec![
            SidebarItem::AssignedTasks,
            SidebarItem::Inbox,
            SidebarItem::Workspace {
                id: test_workspace.id.clone(),
                name: test_workspace.name.clone(),
            },
        ];

        // Select workspace (index 2)
        app.sidebar().select(Some(2));
        app.navigate_into();

        // Navigate to inbox (index 1)
        app.sidebar().select(Some(1));
        app.navigate_into();

        // Wait for load
        let mut iterations = 0;
        let max_iterations = 30;
        while iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        assert!(
            app.inbox_error().is_none(),
            "Should not show error for empty inbox"
        );
        assert_eq!(
            app.inbox_list().activities().len(),
            0,
            "Should have empty activities list"
        );
    });
}

/// Test that dismissing an activity removes it from the list
#[test]
fn test_inbox_dismiss_activity() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let test_activities = fixtures::test_inbox_activities();
        let test_workspace = fixtures::test_workspace();
        let mock_client = MockClickUpClient::new()
            .with_workspaces(vec![test_workspace.clone()])
            .with_inbox_activities(test_activities.clone());

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Set up sidebar
        *app.sidebar().items_mut() = vec![
            SidebarItem::AssignedTasks,
            SidebarItem::Inbox,
            SidebarItem::Workspace {
                id: test_workspace.id.clone(),
                name: test_workspace.name.clone(),
            },
        ];

        // Select workspace (index 2)
        app.sidebar().select(Some(2));
        app.navigate_into();

        // Navigate to inbox (index 1)
        app.sidebar().select(Some(1));
        app.navigate_into();

        // Wait for activities to load
        let mut iterations = 0;
        let max_iterations = 30;
        while app.inbox_list().activities().is_empty() && iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        // Verify initial count
        assert_eq!(app.inbox_list().activities().len(), 4);

        // Select first activity and dismiss it (press 'c')
        app.inbox_list_mut().select(Some(0));
        app.update(InputEvent::Key(key_event(KeyCode::Char('c'))));

        // Process the dismiss action
        let mut iterations = 0;
        let max_iterations = 10;
        while iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            iterations += 1;
        }

        // Should have 3 activities remaining
        assert_eq!(
            app.inbox_list().activities().len(),
            3,
            "Should have 3 activities after dismissing one"
        );
    });
}

/// Test that dismissing all activities clears the inbox
#[test]
fn test_inbox_dismiss_all() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let test_activities = fixtures::test_inbox_activities();
        let test_workspace = fixtures::test_workspace();
        let mock_client = MockClickUpClient::new()
            .with_workspaces(vec![test_workspace.clone()])
            .with_inbox_activities(test_activities.clone());

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Set up sidebar
        *app.sidebar().items_mut() = vec![
            SidebarItem::AssignedTasks,
            SidebarItem::Inbox,
            SidebarItem::Workspace {
                id: test_workspace.id.clone(),
                name: test_workspace.name.clone(),
            },
        ];

        // Select workspace (index 2)
        app.sidebar().select(Some(2));
        app.navigate_into();

        // Navigate to inbox (index 1)
        app.sidebar().select(Some(1));
        app.navigate_into();

        // Wait for activities to load
        let mut iterations = 0;
        let max_iterations = 30;
        while app.inbox_list().activities().is_empty() && iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        // Verify initial count
        assert_eq!(app.inbox_list().activities().len(), 4);

        // Dismiss all (press 'C')
        app.update(InputEvent::Key(key_event(KeyCode::Char('C'))));

        // Process the dismiss all action
        let mut iterations = 0;
        let max_iterations = 10;
        while iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            iterations += 1;
        }

        // Should have 0 activities remaining
        assert_eq!(
            app.inbox_list().activities().len(),
            0,
            "Should have empty inbox after dismissing all"
        );
    });
}

/// Test that pressing 'r' refreshes activities
#[test]
fn test_inbox_refresh() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let test_activities = fixtures::test_inbox_activities();
        let test_workspace = fixtures::test_workspace();
        let mock_client = MockClickUpClient::new()
            .with_workspaces(vec![test_workspace.clone()])
            .with_inbox_activities(test_activities.clone());

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Set up sidebar
        *app.sidebar().items_mut() = vec![
            SidebarItem::AssignedTasks,
            SidebarItem::Inbox,
            SidebarItem::Workspace {
                id: test_workspace.id.clone(),
                name: test_workspace.name.clone(),
            },
        ];

        // Select workspace (index 2)
        app.sidebar().select(Some(2));
        app.navigate_into();

        // Navigate to inbox (index 1)
        app.sidebar().select(Some(1));
        app.navigate_into();

        // Wait for activities to load
        let mut iterations = 0;
        let max_iterations = 30;
        while app.inbox_list().activities().is_empty() && iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        // Verify initial load
        assert_eq!(app.inbox_list().activities().len(), 4);
        assert!(app.status().contains("Loaded") || app.status().contains("activity"), "Should show loaded status");

        // Press 'r' to refresh
        app.update(InputEvent::Key(key_event(KeyCode::Char('r'))));

        // Wait for refresh to complete
        let mut iterations = 0;
        let max_iterations = 20;
        while iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        // Should still have 4 activities after refresh
        assert_eq!(
            app.inbox_list().activities().len(),
            4,
            "Should still have activities after refresh"
        );
    });
}

/// Test inbox navigation with keyboard
#[test]
fn test_inbox_keyboard_navigation() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let test_activities = fixtures::test_inbox_activities();
        let test_workspace = fixtures::test_workspace();
        let mock_client = MockClickUpClient::new()
            .with_workspaces(vec![test_workspace.clone()])
            .with_inbox_activities(test_activities.clone());

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Set up sidebar
        *app.sidebar().items_mut() = vec![
            SidebarItem::AssignedTasks,
            SidebarItem::Inbox,
            SidebarItem::Workspace {
                id: test_workspace.id.clone(),
                name: test_workspace.name.clone(),
            },
        ];

        // Select workspace (index 2)
        app.sidebar().select(Some(2));
        app.navigate_into();

        // Navigate to inbox (index 1)
        app.sidebar().select(Some(1));
        app.navigate_into();

        // Wait for activities to load
        let mut iterations = 0;
        let max_iterations = 30;
        while app.inbox_list().activities().is_empty() && iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        // Navigate down with 'j'
        app.update(InputEvent::Key(key_event(KeyCode::Char('j'))));
        assert_eq!(app.inbox_list().selected(), Some(1), "Should select second item");

        // Navigate down again
        app.update(InputEvent::Key(key_event(KeyCode::Char('j'))));
        assert_eq!(app.inbox_list().selected(), Some(2), "Should select third item");

        // Navigate up with 'k'
        app.update(InputEvent::Key(key_event(KeyCode::Char('k'))));
        assert_eq!(app.inbox_list().selected(), Some(1), "Should select second item");

        // Navigate to first item
        app.update(InputEvent::Key(key_event(KeyCode::Char('k'))));
        assert_eq!(app.inbox_list().selected(), Some(0), "Should select first item");

        // Navigation wraps around (goes to last item)
        app.update(InputEvent::Key(key_event(KeyCode::Char('k'))));
        assert_eq!(app.inbox_list().selected(), Some(3), "Should wrap to last item");
    });
}

/// Test opening activity detail view
#[test]
fn test_inbox_open_activity_detail() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let test_activities = fixtures::test_inbox_activities();
        let test_workspace = fixtures::test_workspace();
        let mock_client = MockClickUpClient::new()
            .with_workspaces(vec![test_workspace.clone()])
            .with_inbox_activities(test_activities.clone());

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Set up sidebar
        *app.sidebar().items_mut() = vec![
            SidebarItem::AssignedTasks,
            SidebarItem::Inbox,
            SidebarItem::Workspace {
                id: test_workspace.id.clone(),
                name: test_workspace.name.clone(),
            },
        ];

        // Select workspace (index 2)
        app.sidebar().select(Some(2));
        app.navigate_into();

        // Navigate to inbox (index 1)
        app.sidebar().select(Some(1));
        app.navigate_into();

        // Wait for activities to load
        let mut iterations = 0;
        let max_iterations = 30;
        while app.inbox_list().activities().is_empty() && iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            iterations += 1;
        }

        // Select first activity and open detail (Enter)
        app.inbox_list_mut().select(Some(0));
        app.update(InputEvent::Key(key_event(KeyCode::Enter)));

        // Process the open action
        let mut iterations = 0;
        let max_iterations = 10;
        while iterations < max_iterations {
            app.process_async_messages();
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            iterations += 1;
        }

        // Detail view should be showing
        assert!(
            app.inbox_showing_detail(),
            "Should show activity detail view"
        );
    });
}
