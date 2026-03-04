//! Notification model for ClickUp inbox

use crate::utils::deserializers::{flexible_string, flexible_timestamp, null_to_empty_string};
use serde::{Deserialize, Serialize};

/// A ClickUp Notification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    #[serde(default, deserialize_with = "flexible_string")]
    pub id: String,
    #[serde(default, rename = "team_id", deserialize_with = "flexible_string")]
    pub workspace_id: String,
    #[serde(default, deserialize_with = "null_to_empty_string")]
    pub title: String,
    #[serde(default, deserialize_with = "null_to_empty_string")]
    pub description: String,
    #[serde(
        default,
        rename = "date_created",
        deserialize_with = "flexible_timestamp"
    )]
    pub created_at: Option<i64>,
    #[serde(default, rename = "date_read", deserialize_with = "flexible_timestamp")]
    pub read_at: Option<i64>,
}

/// API response for getting notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsResponse {
    #[serde(default)]
    pub notifications: Vec<Notification>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_deserialization_basic() {
        let json = r#"{
            "id": "notif_123",
            "team_id": "ws_456",
            "title": "Task assigned",
            "description": "You were assigned to a task",
            "date_created": 1704067200000,
            "date_read": null
        }"#;

        let notif: Notification = serde_json::from_str(json).unwrap();
        assert_eq!(notif.id, "notif_123");
        assert_eq!(notif.workspace_id, "ws_456");
        assert_eq!(notif.title, "Task assigned");
        assert_eq!(notif.description, "You were assigned to a task");
        assert_eq!(notif.created_at, Some(1704067200000));
        assert_eq!(notif.read_at, None);
    }

    #[test]
    fn test_notification_deserialization_with_read_at() {
        let json = r#"{
            "id": "notif_123",
            "team_id": "ws_456",
            "title": "Task assigned",
            "description": "You were assigned to a task",
            "date_created": 1704067200000,
            "date_read": 1704153600000
        }"#;

        let notif: Notification = serde_json::from_str(json).unwrap();
        assert_eq!(notif.read_at, Some(1704153600000));
    }

    #[test]
    fn test_notification_deserialization_missing_optional() {
        let json = r#"{
            "id": "notif_123",
            "team_id": "ws_456",
            "title": "Task assigned"
        }"#;

        let notif: Notification = serde_json::from_str(json).unwrap();
        assert_eq!(notif.id, "notif_123");
        assert_eq!(notif.workspace_id, "ws_456");
        assert_eq!(notif.title, "Task assigned");
        assert_eq!(notif.description, "");
        assert_eq!(notif.created_at, None);
        assert_eq!(notif.read_at, None);
    }

    #[test]
    fn test_notification_deserialization_null_fields() {
        let json = r#"{
            "id": null,
            "team_id": null,
            "title": null,
            "description": null,
            "date_created": null,
            "date_read": null
        }"#;

        let notif: Notification = serde_json::from_str(json).unwrap();
        assert_eq!(notif.id, "");
        assert_eq!(notif.workspace_id, "");
        assert_eq!(notif.title, "");
        assert_eq!(notif.description, "");
        assert_eq!(notif.created_at, None);
        assert_eq!(notif.read_at, None);
    }

    #[test]
    fn test_notification_deserialization_timestamp_formats() {
        // Test integer timestamp (milliseconds)
        let json = r#"{
            "id": "notif_1",
            "team_id": "ws_1",
            "title": "Test",
            "date_created": 1704067200000
        }"#;
        let notif: Notification = serde_json::from_str(json).unwrap();
        assert_eq!(notif.created_at, Some(1704067200000));

        // Test string timestamp (numeric string)
        let json = r#"{
            "id": "notif_2",
            "team_id": "ws_2",
            "title": "Test",
            "date_created": "1704067200000"
        }"#;
        let notif: Notification = serde_json::from_str(json).unwrap();
        assert_eq!(notif.created_at, Some(1704067200000));
    }

    #[test]
    fn test_notification_deserialization_workspace_id_as_integer() {
        // ClickUp API may return workspace_id as integer
        let json = r#"{
            "id": "notif_123",
            "team_id": 456,
            "title": "Test"
        }"#;

        let notif: Notification = serde_json::from_str(json).unwrap();
        assert_eq!(notif.workspace_id, "456");
    }

    #[test]
    fn test_notifications_response_deserialization() {
        let json = r#"{
            "notifications": [
                {
                    "id": "notif_1",
                    "team_id": "ws_1",
                    "title": "First notification",
                    "description": "Description 1",
                    "date_created": 1704067200000
                },
                {
                    "id": "notif_2",
                    "team_id": "ws_1",
                    "title": "Second notification",
                    "date_created": 1704153600000
                }
            ]
        }"#;

        let response: NotificationsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.notifications.len(), 2);
        assert_eq!(response.notifications[0].id, "notif_1");
        assert_eq!(response.notifications[1].id, "notif_2");
    }

    #[test]
    fn test_notifications_response_empty_list() {
        let json = r#"{"notifications": []}"#;
        let response: NotificationsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.notifications.len(), 0);
    }

    #[test]
    fn test_notifications_response_missing_notifications() {
        let json = r#"{}"#;
        let response: NotificationsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.notifications.len(), 0);
    }
}
