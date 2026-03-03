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
    ) -> Self {
        Self {
            screen: format!("{:?}", screen),
            workspace_id,
            space_id,
            folder_id,
            list_id,
            task_id,
            document_id,
        }
    }

    /// Check if the session state is valid (has at least a screen type)
    pub fn is_valid(&self) -> bool {
        !self.screen.is_empty() && self.screen != "Auth"
    }

    /// Check if the session state has any navigation context
    pub fn has_navigation_context(&self) -> bool {
        self.workspace_id.is_some()
            || self.space_id.is_some()
            || self.folder_id.is_some()
            || self.list_id.is_some()
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
    fn test_session_state_is_valid() {
        let valid_state = SessionState {
            screen: String::from("Tasks"),
            workspace_id: Some("ws-123".to_string()),
            ..Default::default()
        };
        assert!(valid_state.is_valid());

        let invalid_auth_state = SessionState {
            screen: String::from("Auth"),
            ..Default::default()
        };
        assert!(!invalid_auth_state.is_valid());

        let invalid_empty_state = SessionState {
            screen: String::new(),
            ..Default::default()
        };
        assert!(!invalid_empty_state.is_valid());
    }

    #[test]
    fn test_session_state_has_navigation_context() {
        let state_with_context = SessionState {
            screen: String::from("Tasks"),
            workspace_id: Some("ws-123".to_string()),
            ..Default::default()
        };
        assert!(state_with_context.has_navigation_context());

        let state_without_context = SessionState {
            screen: String::from("Workspaces"),
            ..Default::default()
        };
        assert!(!state_without_context.has_navigation_context());
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
        };

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: SessionState = serde_json::from_str(&json).unwrap();

        assert_eq!(state.screen, deserialized.screen);
        assert_eq!(state.workspace_id, deserialized.workspace_id);
        assert_eq!(state.space_id, deserialized.space_id);
        assert_eq!(state.list_id, deserialized.list_id);
    }
}
