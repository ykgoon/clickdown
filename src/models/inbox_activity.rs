//! Inbox activity model for smart inbox feature
//!
//! The smart inbox aggregates activity from multiple ClickUp API endpoints:
//! - Task assignments (tasks assigned to the user)
//! - Comments (new comments on user's tasks)
//! - Status changes (tasks with recent status updates)
//! - Due dates (tasks with approaching deadlines)
//!
//! Since ClickUp API v2 has no notifications endpoint, we simulate an inbox
//! by polling these endpoints and normalizing the results into activity items.

use serde::{Deserialize, Serialize};

/// Type of inbox activity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActivityType {
    /// Task was assigned to the user
    #[serde(rename = "assignment")]
    Assignment,
    /// New comment on user's task
    #[serde(rename = "comment")]
    Comment,
    /// Task status changed
    #[serde(rename = "status_change")]
    StatusChange,
    /// Task due date is approaching
    #[serde(rename = "due_date")]
    DueDate,
}

impl ActivityType {
    /// Get the icon for this activity type
    pub fn icon(&self) -> &'static str {
        match self {
            ActivityType::Assignment => "📋",
            ActivityType::Comment => "💬",
            ActivityType::StatusChange => "🔄",
            ActivityType::DueDate => "⏰",
        }
    }

    /// Get a human-readable label for this activity type
    pub fn label(&self) -> &'static str {
        match self {
            ActivityType::Assignment => "Assigned",
            ActivityType::Comment => "Comment",
            ActivityType::StatusChange => "Status",
            ActivityType::DueDate => "Due Date",
        }
    }
}

/// A normalized inbox activity item
///
/// This model unifies different types of activity (assignments, comments, etc.)
/// into a single structure for display in the smart inbox.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InboxActivity {
    /// Unique identifier for this activity
    /// Format: "{activity_type}_{source_id}" (e.g., "assignment_123", "comment_456")
    #[serde(rename = "id")]
    pub id: String,

    /// Type of activity (assignment, comment, etc.)
    #[serde(rename = "activity_type")]
    pub activity_type: ActivityType,

    /// Human-readable title (e.g., "Task assigned to you", "New comment on Task X")
    #[serde(rename = "title")]
    pub title: String,

    /// Optional description with additional context
    #[serde(rename = "description", default)]
    pub description: String,

    /// When the activity occurred (Unix timestamp in milliseconds)
    #[serde(rename = "timestamp")]
    pub timestamp: i64,

    /// ID of the source task (if applicable)
    #[serde(rename = "task_id", default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,

    /// ID of the source comment (if applicable)
    #[serde(rename = "comment_id", default, skip_serializing_if = "Option::is_none")]
    pub comment_id: Option<String>,

    /// Workspace/team ID for context
    #[serde(rename = "workspace_id")]
    pub workspace_id: String,

    /// Name of the source task (for display purposes)
    #[serde(rename = "task_name", default)]
    pub task_name: String,

    /// Previous status (for status changes)
    #[serde(rename = "previous_status", default, skip_serializing_if = "Option::is_none")]
    pub previous_status: Option<String>,

    /// New status (for status changes or assignments)
    #[serde(rename = "new_status", default, skip_serializing_if = "Option::is_none")]
    pub new_status: Option<String>,

    /// Due date in milliseconds (for due date activities)
    #[serde(rename = "due_date", default, skip_serializing_if = "Option::is_none")]
    pub due_date: Option<i64>,
}

impl InboxActivity {
    /// Create a new assignment activity
    pub fn assignment(
        task_id: String,
        task_name: String,
        workspace_id: String,
        timestamp: i64,
        status: Option<String>,
    ) -> Self {
        Self {
            id: format!("assignment_{}", task_id),
            activity_type: ActivityType::Assignment,
            title: format!("📋 Task assigned to you: {}", task_name),
            description: format!("You were assigned to '{}'", task_name),
            timestamp,
            task_id: Some(task_id),
            comment_id: None,
            workspace_id,
            task_name,
            previous_status: None,
            new_status: status,
            due_date: None,
        }
    }

