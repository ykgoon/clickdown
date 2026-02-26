//! Comment models for ClickUp API
//!
//! # Deserializer Design
//!
//! The Comment model uses custom deserializers to handle the ClickUp API's inconsistent
//! response formats. Key design decisions:
//!
//! - **Null handling**: String fields default to empty string, booleans to false, IDs to 0
//! - **Timestamps**: Accept both i64 (milliseconds) and string representations
//! - **Unknown fields**: Silently ignored (serde default) for forward compatibility
//! - **Field renames**: API uses different names (e.g., `comment_text` → `text`, `user` → `commenter`)
//!
//! # Known API Response Variations
//!
//! The ClickUp API may return:
//! - Timestamps as integers (`1234567890000`) or strings (`"1234567890000"`)
//! - User objects as full objects, null, or omitted entirely
//! - `reactions` as an array (ignored until needed)
//! - `parent_id` for threaded replies (optional)
//!
//! # Debugging Parse Errors
//!
//! If parsing fails, use CLI debug mode with `--verbose` to see the full API response:
//! ```bash
//! clickdown debug create-reply <comment_id> --text "Test" --verbose
//! ```
//!
//! The error message will include the field path (via `serde_path_to_error`) to help
//! identify which field caused the failure.

use serde::{Deserialize, Serialize};

/// Helper function to deserialize null as empty string
///
/// Used for string fields that may be null or missing in API responses.
/// Examples: `id`, `comment_text`, `text_preview`, `username`, `reaction`
fn null_to_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

/// Flexible string deserializer that handles both string and integer values
///
/// The ClickUp API may return IDs as either strings or integers.
/// This deserializer converts integers to strings and handles null/missing values.
fn flexible_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(i64),
    }

    let opt = Option::<StringOrInt>::deserialize(deserializer)?;
    match opt {
        None => Ok(String::new()),
        Some(StringOrInt::String(s)) => Ok(s),
        Some(StringOrInt::Int(i)) => Ok(i.to_string()),
    }
}

/// Helper function to deserialize null as false for bool fields
///
/// Used for boolean fields that may be null in API responses.
/// Example: `resolved` → `assigned`
fn null_to_false<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<bool>::deserialize(deserializer).map(|opt| opt.unwrap_or(false))
}

/// Flexible timestamp deserializer for comment fields (handles i64 or string)
///
/// The ClickUp API may return timestamps in two formats:
/// - Integer: `1234567890000` (milliseconds since epoch)
/// - String: `"1234567890000"` (numeric string)
///
/// This deserializer accepts both formats and returns `Option<i64>`.
/// Null or missing fields return `None`.
///
/// # Unsupported Formats
///
/// The following formats will fail to parse (not observed in ClickUp API):
/// - Float: `1234567890.123`
/// - ISO 8601: `"2024-01-15T10:30:00Z"`
/// - Invalid strings: `"yesterday"`
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

/// Helper function to deserialize null as default ID (0)
///
/// Used for user ID fields that may be null in API responses.
/// Example: `user.id`
fn null_to_default_id<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<i64>::deserialize(deserializer).map(|opt| opt.unwrap_or(0))
}

/// A ClickUp Comment
///
/// Represents a comment or reply on a ClickUp task. The struct uses custom
/// deserializers to handle null values and field name mappings from the API.
///
/// # Field Mappings
///
/// | Struct Field | API Field | Notes |
/// |--------------|-----------|-------|
/// | `id` | `id` | Comment identifier (string or int) |
/// | `text` | `comment_text` | Full comment content |
/// | `text_preview` | `text_preview` | Truncated preview |
/// | `commenter` | `user` | Comment author |
/// | `created_at` | `date` | Creation timestamp (ms) |
/// | `updated_at` | `date_updated` | Last update timestamp (ms) |
/// | `assigned_commenter` | `assignee` | Assigned user (if any) |
/// | `assigned_by` | `assigned_by` | Who made the assignment |
/// | `assigned` | `resolved` | Resolved status (bool) |
/// | `reaction` | `reaction` | Single reaction emoji/text |
/// | `parent_id` | `parent_id` | Parent comment for replies |
///
/// # Threading
///
/// - Top-level comments: `parent_id` is `None`
/// - Replies: `parent_id` contains the parent comment's ID
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    /// Comment identifier (empty string if null, converted from int if needed)
    #[serde(default, deserialize_with = "flexible_string")]
    pub id: String,
    /// Full comment content (empty string if null)
    #[serde(default, deserialize_with = "null_to_empty_string", rename = "comment_text")]
    pub text: String,
    /// Truncated preview text (empty string if null)
    #[serde(default, deserialize_with = "null_to_empty_string", rename = "text_preview")]
    pub text_preview: String,
    /// Comment author (None if null or missing)
    #[serde(default, rename = "user")]
    pub commenter: Option<User>,
    /// Creation timestamp in milliseconds since epoch (None if null/missing)
    #[serde(default, rename = "date", deserialize_with = "flexible_timestamp")]
    pub created_at: Option<i64>,
    /// Last update timestamp in milliseconds since epoch (None if null/missing)
    #[serde(default, rename = "date_updated", deserialize_with = "flexible_timestamp")]
    pub updated_at: Option<i64>,
    /// User assigned to this comment (None if null/missing)
    #[serde(default, rename = "assignee")]
    pub assigned_commenter: Option<User>,
    /// User who made the assignment (None if null/missing)
    #[serde(default, rename = "assigned_by")]
    pub assigned_by: Option<User>,
    /// Resolved status (false if null/missing)
    #[serde(default, deserialize_with = "null_to_false", rename = "resolved")]
    pub assigned: bool,
    /// Reaction emoji or text (empty string if null/missing)
    /// Note: API may also return `reactions` array (ignored until needed)
    #[serde(default, deserialize_with = "null_to_empty_string")]
    pub reaction: String,
    /// Parent comment ID for threaded replies (None for top-level comments)
    #[serde(default, rename = "parent_id", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

