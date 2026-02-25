//! Tests for Task JSON parsing
//!
//! These tests verify that Task models can handle various JSON formats
//! that the ClickUp API might return.

use clickdown::models::{Task, TasksResponse};
use serde_json;

// ============================================================================
// Description Tests
// ============================================================================

#[test]
fn test_task_description_as_plain_string() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "description": "This is a plain text description"
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with plain string description: {:?}", task.err());
    let task = task.unwrap();
    assert!(task.description.is_some());
}

#[test]
fn test_task_description_as_object() {
    // ClickUp API can return description as an object with html/markdown
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "description": {
            "html": "<p>This is <strong>HTML</strong> content</p>",
            "markdown": "This is **markdown** content",
            "text": "This is plain text content"
        }
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with object description: {:?}", task.err());
    let task = task.unwrap();
    assert!(task.description.is_some());
}

#[test]
fn test_task_description_as_null() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "description": null
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with null description: {:?}", task.err());
    let task = task.unwrap();
    assert!(task.description.is_none());
}

#[test]
fn test_task_description_missing() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task"
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with missing description: {:?}", task.err());
    let task = task.unwrap();
    assert!(task.description.is_none());
}

// ============================================================================
// Content Tests
// ============================================================================

#[test]
fn test_task_content_as_plain_string() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "content": "Plain text content"
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with plain string content: {:?}", task.err());
}

#[test]
fn test_task_content_as_object() {
    // ClickUp API can return content as an object with HTML
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "content": {
            "html": "<div>HTML content here</div>"
        }
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with object content: {:?}", task.err());
}

#[test]
fn test_task_content_as_null() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "content": null
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with null content: {:?}", task.err());
}

// ============================================================================
// Status Tests
// ============================================================================

#[test]
fn test_task_status_as_object() {
    let json = r##"
    {
        "id": "task-1",
        "name": "Test Task",
        "status": {
            "status": "in progress",
            "color": "#5c7cfa",
            "type": "custom"
        }
    }
    "##;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with status object: {:?}", task.err());
}

#[test]
fn test_task_status_as_null() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "status": null
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with null status: {:?}", task.err());
}

#[test]
fn test_task_status_missing() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task"
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with missing status: {:?}", task.err());
}

// ============================================================================
// Priority Tests
// ============================================================================

#[test]
fn test_task_priority_as_object() {
    let json = r##"
    {
        "id": "task-1",
        "name": "Test Task",
        "priority": {
            "priority": "high",
            "color": "#ff0000"
        }
    }
    "##;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with priority object: {:?}", task.err());
}

#[test]
fn test_task_priority_as_null() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "priority": null
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with null priority: {:?}", task.err());
}

// ============================================================================
// Assignee Tests
// ============================================================================

#[test]
fn test_task_assignees_as_array() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "assignees": [
            {
                "id": 123,
                "username": "john_doe",
                "email": "john@example.com"
            }
        ]
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with assignees array: {:?}", task.err());
}

#[test]
fn test_task_assignees_as_null() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "assignees": null
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with null assignees: {:?}", task.err());
}

#[test]
fn test_task_assignees_missing() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task"
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with missing assignees: {:?}", task.err());
}

// ============================================================================
// Time Estimate/Spent Tests
// ============================================================================

#[test]
fn test_task_time_estimate_as_integer() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "timeEstimate": 3600000
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with integer timeEstimate: {:?}", task.err());
}

#[test]
fn test_task_time_estimate_as_null() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "timeEstimate": null
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with null timeEstimate: {:?}", task.err());
}

#[test]
fn test_task_time_spent_as_integer() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "timeSpent": 1800000
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with integer timeSpent: {:?}", task.err());
}

#[test]
fn test_task_time_spent_as_null() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "timeSpent": null
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with null timeSpent: {:?}", task.err());
}

// ============================================================================
// Checklist Tests
// ============================================================================

#[test]
fn test_task_checklists_as_array() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "checklists": [
            {
                "id": "checklist-1",
                "name": "My Checklist",
                "resolved": false
            }
        ]
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with checklists array: {:?}", task.err());
}

#[test]
fn test_task_checklists_as_null() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "checklists": null
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with null checklists: {:?}", task.err());
}

// ============================================================================
// Custom Fields Tests
// ============================================================================

#[test]
fn test_task_custom_fields_as_array() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "custom_fields": [
            {
                "id": "field-1",
                "name": "Priority",
                "type": "dropdown",
                "value": "High"
            }
        ]
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with custom_fields array: {:?}", task.err());
}

#[test]
fn test_task_custom_fields_as_null() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "custom_fields": null
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with null custom_fields: {:?}", task.err());
}

// ============================================================================
// Tags Tests
// ============================================================================

#[test]
fn test_task_tags_as_array() {
    let json = r##"
    {
        "id": "task-1",
        "name": "Test Task",
        "tags": [
            {
                "id": "tag-1",
                "name": "Urgent",
                "color": "#ff0000"
            }
        ]
    }
    "##;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with tags array: {:?}", task.err());
}

#[test]
fn test_task_tags_as_null() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "tags": null
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with null tags: {:?}", task.err());
}

// ============================================================================
// Attachments Tests
// ============================================================================

#[test]
fn test_task_attachments_as_array() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "attachments": [
            {
                "id": "attach-1",
                "title": "My File.pdf",
                "url": "https://example.com/file.pdf"
            }
        ]
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with attachments array: {:?}", task.err());
}

#[test]
fn test_task_attachments_as_null() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "attachments": null
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with null attachments: {:?}", task.err());
}

// ============================================================================
// Complex Real-World Task Tests
// ============================================================================

