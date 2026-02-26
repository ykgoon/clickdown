//! Task models

use serde::{Deserialize, Deserializer, Serialize};

/// Helper function to deserialize null as empty vec for Vec<T> fields
fn null_to_empty_vec<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

/// Helper function to deserialize null as false for bool fields
fn null_to_false<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<bool>::deserialize(deserializer).map(|opt| opt.unwrap_or(false))
}

/// Flexible deserializer for timestamp fields that can be either i64 or string
/// ClickUp API may return timestamps as either format
fn flexible_timestamp<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TimestampValue {
        Int(i64),
        String(String),
    }
    
    let opt = Option::<TimestampValue>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(TimestampValue::Int(v)) => Ok(Some(v)),
        Some(TimestampValue::String(s)) => {
            s.parse::<i64>().map(Some).map_err(D::Error::custom)
        }
    }
}

/// Flexible deserializer for integer fields that can be either i32 or string
fn flexible_int<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IntValue {
        Int(i32),
        String(String),
    }
    
    let opt = Option::<IntValue>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(IntValue::Int(v)) => Ok(Some(v)),
        Some(IntValue::String(s)) => {
            s.parse::<i32>().map(Some).map_err(D::Error::custom)
        }
    }
}

/// Flexible deserializer for i64 fields that can be either i64 or string
fn flexible_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum I64Value {
        Int(i64),
        String(String),
    }

    let opt = Option::<I64Value>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(I64Value::Int(v)) => Ok(Some(v)),
        Some(I64Value::String(s)) => {
            s.parse::<i64>().map(Some).map_err(D::Error::custom)
        }
    }
}

/// Flexible deserializer for resolved field that can be either bool or i64
/// ClickUp API may return resolved as either format
fn flexible_resolved<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ResolvedValue {
        Bool(bool),
        Int(i64),
    }

    let opt = Option::<ResolvedValue>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(ResolvedValue::Bool(b)) => Ok(Some(if b { 1 } else { 0 })),
        Some(ResolvedValue::Int(v)) => Ok(Some(v)),
    }
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
            TaskDescription::Rich { markdown, text, html, .. } => {
                markdown.clone()
                    .or_else(|| text.clone())
                    .or_else(|| html.clone())
                    .unwrap_or_default()
            }
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

impl TaskContent {
    /// Get the content as a string
    pub fn as_text(&self) -> String {
        match self {
            TaskContent::Plain(s) => s.clone(),
            TaskContent::Rich { html } => html.clone().unwrap_or_default(),
        }
    }
}

/// A ClickUp Task
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    #[serde(default, rename = "date_created", deserialize_with = "flexible_timestamp")]
    pub created_at: Option<i64>,
    #[serde(default, rename = "date_updated", deserialize_with = "flexible_timestamp")]
    pub updated_at: Option<i64>,
    #[serde(default, rename = "date_closed", deserialize_with = "flexible_timestamp")]
    pub closed_at: Option<i64>,
    #[serde(default, rename = "date_done", deserialize_with = "flexible_timestamp")]
    pub done_at: Option<i64>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub creator: Option<User>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub assignees: Vec<User>,
    #[serde(default, deserialize_with = "null_to_empty_vec", rename = "group_assignees")]
    pub group_assignees: Vec<User>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub watchers: Vec<User>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub checklists: Vec<Checklist>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub tags: Vec<Tag>,
    #[serde(default)]
    pub parent: Option<TaskReference>,
    #[serde(default, rename = "top_level_parent")]
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
    #[serde(default, deserialize_with = "null_to_empty_vec", rename = "linked_tasks")]
    pub linked_tasks: Vec<LinkedTask>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub locations: Vec<serde_json::Value>,
    #[serde(default)]
    pub list: Option<ListReference>,
    #[serde(default)]
    pub folder: Option<FolderReference>,
    #[serde(default)]
    pub space: Option<SpaceReference>,
    #[serde(default, rename = "project")]
    pub project: Option<ProjectReference>,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default, rename = "profilePicture")]
    pub profile_picture: Option<String>,
    #[serde(default)]
    pub initials: Option<String>,
}

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

/// Task dependency reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskDependency {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, rename = "status")]
    pub dependency_status: Option<String>,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskReference {
    pub id: String,
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
    #[serde(default, rename = "date_created", deserialize_with = "flexible_timestamp")]
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
    #[serde(default)]
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

/// Reference to a List
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListReference {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub access: Option<bool>,
}

/// Reference to a Folder
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FolderReference {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub hidden: Option<bool>,
    #[serde(default)]
    pub access: Option<bool>,
}

/// Reference to a Project (folder in API response)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectReference {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub hidden: Option<bool>,
    #[serde(default)]
    pub access: Option<bool>,
}

