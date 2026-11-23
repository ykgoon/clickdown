//! Task models

use crate::models::entity_ref::EntityRef;
use crate::utils::deserializers::{
    flexible_i64, flexible_int, flexible_resolved, flexible_timestamp, null_to_empty_vec,
    null_to_false,
};
use serde::{Deserialize, Serialize};

/// Deserialize a task reference that the API may send as a bare ID string.
///
/// Verified against the live API: `parent` and `top_level_parent` come back as
/// plain ID strings (`"parent": "862kb0dqp"`) on `GET /team/{id}/task` (with
/// `subtasks=true`), on `GET /list/{id}/task`, and on `GET /task/{id}`. Only
/// some older payloads carry the object form (`{"id": ..., "name": ...}`), so
/// both are accepted and normalised to a [`TaskReference`]; `null` or a
/// missing field stays `None`.
fn task_reference<'de, D>(deserializer: D) -> Result<Option<TaskReference>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ReferenceValue {
        Id(String),
        Object(TaskReference),
    }

    let opt = Option::<ReferenceValue>::deserialize(deserializer)?;
    Ok(match opt {
        None => None,
        Some(ReferenceValue::Id(id)) => {
            let id = id.trim().to_string();
            if id.is_empty() {
                None
            } else {
                Some(TaskReference { id, name: None })
            }
        }
        Some(ReferenceValue::Object(reference)) => Some(reference),
    })
}

/// Flexible description type that can be either a plain string or an object
/// with HTML/markdown content (ClickUp API can return both formats)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskDescription {
    /// Plain text description
    Plain(String),
    /// Rich text description with HTML and/or markdown
    Rich {
        #[serde(default)]
        html: Option<String>,
        #[serde(default)]
        markdown: Option<String>,
        #[serde(default)]
        text: Option<String>,
    },
}

impl TaskDescription {
    /// Get the description as a string, preferring markdown > text > html > empty
    pub fn as_text(&self) -> String {
        match self {
            TaskDescription::Plain(s) => s.clone(),
            TaskDescription::Rich {
                markdown,
                text,
                html,
                ..
            } => markdown
                .clone()
                .or_else(|| text.clone())
                .or_else(|| html.clone())
                .unwrap_or_default(),
        }
    }
}

/// Flexible content type that can be either a plain string or an object
/// with HTML content (ClickUp API can return both formats)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskContent {
    /// Plain text content
    Plain(String),
    /// HTML content
    Rich {
        #[serde(default)]
        html: Option<String>,
    },
}

/// A ClickUp Task
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Task {
    pub id: String,
    #[serde(default)]
    pub custom_id: Option<String>,
    #[serde(default, rename = "custom_item_id", deserialize_with = "flexible_int")]
    pub custom_item_id: Option<i32>,
    pub name: String,
    #[serde(default)]
    pub text_content: Option<String>,
    #[serde(default)]
    pub description: Option<TaskDescription>,
    #[serde(default, rename = "markdown_description")]
    pub markdown_description: Option<String>,
    #[serde(default)]
    pub status: Option<TaskStatus>,
    #[serde(default)]
    pub orderindex: Option<String>,
    #[serde(default)]
    pub content: Option<TaskContent>,
    #[serde(
        default,
        rename = "date_created",
        deserialize_with = "flexible_timestamp"
    )]
    pub created_at: Option<i64>,
    #[serde(
        default,
        rename = "date_updated",
        deserialize_with = "flexible_timestamp"
    )]
    pub updated_at: Option<i64>,
    #[serde(
        default,
        rename = "date_closed",
        deserialize_with = "flexible_timestamp"
    )]
    pub closed_at: Option<i64>,
    #[serde(default, rename = "date_done", deserialize_with = "flexible_timestamp")]
    pub done_at: Option<i64>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub creator: Option<User>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub assignees: Vec<User>,
    #[serde(
        default,
        deserialize_with = "null_to_empty_vec",
        rename = "group_assignees"
    )]
    pub group_assignees: Vec<User>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub watchers: Vec<User>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub checklists: Vec<Checklist>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub tags: Vec<Tag>,
    #[serde(default, deserialize_with = "task_reference")]
    pub parent: Option<TaskReference>,
    #[serde(default, rename = "top_level_parent", deserialize_with = "task_reference")]
    pub top_level_parent: Option<TaskReference>,
    #[serde(default)]
    pub priority: Option<Priority>,
    #[serde(default, deserialize_with = "flexible_timestamp")]
    pub due_date: Option<i64>,
    #[serde(default, deserialize_with = "flexible_timestamp")]
    pub start_date: Option<i64>,
    #[serde(default, deserialize_with = "flexible_int")]
    pub points: Option<i32>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub custom_fields: Vec<CustomField>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub attachments: Vec<Attachment>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub dependencies: Vec<TaskDependencyRef>,
    #[serde(
        default,
        deserialize_with = "null_to_empty_vec",
        rename = "linked_tasks"
    )]
    pub linked_tasks: Vec<LinkedTask>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub locations: Vec<serde_json::Value>,
    #[serde(default)]
    pub list: Option<EntityRef>,
    #[serde(default)]
    pub folder: Option<EntityRef>,
    #[serde(default)]
    pub space: Option<EntityRef>,
    #[serde(default, rename = "project")]
    pub project: Option<EntityRef>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default, rename = "team_id")]
    pub team_id: Option<String>,
    #[serde(default)]
    pub sharing: Option<TaskSharing>,
    #[serde(default)]
    pub permission_level: Option<String>,
    #[serde(default, alias = "timeEstimate", deserialize_with = "flexible_i64")]
    pub time_estimate: Option<i64>,
    #[serde(default, alias = "timeSpent", deserialize_with = "flexible_i64")]
    pub time_spent: Option<i64>,
}

