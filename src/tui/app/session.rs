//! Quitting and session persistence.

use super::*;

impl TuiApp {

    /// Save the current session state to the cache
    ///
    /// This captures the current navigation context for restoration on next startup.
    /// Signal the run loop to quit, saving session state on the way out.
    pub(super) fn quit(&mut self) {
        if let Err(e) = self.save_session_state() {
            tracing::error!("Failed to save session state: {}", e);
        }
        self.state = AppState::Quitting;
        self.status = "Bye".to_string();
    }

    pub fn save_session_state(&mut self) -> Result<()> {
        // Built field-by-field rather than through a positional constructor:
        // eight adjacent `Option`s are otherwise easy to transpose.
        let state = SessionState {
            screen: format!("{:?}", self.screen),
            workspace_id: self.current_workspace_id.clone(),
            space_id: self.current_space_id.clone(),
            folder_id: self.current_folder_id.clone(),
            list_id: self.current_list_id.clone(),
            task_id: self.task_detail.task.as_ref().map(|t| t.id.clone()),
            document_id: self.documents.first().map(|d| d.id.clone()),
            user_id: self.current_user_id,
        };
        self.cache.save_session_state(&state)
    }

    /// Restore session state from the cache
    ///
    /// Returns true if session restore is in progress, false if no saved state exists.
    /// Sets restoring_session flag and stores target IDs for progressive restore.
    /// The actual navigation chain replay happens in async message handlers.
    pub fn restore_session_state(&mut self) -> Result<bool> {
        let saved_state = match self.cache.load_session_state()? {
            Some(state) => state,
            None => {
                tracing::debug!("No saved session state found");
                return Ok(false);
            }
        };

        tracing::info!("Restoring session state: screen={}, workspace={:?}, space={:?}, folder={:?}, list={:?}",
            saved_state.screen,
            saved_state.workspace_id,
            saved_state.space_id,
            saved_state.folder_id,
            saved_state.list_id);

        // Set restoring session flag
        self.restoring_session = true;

        // Honour a saved query-style screen. The replay below walks the
        // hierarchy (workspace -> space -> folder -> list), which cannot
        // express "Assigned to Me": that view has no location to navigate to.
        self.restored_screen = match saved_state.screen.as_str() {
            "Assigned" => Some(Screen::Assigned),
            _ => None,
        };

        // Store target IDs for progressive restore
        self.restored_workspace_id = saved_state.workspace_id.clone();
        self.restored_space_id = saved_state.space_id.clone();
        self.restored_folder_id = saved_state.folder_id.clone();
        self.restored_list_id = saved_state.list_id.clone();
        self.restored_task_id = saved_state.task_id.clone();

        // Restore current navigation IDs (used for URL generation etc.)
        self.current_workspace_id = saved_state.workspace_id.clone();
        self.current_space_id = saved_state.space_id.clone();
        self.current_folder_id = saved_state.folder_id.clone();
        self.current_list_id = saved_state.list_id.clone();

        // Restore user ID for assignee filtering (only if valid - not 0)
        if let Some(uid) = saved_state.user_id {
            if uid != 0 {
                self.current_user_id = Some(uid);
                tracing::debug!("Restored user_id from session cache: {}", uid);
            } else {
                tracing::debug!("Ignoring invalid cached user_id: 0, will fetch fresh");
            }
        }

        // Start at Workspaces - the navigation chain will replay from here
        // The async handlers will navigate through each level and select the restored items
        self.screen = Screen::Workspaces;
        self.screen_title = generate_screen_title("Workspaces");

        Ok(true) // Return true to indicate restore is in progress
    }
}
