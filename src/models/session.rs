//! Session state management for persisting navigation state across sessions

use serde::{Deserialize, Serialize};

/// Represents the application's navigation state for session persistence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionState {
    /// Current screen type (Auth, Workspaces, Spaces, Folders, Lists, Tasks, TaskDetail, Document)
    pub screen: String,
    /// Current workspace ID (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    /// Current space ID (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
    /// Current folder ID (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,
    /// Current list ID (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_id: Option<String>,
    /// Current task ID (when in TaskDetail view)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    /// Current document ID (when in Document view)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    /// Current user ID for assignee filtering (if detected)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i32>,
}

impl SessionState {
    /// Create a new SessionState from the current TuiApp state
    ///
    /// This captures the current navigation context for later restoration.
    pub fn from_app(
        screen: &crate::tui::app::Screen,
        workspace_id: Option<String>,
        space_id: Option<String>,
        folder_id: Option<String>,
        list_id: Option<String>,
        task_id: Option<String>,
        document_id: Option<String>,
        user_id: Option<i32>,
    ) -> Self {
        Self {
            screen: format!("{:?}", screen),
            workspace_id,
            space_id,
            folder_id,
            list_id,
            task_id,
            document_id,
            user_id,
        }
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            screen: String::from("Workspaces"),
            workspace_id: None,
            space_id: None,
            folder_id: None,
            list_id: None,
            task_id: None,
            document_id: None,
            user_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_default() {
        let state = SessionState::default();
        assert_eq!(state.screen, "Workspaces");
        assert!(state.workspace_id.is_none());
        assert!(state.space_id.is_none());
        assert!(state.folder_id.is_none());
        assert!(state.list_id.is_none());
        assert!(state.task_id.is_none());
        assert!(state.document_id.is_none());
    }

    #[test]
    fn test_session_state_serialization() {
        let state = SessionState {
            screen: String::from("Tasks"),
            workspace_id: Some("ws-123".to_string()),
            space_id: Some("space-456".to_string()),
            folder_id: None,
            list_id: Some("list-789".to_string()),
            task_id: None,
            document_id: None,
            user_id: None,
        };

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: SessionState = serde_json::from_str(&json).unwrap();

        assert_eq!(state.screen, deserialized.screen);
        assert_eq!(state.workspace_id, deserialized.workspace_id);
        assert_eq!(state.space_id, deserialized.space_id);
        assert_eq!(state.list_id, deserialized.list_id);
    }
}