/// Task status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskStatus {
    #[serde(default)]
    pub id: Option<String>,
    pub status: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default, rename = "type")]
    pub type_field: Option<String>,
    #[serde(default)]
    pub orderindex: Option<u32>,
    #[serde(default, rename = "status_group")]
    pub status_group: Option<String>,
}

/// User/Assignee reference
/// Re-exported from crate::models::User for backwards compatibility
pub use crate::models::user::User;

/// Task checklist item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub orderindex: Option<f64>,
    #[serde(default)]
    pub assignee: Option<serde_json::Value>,
    #[serde(default)]
    pub group_assignee: Option<serde_json::Value>,
    #[serde(default, deserialize_with = "null_to_false")]
    pub resolved: bool,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub date_created: Option<String>,
    #[serde(default)]
    pub start_date: Option<serde_json::Value>,
    #[serde(default)]
    pub start_date_time: Option<bool>,
    #[serde(default)]
    pub due_date: Option<serde_json::Value>,
    #[serde(default)]
    pub due_date_time: Option<bool>,
    #[serde(default)]
    pub sent_due_date_notif: Option<serde_json::Value>,
    #[serde(default)]
    pub children: Vec<serde_json::Value>,
}

/// Task checklist
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Checklist {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub orderindex: Option<u32>,
    #[serde(default, deserialize_with = "flexible_resolved")]
    pub resolved: Option<i64>,
    #[serde(default, deserialize_with = "flexible_resolved")]
    pub unresolved: Option<i64>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub date_created: Option<String>,
    #[serde(default)]
    pub creator: Option<i64>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub items: Vec<ChecklistItem>,
}

/// Task tag
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default, alias = "tag_fg")]
    pub tag_fg: Option<String>,
    #[serde(default, alias = "tag_bg")]
    pub tag_bg: Option<String>,
    #[serde(default)]
    pub creator: Option<i64>,
}

/// Task dependency reference (for dependencies array in task response)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskDependencyRef {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, alias = "task_id")]
    pub task_id: Option<String>,
    #[serde(default, alias = "depends_on")]
    pub depends_on: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, rename = "status")]
    pub dependency_status: Option<String>,
    #[serde(default)]
    pub type_field: Option<i64>,
    #[serde(default)]
    pub date_created: Option<String>,
    #[serde(default)]
    pub userid: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub chain_id: Option<serde_json::Value>,
}

/// Linked task reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkedTask {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, alias = "task_id")]
    pub task_id: Option<String>,
    #[serde(default)]
    pub link_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub date_created: Option<String>,
    #[serde(default)]
    pub userid: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
}

