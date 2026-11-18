//! Inbox model for the undocumented ClickUp inbox endpoint.
//!
//! `GET /team/{team_id}/inbox` returns the account's attention list — the same
//! set the ClickUp web app shows in its Inbox. It is **not** in the published
//! v2 reference and its payload is the internal web-app task shape, not the
//! public `Task` shape, so it gets a model of its own rather than being forced
//! through `Task` (which fails on it: `permission_level` is an integer here but
//! a string in the public API).
//!
//! Only the fields this client needs are modelled; serde ignores the rest.
//! Two naming differences matter when reading this:
//!
//! | Inbox field   | Means                                      |
//! |---------------|--------------------------------------------|
//! | `subcategory` | the List (public API calls it `list`)      |
//! | `project`     | the Folder (public API calls it `folder`)  |
//!
//! The endpoint accepts no parameters: `page`, `limit`, `cursor` and `unread`
//! are all silently ignored, and it is GET-only. Dismiss/read state therefore
//! cannot be written back and is tracked locally in the cache instead.

use crate::utils::deserializers::{flexible_string, flexible_timestamp, null_to_empty_string};
use serde::Deserialize;

/// One entry of the ClickUp inbox.
///
/// The list entry is the task itself: clicking it must open that task's detail
/// view, so `id` is the important field. `subcategory`/`project` give the row
/// its location context.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct InboxTask {
    #[serde(default, deserialize_with = "flexible_string")]
    pub id: String,
    #[serde(default, deserialize_with = "null_to_empty_string")]
    pub name: String,
    /// Last time anything on the task changed. Drives "most recent first".
    #[serde(
        default,
        rename = "date_updated",
        deserialize_with = "flexible_timestamp"
    )]
    pub updated_at: Option<i64>,
    #[serde(
        default,
        rename = "date_created",
        deserialize_with = "flexible_timestamp"
    )]
    pub created_at: Option<i64>,
    /// Number of comments on the task (inbox payload only).
    #[serde(default)]
    pub comments_count: Option<i64>,
    /// Discussion threads assigned to the current user (inbox payload only).
    #[serde(default)]
    pub assigned_comments_count: Option<i64>,
    /// Unresolved comment threads (inbox payload only).
    #[serde(default)]
    pub unresolved_comments_count: Option<i64>,
    /// The task's List, named `subcategory` in this payload.
    #[serde(default)]
    pub subcategory: Option<InboxLocation>,
    /// The task's Folder, named `project` in this payload.
    #[serde(default)]
    pub project: Option<InboxLocation>,
    #[serde(default)]
    pub status: Option<InboxStatus>,
}

/// A List/Folder reference inside the inbox payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct InboxLocation {
    #[serde(default, deserialize_with = "flexible_string")]
    pub id: String,
    #[serde(default, deserialize_with = "null_to_empty_string")]
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
}

/// A status reference inside the inbox payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct InboxStatus {
    #[serde(default, deserialize_with = "null_to_empty_string")]
    pub status: String,
    #[serde(default)]
    pub color: Option<String>,
    /// `custom`, `open`, `closed`, `done` — used to show progress at a glance.
    #[serde(default, rename = "type")]
    pub status_type: Option<String>,
}

impl InboxTask {
    /// The List name for display, when the payload carried one.
    pub fn list_name(&self) -> Option<&str> {
        self.subcategory
            .as_ref()
            .map(|location| location.name.as_str())
            .filter(|name| !name.is_empty())
    }

    /// The Folder name for display, when the payload carried one.
    pub fn folder_name(&self) -> Option<&str> {
        self.project
            .as_ref()
            .map(|location| location.name.as_str())
            .filter(|name| !name.is_empty())
    }

    /// True when the task is closed/done according to its status type.
    #[allow(dead_code)]
    pub fn is_closed(&self) -> bool {
        self.status
            .as_ref()
            .and_then(|status| status.status_type.as_deref())
            .map(|kind| kind == "closed" || kind == "done")
            .unwrap_or(false)
    }
}

