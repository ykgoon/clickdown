//! Tests for malformed JSON parsing and error handling
//!
//! These tests verify that the Task models handle edge cases and malformed data gracefully,
//! and that errors are properly reported when parsing fails.

use clickdown::models::{Task, TasksResponse};
use serde_json;

// ============================================================================
// Malformed Data Tests - Should Still Parse (Graceful Degradation)
// ============================================================================

#[test]
fn test_task_with_completely_wrong_type_for_id() {
    // ID should be string, but API might return number
    let json = r#"
    {
        "id": 12345,
        "name": "Test Task"
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    // This should fail because id is required and must be String
    assert!(task.is_err(), "Task with numeric id should fail to parse");
    let err = task.unwrap_err();
    assert!(err.to_string().contains("invalid type") || err.to_string().contains("id"));
}

#[test]
fn test_task_with_missing_required_name_field() {
    // name is required
    let json = r#"
    {
        "id": "task-1"
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    // This should fail because name is required
    assert!(task.is_err(), "Task without name should fail to parse");
}

#[test]
fn test_task_with_null_for_required_string_field() {
    let json = r#"
    {
        "id": "task-1",
        "name": null
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    // This should fail because name cannot be null
    assert!(task.is_err(), "Task with null name should fail to parse");
}

#[test]
fn test_task_with_invalid_json_syntax() {
    // Missing closing brace
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task"
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_err(), "Invalid JSON syntax should fail to parse");
}

#[test]
fn test_task_with_unexpected_nested_object() {
    // status should be an object, but what if it's deeply nested unexpectedly?
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "status": {
            "status": "done",
            "nested": {
                "unexpected": "value"
            }
        }
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    // Should still parse, ignoring unknown fields
    assert!(task.is_ok(), "Task with unexpected nested object should still parse: {:?}", task.err());
}

#[test]
fn test_task_with_unicode_in_name() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task 日本語 🚀 émojis"
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Task with unicode should parse: {:?}", task.err());
    let task = task.unwrap();
    assert_eq!(task.name, "Test Task 日本語 🚀 émojis");
}

#[test]
fn test_task_with_extremely_long_string() {
    let long_desc = "A".repeat(100000);
    let json = format!(r#"
    {{
        "id": "task-1",
        "name": "Test",
        "description": "{}"
    }}
    "#, long_desc);

    let task: Result<Task, _> = serde_json::from_str(&json);
    assert!(task.is_ok(), "Task with very long string should parse: {:?}", task.err());
}

#[test]
fn test_task_with_empty_object() {
    let json = r#"{}"#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_err(), "Empty object should fail (missing required fields)");
}

// ============================================================================
// Field Type Mismatch Tests
// ============================================================================

#[test]
fn test_task_status_as_string_instead_of_object() {
    // status should be object, but API might return string
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "status": "done"
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    // This should fail because status type doesn't match
    assert!(task.is_err(), "Task with string status should fail to parse");
}

#[test]
fn test_task_with_array_where_object_expected() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "creator": ["user1", "user2"]
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    // creator should be object, not array
    assert!(task.is_err(), "Task with array for creator should fail to parse");
}

#[test]
fn test_task_with_string_where_number_expected() {
    // User id should be i64
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "creator": {
            "id": "not-a-number",
            "username": "test"
        }
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_err(), "Task with string for user id should fail to parse");
}

// ============================================================================
// Edge Case Tests - Should Parse Successfully
// ============================================================================

#[test]
fn test_task_with_all_optional_fields_null() {
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "description": null,
        "status": null,
        "content": null,
        "creator": null,
        "parent": null,
        "priority": null,
        "due_date": null,
        "start_date": null,
        "points": null,
        "list": null,
        "folder": null,
        "space": null,
        "url": null,
        "timeEstimate": null,
        "timeSpent": null
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Task with all null optional fields should parse: {:?}", task.err());
}

