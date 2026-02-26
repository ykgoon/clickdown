//! Comment models for ClickUp API

use serde::{Deserialize, Serialize};

/// Helper function to deserialize null as empty string
fn null_to_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

/// Helper function to deserialize null as false for bool fields
fn null_to_false<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<bool>::deserialize(deserializer).map(|opt| opt.unwrap_or(false))
}

/// Flexible timestamp deserializer for comment fields (handles i64 or string)
fn flexible_timestamp<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
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

/// A ClickUp Comment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    #[serde(default, deserialize_with = "null_to_empty_string", rename = "comment_text")]
    pub text: String,
    #[serde(default, deserialize_with = "null_to_empty_string", rename = "text_preview")]
    pub text_preview: String,
    #[serde(default, rename = "user")]
    pub commenter: Option<User>,
    #[serde(default, rename = "date", deserialize_with = "flexible_timestamp")]
    pub created_at: Option<i64>,
    #[serde(default, rename = "date_updated", deserialize_with = "flexible_timestamp")]
    pub updated_at: Option<i64>,
    #[serde(default, rename = "assignee")]
    pub assigned_commenter: Option<User>,
    #[serde(default, rename = "assigned_by")]
    pub assigned_by: Option<User>,
    #[serde(default, deserialize_with = "null_to_false", rename = "resolved")]
    pub assigned: bool,
    #[serde(default)]
    pub reaction: String,
    #[serde(default, rename = "parent_id", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

/// User reference in comment context
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

/// API response for getting task comments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentsResponse {
    #[serde(default, rename = "comments")]
    pub comments: Vec<Comment>,
}

/// Request body for creating a comment
#[derive(Debug, Clone, Serialize)]
pub struct CreateCommentRequest {
    pub comment_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_commenter: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "parent_id")]
    pub parent_id: Option<String>,
}

/// Request body for updating a comment
#[derive(Debug, Clone, Serialize)]
pub struct UpdateCommentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_commenter: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_comment_deserialize_minimal() {
        let json = r#"{"id": "123", "comment_text": "Hello world"}"#;
        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, "123");
        assert_eq!(comment.text, "Hello world");
        assert_eq!(comment.text_preview, "");
        assert!(comment.commenter.is_none());
        assert!(comment.created_at.is_none());
        assert!(comment.updated_at.is_none());
        assert!(!comment.assigned);
        assert_eq!(comment.reaction, "");
        assert!(comment.parent_id.is_none());
    }

    #[test]
    fn test_comment_deserialize_full() {
        let json = r#"{
            "id": "456",
            "comment_text": "This is a comment",
            "text_preview": "This is a...",
            "user": {"id": 789, "username": "testuser"},
            "date": "1234567890",
            "date_updated": "1234567899",
            "resolved": false,
            "reactions": ["thumbsup"]
        }"#;
        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, "456");
        assert_eq!(comment.text, "This is a comment");
        assert_eq!(comment.text_preview, "This is a...");
        assert!(comment.commenter.is_some());
        assert_eq!(comment.commenter.as_ref().unwrap().id, 789);
        assert_eq!(comment.created_at, Some(1234567890));
        assert_eq!(comment.updated_at, Some(1234567899));
        assert!(!comment.assigned);
        // Note: reactions field deserialization not yet implemented
        assert_eq!(comment.reaction, "");
        assert!(comment.parent_id.is_none());
    }

    #[test]
    fn test_comment_deserialize_null_fields() {
        let json = r#"{
            "id": "789",
            "comment_text": null,
            "text_preview": null,
            "user": null,
            "date": null,
            "date_updated": null,
            "resolved": null,
            "reactions": null
        }"#;
        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, "789");
        assert_eq!(comment.text, "");
        assert_eq!(comment.text_preview, "");
        assert!(comment.commenter.is_none());
        assert!(comment.created_at.is_none());
        assert!(comment.updated_at.is_none());
        assert!(!comment.assigned);
        assert_eq!(comment.reaction, "");
    }

    #[test]
    fn test_comment_deserialize_api_format() {
        // Test with actual ClickUp API response format
        let json = r##"{
            "id": "90160160381021",
            "comment": [{"text": "Task comment content"}],
            "comment_text": "Task comment content",
            "user": {
                "id": 183,
                "username": "John Doe",
                "email": "johndoe@gmail.com",
                "color": "#827718",
                "profilePicture": "https://example.com/pic.jpg",
                "initials": "JD"
            },
            "resolved": false,
            "assignee": null,
            "assigned_by": null,
            "reactions": [],
            "date": "1568036964079"
        }"##;
        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, "90160160381021");
        assert_eq!(comment.text, "Task comment content");
        assert!(comment.commenter.is_some());
        assert_eq!(comment.commenter.as_ref().unwrap().username, "John Doe");
        assert_eq!(comment.commenter.as_ref().unwrap().id, 183);
        assert_eq!(comment.created_at, Some(1568036964079));
        assert!(!comment.assigned);
        assert_eq!(comment.reaction, "");
    }

    #[test]
    fn test_comment_timestamp_as_int() {
        // Test timestamp as integer (some API responses may use this)
        let json = r#"{
            "id": "123",
            "comment_text": "Test",
            "user": {"id": 1, "username": "test"},
            "date": 1234567890,
            "resolved": false,
            "reactions": []
        }"#;
        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.created_at, Some(1234567890));
    }

    #[test]
    fn test_comment_reactions_array() {
        // Note: reactions array deserialization not yet implemented
        // This test documents expected behavior for future implementation
        let json = r#"{
            "id": "123",
            "comment_text": "Test",
            "user": {"id": 1, "username": "test"},
            "date": "1234567890",
            "resolved": false,
            "reactions": ["thumbsup", "heart", "laugh"]
        }"#;
        let comment: Comment = serde_json::from_str(json).unwrap();
        // Currently reactions are not deserialized - future enhancement
        assert_eq!(comment.reaction, "");
    }

    #[test]
    fn test_create_comment_request() {
        let request = CreateCommentRequest {
            comment_text: "New comment".to_string(),
            assignee: None,
            assigned_commenter: None,
            parent_id: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"comment_text\":\"New comment\""));
    }

    #[test]
    fn test_create_comment_request_with_parent() {
        let request = CreateCommentRequest {
            comment_text: "Reply to thread".to_string(),
            assignee: None,
            assigned_commenter: None,
            parent_id: Some("parent123".to_string()),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"comment_text\":\"Reply to thread\""));
        assert!(json.contains("\"parent_id\":\"parent123\""));
    }

    #[test]
    fn test_update_comment_request() {
        let request = UpdateCommentRequest {
            comment_text: Some("Updated text".to_string()),
            assigned: None,
            assignee: None,
            assigned_commenter: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"comment_text\":\"Updated text\""));
    }

    #[test]
    fn test_comment_with_parent_id() {
        // Test deserialization of a reply comment with parent_id
        let json = r#"{
            "id": "reply123",
            "comment_text": "This is a reply",
            "parent_id": "parent456",
            "user": {"id": 789, "username": "testuser"}
        }"#;
        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, "reply123");
        assert_eq!(comment.text, "This is a reply");
        assert_eq!(comment.parent_id, Some("parent456".to_string()));
    }

    #[test]
    fn test_comment_top_level_no_parent() {
        // Test that top-level comments have parent_id = None
        let json = r#"{
            "id": "top123",
            "comment_text": "Top level comment",
            "user": {"id": 789, "username": "testuser"}
        }"#;
        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, "top123");
        assert!(comment.parent_id.is_none());
    }
}