/// Response envelope for the inbox endpoint.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct InboxResponse {
    #[serde(default)]
    pub inbox: Inbox,
}

/// The `inbox` object: ordered ids plus the task records behind them.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Inbox {
    /// Entry ids, in the order the web app shows them.
    #[serde(default)]
    pub ids: Vec<String>,
    /// Reminders attached to the inbox (not modelled further; counted only).
    #[serde(default)]
    pub reminders: Vec<serde_json::Value>,
    #[serde(default)]
    pub tasks: Vec<InboxTask>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shape taken from a real response, trimmed to the modelled fields.
    const SAMPLE: &str = r##"{
        "inbox": {
            "ids": ["86cyz1tw0", "86czcta6f"],
            "reminders": [],
            "tasks": [
                {
                    "id": "86cyz1tw0",
                    "name": "Button should update the colour after edited",
                    "date_created": "1747377821347",
                    "date_updated": "1749012385261",
                    "comments_count": 1,
                    "assigned_comments_count": 0,
                    "unresolved_comments_count": 0,
                    "subcategory": {
                        "id": "901608512233",
                        "name": "V1.5 Single Login",
                        "color": "#f9d900"
                    },
                    "project": {
                        "id": "90161214661",
                        "name": "Future Improvement"
                    },
                    "status": {
                        "status": "viniss",
                        "color": "#64c6a2",
                        "type": "custom"
                    },
                    "permission_level": 5,
                    "customFields": [],
                    "followers": []
                },
                {
                    "id": "86czcta6f",
                    "name": "Error message under list of company",
                    "date_updated": "1751266267301",
                    "subcategory": {"id": "1", "name": "V1.5 Single Login"},
                    "status": {"status": "done", "type": "closed"}
                }
            ]
        }
    }"##;

    #[test]
    fn parses_real_payload_shape() {
        let response: InboxResponse = serde_json::from_str(SAMPLE).unwrap();
        assert_eq!(response.inbox.ids.len(), 2);
        assert_eq!(response.inbox.tasks.len(), 2);

        let first = &response.inbox.tasks[0];
        assert_eq!(first.id, "86cyz1tw0");
        assert_eq!(first.updated_at, Some(1749012385261));
        assert_eq!(first.comments_count, Some(1));
        assert_eq!(first.list_name(), Some("V1.5 Single Login"));
        assert_eq!(first.folder_name(), Some("Future Improvement"));
        assert!(!first.is_closed());
    }

    /// Web-app extras (`permission_level` as an int, camelCase `customFields`,
    /// `followers`) must not break parsing — they are simply ignored.
    #[test]
    fn ignores_web_app_only_fields() {
        let response: InboxResponse = serde_json::from_str(SAMPLE).unwrap();
        assert_eq!(
            response.inbox.tasks[0].name,
            "Button should update the colour after edited"
        );
    }

    #[test]
    fn detects_closed_status() {
        let response: InboxResponse = serde_json::from_str(SAMPLE).unwrap();
        assert!(response.inbox.tasks[1].is_closed());
    }

    #[test]
    fn tolerates_missing_optional_context() {
        let json = r#"{"inbox":{"ids":["x"],"reminders":[],"tasks":[{"id":"x","name":"Bare"}]}}"#;
        let response: InboxResponse = serde_json::from_str(json).unwrap();
        let task = &response.inbox.tasks[0];
        assert_eq!(task.list_name(), None);
        assert_eq!(task.folder_name(), None);
        assert_eq!(task.updated_at, None);
        assert!(!task.is_closed());
    }

    #[test]
    fn empty_inbox_is_valid() {
        let response: InboxResponse =
            serde_json::from_str(r#"{"inbox":{"ids":[],"reminders":[],"tasks":[]}}"#).unwrap();
        assert!(response.inbox.tasks.is_empty());
        assert!(response.inbox.ids.is_empty());
    }
}