#[test]
fn test_task_minimal_real_world() {
    // Minimal task as might be returned by API
    let json = r##"
    {
        "id": "8j1k2l3m4n5o",
        "name": "Review pull request",
        "status": {
            "status": "in progress",
            "color": "#5c7cfa",
            "type": "custom",
            "orderindex": 2
        },
        "priority": {
            "priority": "high",
            "color": "#ff0000"
        },
        "assignees": [{
            "id": 12345,
            "username": "developer",
            "email": "dev@example.com",
            "color": "#3498db"
        }],
        "description": {
            "html": "<p>Please review the changes in PR #42</p>",
            "markdown": "Please review the changes in PR #42"
        },
        "due_date": 1709251200000,
        "timeEstimate": 7200000,
        "timeSpent": 3600000,
        "tags": [],
        "checklists": [],
        "custom_fields": [],
        "attachments": []
    }
    "##;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse real-world task: {:?}", task.err());
}

#[test]
fn test_task_with_null_values_real_world() {
    // Task with many null values as API might return
    let json = r#"
    {
        "id": "8j1k2l3m4n5o",
        "name": "Simple task",
        "status": null,
        "priority": null,
        "assignees": [],
        "description": null,
        "content": null,
        "due_date": null,
        "start_date": null,
        "timeEstimate": null,
        "timeSpent": null,
        "tags": null,
        "checklists": null,
        "custom_fields": null,
        "attachments": null,
        "parent": null,
        "folder": null,
        "space": null,
        "list": null,
        "creator": null,
        "closed_at": null
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Failed to parse task with many null values: {:?}", task.err());
}

#[test]
fn test_tasks_response_array() {
    // Full API response with tasks array
    let json = r#"
    {
        "tasks": [
            {
                "id": "task-1",
                "name": "First task",
                "status": {"status": "open"},
                "description": {"html": "<p>First description</p>"}
            },
            {
                "id": "task-2",
                "name": "Second task",
                "status": {"status": "complete"},
                "description": "Plain text description"
            },
            {
                "id": "task-3",
                "name": "Third task",
                "status": null,
                "description": null
            }
        ]
    }
    "#;

    let response: Result<TasksResponse, _> = serde_json::from_str(json);
    assert!(response.is_ok(), "Failed to parse TasksResponse: {:?}", response.err());
    let response = response.unwrap();
    assert_eq!(response.tasks.len(), 3);
}

// ============================================================================
// Parsing Failure Reproduction Tests - NOW FIXED
// These tests verify that previously failing cases now work correctly
// ============================================================================

#[test]
fn test_task_timestamp_as_string_now_works() {
    // FIXED: ClickUp API sometimes returns timestamps as strings
    // This test verifies the fix works
    let json = r#"
    {
        "id": "task-1",
        "name": "Test",
        "date_created": "1709251200000",
        "due_date": "1709424000000"
    }
    "#;

    let result: Result<Task, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Should now handle string timestamps: {:?}", result.err());
    let task = result.unwrap();
    assert_eq!(task.created_at, Some(1709251200000));
    assert_eq!(task.due_date, Some(1709424000000));
}

#[test]
fn test_task_points_as_string_now_works() {
    // FIXED: ClickUp API might return points as string
    let json = r#"
    {
        "id": "task-1",
        "name": "Test",
        "points": "5"
    }
    "#;

    let result: Result<Task, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Should now handle string points: {:?}", result.err());
    let task = result.unwrap();
    assert_eq!(task.points, Some(5));
}

#[test]
fn test_task_time_estimate_as_string_now_works() {
    // FIXED: timeEstimate might be returned as string
    let json = r#"
    {
        "id": "task-1",
        "name": "Test",
        "timeEstimate": "3600000"
    }
    "#;

    let result: Result<Task, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Should now handle string timeEstimate: {:?}", result.err());
    let task = result.unwrap();
    assert_eq!(task.time_estimate, Some(3600000));
}

#[test]
fn test_tasks_response_null_tasks_now_works() {
    // FIXED: API might return null tasks field - now defaults to empty vec
    let json = r#"{"tasks": null}"#;

    let result: Result<TasksResponse, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Should now handle null tasks field: {:?}", result.err());
    let response = result.unwrap();
    assert_eq!(response.tasks.len(), 0);
}

#[test]
fn test_tasks_response_missing_tasks_now_works() {
    // FIXED: API might return missing tasks field - now defaults to empty vec
    let json = r#"{}"#;

    let result: Result<TasksResponse, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Should now handle missing tasks field: {:?}", result.err());
    let response = result.unwrap();
    assert_eq!(response.tasks.len(), 0);
}

#[test]
fn test_task_status_as_string_still_fails() {
    // NOTE: API returning status as string instead of object still fails
    // This is a known limitation - status should always be an object
    let json = r##"
    {
        "id": "task-1",
        "name": "Test",
        "status": "open"
    }
    "##;

    let result: Result<Task, _> = serde_json::from_str(json);
    // This still fails because status should be an object with metadata
    // If the API actually returns this format, we'd need to add a flexible status deserializer
    assert!(result.is_err(), "Status as string still fails (expected - status should be object)");
}

#[test]
fn test_task_assignees_as_single_object_still_fails() {
    // NOTE: API returning single assignee object instead of array still fails
    // This is a known limitation - assignees should always be an array
    let json = r#"
    {
        "id": "task-1",
        "name": "Test",
        "assignees": {"id": 123, "username": "single_assignee"}
    }
    "#;

    let result: Result<Task, _> = serde_json::from_str(json);
    // This still fails because assignees should be an array
    // If the API actually returns this format, we'd need a flexible assignees deserializer
    assert!(result.is_err(), "Single object assignees still fails (expected - should be array)");
}