/// Task reference (for parent tasks)
///
/// The API sends this as a bare ID string, so `name` is only present when a
/// payload happens to include the object form. Treat `id` as the source of
/// truth and never rely on `name`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskReference {
    #[serde(default, deserialize_with = "crate::utils::deserializers::flexible_string")]
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
}

/// Priority level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Priority {
    pub priority: String,
    #[serde(default)]
    pub color: Option<String>,
}

/// Custom field value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomField {
    pub id: String,
    pub name: String,
    #[serde(default, rename = "type")]
    pub type_field: Option<String>,
    #[serde(default)]
    pub value: Option<serde_json::Value>,
    #[serde(default, rename = "value_richtext")]
    pub value_richtext: Option<serde_json::Value>,
    #[serde(default)]
    pub type_config: Option<serde_json::Value>,
    #[serde(
        default,
        rename = "date_created",
        deserialize_with = "flexible_timestamp"
    )]
    pub date_created: Option<i64>,
    #[serde(default, rename = "hide_from_guests")]
    pub hide_from_guests: Option<bool>,
    #[serde(default)]
    pub required: Option<bool>,
}

/// Attachment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    #[serde(default)]
    pub version: Option<i64>,
    /// ClickUp sends the attachment epoch-millis timestamp as a JSON string
    /// (e.g. "1793777721572"), unlike most other date fields which arrive
    /// as numbers. Accept both or tasks with attachments fail to parse.
    #[serde(default, deserialize_with = "flexible_i64")]
    pub date: Option<i64>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub extension: Option<String>,
    #[serde(default, rename = "thumbnailSmall")]
    pub thumbnail_small: Option<String>,
    #[serde(default, rename = "thumbnailLarge")]
    pub thumbnail_large: Option<String>,
}

/// Task sharing settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskSharing {
    #[serde(default)]
    pub public: Option<bool>,
    #[serde(default, rename = "public_share_expires_on")]
    pub public_share_expires_on: Option<i64>,
    #[serde(default, rename = "public_fields")]
    pub public_fields: Vec<String>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default, rename = "seo_optimized")]
    pub seo_optimized: Option<bool>,
}

/// API response for getting tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksResponse {
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub tasks: Vec<Task>,
}

/// Parameters for filtering tasks
#[derive(Debug, Clone, Default)]
pub struct TaskFilters {
    pub archived: Option<bool>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub order_by: Option<String>,
    pub reverse: Option<bool>,
    pub subtasks: Option<bool>,
    pub statuses: Vec<String>,
    pub assignees: Vec<i64>,
    pub include_markdown_description: Option<bool>,
    /// Filter tasks updated after this timestamp (milliseconds)
    pub date_updated_gt: Option<i64>,
    /// Filter tasks with due date less than this timestamp (milliseconds)
    pub due_date_lt: Option<i64>,
    /// Include closed tasks in results
    pub include_closed: Option<bool>,
}

impl TaskFilters {
    /// Convert filters to query parameters
    pub fn to_query_string(&self) -> String {
        use crate::utils::QueryParams;

        let mut params = QueryParams::new();
        params.add_opt("archived", self.archived);
        params.add_opt("page", self.page);
        params.add_opt("limit", self.limit);
        params.add_opt("order_by", self.order_by.as_ref());
        params.add_opt("reverse", self.reverse);
        params.add_opt("subtasks", self.subtasks);
        params.add_all("statuses", &self.statuses);
        // Use array format for assignees (ClickUp API expects assignees[]=123&assignees[]=456)
        params.add_all("assignees", &self.assignees);
        params.add_opt(
            "include_markdown_description",
            self.include_markdown_description,
        );
        params.add_opt("date_updated_gt", self.date_updated_gt);
        params.add_opt("due_date_lt", self.due_date_lt);
        params.add_opt("include_closed", self.include_closed);

        params.to_query_string()
    }
}

/// Request body for creating a task
#[derive(Debug, Clone, Serialize)]
pub struct CreateTaskRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<i64>,
}

