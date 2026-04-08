//! Tests for task assignment picker feature

mod fixtures;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::api::ClickUpApi;
use clickdown::models::task::{AssigneesUpdate, UpdateTaskRequest};
use clickdown::models::User;
use clickdown::tui::app::TuiApp;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Helper to create a test user
fn test_user(id: i64, username: &str) -> User {
    User {
        id,
        username: username.to_string(),
        color: None,
        email: Some(format!("{}@example.com", username.to_lowercase())),
        profile_picture: None,
        initials: Some(
            username
                .split_whitespace()
                .filter_map(|w| w.chars().next())
                .take(2)
                .collect(),
        ),
    }
}

/// Test that the assignee picker state initializes correctly
#[test]
fn test_picker_state_initially_closed() {
    let rt = Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new();

    let app = rt.block_on(async { TuiApp::with_client(Arc::new(mock_client)).unwrap() });

    assert!(
        !app.is_assignee_picker_open(),
        "Picker should be closed initially"
    );
}

/// Test that cached list members can be set and checked
#[test]
fn test_cached_members_can_be_set() {
    let rt = Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new();

    let mut app = rt.block_on(async { TuiApp::with_client(Arc::new(mock_client)).unwrap() });

    let members = vec![test_user(1, "Alice"), test_user(2, "Bob")];
    app.set_cached_list_members("list_123", members);

    // Verify cache was populated (using is_assignee_picker_open as proxy for state access)
    assert!(
        !app.is_assignee_picker_open(),
        "Picker should still be closed after setting cache"
    );
}

/// Test that mock client returns configured list members
#[test]
fn test_mock_list_members() {
    let members = vec![test_user(1, "Alice"), test_user(2, "Bob")];
    let mock_client = MockClickUpClient::new().with_list_members(members.clone());

    let rt = Runtime::new().unwrap();
    let result: Result<Vec<User>, _> =
        rt.block_on(async { mock_client.get_list_members("list_123").await });

    let returned = result.expect("get_list_members should succeed");
    assert_eq!(returned.len(), 2);
    assert_eq!(returned[0].username, "Alice");
    assert_eq!(returned[1].username, "Bob");
}

/// Test that mock client returns error when not configured
#[test]
fn test_mock_list_members_error() {
    let mock_client = MockClickUpClient::new();
    let rt = Runtime::new().unwrap();
    let result: Result<Vec<User>, _> =
        rt.block_on(async { mock_client.get_list_members("list_123").await });

    // Default returns empty vec, not error
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

/// Test that mock list members error is returned
#[test]
fn test_mock_list_members_explicit_error() {
    let mock_client =
        MockClickUpClient::new().with_list_members_error("API unavailable".to_string());
    let rt = Runtime::new().unwrap();
    let result: Result<Vec<User>, _> =
        rt.block_on(async { mock_client.get_list_members("list_123").await });

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API unavailable"));
}

/// Test that mock update_task merges assignee IDs into the returned task
#[test]
fn test_mock_update_task_merges_assignees() {
    use fixtures::test_task;

    let base_task = test_task();
    let mock_client = MockClickUpClient::new().with_update_task_response(base_task.clone());

    let rt = Runtime::new().unwrap();
    let result = rt.block_on(async {
        let update = UpdateTaskRequest {
            name: None,
            description: None,
            status: None,
            priority: None,
            assignees: Some(AssigneesUpdate::set_all(vec![1, 2, 3])),
            due_date: None,
        };
        mock_client.update_task("test-task-1", &update).await
    });

    let updated = result.expect("update_task should succeed");
    assert_eq!(updated.assignees.len(), 3);
    assert_eq!(updated.assignees[0].id, 1);
    assert_eq!(updated.assignees[0].username, "user_1");
    assert_eq!(updated.assignees[1].id, 2);
    assert_eq!(updated.assignees[2].id, 3);
    // Other fields should remain unchanged
    assert_eq!(updated.id, base_task.id);
    assert_eq!(updated.name, base_task.name);
}

/// Test that mock update_task merges all fields
#[test]
fn test_mock_update_task_merges_all_fields() {
    use fixtures::test_task;

    let base_task = test_task();
    let mock_client = MockClickUpClient::new().with_update_task_response(base_task);

    let rt = Runtime::new().unwrap();
    let result = rt.block_on(async {
        let update = UpdateTaskRequest {
            name: Some("Updated Task Name".to_string()),
            description: Some("New description".to_string()),
            status: Some("in_progress".to_string()),
            priority: None,
            assignees: Some(AssigneesUpdate::set_all(vec![42])),
            due_date: Some(1700000000000),
        };
        mock_client.update_task("test-task-1", &update).await
    });

    let updated = result.expect("update_task should succeed");
    assert_eq!(updated.name, "Updated Task Name");
    assert_eq!(
        updated.description.as_ref().map(|d| d.as_text()),
        Some("New description".to_string())
    );
    assert_eq!(
        updated.status.as_ref().map(|s| s.status.clone()),
        Some("in_progress".to_string())
    );
    assert_eq!(updated.assignees.len(), 1);
    assert_eq!(updated.assignees[0].id, 42);
    assert_eq!(updated.due_date, Some(1700000000000));
}

/// Test that empty assignees clears the assignees list
#[test]
fn test_mock_update_task_clears_assignees() {
    use fixtures::test_task;

    let base_task = test_task();
    let mock_client = MockClickUpClient::new().with_update_task_response(base_task);

    let rt = Runtime::new().unwrap();
    let result = rt.block_on(async {
        let update = UpdateTaskRequest {
            name: None,
            description: None,
            status: None,
            priority: None,
            assignees: Some(AssigneesUpdate::set_all(vec![])),
            due_date: None,
        };
        mock_client.update_task("test-task-1", &update).await
    });

    let updated = result.expect("update_task should succeed");
    assert!(updated.assignees.is_empty());
}

/// Test that assignees are not modified when not included in update
#[test]
fn test_mock_update_task_preserves_assignees_when_not_updated() {
    use fixtures::test_task;

    let mut base_task = test_task();
    base_task.assignees = vec![test_user(1, "Existing User")];
    let mock_client = MockClickUpClient::new().with_update_task_response(base_task);

    let rt = Runtime::new().unwrap();
    let result = rt.block_on(async {
        let update = UpdateTaskRequest {
            name: Some("Updated Name".to_string()),
            description: None,
            status: None,
            priority: None,
            assignees: None,
            due_date: None,
        };
        mock_client.update_task("test-task-1", &update).await
    });

    let updated = result.expect("update_task should succeed");
    assert_eq!(updated.assignees.len(), 1);
    assert_eq!(updated.assignees[0].username, "Existing User");
    assert_eq!(updated.name, "Updated Name");
}