    /// Create a new comment activity
    pub fn comment(
        comment_id: String,
        task_id: String,
        task_name: String,
        workspace_id: String,
        timestamp: i64,
        comment_text: String,
    ) -> Self {
        Self {
            id: format!("comment_{}", comment_id),
            activity_type: ActivityType::Comment,
            title: format!("💬 New comment on: {}", task_name),
            description: truncate_text(&comment_text, 100),
            timestamp,
            task_id: Some(task_id),
            comment_id: Some(comment_id),
            workspace_id,
            task_name,
            previous_status: None,
            new_status: None,
            due_date: None,
        }
    }

    /// Create a new status change activity
    pub fn status_change(
        task_id: String,
        task_name: String,
        workspace_id: String,
        timestamp: i64,
        old_status: String,
        new_status: String,
    ) -> Self {
        Self {
            id: format!("status_{}_{}", task_id, timestamp),
            activity_type: ActivityType::StatusChange,
            title: format!("🔄 Status changed: {}", task_name),
            description: format!("Changed from '{}' to '{}'", old_status, new_status),
            timestamp,
            task_id: Some(task_id),
            comment_id: None,
            workspace_id,
            task_name,
            previous_status: Some(old_status),
            new_status: Some(new_status),
            due_date: None,
        }
    }

    /// Create a new due date activity
    pub fn due_date(
        task_id: String,
        task_name: String,
        workspace_id: String,
        due_date: i64,
    ) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        let days_until_due = (due_date - now) / (1000 * 60 * 60 * 24);

        let title = if days_until_due < 0 {
            format!("⏰ Task overdue: {}", task_name)
        } else if days_until_due == 0 {
            format!("⏰ Task due today: {}", task_name)
        } else if days_until_due <= 3 {
            format!("⏰ Task due in {} days: {}", days_until_due, task_name)
        } else {
            format!("⏰ Task due soon: {}", task_name)
        };

        Self {
            id: format!("due_{}", task_id),
            activity_type: ActivityType::DueDate,
            title,
            description: format_due_date(due_date),
            timestamp: now, // Use current time for due date reminders
            task_id: Some(task_id),
            comment_id: None,
            workspace_id,
            task_name,
            previous_status: None,
            new_status: None,
            due_date: Some(due_date),
        }
    }

    /// Get the icon for this activity
    pub fn icon(&self) -> &'static str {
        self.activity_type.icon()
    }
}

/// Truncate text to a maximum length, adding ellipsis if needed
fn truncate_text(text: &str, max_len: usize) -> String {
    let stripped = text.trim();
    if stripped.len() <= max_len {
        stripped.to_string()
    } else {
        format!("{}...", &stripped[..max_len.saturating_sub(3)])
    }
}

/// Format a due date timestamp into a human-readable string
fn format_due_date(timestamp: i64) -> String {
    use chrono::{DateTime, Local};

    let secs = timestamp / 1000;
    match DateTime::from_timestamp(secs, 0) {
        Some(dt) => {
            let local_dt: DateTime<Local> = dt.into();
            local_dt.format("%b %d, %Y").to_string()
        }
        None => "Unknown date".to_string(),
    }
}

/// API response for inbox activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxActivityResponse {
    #[serde(default)]
    pub activities: Vec<InboxActivity>,
}

/// Deduplicate activities by ID, keeping the most recent occurrence
///
/// When multiple activities have the same ID (e.g., same task appears in
/// multiple fetch results), this function keeps only the most recent one
/// based on timestamp.
///
/// # Arguments
/// * `activities` - Vector of activities, potentially with duplicates
///
/// # Returns
/// A new vector with duplicates removed, sorted by timestamp (newest first)
pub fn deduplicate_activities(activities: Vec<InboxActivity>) -> Vec<InboxActivity> {
    use std::collections::HashMap;

    // Build a map of id -> activity, keeping the most recent
    let mut activity_map: HashMap<String, InboxActivity> = HashMap::new();

    for activity in activities {
        activity_map
            .entry(activity.id.clone())
            .and_modify(|existing| {
                // Keep the more recent activity
                if activity.timestamp > existing.timestamp {
                    *existing = activity.clone();
                }
            })
            .or_insert(activity);
    }

    // Convert back to vector and sort by timestamp (newest first)
    let mut unique_activities: Vec<InboxActivity> = activity_map.into_values().collect();
    unique_activities.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    unique_activities
}