/// User reference in comment context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    #[serde(default, deserialize_with = "null_to_default_id")]
    pub id: i64,
    #[serde(default, deserialize_with = "null_to_empty_string")]
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

    #[test]
    fn test_comment_create_response_with_null_user() {
        // Test create comment response where user fields might be null
        // This simulates the API response when creating a comment
        let json = r#"{
            "id": "newcomment123",
            "comment_text": "Just created this comment",
            "text_preview": "Just created...",
            "user": null,
            "date": "1700000000000",
            "date_updated": null,
            "assignee": null,
            "assigned_by": null,
            "resolved": null,
            "reactions": null,
            "parent_id": null
        }"#;
        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, "newcomment123");
        assert_eq!(comment.text, "Just created this comment");
        assert!(comment.commenter.is_none());
        assert_eq!(comment.created_at, Some(1700000000000));
        assert!(comment.parent_id.is_none());
    }

    #[test]
    fn test_comment_update_response_with_minimal_fields() {
        // Test update comment response with minimal fields
        // Some API responses may return only a subset of fields
        let json = r#"{
            "id": "updatedcomment456",
            "comment_text": "Updated comment text",
            "date_updated": "1700000001000"
        }"#;
        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, "updatedcomment456");
        assert_eq!(comment.text, "Updated comment text");
        assert_eq!(comment.updated_at, Some(1700000001000));
        assert!(comment.text_preview.is_empty());
        assert!(comment.commenter.is_none());
    }

    #[test]
    fn test_comment_with_null_id() {
        // Test that null ID is handled gracefully (converted to empty string)
        let json = r#"{
            "id": null,
            "comment_text": "Comment with null ID"
        }"#;
        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, "");
        assert_eq!(comment.text, "Comment with null ID");
    }

    #[test]
    fn test_user_with_null_fields() {
        // Test user deserialization with null fields
        let json = r#"{
            "id": null,
            "username": null,
            "color": null,
            "email": null
        }"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, 0);
        assert_eq!(user.username, "");
        assert!(user.color.is_none());
        assert!(user.email.is_none());
    }

    #[test]
    fn test_comment_with_float_timestamp_should_fail_gracefully() {
        // Test that float timestamps fail with a clear error
        let json = r#"{
            "id": "123",
            "comment_text": "Test",
            "date": 1234567890.123
        }"#;
        let result: Result<Comment, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Float timestamp should fail");
        let err = result.unwrap_err();
        // Error should mention the type issue (float not matching int/string variants)
        let err_msg = err.to_string().to_lowercase();
        assert!(err_msg.contains("invalid") || err_msg.contains("type") || err_msg.contains("variant") || err_msg.contains("timestampvalue"),
                "Error should indicate the problem: {}", err);
    }

    #[test]
    fn test_comment_with_iso8601_timestamp_should_fail_gracefully() {
        // Test that ISO 8601 timestamps fail (not yet supported)
        let json = r#"{
            "id": "123",
            "comment_text": "Test",
            "date": "2024-01-15T10:30:00Z"
        }"#;
        let result: Result<Comment, _> = serde_json::from_str(json);
        assert!(result.is_err(), "ISO 8601 timestamp should fail");
    }

    #[test]
    fn test_comment_id_as_integer() {
        // Test that integer ID is converted to string
        // This is the actual bug: API returns id as integer 90160168435702
        let json = r#"{
            "id": 90160168435702,
            "comment_text": "Test comment"
        }"#;
        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, "90160168435702");
    }

    #[test]
    fn test_comment_id_as_string() {
        // Test that string ID still works
        let json = r#"{
            "id": "abc123",
            "comment_text": "Test comment"
        }"#;
        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, "abc123");
    }

    #[test]
    fn test_comment_id_null() {
        // Test that null ID becomes empty string
        let json = r#"{
            "id": null,
            "comment_text": "Test comment"
        }"#;
        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, "");
    }
}
