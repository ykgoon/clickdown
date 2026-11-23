//! Async message handling: the single place API results become state.

use super::*;

impl TuiApp {

    /// Process async messages from API calls (public for testing)
    pub fn process_async_messages(&mut self) {
        if let Some(ref mut rx) = self.message_rx {
            // Try to receive messages without blocking
            // We need to collect messages first to avoid borrow conflicts with load_* methods
            let mut messages = Vec::new();
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }

            // Now process messages without holding the borrow
            for msg in messages {
                match msg {
                    AppMessage::WorkspacesVerified(result, token) => {
                        self.loading = false;
                        self.auth_state.loading = false;
                        match result {
                            Ok(workspaces) => {
                                // Token proven working - only now persist it.
                                if self.manage_stored_token {
                                    if let Err(e) = self.auth.save_token(&token) {
                                        tracing::warn!("Failed to save verified token: {}", e);
                                        self.status =
                                            "Authenticated, but failed to save token".to_string();
                                    }
                                }
                                self.auth_invalid = false;
                                self.error = None;
                                self.workspaces = workspaces.clone();
                                if self.current_team_id.is_none() {
                                    self.current_team_id =
                                        self.workspaces.first().map(|w| w.id.clone());
                                }
                                *self.sidebar.items_mut() = self.root_sidebar_items();
                                self.sidebar.select_first();
                                self.auth_state.clear();
                                self.state = AppState::Main;
                                self.screen = Screen::Workspaces;
                                self.screen_title = generate_screen_title("Workspaces");
                                self.status = format!(
                                    "Authenticated! Loaded {} workspace(s)",
                                    workspaces.len()
                                );
                            }
                            Err(e) => {
                                self.auth_invalid = true;
                                self.state = AppState::Unauthenticated;
                                self.screen = Screen::Auth;
                                self.auth_state.clear();
                                self.auth_state.error = Some(format!(
                                    "Token rejected by ClickUp ({}). Enter a valid personal API token (pk_...).",
                                    e
                                ));
                                self.status = "Authentication failed: token rejected".to_string();
                            }
                        }
                    }
                    AppMessage::WorkspacesLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(workspaces) => {
                                self.workspaces = workspaces.clone();
                                if self.current_team_id.is_none() {
                                    self.current_team_id =
                                        self.workspaces.first().map(|w| w.id.clone());
                                }
                                *self.sidebar.items_mut() = self.root_sidebar_items();

                                // A saved query-style screen needs no
                                // hierarchy replay, and replaying would drag
                                // the user back into it.
                                if self.restored_screen.is_some() {
                                    self.restoring_session = false;
                                    self.try_restore_query_screen();
                                } else if self.restoring_session {
                                    // Try to select the restored workspace
                                    if let Some(restored_id) = self.restored_workspace_id.clone() {
                                        if self.sidebar.select_by_id(&restored_id) {
                                            // Found the workspace, load its spaces
                                            let workspace_name = self
                                                .workspaces
                                                .iter()
                                                .find(|w| w.id == restored_id)
                                                .map(|w| w.name.clone())
                                                .unwrap_or_default();
                                            self.load_spaces(restored_id.clone());
                                            self.screen = Screen::Spaces;
                                            self.screen_title =
                                                generate_screen_title(&workspace_name);
                                        } else {
                                            // Workspace not found, fallback to Workspaces screen
                                            self.restoring_session = false;
                                            self.screen = Screen::Workspaces;
                                            self.screen_title = generate_screen_title("Workspaces");
                                            self.status =
                                                "Saved workspace not found, showing workspaces"
                                                    .to_string();
                                            tracing::warn!("Restored workspace {} not found, falling back to Workspaces", restored_id);
                                        }
                                    } else {
                                        // No workspace ID saved, stay at Workspaces
                                        self.restoring_session = false;
                                        self.sidebar.select_first();
                                        self.status = format!(
                                            "Loaded {} workspace(s)",
                                            self.workspaces.len()
                                        );
                                    }
                                } else {
                                    // Normal behavior (not restoring)
                                    // Check if we have a current_workspace_id from navigation context
                                    if let Some(ref workspace_id) = self.current_workspace_id {
                                        if !self.sidebar.select_by_id(workspace_id) {
                                            // Workspace not found, fallback to first
                                            self.sidebar.select_first();
                                        }
                                    } else {
                                        self.sidebar.select_first();
                                    }
                                    self.status =
                                        format!("Loaded {} workspace(s)", self.workspaces.len());
                                }

                                self.state = AppState::Main;
                                // Clear any previous error state
                                self.error = None;
                            }
                            Err(e) => {
                                self.loading = false;
                                // A dead stored token must not strand the user on
                                // an error banner: drop it and return to auth.
                                if e.contains("Token invalid or expired") {
                                    self.auth_invalid = true;
                                    if self.manage_stored_token {
                                        let _ = self.auth.clear_token();
                                    }
                                    self.client = None;
                                    self.state = AppState::Unauthenticated;
                                    self.screen = Screen::Auth;
                                    self.auth_state.clear();
                                    self.auth_state.error = Some(
                                        "Stored token was rejected by ClickUp (invalid or expired). Enter a fresh personal API token (pk_...)."
                                            .to_string(),
                                    );
                                    self.status = "Re-authentication required".to_string();
                                } else {
                                    self.error =
                                        Some(format!("Failed to load workspaces: {}", e));
                                    self.status = "Failed to load workspaces".to_string();
                                }
                                if self.restoring_session {
                                    self.restoring_session = false;
                                }
                            }
                        }
                    }
                    AppMessage::SpacesLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(spaces) => {
                                self.spaces = spaces.clone();
                                // Populate sidebar with spaces
                                let mut items = Vec::new();
                                items.extend(self.spaces.iter().map(|s| SidebarItem::Space {
                                    name: s.name.clone(),
                                    id: s.id.clone(),
                                }));
                                *self.sidebar.items_mut() = items;

                                // Check if we're restoring a session
                                if self.restoring_session {
                                    // Try to select the restored space
                                    if let Some(restored_id) = self.restored_space_id.clone() {
                                        if self.sidebar.select_by_id(&restored_id) {
                                            // Found the space, load its folders
                                            let space_name = self
                                                .spaces
                                                .iter()
                                                .find(|s| s.id == restored_id)
                                                .map(|s| s.name.clone())
                                                .unwrap_or_default();
                                            self.load_folders(restored_id.clone());
                                            self.screen = Screen::Folders;
                                            self.screen_title = generate_screen_title(&space_name);
                                        } else {
                                            // Space not found, fallback to Spaces screen
                                            self.restoring_session = false;
                                            self.screen = Screen::Spaces;
                                            self.screen_title = generate_screen_title("Spaces");
                                            self.status =
                                                "Saved space not found, showing spaces".to_string();
                                            tracing::warn!("Restored space {} not found, falling back to Spaces", restored_id);
                                        }
                                    } else {
                                        // No space ID saved, stay at Spaces
                                        self.restoring_session = false;
                                        self.sidebar.select_first();
                                        self.status =
                                            format!("Loaded {} space(s)", self.spaces.len());
                                    }
                                } else {
                                    // Normal behavior (not restoring)
                                    // Check if we have a current_space_id from navigation context
                                    if let Some(ref space_id) = self.current_space_id {
                                        if !self.sidebar.select_by_id(space_id) {
                                            // Space not found, fallback to first
                                            self.sidebar.select_first();
                                        }
                                    } else {
                                        self.sidebar.select_first();
                                    }
                                    self.status = format!("Loaded {} space(s)", self.spaces.len());
                                }

                                // Clear any previous error state
                                self.error = None;
                            }
                            Err(e) => {
                                self.loading = false;
                                self.error = Some(format!("Failed to load spaces: {}", e));
                                self.status = "Failed to load spaces".to_string();
                                if self.restoring_session {
                                    self.restoring_session = false;
                                }
                            }
                        }
                    }
                    AppMessage::FoldersLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(folders) => {
                                self.folders = folders.clone();
                                // Populate sidebar with folders
                                let mut items = Vec::new();
                                items.extend(self.folders.iter().map(|f| SidebarItem::Folder {
                                    name: f.name.clone(),
                                    id: f.id.clone(),
                                }));
                                *self.sidebar.items_mut() = items;

                                // Check if we're restoring a session
                                if self.restoring_session {
                                    // Try to select the restored folder
                                    if let Some(restored_id) = self.restored_folder_id.clone() {
                                        if self.sidebar.select_by_id(&restored_id) {
                                            // Found the folder, load its lists
                                            let folder_name = self
                                                .folders
                                                .iter()
                                                .find(|f| f.id == restored_id)
                                                .map(|f| f.name.clone())
                                                .unwrap_or_default();
                                            self.load_lists(restored_id.clone());
                                            self.screen = Screen::Lists;
                                            self.screen_title = generate_screen_title(&folder_name);
                                        } else {
                                            // Folder not found, fallback to Folders screen
                                            self.restoring_session = false;
                                            self.screen = Screen::Folders;
                                            self.screen_title = generate_screen_title("Folders");
                                            self.status = "Saved folder not found, showing folders"
                                                .to_string();
                                            tracing::warn!("Restored folder {} not found, falling back to Folders", restored_id);
                                        }
                                    } else {
                                        // No folder ID saved, stay at Folders
                                        self.restoring_session = false;
                                        self.sidebar.select_first();
                                        self.status =
                                            format!("Loaded {} folder(s)", self.folders.len());
                                    }
                                } else {
                                    // Normal behavior (not restoring)
                                    // Check if we have a current_folder_id from navigation context
                                    if let Some(ref folder_id) = self.current_folder_id {
                                        if !self.sidebar.select_by_id(folder_id) {
                                            // Folder not found, fallback to first
                                            self.sidebar.select_first();
                                        }
                                    } else {
                                        self.sidebar.select_first();
                                    }
                                    self.status =
                                        format!("Loaded {} folder(s)", self.folders.len());
                                }

                                // Clear any previous error state
                                self.error = None;
                            }
                            Err(e) => {
                                self.loading = false;
                                self.error = Some(format!("Failed to load folders: {}", e));
                                self.status = "Failed to load folders".to_string();
                                if self.restoring_session {
                                    self.restoring_session = false;
                                }
                            }
                        }
                    }
                    AppMessage::ListsLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(lists) => {
                                self.lists = lists.clone();
                                // Cache any status configuration that came with the lists
                                for l in &self.lists {
                                    if !l.statuses.is_empty() {
                                        self.cached_list_statuses
                                            .insert(l.id.clone(), l.statuses.clone());
                                    }
                                }
                                // Populate sidebar with lists
                                let mut items = Vec::new();
                                items.extend(self.lists.iter().map(|l| SidebarItem::List {
                                    name: l.name.clone(),
                                    id: l.id.clone(),
                                }));
                                *self.sidebar.items_mut() = items;

                                // Check if we're restoring a session
                                if self.restoring_session {
                                    // Try to select the restored list
                                    if let Some(restored_id) = self.restored_list_id.clone() {
                                        if self.sidebar.select_by_id(&restored_id) {
                                            // Found the list, load its tasks
                                            self.load_tasks(restored_id.clone());
                                            self.screen = Screen::Tasks;
                                            self.update_screen_title();
                                        } else {
                                            // List not found, fallback to Lists screen
                                            self.restoring_session = false;
                                            self.screen = Screen::Lists;
                                            self.screen_title = generate_screen_title("Lists");
                                            self.status =
                                                "Saved list not found, showing lists".to_string();
                                            tracing::warn!(
                                                "Restored list {} not found, falling back to Lists",
                                                restored_id
                                            );
                                        }
                                    } else {
                                        // No list ID saved, stay at Lists
                                        self.restoring_session = false;
                                        self.sidebar.select_first();
                                        self.status =
                                            format!("Loaded {} list(s)", self.lists.len());
                                    }
                                } else {
                                    // Normal behavior (not restoring)
                                    // Check if we have a current_list_id from navigation context
                                    if let Some(ref list_id) = self.current_list_id {
                                        if !self.sidebar.select_by_id(list_id) {
                                            // List not found, fallback to first
                                            self.sidebar.select_first();
                                        }
                                    } else {
                                        self.sidebar.select_first();
                                    }
                                    self.status = format!("Loaded {} list(s)", self.lists.len());
                                }

                                // Clear any previous error state
                                self.error = None;
                            }
                            Err(e) => {
                                self.loading = false;
                                self.error = Some(format!("Failed to load lists: {}", e));
                                self.status = "Failed to load lists".to_string();
                                if self.restoring_session {
                                    self.restoring_session = false;
                                }
                            }
                        }
                    }
                    AppMessage::TasksLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(tasks) => {
                                // Store tasks as source of truth
                                self.tasks = tasks;
                                // Record the statuses we just saw: the next
                                // load may be filtered down to one of them.
                                self.remember_status_names();
                                // Build grouped task list (preserving fold state)
                                self.task_list = GroupedTaskList::from_tasks_with_folds(
                                    self.tasks.clone(),
                                    self.task_list.folded_groups().clone(),
                                );

                                // Check if we're restoring a session
                                if self.restoring_session {
                                    // Try to select the restored task
                                    if let Some(ref restored_id) = self.restored_task_id {
                                        // Find and select the restored task by ID
                                        let found = self
                                            .task_list
                                            .rows()
                                            .iter()
                                            .enumerate()
                                            .find(|(_, r)| matches!(r, ListRow::Task(t) if &t.id == restored_id))
                                            .map(|(idx, _)| idx);

                                        if let Some(task_idx) = found {
                                            self.task_list.select(Some(task_idx));
                                            self.restoring_session = false;
                                            self.status = format!(
                                                "Restored to Tasks view - {} task(s) loaded",
                                                self.task_list
                                                .rows()
                                                .iter()
                                                .filter(|r| matches!(r, ListRow::Task(_)))
                                                .count()
                                            );
                                            tracing::info!("Session restore complete: tasks loaded, task {} selected", restored_id);
                                        } else {
                                            self.restoring_session = false;
                                            self.task_list.select_first();
                                            self.status =
                                                "Saved task not found, showing tasks".to_string();
                                            tracing::warn!(
                                                "Restored task {} not found, falling back to Tasks",
                                                restored_id
                                            );
                                        }
                                    } else {
                                        self.restoring_session = false;
                                        self.task_list.select_first();
                                        self.status = format!(
                                            "Loaded {} task(s)",
                                            self.task_list
                                            .rows()
                                            .iter()
                                            .filter(|r| matches!(r, ListRow::Task(_)))
                                            .count()
                                        );
                                    }
                                } else if let Some(ref pending_id) = self.pending_task_selection.take()
                                {
                                    let found = self
                                        .task_list
                                        .rows()
                                        .iter()
                                        .enumerate()
                                        .find(|(_, r)| matches!(r, ListRow::Task(t) if &t.id == pending_id))
                                        .map(|(idx, _)| idx);
                                    match found {
                                        Some(idx) => self.task_list.select(Some(idx)),
                                        None => self.task_list.select_first(),
                                    }
                                    self.status = format!(
                                        "Refreshed - {} task(s)",
                                        self.task_list
                                            .rows()
                                            .iter()
                                            .filter(|r| matches!(r, ListRow::Task(_)))
                                            .count()
                                    );
                                } else {
                                    self.task_list.select_first();
                                    self.status = format!(
                                        "Loaded {} task(s)",
                                        self.task_list
                                        .rows()
                                        .iter()
                                        .filter(|r| matches!(r, ListRow::Task(_)))
                                        .count()
                                    );
                                }

                                // Clear any previous error state
                                self.error = None;
                            }
                            Err(e) => {
                                self.loading = false;
                                self.error = Some(format!("Failed to load tasks: {}", e));
                                self.status = "Failed to load tasks".to_string();
                                // Clear tasks on error to prevent stale data
                                self.tasks.clear();
                                self.task_list = GroupedTaskList::new();
                                if self.restoring_session {
                                    self.restoring_session = false;
                                }
                            }
                        }
                    }
                    AppMessage::CommentsLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(comments) => {
                                tracing::debug!("Loaded {} comments", comments.len());
                                for (i, comment) in comments.iter().enumerate() {
                                    tracing::debug!(
                                        "Comment {}: id={}, parent_id={:?}, author={:?}",
                                        i,
                                        comment.id,
                                        comment.parent_id,
                                        comment.commenter.as_ref().map(|c| &c.username)
                                    );
                                }
                                self.comments = comments;
                                // A refresh reload tries to keep the user on
                                // the same comment instead of jumping to top.
                                if let Some(ref pending_id) = self.pending_comment_selection {
                                    self.comment_selected_index = self
                                        .comments
                                        .iter()
                                        .position(|c| &c.id == pending_id)
                                        .unwrap_or(0);
                                    self.pending_comment_selection = None;
                                } else {
                                    self.comment_selected_index = 0;
                                }
                                self.error = None;
                                self.status = if self.refresh_reloading_comments {
                                    self.refresh_reloading_comments = false;
                                    format!("Refreshed - {} comment(s)", self.comments.len())
                                } else {
                                    format!("Loaded {} comment(s)", self.comments.len())
                                };
                            }
                            Err(e) => {
                                self.refresh_reloading_comments = false;
                                self.error = Some(format!("Failed to load comments: {}", e));
                                self.status = "Failed to load comments".to_string();
                                self.comments.clear();
                                self.comment_selected_index = 0;
                                self.comment_editing_index = None;
                                self.comment_new_text.clear();
                                self.comment_cursor = 0;
                            }
                        }
                    }
                    AppMessage::CommentCreated(result, comment_meta) => {
                        self.loading = false;
                        match result {
                            Ok(comment) => {
                                self.comments.insert(0, comment);
                                self.comment_new_text.clear();
                                self.comment_cursor = 0;
                                self.comment_editing_index = None;
                                self.status = if comment_meta.is_reply {
                                    "Reply added".to_string()
                                } else {
                                    "Comment added".to_string()
                                };

                                // refetch comments on comment created
                                self.load_comments(comment_meta.task_id);
                            }
                            Err(e) => {
                                self.error = Some(format!(
                                    "Failed to create {}: {}",
                                    if comment_meta.is_reply {
                                        "reply"
                                    } else {
                                        "comment"
                                    },
                                    e
                                ));
                                self.status = format!(
                                    "Failed to create {}",
                                    if comment_meta.is_reply {
                                        "reply"
                                    } else {
                                        "comment"
                                    }
                                );
                            }
                        }
                    }
                    AppMessage::CommentUpdated(result) => {
                        self.loading = false;
                        match result {
                            Ok(comment) => {
                                if let Some(idx) =
                                    self.comments.iter().position(|c| c.id == comment.id)
                                {
                                    self.comments[idx] = comment;
                                }
                                self.comment_new_text.clear();
                                self.comment_cursor = 0;
                                self.comment_editing_index = None;
                                self.status = "Comment updated".to_string();
                            }
                            Err(e) => {
                                self.error = Some(format!("Failed to update comment: {}", e));
                                self.status = "Failed to update comment".to_string();
                            }
                        }
                    }
                    AppMessage::CurrentUserLoaded(result) => {
                        match result {
                            Ok(user) => {
                                self.current_user_id = Some(user.id as i32);
                                tracing::info!(
                                    "Detected current user ID from API: {} ({})",
                                    user.id,
                                    user.username
                                );

                                // If assigned filter is active, re-fetch with the fresh user ID
                                // to replace any stale results from a cached ID
                                if self.assigned_filter_active {
                                    if let Some(list_id) = &self.current_list_id {
                                        self.load_tasks_with_assigned_filter(list_id.clone());
                                    }
                                }

                                // A session saved in the Assigned view reopens
                                // there. This needs the user id, which only
                                // exists now, so it cannot happen during the
                                // hierarchy replay.
                                self.try_restore_query_screen();
                            }
                            Err(e) => {
                                tracing::warn!("Background user profile fetch failed: {}", e);
                            }
                        }
                    }
                    AppMessage::ListStatusesLoaded(list_id, result) => {
                        self.loading = false;
                        match result {
                            Ok(statuses) => {
                                if !statuses.is_empty() {
                                    self.cached_list_statuses
                                        .insert(list_id.clone(), statuses.clone());
                                }
                                // If the picker is open waiting for statuses, refresh it
                                if self.status_picker_open {
                                    if !statuses.is_empty() {
                                        self.status_picker_statuses = statuses;
                                        if self.status_picker_cursor
                                            >= self.status_picker_statuses.len()
                                        {
                                            self.status_picker_cursor = 0;
                                        }
                                    }
                                    self.status = "Statuses loaded".to_string();
                                }
                            }
                            Err(e) => {
                                // Keep whatever options the picker already has
                                if self.status_picker_open {
                                    self.status = format!(
                                        "Could not load full status list ({}); showing known statuses",
                                        e
                                    );
                                }
                            }
                        }
                    }
                    AppMessage::MembersLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(members) => {
                                // Cache the members
                                if let Some(list_id) = &self.current_list_id {
                                    self.cached_list_members
                                        .insert(list_id.clone(), members.clone());
                                }
                                // Open the picker
                                self.open_assignee_picker(members);
                            }
                            Err(e) => {
                                self.status = format!("Failed to load members: {}", e);
                            }
                        }
                    }
                    AppMessage::AssigneesUpdated(result) => {
                        self.loading = false;
                        match result {
                            Ok(updated_task) => {
                                // Update the task in the tasks list (app cache)
                                for task in &mut self.tasks {
                                    if task.id == updated_task.id {
                                        *task = updated_task.clone();
                                        break;
                                    }
                                }
                                // Rebuild grouped task list (status may have changed group)
                                self.rebuild_task_list();
                                // Update task detail view
                                self.task_detail.task = Some(updated_task.clone());
                                self.assignee_picker_open = false;
                                self.status = "Assignees updated".to_string();
                            }
                            Err(e) => {
                                self.status = format!("Failed to update assignees: {}", e);
                            }
                        }
                    }
                    AppMessage::TaskStatusUpdated(result) => {
                        self.loading = false;
                        match result {
                            Ok(updated_task) => {
                                // Update the task in the tasks list (app cache)
                                for task in &mut self.tasks {
                                    if task.id == updated_task.id {
                                        *task = updated_task.clone();
                                        break;
                                    }
                                }
                                // Rebuild grouped task list (status may have changed group)
                                self.rebuild_task_list();
                                // Update task detail view
                                self.task_detail.task = Some(updated_task.clone());
                                self.status_picker_open = false;
                                self.status = "Status updated".to_string();
                            }
                            Err(e) => {
                                // Rollback: restore original status in tasks
                                if let Some(ref original_status) =
                                    self.status_picker_original_status
                                {
                                    if let Some(ref task_id) = self.status_picker_task_id {
                                        for task in &mut self.tasks {
                                            if task.id == *task_id {
                                                task.status = Some(original_status.clone());
                                                break;
                                            }
                                        }
                                        if let Some(detail_task) = &mut self.task_detail.task {
                                            if detail_task.id == *task_id {
                                                detail_task.status = Some(original_status.clone());
                                            }
                                        }
                                    }
                                }
                                // Rebuild grouped task list after rollback
                                self.rebuild_task_list();
                                self.status_picker_open = false;
                                self.status = format!("Failed to update status: {}", e);
                            }
                        }
                    }
                    AppMessage::TaskDescriptionUpdated(result) => {
                        self.loading = false;
                        match result {
                            Ok(updated_task) => {
                                for task in &mut self.tasks {
                                    if task.id == updated_task.id {
                                        *task = updated_task.clone();
                                        break;
                                    }
                                }
                                self.rebuild_task_list();
                                self.task_detail.task = Some(updated_task.clone());
                                self.description_edit_buffer.clear();
                                self.description_edit_cursor = 0;
                                self.status = "Description saved".to_string();
                            }
                            Err(e) => {
                                self.error = Some(format!("Failed to save description: {}", e));
                                self.status = "Failed to save description".to_string();
                            }
                        }
                    }
                    AppMessage::MentionMembersLoaded(result) => match result {
                        Ok(members) => {
                            self.mention_members = members;
                            // Cache per list so re-opening the picker (or
                            // another task in the same list) is instant.
                            if let Some(list_id) = self.current_list_id.clone() {
                                self.cached_list_members
                                    .insert(list_id, self.mention_members.clone());
                            }
                            // Seed the mention registry so a hand-typed
                            // "@Some Name" still resolves to a real tag.
                            self.seed_mention_registry();
                            if self.mention_members.is_empty() {
                                self.status = "No members found for @mentions".to_string();
                            }
                        }
                        Err(e) => {
                            self.mention_picker_open = false;
                            self.insert_mode_target = self.mention_prev_target.take();
                            self.status = format!("Could not load members for @: {}", e);
                        }
                    }
                    AppMessage::TaskFetchedForNavigation(result, prev_screen) => {
                        match result {
                            Ok(task) => {
                                self.navigating = false;
                                self.navigating_level.clear();
                                // Open the task in detail view
                                self.task_detail.task = Some(task.clone());
                                self.reset_task_detail_view();
                                self.screen = Screen::TaskDetail;
                                self.comment_view_mode = CommentViewMode::TopLevel;
                                self.comments.clear();
                                self.comment_selected_index = 0;
                                self.status = format!("Navigated to task: {}", task.name);
                                // Load comments for the task
                                self.load_comments(task.id);
                            }
                            Err(e) => {
                                self.navigating = false;
                                self.navigating_level.clear();
                                if e.contains("Token invalid or expired") {
                                    self.status = "Token invalid - re-authenticate with a fresh ClickUp API token (pk_...)".to_string();
                                } else {
                                    self.status = format!("Resource not found: {}", e);
                                }
                                // Restore previous screen
                                self.screen = prev_screen;
                            }
                        }
                    }
                    AppMessage::CommentFetchedForNavigation(result, comment_id, prev_screen) => {
                        match result {
                            Ok(task) => {
                                // Navigate to task detail, then find the comment
                                self.task_detail.task = Some(task.clone());
                                self.reset_task_detail_view();
                                self.screen = Screen::TaskDetail;
                                self.comment_view_mode = CommentViewMode::TopLevel;
                                self.comments.clear();
                                self.comment_selected_index = 0;

                                // Load comments and then find the target one
                                let tx = self.message_tx.clone().unwrap();
                                let task_id = task.id.clone();
                                let client = match &self.client {
                                    Some(c) => c.clone(),
                                    None => {
                                        self.navigating = false;
                                        self.navigating_level.clear();
                                        self.status = "Not authenticated".to_string();
                                        self.screen = prev_screen;
                                        return;
                                    }
                                };
                                tokio::spawn(async move {
                                    let result = client.get_task_comments(&task_id).await;
                                    let msg = match result {
                                        Ok(comments) => {
                                            AppMessage::CommentsLoadedForCommentNavigation(
                                                Ok(comments),
                                                comment_id,
                                            )
                                        }
                                        Err(e) => AppMessage::CommentsLoadedForCommentNavigation(
                                            Err(e.to_string()),
                                            comment_id,
                                        ),
                                    };
                                    let _ = tx.send(msg).await;
                                });
                            }
                            Err(e) => {
                                self.navigating = false;
                                self.navigating_level.clear();
                                if e.contains("Token invalid or expired") {
                                    self.status = "Token invalid - re-authenticate with a fresh ClickUp API token (pk_...)".to_string();
                                } else {
                                    self.status = format!("Resource not found: {}", e);
                                }
                                self.screen = prev_screen;
                            }
                        }
                    }
                    AppMessage::DocumentFetchedForNavigation(result, prev_screen) => match result {
                        Ok(doc) => {
                            self.navigating = false;
                            self.navigating_level.clear();
                            self.documents = vec![doc.clone()];
                            self.screen = Screen::Document;
                            self.screen_title = generate_screen_title(&doc.name);
                            self.status = format!("Navigated to document: {}", doc.name);
                        }
                        Err(e) => {
                            self.navigating = false;
                            self.navigating_level.clear();
                            self.status = format!("Document not found: {}", e);
                            self.screen = prev_screen;
                        }
                    },
                    AppMessage::CommentsLoadedForCommentNavigation(result, comment_id) => {
                        match result {
                            Ok(comments) => {
                                self.comments = comments;
                                self.comment_view_mode = CommentViewMode::TopLevel;
                                // Find and select the target comment
                                let found = self.comments.iter().position(|c| c.id == comment_id);
                                if let Some(idx) = found {
                                    self.comment_selected_index = idx;
                                    self.navigating = false;
                                    self.navigating_level.clear();
                                    self.status = format!(
                                        "Navigated to comment: {}",
                                        crate::utils::truncate_chars(&self.comments[idx].text, 40)
                                    );
                                } else {
                                    self.navigating = false;
                                    self.navigating_level.clear();
                                    self.status = "Comment not found in task".to_string();
                                }
                            }
                            Err(e) => {
                                self.navigating = false;
                                self.navigating_level.clear();
                                self.status = format!("Failed to load comments: {}", e);
                            }
                        }
                    }
                    AppMessage::CommentReactionUpdated(result, comment_id, reaction, added) => {
                        match result {
                            Ok(()) => {
                                self.status = if added {
                                    "Reacted 👍".to_string()
                                } else {
                                    "Removed 👍".to_string()
                                };
                            }
                            Err(e) => {
                                // Roll the optimistic flip back so the UI keeps
                                // matching what ClickUp actually stored.
                                self.apply_reaction_locally(&comment_id, &reaction, !added);
                                self.error = Some(format!("Failed to update reaction: {}", e));
                                self.status = "Reaction failed".to_string();
                            }
                        }
                    }
                    AppMessage::InboxLoaded(result) => {
                        self.inbox_loading = false;
                        self.loading = false;
                        match result {
                            Ok(inbox) => {
                                self.apply_inbox(inbox);
                                self.error = None;
                                self.status = if self.inbox_tasks.is_empty() {
                                    "Inbox clear".to_string()
                                } else {
                                    format!(
                                        "Inbox: {} item(s), {} unread",
                                        self.inbox_tasks.len(),
                                        self.inbox_unread_count()
                                    )
                                };
                                // Badge can change without a screen switch.
                                *self.sidebar.items_mut() = self.root_sidebar_items();
                                self.sync_sidebar_selection_to_screen();
                                self.update_screen_title();
                            }
                            Err(e) => {
                                // Undocumented endpoint: report it as
                                // unavailable rather than as a hard failure,
                                // and keep the user on a usable screen.
                                tracing::warn!("Inbox unavailable: {}", e);
                                self.inbox_tasks.clear();
                                self.status = "Inbox unavailable".to_string();
                                *self.sidebar.items_mut() = self.root_sidebar_items();
                            }
                        }
                    }
                    AppMessage::AssignedTasksLoaded(result, page) => {
                        match result {
                            Ok(tasks) => {
                                let fetched = tasks.len();
                                // One row store for every screen: the
                                // Assigned view renders from `tasks` exactly
                                // like a single list does.
                                self.tasks.extend(tasks);
                                // Keep the status registry growing so the
                                // filter menu survives an active filter.
                                self.remember_status_names();
                                // Ordering lives in the rebuild, so every
                                // surface sorts the same way.
                                self.rebuild_task_list();

                                // A full page means there is probably more; a
                                // short page ends the walk.
                                const PAGE_SIZE: usize = 100;
                                if fetched == PAGE_SIZE {
                                    if let Some(team_id) = self.current_team_id.clone() {
                                        self.fetch_assigned_page(team_id, page + 1);
                                        return; // keep the loading state up
                                    }
                                }

                                self.loading = false;
                                self.assigned_loading = false;
                                self.assigned_page_in_flight = None;
                                self.error = None;
                                self.status = format!(
                                    "Loaded {} assigned task(s)",
                                    self.tasks.len()
                                );
                            }
                            Err(e) => {
                                self.loading = false;
                                self.assigned_loading = false;
                                self.assigned_page_in_flight = None;
                                self.error = Some(format!("Failed to load assigned tasks: {}", e));
                                self.status = "Failed to load assigned tasks".to_string();
                            }
                        }
                    }
                    AppMessage::TaskRefreshed(result) => {
                        self.loading = false;
                        match result {
                            Ok(task) => {
                                // The user may have navigated away before the
                                // response landed; only repaint the detail view
                                // if it still shows this task.
                                let still_viewing = self.screen == Screen::TaskDetail
                                    && self
                                        .task_detail
                                        .task
                                        .as_ref()
                                        .map(|t| t.id == task.id)
                                        .unwrap_or(false);
                                for cached in &mut self.tasks {
                                    if cached.id == task.id {
                                        *cached = task.clone();
                                        break;
                                    }
                                }
                                self.rebuild_task_list();
                                if still_viewing {
                                    self.task_detail.task = Some(task.clone());
                                    self.update_screen_title();
                                    // Pull the latest comments as well.
                                    self.load_comments(task.id);
                                } else {
                                    self.refresh_reloading_comments = false;
                                    self.pending_comment_selection = None;
                                }
                                self.error = None;
                                self.status = format!("Refreshed task: {}", task.name);
                            }
                            Err(e) => {
                                self.error = Some(format!("Failed to refresh task: {}", e));
                                self.status = "Refresh failed".to_string();
                            }
                        }
                    }
                    AppMessage::DocumentRefreshed(result) => {
                        self.loading = false;
                        match result {
                            Ok(doc) if self.screen == Screen::Document => {
                                self.documents = vec![doc.clone()];
                                self.document.title = doc.name.clone();
                                if let Some(page) = doc.pages.first() {
                                    if let Some(content) = page
                                        .content_markdown
                                        .clone()
                                        .or_else(|| page.content.clone())
                                    {
                                        self.document.content = content;
                                    }
                                }
                                self.update_screen_title();
                                self.error = None;
                                self.status = format!("Refreshed document: {}", doc.name);
                            }
                            // Response arrived after the user left the
                            // document screen: nothing to repaint.
                            Ok(_) => {
                                self.status = "Refreshed".to_string();
                            }
                            Err(e) => {
                                self.error = Some(format!("Failed to refresh document: {}", e));
                                self.status = "Refresh failed".to_string();
                            }
                        }
                    }
                    AppMessage::TaskCreated(result) => {
                        match result {
                            Ok(task) => {
                                self.loading = false;
                                self.task_name_input.clear();
                                self.task_name_cursor = 0;
                                self.task_description_input.clear();
                                self.task_description_cursor = 0;
                                self.task_creating = false;
                                self.task_detail.creating = false;
                                self.status = format!("Task created: {}", task.name);
                                // Reload the task list and return to tasks view
                                if let Some(list_id) = &self.current_list_id {
                                    let list_id_clone = list_id.clone();
                                    let filter_active = self.assigned_filter_active;
                                    if filter_active {
                                        self.load_tasks_with_assigned_filter(list_id_clone);
                                    } else {
                                        self.load_tasks(list_id_clone);
                                    }
                                }
                                self.screen = Screen::Tasks;
                                self.update_screen_title();
                            }
                            Err(e) => {
                                self.loading = false;
                                self.error = Some(format!("Failed to create task: {}", e));
                                self.status = "Task creation failed".to_string();
                                // Keep task_creating = true so the form stays open
                            }
                        }
                    }
                    AppMessage::TaskDeleted(result) => {
                        match result {
                            Ok(task_id) => {
                                // Remove the task from the local list
                                self.tasks.retain(|task| task.id != task_id);

                                let deleted_open_task = self
                                    .task_detail
                                    .task
                                    .as_ref()
                                    .is_some_and(|task| task.id == task_id);
                                if deleted_open_task {
                                    self.task_detail.task = None;
                                    self.task_detail.editing = false;
                                    self.task_detail.creating = false;
                                    self.comments.clear();
                                    self.comment_selected_index = 0;
                                    self.comment_editing_index = None;
                                    self.comment_new_text.clear();
                                    self.comment_cursor = 0;
                                    self.comment_focus = false;
                                    self.comment_view_mode = CommentViewMode::TopLevel;
                                    if self.screen == Screen::TaskDetail {
                                        self.screen = Screen::Tasks;
                                    }
                                }

                                // Rebuild grouped rows so the deleted task disappears
                                // from the widget, not just from `self.tasks`.
                                self.rebuild_task_list();
                                self.update_screen_title();
                                self.status = "Task deleted".to_string();
                            }
                            Err(e) => {
                                self.error = Some(format!("Failed to delete task: {}", e));
                                self.status = "Failed to delete task".to_string();
                                // Task remains in list — user can retry
                            }
                        }
                    }
                }
            }
        }
    }
}
