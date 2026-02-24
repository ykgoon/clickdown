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
    pub name: String,
    #[serde(default)]
    pub description: Option<TaskDescription>,
    #[serde(default)]
    pub status: Option<TaskStatus>,
    #[serde(default)]
    pub orderindex: Option<String>,
    #[serde(default)]
    pub content: Option<TaskContent>,
    #[serde(default, deserialize_with = "flexible_timestamp")]
    pub created_at: Option<i64>,
    #[serde(default, deserialize_with = "flexible_timestamp")]
    pub updated_at: Option<i64>,
    #[serde(default, deserialize_with = "flexible_timestamp")]
    pub closed_at: Option<i64>,
    #[serde(default)]
    pub creator: Option<User>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub assignees: Vec<User>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub checklists: Vec<Checklist>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub tags: Vec<Tag>,
    #[serde(default)]
    pub parent: Option<TaskReference>,
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
    #[serde(default)]
    pub list: Option<ListReference>,
    #[serde(default)]
    pub folder: Option<FolderReference>,
    #[serde(default)]
    pub space: Option<SpaceReference>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default, rename = "timeEstimate", deserialize_with = "flexible_i64")]
    pub time_estimate: Option<i64>,
    #[serde(default, rename = "timeSpent", deserialize_with = "flexible_i64")]
    pub time_spent: Option<i64>,
}

/// Task status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Checklist {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub orderindex: Option<u32>,
    #[serde(default)]
    pub resolved: bool,
}

/// Task tag
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
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
    #[serde(default)]
    pub type_field: Option<String>,
    #[serde(default)]
    pub value: Option<serde_json::Value>,
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
}

/// Reference to a List
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListReference {
    pub id: String,
    pub name: Option<String>,
}

/// Reference to a Folder
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FolderReference {
    pub id: String,
    pub name: Option<String>,
}

/// Reference to a Space
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceReference {
    pub id: String,
    pub name: Option<String>,
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