/// Request body for updating assignees
/// ClickUp API expects: {"add": [ids], "rem": [ids]}
#[derive(Debug, Clone, Serialize)]
pub struct AssigneesUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rem: Option<Vec<i64>>,
}

impl AssigneesUpdate {
    pub fn replace_all(
        original: std::collections::HashSet<i64>,
        new: std::collections::HashSet<i64>,
    ) -> Self {
        let mut add: Vec<i64> = new.difference(&original).copied().collect();
        add.sort();
        let mut rem: Vec<i64> = original.difference(&new).copied().collect();
        rem.sort();
        Self {
            add: if add.is_empty() { None } else { Some(add) },
            rem: if rem.is_empty() { None } else { Some(rem) },
        }
    }
}

/// Request body for updating a task
#[derive(Debug, Clone, Serialize)]
pub struct UpdateTaskRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Markdown form of the description. Used instead of `description` when
    /// the text contains an @mention, because that is the only form ClickUp
    /// turns into a real user tag (`[@Name](#user_mention#<id>)`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<AssigneesUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<i64>,
}

/// Resolve the canonical status group name from a `TaskStatus`.
///
/// Resolution order:
/// 1. `status_group` field if present
/// 2. `status` field (the human-readable status name) as fallback
/// 3. `"other"` as the last resort
///
/// This ensures that tasks are grouped by their actual status even when the
/// API does not return a `status_group` value.
pub fn resolve_status_group(status: Option<&TaskStatus>) -> String {
    match status {
        Some(s) => s
            .status_group
            .clone()
            .filter(|g| !g.is_empty())
            .unwrap_or_else(|| {
                if s.status.is_empty() {
                    "other".to_string()
                } else {
                    s.status.clone()
                }
            }),
        None => "other".to_string(),
    }
}

/// Recency key for a task: last touched, then created, then id.
///
/// Ties break deterministically so repeated loads do not reshuffle equal rows.
/// Missing timestamps sort last rather than jumping to the top.
pub fn recency_key(task: &Task) -> (i64, i64, &str) {
    (
        task.updated_at.unwrap_or(i64::MIN),
        task.created_at.unwrap_or(i64::MIN),
        task.id.as_str(),
    )
}