/// Merge multiple activity sources, deduplicating by ID
///
/// This is a convenience function that combines multiple activity vectors
/// and removes duplicates.
///
/// # Arguments
/// * `sources` - Multiple vectors of activities from different sources
///
/// # Returns
/// A single deduplicated vector of activities
pub fn merge_activity_sources(sources: Vec<Vec<InboxActivity>>) -> Vec<InboxActivity> {
    let all_activities: Vec<InboxActivity> = sources.into_iter().flatten().collect();
    deduplicate_activities(all_activities)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_type_icon() {
        assert_eq!(ActivityType::Assignment.icon(), "📋");
        assert_eq!(ActivityType::Comment.icon(), "💬");
        assert_eq!(ActivityType::StatusChange.icon(), "🔄");
        assert_eq!(ActivityType::DueDate.icon(), "⏰");
    }

    #[test]
    fn test_activity_type_label() {
        assert_eq!(ActivityType::Assignment.label(), "Assigned");
        assert_eq!(ActivityType::Comment.label(), "Comment");
        assert_eq!(ActivityType::StatusChange.label(), "Status");
        assert_eq!(ActivityType::DueDate.label(), "Due Date");
    }

    #[test]
    fn test_create_assignment_activity() {
        let activity = InboxActivity::assignment(
            "task_123".to_string(),
            "Review PR".to_string(),
            "ws_456".to_string(),
            1704067200000,
            Some("Open".to_string()),
        );

        assert_eq!(activity.id, "assignment_task_123");
        assert_eq!(activity.activity_type, ActivityType::Assignment);
        assert!(activity.title.contains("Review PR"));
        assert_eq!(activity.task_id, Some("task_123".to_string()));
    }

    #[test]
    fn test_create_comment_activity() {
        let activity = InboxActivity::comment(
            "comment_789".to_string(),
            "task_123".to_string(),
            "Review PR".to_string(),
            "ws_456".to_string(),
            1704067200000,
            "Looks good! Just a few minor changes needed.".to_string(),
        );

        assert_eq!(activity.id, "comment_comment_789");
        assert_eq!(activity.activity_type, ActivityType::Comment);
        assert!(activity.title.contains("Review PR"));
        assert_eq!(activity.comment_id, Some("comment_789".to_string()));
    }

    #[test]
    fn test_create_status_change_activity() {
        let activity = InboxActivity::status_change(
            "task_123".to_string(),
            "Review PR".to_string(),
            "ws_456".to_string(),
            1704067200000,
            "Open".to_string(),
            "In Progress".to_string(),
        );

        assert!(activity.id.starts_with("status_task_123_"));
        assert_eq!(activity.activity_type, ActivityType::StatusChange);
        assert_eq!(activity.previous_status, Some("Open".to_string()));
        assert_eq!(activity.new_status, Some("In Progress".to_string()));
    }

    #[test]
    fn test_create_due_date_activity() {
        let now = chrono::Utc::now().timestamp_millis();
        let due_date = now + (7 * 24 * 60 * 60 * 1000); // 7 days from now

        let activity = InboxActivity::due_date(
            "task_123".to_string(),
            "Review PR".to_string(),
            "ws_456".to_string(),
            due_date,
        );

        assert_eq!(activity.id, "due_task_123");
        assert_eq!(activity.activity_type, ActivityType::DueDate);
        assert_eq!(activity.due_date, Some(due_date));
        assert!(activity.title.contains("Review PR"));
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("Short text", 20), "Short text");
        assert_eq!(truncate_text("This is a longer text that should be truncated", 20), "This is a longer ...");
        assert_eq!(truncate_text("  Trimmed and truncated  ", 10), "Trimmed...");
    }

    #[test]
    fn test_inbox_activity_serialization() {
        let activity = InboxActivity::assignment(
            "task_123".to_string(),
            "Test Task".to_string(),
            "ws_456".to_string(),
            1704067200000,
            None,
        );

        let json = serde_json::to_string(&activity).unwrap();
        assert!(json.contains("assignment"));
        assert!(json.contains("task_123"));

        let deserialized: InboxActivity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, activity.id);
        assert_eq!(deserialized.activity_type, activity.activity_type);
    }

    #[test]
    fn test_assignment_id_uniqueness() {
        // Verify assignment IDs are unique per task
        let activity1 = InboxActivity::assignment(
            "task_123".to_string(),
            "Task 1".to_string(),
            "ws_456".to_string(),
            1704067200000,
            None,
        );
        let activity2 = InboxActivity::assignment(
            "task_456".to_string(),
            "Task 2".to_string(),
            "ws_456".to_string(),
            1704067200000,
            None,
        );

        assert_ne!(activity1.id, activity2.id);
        assert_eq!(activity1.id, "assignment_task_123");
        assert_eq!(activity2.id, "assignment_task_456");
    }

    #[test]
    fn test_comment_with_long_text_truncation() {
        let long_comment = "This is a very long comment that should be truncated to fit within the display constraints. ".repeat(5);
        let activity = InboxActivity::comment(
            "comment_789".to_string(),
            "task_123".to_string(),
            "Review PR".to_string(),
            "ws_456".to_string(),
            1704067200000,
            long_comment.clone(),
        );

        assert!(activity.description.len() <= 103); // 100 chars + "..."
        assert!(activity.description.ends_with("..."));
    }

    #[test]
    fn test_status_change_id_includes_timestamp() {
        let activity1 = InboxActivity::status_change(
            "task_123".to_string(),
            "Task".to_string(),
            "ws_456".to_string(),
            1704067200000,
            "Open".to_string(),
            "Closed".to_string(),
        );
        let activity2 = InboxActivity::status_change(
            "task_123".to_string(),
            "Task".to_string(),
            "ws_456".to_string(),
            1704067300000, // Different timestamp
            "Open".to_string(),
            "In Progress".to_string(),
        );

        // IDs should be different due to different timestamps
        assert_ne!(activity1.id, activity2.id);
        assert!(activity1.id.starts_with("status_task_123_"));
        assert!(activity2.id.starts_with("status_task_123_"));
    }

    #[test]
    fn test_due_date_variants() {
        let now = chrono::Utc::now().timestamp_millis();
        let day_ms = 24 * 60 * 60 * 1000;

        // Overdue (past due date)
        let overdue = InboxActivity::due_date(
            "task_1".to_string(),
            "Overdue Task".to_string(),
            "ws_1".to_string(),
            now - day_ms, // 1 day ago
        );
        assert!(overdue.title.contains("overdue"));

        // Due today
        let today = InboxActivity::due_date(
            "task_2".to_string(),
            "Today Task".to_string(),
            "ws_1".to_string(),
            now,
        );
        assert!(today.title.contains("due today"));

        // Due in 2 days (within 3 days)
        let soon = InboxActivity::due_date(
            "task_3".to_string(),
            "Soon Task".to_string(),
            "ws_1".to_string(),
            now + (2 * day_ms),
        );
        assert!(soon.title.contains("2 days"));

        // Due in 7 days (general "due soon")
        let later = InboxActivity::due_date(
            "task_4".to_string(),
            "Later Task".to_string(),
            "ws_1".to_string(),
            now + (7 * day_ms),
        );
        assert!(later.title.contains("due soon"));
    }

    #[test]
    fn test_activity_equality() {
        let activity1 = InboxActivity::assignment(
            "task_123".to_string(),
            "Task".to_string(),
            "ws_456".to_string(),
            1704067200000,
            None,
        );
        let activity2 = InboxActivity::assignment(
            "task_123".to_string(),
            "Task".to_string(),
            "ws_456".to_string(),
            1704067200000,
            None,
        );
        let activity3 = InboxActivity::assignment(
            "task_456".to_string(),
            "Task".to_string(),
            "ws_456".to_string(),
            1704067200000,
            None,
        );

        assert_eq!(activity1, activity2);
        assert_ne!(activity1, activity3);
    }

    #[test]
    fn test_optional_fields_default() {
        let activity = InboxActivity::assignment(
            "task_123".to_string(),
            "Task".to_string(),
            "ws_456".to_string(),
            1704067200000,
            None,
        );

        assert_eq!(activity.comment_id, None);
        assert_eq!(activity.previous_status, None);
        assert_eq!(activity.due_date, None);
        assert_eq!(activity.task_name, "Task".to_string()); // task_name is set from parameter
    }

    #[test]
    fn test_format_due_date_invalid() {
        // Test with a timestamp that would overflow
        let result = format_due_date(i64::MAX);
        assert_eq!(result, "Unknown date");
    }

    #[test]
    fn test_deduplicate_removes_exact_duplicates() {
        // Create two identical activities
        let activity1 = InboxActivity::assignment(
            "task_123".to_string(),
            "Task".to_string(),
            "ws_456".to_string(),
            1704067200000,
            None,
        );
        let activity2 = activity1.clone();

        let activities = vec![activity1.clone(), activity2.clone()];
        let result = deduplicate_activities(activities);

        // Should have only one activity
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "assignment_task_123");
    }

    #[test]
    fn test_deduplicate_keeps_most_recent() {
        // Create two activities with same ID but different timestamps
        let older = InboxActivity::assignment(
            "task_123".to_string(),
            "Task".to_string(),
            "ws_456".to_string(),
            1704067200000, // Older
            Some("Open".to_string()),
        );
        let newer = InboxActivity::assignment(
            "task_123".to_string(),
            "Task".to_string(),
            "ws_456".to_string(),
            1704067300000, // Newer
            Some("In Progress".to_string()),
        );

        // Add in reverse order to test that order doesn't matter
        let activities = vec![newer.clone(), older.clone()];
        let result = deduplicate_activities(activities);

        // Should keep the newer one
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].timestamp, 1704067300000);
        assert_eq!(result[0].new_status, Some("In Progress".to_string()));
    }

    #[test]
    fn test_deduplicate_preserves_different_activities() {
        // Create activities with different IDs
        let assignment = InboxActivity::assignment(
            "task_123".to_string(),
            "Task 1".to_string(),
            "ws_456".to_string(),
            1704067200000,
            None,
        );
        let comment = InboxActivity::comment(
            "comment_789".to_string(),
            "task_456".to_string(),
            "Task 2".to_string(),
            "ws_456".to_string(),
            1704067300000,
            "Test comment".to_string(),
        );

        let activities = vec![assignment.clone(), comment.clone()];
        let result = deduplicate_activities(activities);

        // Should keep both
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_deduplicate_sorts_by_timestamp() {
        // Create activities with different timestamps
        let oldest = InboxActivity::assignment(
            "task_1".to_string(),
            "Task 1".to_string(),
            "ws_1".to_string(),
            1704067200000,
            None,
        );
        let middle = InboxActivity::assignment(
            "task_2".to_string(),
            "Task 2".to_string(),
            "ws_1".to_string(),
            1704067300000,
            None,
        );
        let newest = InboxActivity::assignment(
            "task_3".to_string(),
            "Task 3".to_string(),
            "ws_1".to_string(),
            1704067400000,
            None,
        );

        // Add in random order
        let activities = vec![oldest.clone(), newest.clone(), middle.clone()];
        let result = deduplicate_activities(activities);

        // Should be sorted newest first
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].timestamp, 1704067400000);
        assert_eq!(result[1].timestamp, 1704067300000);
        assert_eq!(result[2].timestamp, 1704067200000);
    }

    #[test]
    fn test_deduplicate_empty_input() {
        let activities: Vec<InboxActivity> = vec![];
        let result = deduplicate_activities(activities);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_merge_activity_sources() {
        // Create two sources with overlapping activities
        let source1 = vec![
            InboxActivity::assignment(
                "task_1".to_string(),
                "Task 1".to_string(),
                "ws_1".to_string(),
                1704067200000,
                None,
            ),
            InboxActivity::assignment(
                "task_2".to_string(),
                "Task 2".to_string(),
                "ws_1".to_string(),
                1704067300000,
                None,
            ),
        ];

        let source2 = vec![
            InboxActivity::assignment(
                "task_2".to_string(),
                "Task 2".to_string(),
                "ws_1".to_string(),
                1704067400000, // Newer version of task_2
                None,
            ),
            InboxActivity::assignment(
                "task_3".to_string(),
                "Task 3".to_string(),
                "ws_1".to_string(),
                1704067500000,
                None,
            ),
        ];

        let result = merge_activity_sources(vec![source1, source2]);

        // Should have 3 unique activities (task_1, task_2, task_3)
        assert_eq!(result.len(), 3);

        // task_2 should be the newer version
        let task2 = result.iter().find(|a| a.task_id == Some("task_2".to_string())).unwrap();
        assert_eq!(task2.timestamp, 1704067400000);

        // Should be sorted newest first
        assert_eq!(result[0].timestamp, 1704067500000); // task_3
        assert_eq!(result[1].timestamp, 1704067400000); // task_2 (newer)
        assert_eq!(result[2].timestamp, 1704067200000); // task_1
    }

    #[test]
    fn test_merge_empty_sources() {
        let result = merge_activity_sources(vec![]);
        assert_eq!(result.len(), 0);

        let result = merge_activity_sources(vec![vec![], vec![]]);
        assert_eq!(result.len(), 0);
    }
}