#[test]
fn test_task_with_camelcase_field_names() {
    // API returns camelCase, our model should handle it
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "timeEstimate": 3600000,
        "timeSpent": 1800000,
        "date_created": "1234567890",
        "date_updated": "1234567891",
        "markdown_description": "Test markdown"
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Task with camelCase fields should parse: {:?}", task.err());
    let task = task.unwrap();
    assert_eq!(task.time_estimate, Some(3600000));
    assert_eq!(task.time_spent, Some(1800000));
}

#[test]
fn test_task_with_profile_picture_camelcase() {
    // API returns profilePicture (camelCase)
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "creator": {
            "id": 123,
            "username": "test",
            "profilePicture": "https://example.com/pic.jpg"
        }
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Task with profilePicture should parse: {:?}", task.err());
    let task = task.unwrap();
    assert_eq!(task.creator.unwrap().profile_picture, Some("https://example.com/pic.jpg".to_string()));
}

#[test]
fn test_task_status_with_type_field() {
    // API returns "type" but we use type_field
    let json = r##"
    {
        "id": "task-1",
        "name": "Test Task",
        "status": {
            "status": "done",
            "type": "custom",
            "color": "#00ff00"
        }
    }
    "##;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Task with status type should parse: {:?}", task.err());
    let task = task.unwrap();
    assert_eq!(task.status.unwrap().type_field, Some("custom".to_string()));
}

#[test]
fn test_custom_field_with_type_field() {
    // API returns "type" but we use type_field
    let json = r##"
    {
        "id": "task-1",
        "name": "Test Task",
        "custom_fields": [
            {
                "id": "field-1",
                "name": "Test Field",
                "type": "date",
                "value": "2024-01-01"
            }
        ]
    }
    "##;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Task with custom field type should parse: {:?}", task.err());
    let task = task.unwrap();
    assert_eq!(task.custom_fields[0].type_field, Some("date".to_string()));
}

// ============================================================================
// TasksResponse Parsing Tests
// ============================================================================

#[test]
fn test_tasks_response_with_null_tasks() {
    let json = r#"
    {
        "tasks": null
    }
    "#;

    let response: Result<TasksResponse, _> = serde_json::from_str(json);
    assert!(response.is_ok(), "TasksResponse with null tasks should parse: {:?}", response.err());
    let response = response.unwrap();
    assert!(response.tasks.is_empty());
}

#[test]
fn test_tasks_response_with_missing_tasks_field() {
    let json = r#"{}"#;

    let response: Result<TasksResponse, _> = serde_json::from_str(json);
    assert!(response.is_ok(), "TasksResponse without tasks field should parse: {:?}", response.err());
    let response = response.unwrap();
    assert!(response.tasks.is_empty());
}

#[test]
fn test_tasks_response_with_malformed_task_in_array() {
    // One valid task, one malformed
    let json = r#"
    {
        "tasks": [
            {
                "id": "task-1",
                "name": "Valid Task"
            },
            {
                "id": "task-2"
            }
        ]
    }
    "#;

    let response: Result<TasksResponse, _> = serde_json::from_str(json);
    // Should fail because second task is missing required name field
    assert!(response.is_err(), "TasksResponse with malformed task should fail");
}

// ============================================================================
// Real-world Malformed Data Tests
// ============================================================================

