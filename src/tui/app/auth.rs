//! Token verification and the workspaces/spaces/folders/lists loaders.

use super::*;

impl TuiApp {

    /// Authenticate with the token typed on the auth screen.
    /// Public so tests can drive the auth flow end-to-end.
    pub fn authenticate(&mut self) {
        let token = self.auth_state.token_input.clone();
        if token.is_empty() {
            self.auth_state.error = Some("Token cannot be empty".to_string());
            self.status = "Authentication failed".to_string();
            return;
        }

        // Reject obviously malformed input before anything else: ClickUp
        // personal API tokens start with pk_. Pasting a task URL (or any
        // other non-token text) previously got saved verbatim and silently
        // broke every API call until the file was fixed manually.
        if !token.starts_with("pk_") {
            self.auth_state.error =
                Some("Token must be a ClickUp API token (starts with pk_)".to_string());
            self.status = "Authentication failed".to_string();
            return;
        }

        self.loading = true;
        self.auth_state.loading = true;
        self.status = "Verifying token...".to_string();

        // Reuse an injected client when present (tests); otherwise create a
        // real HTTP client for the verification call.
        let client = match &self.client {
            Some(c) => c.clone(),
            None => Arc::new(ClickUpClient::new(token.clone())),
        };
        self.client = Some(client.clone());

        // Do NOT persist the token yet. Verify it against the API first so a
        // bad token never overwrites a previously working one on disk. The
        // token travels with the result and is only saved on success.
        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client.get_workspaces().await;
            let msg = match result {
                Ok(workspaces) => AppMessage::WorkspacesVerified(Ok(workspaces), token),
                Err(e) => AppMessage::WorkspacesVerified(Err(e.to_string()), token),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Load workspaces from API (public for testing)
    pub fn load_workspaces(&mut self) {
        self.loading = true;
        self.status = "Loading workspaces...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client.get_workspaces().await;
            let msg = match result {
                Ok(workspaces) => AppMessage::WorkspacesLoaded(Ok(workspaces)),
                Err(e) => AppMessage::WorkspacesLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    pub(super) fn load_spaces(&mut self, workspace_id: String) {
        self.loading = true;
        self.status = "Loading spaces...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client.get_spaces(&workspace_id).await;
            let msg = match result {
                Ok(spaces) => AppMessage::SpacesLoaded(Ok(spaces)),
                Err(e) => AppMessage::SpacesLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    pub(super) fn load_folders(&mut self, space_id: String) {
        self.loading = true;
        self.status = "Loading folders...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client.get_folders(&space_id).await;
            let msg = match result {
                Ok(folders) => AppMessage::FoldersLoaded(Ok(folders)),
                Err(e) => AppMessage::FoldersLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    pub(super) fn load_lists(&mut self, folder_id: String) {
        self.loading = true;
        self.status = "Loading lists...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client.get_lists_in_folder(&folder_id, None).await;
            let msg = match result {
                Ok(lists) => AppMessage::ListsLoaded(Ok(lists)),
                Err(e) => AppMessage::ListsLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Load tasks filtered by the current user as assignee
    pub(super) fn load_tasks_with_assigned_filter(&mut self, list_id: String) {
        self.loading = true;
        self.status = "Loading assigned tasks...".to_string();

        let user_id = match self.current_user_id {
            Some(id) => id,
            None => {
                self.loading = false;
                self.status = "User ID not available for filtering".to_string();
                return;
            }
        };

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client
                .get_tasks_with_assignee(&list_id, user_id, Some(100))
                .await;
            let msg = match result {
                Ok(tasks) => AppMessage::TasksLoaded(Ok(tasks)),
                Err(e) => AppMessage::TasksLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    pub fn load_tasks(&mut self, list_id: String) {
        // If the assigned filter is active, use the filtered version
        if self.assigned_filter_active {
            self.load_tasks_with_assigned_filter(list_id);
            return;
        }

        self.loading = true;
        self.status = "Loading tasks...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        use crate::models::TaskFilters;
        let tx = self.message_tx.clone().unwrap();
        let mut filters = TaskFilters::default();
        if !self.status_filters.is_empty() {
            filters.statuses = self.status_filters.clone();
        }
        tokio::spawn(async move {
            let result = client.get_tasks(&list_id, &filters).await;
            let msg = match result {
                Ok(tasks) => AppMessage::TasksLoaded(Ok(tasks)),
                Err(e) => AppMessage::TasksLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Open the assignee picker with the given members list
    pub(super) fn open_assignee_picker(&mut self, members: Vec<User>) {
        // Build the set of currently assigned user IDs from the selected task
        let current_assignee_ids: std::collections::HashSet<i64> = self
            .task_detail
            .task
            .as_ref()
            .map(|t| t.assignees.iter().map(|u| u.id).collect())
            .unwrap_or_default();

        self.assignee_picker_members = members;
        self.assignee_picker_selected = current_assignee_ids.clone();
        self.assignee_picker_original = current_assignee_ids;
        self.assignee_picker_cursor = 0;
        self.assignee_picker_open = true;
    }

    /// Open the assignee picker, fetching members if not cached
    pub(super) fn open_assignee_picker_flow(&mut self) {
        // Guard: need a task
        if self.task_detail.task.is_none() {
            self.status = "No task selected".to_string();
            return;
        }

        // Members are list-scoped, and a task opened from the Assigned view
        // has no navigation context, so resolve from the task's own list.
        let list_id = match self.resolve_task_list_id() {
            Some(id) => id,
            None => {
                self.status = "Cannot assign: this task's list is unknown".to_string();
                return;
            }
        };

        // Check cache first
        if let Some(cached) = self.cached_list_members.get(&list_id) {
            self.open_assignee_picker(cached.clone());
            return;
        }

        // Cache miss: fetch from API
        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.status = "Not authenticated".to_string();
                return;
            }
        };

        self.loading = true;
        self.status = "Loading members...".to_string();

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client.get_list_members(&list_id).await;
            let msg = match result {
                Ok(members) => AppMessage::MembersLoaded(Ok(members)),
                Err(e) => AppMessage::MembersLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Save current assignee selection to the task
    pub(super) fn save_assignees(&mut self) {
        let task = match &self.task_detail.task {
            Some(t) => t.clone(),
            None => {
                self.status = "No task selected".to_string();
                return;
            }
        };

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.status = "Not authenticated".to_string();
                return;
            }
        };

        let task_id = task.id.clone();
        let tx = self.message_tx.clone().unwrap();

        use crate::models::{AssigneesUpdate, UpdateTaskRequest};
        let update = UpdateTaskRequest {
            name: None,
            description: None,
            markdown_description: None,
            status: None,
            priority: None,
            assignees: Some(AssigneesUpdate::replace_all(
                self.assignee_picker_original.clone(),
                self.assignee_picker_selected.clone(),
            )),
            due_date: None,
        };

        // Immediate UI feedback while the request is in flight
        self.loading = true;
        self.status = "Saving assignees...".to_string();

        tokio::spawn(async move {
            let result = client.update_task(&task_id, &update).await;
            let msg = match result {
                Ok(task) => AppMessage::AssigneesUpdated(Ok(task)),
                Err(e) => AppMessage::AssigneesUpdated(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }
}
