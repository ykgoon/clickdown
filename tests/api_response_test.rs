//! Regression tests for real ClickUp API response payloads.

use clickdown::models::Task;

/// The real JSON body ClickUp returns for `PUT /task/{id}` when updating
/// assignees (captured 2026-10-30 from the production API).
///
/// Regression: this payload previously failed to deserialize into `Task`,
/// which made the assignee picker show "Failed to update assignees" even
/// though the assignment had been applied server-side.
#[test]
fn test_assignee_update_response_deserializes() {
    let body = include_str!("fixtures/assignee_update_response.json");
    let task: Task = serde_json::from_str(body)
        .expect("real assignee-update response must deserialize into Task");
    assert!(!task.assignees.is_empty(), "assignees must be present");
    assert_eq!(task.assignees[0].id, 38650140);
}

/// The real JSON body returned by `PUT /task/{id}` on a production task
/// (captured 2026-10-30, includes custom_fields, sharing, locations etc.).
#[test]
fn test_real_task_response_deserializes() {
    let body = include_str!("fixtures/real_task_response.json");
    let task: Task = serde_json::from_str(body)
        .expect("real task response must deserialize into Task");
    assert_eq!(task.id, "14zrc6q9vr2");
}