/// Reference to a Space
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceReference {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub access: Option<bool>,
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
    pub order_by: Option<String>,
    pub reverse: Option<bool>,
    pub subtasks: Option<bool>,
    pub statuses: Vec<String>,
    pub assignees: Vec<i64>,
    pub include_markdown_description: Option<bool>,
}

impl TaskFilters {
    /// Convert filters to query parameters
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if let Some(archived) = self.archived {
            params.push(format!("archived={}", archived));
        }
        if let Some(page) = self.page {
            params.push(format!("page={}", page));
        }
        if let Some(ref order_by) = self.order_by {
            params.push(format!("order_by={}", order_by));
        }
        if let Some(reverse) = self.reverse {
            params.push(format!("reverse={}", reverse));
        }
        if let Some(subtasks) = self.subtasks {
            params.push(format!("subtasks={}", subtasks));
        }
        for status in &self.statuses {
            params.push(format!("statuses[]={}", status));
        }
        for assignee in &self.assignees {
            params.push(format!("assignees[]={}", assignee));
        }
        if let Some(include_md) = self.include_markdown_description {
            params.push(format!("include_markdown_description={}", include_md));
        }

        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
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

/// Request body for updating a task
#[derive(Debug, Clone, Serialize)]
pub struct UpdateTaskRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
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

/// Status group priority for sorting
/// Lower priority value = higher in the list
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatusGroupPriority {
    InProgress = 1,
    ToDo = 2,
    Done = 3,
    Fallback = 4,
}

/// Get the sort priority for a status group string
/// Returns the priority value (lower = higher priority in sort order)
pub fn get_status_group_priority(status_group: Option<&str>) -> StatusGroupPriority {
    match status_group {
        Some(group) => {
            match group.to_lowercase().as_str() {
                "in_progress" | "in progress" => StatusGroupPriority::InProgress,
                "todo" | "to_do" => StatusGroupPriority::ToDo,
                "done" | "complete" => StatusGroupPriority::Done,
                _ => {
                    // Unknown status group - log for debugging and use fallback
                    tracing::debug!("Unknown status_group: '{}', using fallback priority", group);
                    StatusGroupPriority::Fallback
                }
            }
        }
        None => StatusGroupPriority::Fallback,
    }
}

/// Get the sort key for a task
/// Returns a tuple of (status_priority, updated_at) for sorting
/// - status_priority: lower value = higher priority (in-progress first)
/// - updated_at: higher value = more recent (sorted descending within group)
///   Tasks without updated_at get i64::MIN to sort last within their group
fn get_task_sort_key(task: &Task) -> (StatusGroupPriority, i64) {
    let status_priority = get_status_group_priority(
        task.status.as_ref().and_then(|s| s.status_group.as_deref())
    );
    
    // Use updated_at for recency sorting within status group
    // Tasks without updated_at get MIN value to sort last
    let updated_at = task.updated_at.unwrap_or(i64::MIN);
    
    (status_priority, updated_at)
}

/// Sort tasks by status priority and recency
/// 
/// Sorting rules:
/// 1. Group by status: in-progress → to-do → done → fallback
/// 2. Within each group, sort by updated_at descending (most recent first)
/// 3. Tasks without updated_at are placed last within their status group
/// 
/// This function sorts in-place and returns the sorted vector for convenience.
pub fn sort_tasks(mut tasks: Vec<Task>) -> Vec<Task> {
    tasks.sort_by(|a, b| {
        let (priority_a, updated_a) = get_task_sort_key(a);
        let (priority_b, updated_b) = get_task_sort_key(b);
        
        // First sort by status group priority (ascending - lower priority value first)
        match priority_a.cmp(&priority_b) {
            std::cmp::Ordering::Equal => {
                // Within same status group, sort by updated_at descending (most recent first)
                updated_b.cmp(&updated_a)
            }
            other => other,
        }
    });
    
    tasks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_task_with_status_and_updated_at(
        id: &str,
        status_group: Option<&str>,
        updated_at: Option<i64>,
    ) -> Task {
        Task {
            id: id.to_string(),
            custom_id: None,
            custom_item_id: None,
            name: format!("Task {}", id),
            text_content: None,
            description: None,
            markdown_description: None,
            status: Some(TaskStatus {
                id: None,
                status: "test".to_string(),
                color: None,
                type_field: None,
                orderindex: None,
                status_group: status_group.map(|s| s.to_string()),
            }),
            orderindex: None,
            content: None,
            created_at: None,
            updated_at,
            closed_at: None,
            done_at: None,
            archived: None,
            creator: None,
            assignees: vec![],
            group_assignees: vec![],
            watchers: vec![],
            checklists: vec![],
            tags: vec![],
            parent: None,
            top_level_parent: None,
            priority: None,
            due_date: None,
            start_date: None,
            points: None,
            custom_fields: vec![],
            attachments: vec![],
            dependencies: vec![],
            linked_tasks: vec![],
            locations: vec![],
            list: None,
            folder: None,
            space: None,
            project: None,
            url: None,
            team_id: None,
            sharing: None,
            permission_level: None,
            time_estimate: None,
            time_spent: None,
        }
    }

    #[test]
    fn test_status_group_priority_mapping() {
        assert_eq!(get_status_group_priority(Some("in_progress")), StatusGroupPriority::InProgress);
        assert_eq!(get_status_group_priority(Some("IN_PROGRESS")), StatusGroupPriority::InProgress);
        assert_eq!(get_status_group_priority(Some("in progress")), StatusGroupPriority::InProgress);
        
        assert_eq!(get_status_group_priority(Some("todo")), StatusGroupPriority::ToDo);
        assert_eq!(get_status_group_priority(Some("to_do")), StatusGroupPriority::ToDo);
        
        assert_eq!(get_status_group_priority(Some("done")), StatusGroupPriority::Done);
        assert_eq!(get_status_group_priority(Some("complete")), StatusGroupPriority::Done);
        
        assert_eq!(get_status_group_priority(Some("unknown")), StatusGroupPriority::Fallback);
        assert_eq!(get_status_group_priority(None), StatusGroupPriority::Fallback);
    }

    #[test]
    fn test_sort_tasks_by_status_priority() {
        let tasks = vec![
            create_task_with_status_and_updated_at("done1", Some("done"), Some(1000)),
            create_task_with_status_and_updated_at("todo1", Some("todo"), Some(1000)),
            create_task_with_status_and_updated_at("progress1", Some("in_progress"), Some(1000)),
        ];

        let sorted = sort_tasks(tasks);

        assert_eq!(sorted[0].id, "progress1");
        assert_eq!(sorted[1].id, "todo1");
        assert_eq!(sorted[2].id, "done1");
    }

    #[test]
    fn test_sort_tasks_by_recency_within_status_group() {
        let tasks = vec![
            create_task_with_status_and_updated_at("old", Some("in_progress"), Some(1000)),
            create_task_with_status_and_updated_at("new", Some("in_progress"), Some(3000)),
            create_task_with_status_and_updated_at("mid", Some("in_progress"), Some(2000)),
        ];

        let sorted = sort_tasks(tasks);

        assert_eq!(sorted[0].id, "new");
        assert_eq!(sorted[1].id, "mid");
        assert_eq!(sorted[2].id, "old");
    }

    #[test]
    fn test_sort_tasks_missing_updated_at() {
        let tasks = vec![
            create_task_with_status_and_updated_at("with_date", Some("todo"), Some(1000)),
            create_task_with_status_and_updated_at("no_date", Some("todo"), None),
        ];

        let sorted = sort_tasks(tasks);

        assert_eq!(sorted[0].id, "with_date");
        assert_eq!(sorted[1].id, "no_date");
    }

    #[test]
    fn test_sort_tasks_missing_status() {
        let tasks = vec![
            create_task_with_status_and_updated_at("with_status", Some("todo"), Some(1000)),
            create_task_with_status_and_updated_at("no_status", None, Some(1000)),
        ];

        let sorted = sort_tasks(tasks);

        assert_eq!(sorted[0].id, "with_status");
        assert_eq!(sorted[1].id, "no_status");
    }

    #[test]
    fn test_sort_tasks_empty_list() {
        let tasks: Vec<Task> = vec![];
        let sorted = sort_tasks(tasks);
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_sort_tasks_single_task() {
        let tasks = vec![
            create_task_with_status_and_updated_at("single", Some("in_progress"), Some(1000)),
        ];
        let sorted = sort_tasks(tasks);
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted[0].id, "single");
    }

    #[test]
    fn test_sort_tasks_all_same_status() {
        let tasks = vec![
            create_task_with_status_and_updated_at("a", Some("done"), Some(1000)),
            create_task_with_status_and_updated_at("b", Some("done"), Some(3000)),
            create_task_with_status_and_updated_at("c", Some("done"), Some(2000)),
        ];

        let sorted = sort_tasks(tasks);

        assert_eq!(sorted[0].id, "b");
        assert_eq!(sorted[1].id, "c");
        assert_eq!(sorted[2].id, "a");
    }
}