/// Sort tasks newest-first.
///
/// This is the only ordering the task list uses: the freshest work is at the
/// top regardless of status, and status groups follow the same rule (see
/// `GroupedTaskList`).
pub fn sort_tasks_by_recency(tasks: &mut [Task]) {
    tasks.sort_by(|a, b| {
        let (updated_a, created_a, id_a) = recency_key(a);
        let (updated_b, created_b, id_b) = recency_key(b);
        updated_b
            .cmp(&updated_a)
            .then_with(|| created_b.cmp(&created_a))
            .then_with(|| id_a.cmp(id_b))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_status_group() {
        use super::{resolve_status_group, TaskStatus};

        fn make_status(status_group: Option<&str>, status: &str) -> TaskStatus {
            TaskStatus {
                id: None,
                status: status.to_string(),
                color: None,
                type_field: None,
                orderindex: None,
                status_group: status_group.map(|s| s.to_string()),
            }
        }

        // status_group present → use it
        assert_eq!(
            resolve_status_group(Some(&make_status(Some("in_progress"), "In Progress"))),
            "in_progress"
        );

        // status_group None → fall back to status.status
        assert_eq!(
            resolve_status_group(Some(&make_status(None, "In Progress"))),
            "In Progress"
        );

        // status_group empty string → fall back to status.status
        assert_eq!(
            resolve_status_group(Some(&make_status(Some(""), "Review"))),
            "Review"
        );

        // Both empty → "other"
        assert_eq!(resolve_status_group(Some(&make_status(None, ""))), "other");

        // None status → "other"
        assert_eq!(resolve_status_group(None), "other");
    }

    /// Regression: "Assigned to Me" failed with "Failed to parse team tasks"
    /// because `GET /team/{id}/task?subtasks=true` returns subtasks whose
    /// `parent` is a bare ID string, not the object the model expected.
    fn task_at(id: &str, status_group: Option<&str>, updated_at: Option<i64>) -> Task {
        Task {
            id: id.to_string(),
            name: format!("Task {}", id),
            status: Some(TaskStatus {
                id: None,
                status: "test".to_string(),
                color: None,
                type_field: None,
                orderindex: None,
                status_group: status_group.map(|s| s.to_string()),
            }),
            updated_at,
            ..Default::default()
        }
    }

    #[test]
    fn test_sort_tasks_orders_by_recency_regardless_of_status() {
        // Deliberately interleaved statuses: recency must win over status.
        let mut tasks = vec![
            task_at("done", Some("done"), Some(1000)),
            task_at("todo", Some("todo"), Some(3000)),
            task_at("progress", Some("in_progress"), Some(2000)),
        ];

        sort_tasks_by_recency(&mut tasks);

        assert_eq!(
            tasks.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(),
            vec!["todo", "progress", "done"]
        );
    }

    #[test]
    fn test_sort_tasks_missing_timestamp_sorts_last() {
        let mut tasks = vec![
            task_at("undated", Some("todo"), None),
            task_at("dated", Some("todo"), Some(1000)),
        ];

        sort_tasks_by_recency(&mut tasks);

        assert_eq!(tasks[0].id, "dated");
        assert_eq!(tasks[1].id, "undated");
    }

    #[test]
    fn test_sort_tasks_is_stable_on_equal_timestamps() {
        let mut tasks = vec![
            task_at("b", Some("todo"), Some(1000)),
            task_at("a", Some("done"), Some(1000)),
        ];

        sort_tasks_by_recency(&mut tasks);

        // Same timestamps: the id breaks the tie, so repeated loads agree.
        assert_eq!(tasks[0].id, "a");
        assert_eq!(tasks[1].id, "b");
    }

    #[test]
    fn test_sort_tasks_handles_empty_and_single() {
        let mut empty: Vec<Task> = vec![];
        sort_tasks_by_recency(&mut empty);
        assert!(empty.is_empty());

        let mut one = vec![task_at("only", Some("todo"), Some(1000))];
        sort_tasks_by_recency(&mut one);
        assert_eq!(one.len(), 1);
    }

    #[test]
    fn test_resolve_status_group_prefers_status_group_then_status_then_other() {
        use super::{resolve_status_group, TaskStatus};

        fn make(status_group: Option<&str>, status: &str) -> TaskStatus {
            TaskStatus {
                id: None,
                status: status.to_string(),
                color: None,
                type_field: None,
                orderindex: None,
                status_group: status_group.map(|s| s.to_string()),
            }
        }

        assert_eq!(
            resolve_status_group(Some(&make(Some("in_progress"), "In Progress"))),
            "in_progress"
        );
        assert_eq!(
            resolve_status_group(Some(&make(None, "In Progress"))),
            "In Progress"
        );
        assert_eq!(resolve_status_group(Some(&make(Some(""), "Review"))), "Review");
        assert_eq!(resolve_status_group(Some(&make(None, ""))), "other");
        assert_eq!(resolve_status_group(None), "other");
    }

    #[test]
    fn test_task_accepts_parent_as_bare_id_string() {
        let json = r#"{
            "id": "862kbvqdr",
            "name": "Run FusionAuth container",
            "parent": "862kb0dqp",
            "top_level_parent": "862kb0dqp"
        }"#;
        let task: Task = serde_json::from_str(json).expect("string parent must parse");
        let parent = task.parent.expect("parent must be present");
        assert_eq!(parent.id, "862kb0dqp");
        assert_eq!(parent.name, None, "the string form carries no name");
        assert_eq!(
            task.top_level_parent.map(|p| p.id),
            Some("862kb0dqp".to_string())
        );
    }

    /// The object form still parses, so older payloads keep working.
    #[test]
    fn test_task_accepts_parent_as_object() {
        let json = r#"{
            "id": "1",
            "name": "Child",
            "parent": {"id": "9", "name": "Parent"},
            "top_level_parent": {"id": "9", "name": "Parent"}
        }"#;
        let task: Task = serde_json::from_str(json).expect("object parent must parse");
        let parent = task.parent.expect("parent must be present");
        assert_eq!(parent.id, "9");
        assert_eq!(parent.name, Some("Parent".to_string()));
    }

    /// Null and missing parents stay `None` rather than becoming empty refs.
    #[test]
    fn test_task_parent_null_and_missing_are_none() {
        let nulled: Task = serde_json::from_str(
            r#"{"id": "1", "name": "x", "parent": null, "top_level_parent": null}"#,
        )
        .expect("null parent must parse");
        assert!(nulled.parent.is_none());
        assert!(nulled.top_level_parent.is_none());

        let missing: Task =
            serde_json::from_str(r#"{"id": "1", "name": "x"}"#).expect("missing parent must parse");
        assert!(missing.parent.is_none());
        assert!(missing.top_level_parent.is_none());

        // An empty string is not a usable reference.
        let empty: Task = serde_json::from_str(r#"{"id": "1", "name": "x", "parent": ""}"#)
            .expect("empty parent must parse");
        assert!(empty.parent.is_none());
    }

    /// The real failure came from a list of tasks: one subtask poisoned the
    /// whole page. Structurally the page is a `Vec<Task>`, so prove a mixed
    /// page parses.
    #[test]
    fn test_team_task_page_with_mixed_parents_parses() {
        let json = r#"{
            "tasks": [
                {"id": "86cvd1p4n", "name": "Top level", "parent": null},
                {"id": "862kbvqdr", "name": "Subtask", "parent": "862kb0dqp",
                 "top_level_parent": "862kb0dqp"}
            ]
        }"#;
        #[derive(serde::Deserialize)]
        struct Page {
            tasks: Vec<Task>,
        }
        let page: Page = serde_json::from_str(json).expect("mixed page must parse");
        assert_eq!(page.tasks.len(), 2);
        assert!(page.tasks[0].parent.is_none());
        assert_eq!(
            page.tasks[1].parent.as_ref().map(|p| p.id.as_str()),
            Some("862kb0dqp")
        );
    }

    #[test]
    fn test_task_filters_assignees_format() {
        // Test that assignees are formatted as array values (assignees[]=123&assignees[]=456)
        let filters = TaskFilters {
            assignees: vec![123, 456, 789],
            ..Default::default()
        };

        let query = filters.to_query_string();

        // Should use array format, not comma-separated
        assert!(query.contains("assignees[]=123"));
        assert!(query.contains("assignees[]=456"));
        assert!(query.contains("assignees[]=789"));
        assert!(!query.contains("assignees=123,456"));
    }

    #[test]
    fn test_task_filters_single_assignee() {
        let filters = TaskFilters {
            assignees: vec![123],
            ..Default::default()
        };

        let query = filters.to_query_string();

        assert_eq!(query, "?assignees[]=123");
    }

    #[test]
    fn test_task_filters_no_assignees() {
        let filters = TaskFilters::default();
        let query = filters.to_query_string();

        // Empty filters should produce empty query or no assignees param
        assert!(!query.contains("assignees"));
    }

    #[test]
    fn test_task_filters_with_limit() {
        let filters = TaskFilters {
            assignees: vec![123],
            limit: Some(100),
            ..Default::default()
        };

        let query = filters.to_query_string();

        // Should use array format for assignees and include limit
        assert!(query.contains("assignees[]=123"));
        assert!(query.contains("limit=100"));
    }

    #[test]
    fn test_task_filters_mixed_with_assignees() {
        let filters = TaskFilters {
            archived: Some(false),
            assignees: vec![123, 456],
            statuses: vec!["todo".to_string(), "in progress".to_string()],
            ..Default::default()
        };

        let query = filters.to_query_string();

        assert!(query.contains("archived=false"));
        assert!(query.contains("assignees[]=123"));
        assert!(query.contains("assignees[]=456"));
        assert!(query.contains("statuses[]=todo"));
        assert!(query.contains("statuses[]=in%20progress"));
    }

    #[test]
    fn test_task_filters_assigned_with_include_closed() {
        let filters = TaskFilters {
            assignees: vec![42],
            include_closed: Some(true),
            ..Default::default()
        };

        let query = filters.to_query_string();

        assert!(
            query.contains("assignees[]=42"),
            "Should include assignee: {}",
            query
        );
        assert!(
            query.contains("include_closed=true"),
            "Should include include_closed: {}",
            query
        );
    }
}