#[test]
fn test_task_with_real_api_response_format() {
    // Simulating a real API response with all the fields we've seen
    let json = r##"
    {
        "id": "86d20qjkb",
        "custom_id": null,
        "custom_item_id": 0,
        "name": "Test Task",
        "text_content": "",
        "description": "",
        "markdown_description": null,
        "status": {
            "id": "sc901610671319_6vzAiFuB",
            "status": "to do",
            "color": "#87909e",
            "orderindex": 0,
            "type": "open",
            "status_group": "subcat_901610671319"
        },
        "orderindex": "120305339.00000000000000000000000000000000",
        "date_created": "1771464110921",
        "date_updated": "1771464116942",
        "date_closed": null,
        "date_done": null,
        "archived": false,
        "creator": {
            "id": 94803855,
            "username": "Test User",
            "color": "#827718",
            "email": "test@example.com",
            "profilePicture": null,
            "initials": "TU"
        },
        "assignees": [],
        "group_assignees": [],
        "watchers": [],
        "checklists": [],
        "tags": [],
        "parent": null,
        "top_level_parent": null,
        "priority": null,
        "due_date": null,
        "start_date": null,
        "points": null,
        "time_estimate": null,
        "time_spent": 0,
        "custom_fields": [],
        "attachments": [],
        "dependencies": [],
        "linked_tasks": [],
        "locations": [],
        "list": {
            "id": "901610671319",
            "name": "Test List",
            "access": true
        },
        "project": {
            "id": "121765286",
            "name": "Active",
            "hidden": false,
            "access": true
        },
        "folder": {
            "id": "121765286",
            "name": "Active",
            "hidden": false,
            "access": true
        },
        "space": {
            "id": "44426444",
            "name": null,
            "access": true
        },
        "team_id": "26408409",
        "url": "https://app.clickup.com/t/86d20qjkb",
        "sharing": {
            "public": false,
            "public_share_expires_on": null,
            "public_fields": ["assignees", "priority"],
            "token": null,
            "seo_optimized": false
        },
        "permission_level": "create"
    }
    "##;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Real API response format should parse: {:?}", task.err());
    let task = task.unwrap();
    assert_eq!(task.id, "86d20qjkb");
    assert_eq!(task.name, "Test Task");
    assert!(task.status.is_some());
    assert_eq!(task.status.as_ref().unwrap().id, Some("sc901610671319_6vzAiFuB".to_string()));
}

#[test]
fn test_task_with_string_timestamp_values() {
    // API sometimes returns timestamps as strings
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "date_created": "1234567890",
        "date_updated": "1234567891",
        "due_date": "1234567892"
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Task with string timestamps should parse: {:?}", task.err());
    let task = task.unwrap();
    assert_eq!(task.created_at, Some(1234567890));
    assert_eq!(task.updated_at, Some(1234567891));
    assert_eq!(task.due_date, Some(1234567892));
}

#[test]
fn test_task_with_numeric_timestamp_values() {
    // API sometimes returns timestamps as numbers
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "date_created": 1234567890,
        "date_updated": 1234567891,
        "due_date": 1234567892
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Task with numeric timestamps should parse: {:?}", task.err());
    let task = task.unwrap();
    assert_eq!(task.created_at, Some(1234567890));
    assert_eq!(task.updated_at, Some(1234567891));
    assert_eq!(task.due_date, Some(1234567892));
}

#[test]
fn test_task_with_mixed_timestamp_types() {
    // Mix of string and numeric timestamps
    let json = r#"
    {
        "id": "task-1",
        "name": "Test Task",
        "date_created": "1234567890",
        "date_updated": 1234567891,
        "due_date": null
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    assert!(task.is_ok(), "Task with mixed timestamp types should parse: {:?}", task.err());
    let task = task.unwrap();
    assert_eq!(task.created_at, Some(1234567890));
    assert_eq!(task.updated_at, Some(1234567891));
    assert_eq!(task.due_date, None);
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[test]
fn test_error_message_contains_field_name() {
    let json = r#"
    {
        "id": "task-1"
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    let err = task.unwrap_err();
    let err_msg = err.to_string().to_lowercase();
    
    // Error should mention the missing field
    assert!(err_msg.contains("name") || err_msg.contains("missing"), 
            "Error message should indicate missing field: {}", err);
}

#[test]
fn test_error_message_for_type_mismatch() {
    let json = r#"
    {
        "id": 12345,
        "name": "Test"
    }
    "#;

    let task: Result<Task, _> = serde_json::from_str(json);
    let err = task.unwrap_err();
    let err_msg = err.to_string().to_lowercase();
    
    // Error should indicate type problem
    assert!(err_msg.contains("invalid") || err_msg.contains("type") || err_msg.contains("expected"),
            "Error message should indicate type mismatch: {}", err);
}
