//! Task models

use serde::{Deserialize, Serialize};

/// A ClickUp Task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<TaskStatus>,
    #[serde(default)]
    pub orderindex: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub updated_at: Option<i64>,
    #[serde(default)]
    pub closed_at: Option<i64>,
    #[serde(default)]
    pub creator: Option<User>,
    #[serde(default)]
    pub assignees: Vec<User>,
    #[serde(default)]
    pub checklists: Vec<Checklist>,
    #[serde(default)]
    pub tags: Vec<Tag>,
    #[serde(default)]
    pub parent: Option<TaskReference>,
    #[serde(default)]
    pub priority: Option<Priority>,
    #[serde(default)]
    pub due_date: Option<i64>,
    #[serde(default)]
    pub start_date: Option<i64>,
    #[serde(default)]
    pub points: Option<i32>,
    #[serde(default)]
    pub custom_fields: Vec<CustomField>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(default)]
    pub list: Option<ListReference>,
    #[serde(default)]
    pub folder: Option<FolderReference>,
    #[serde(default)]
    pub space: Option<SpaceReference>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default, rename = "timeEstimate")]
    pub time_estimate: Option<i64>,
    #[serde(default, rename = "timeSpent")]
    pub time_spent: Option<i64>,
}

/// Task status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    pub status: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub type_field: Option<String>,
    #[serde(default)]
    pub orderindex: Option<u32>,
}

/// User/Assignee reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub profile_picture: Option<String>,
}

/// Task checklist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checklist {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub orderindex: Option<u32>,
    #[serde(default)]
    pub resolved: bool,
}

/// Task tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
}

/// Task reference (for parent tasks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskReference {
    pub id: String,
    pub name: Option<String>,
}

/// Priority level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Priority {
    pub priority: String,
    #[serde(default)]
    pub color: Option<String>,
}

/// Custom field value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomField {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub type_field: Option<String>,
    #[serde(default)]
    pub value: Option<serde_json::Value>,
}

/// Attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

/// Reference to a List
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListReference {
    pub id: String,
    pub name: Option<String>,
}

/// Reference to a Folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderReference {
    pub id: String,
    pub name: Option<String>,
}

/// Reference to a Space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceReference {
    pub id: String,
    pub name: Option<String>,
}

/// API response for getting tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksResponse {
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
