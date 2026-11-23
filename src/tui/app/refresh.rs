//! Reloading the current view and the comment/task mutations.

use super::*;

impl TuiApp {

    /// Re-query ClickUp for whatever is currently displayed (`SPC r`).
    ///
    /// Each screen maps to the request that produced it, so the refresh
    /// re-runs that request and the usual message handlers repaint the view:
    /// a task list picks up renamed tasks, task detail picks up new comments,
    /// and so on. Selection/fold state is preserved where it still applies.
    pub fn refresh_current_view(&mut self) {
        if self.client.is_none() {
            self.status = "Not authenticated".to_string();
            return;
        }

        match self.screen {
            Screen::Auth => {
                self.status = "Nothing to refresh before sign-in".to_string();
            }
            Screen::Workspaces => {
                self.load_workspaces();
                self.status = "Refreshing workspaces...".to_string();
            }
            Screen::Inbox => {
                self.load_inbox();
                self.status = "Refreshing inbox...".to_string();
            }
            Screen::Assigned => {
                self.load_assigned_tasks();
                self.status = "Refreshing assigned tasks...".to_string();
            }
            Screen::Spaces => match self.current_workspace_id.clone() {
                Some(id) => {
                    self.load_spaces(id);
                    self.status = "Refreshing spaces...".to_string();
                }
                None => self.status = "Nothing to refresh".to_string(),
            },
            Screen::Folders => match self.current_space_id.clone() {
                Some(id) => {
                    self.load_folders(id);
                    self.status = "Refreshing folders...".to_string();
                }
                None => self.status = "Nothing to refresh".to_string(),
            },
            Screen::Lists => match self.current_folder_id.clone() {
                Some(id) => {
                    self.load_lists(id);
                    self.status = "Refreshing lists...".to_string();
                }
                None => self.status = "Nothing to refresh".to_string(),
            },
            Screen::Tasks => match self.current_list_id.clone() {
                Some(id) => {
                    // Remember the selected task so the rebuilt list can
                    // land back on it even if the order changed upstream.
                    self.pending_task_selection =
                        self.task_list.selected_task().map(|t| t.id.clone());
                    self.load_tasks(id);
                    self.status = "Refreshing tasks...".to_string();
                }
                None => self.status = "Nothing to refresh".to_string(),
            },
            Screen::TaskDetail => match self.task_detail.task.as_ref().map(|t| t.id.clone()) {
                Some(task_id) => {
                    let client = self.client.clone().unwrap();
                    let tx = self.message_tx.clone().unwrap();
                    let comment_id = self.selected_comment_id();
                    self.pending_comment_selection = comment_id;
                    self.refresh_reloading_comments = true;
                    self.loading = true;
                    self.status = "Refreshing task...".to_string();
                    tokio::spawn(async move {
                        let result = client.get_task(&task_id).await;
                        let msg = match result {
                            Ok(task) => AppMessage::TaskRefreshed(Ok(task)),
                            Err(e) => AppMessage::TaskRefreshed(Err(e.to_string())),
                        };
                        let _ = tx.send(msg).await;
                    });
                }
                None => {
                    // Inside the task screen without a task (e.g. creation
                    // form): refreshing the underlying list is still useful.
                    self.status = "Nothing to refresh".to_string();
                }
            },
            Screen::Document => match self.documents.first().map(|d| d.id.clone()) {
                Some(doc_id) => {
                    let client = self.client.clone().unwrap();
                    let tx = self.message_tx.clone().unwrap();
                    self.loading = true;
                    self.status = "Refreshing document...".to_string();
                    tokio::spawn(async move {
                        use crate::models::DocumentFilters;
                        let filters = DocumentFilters::default();
                        let result = client.search_docs(&filters).await;
                        let msg = match result {
                            Ok(docs) => match docs.into_iter().find(|d| d.id == doc_id) {
                                Some(doc) => AppMessage::DocumentRefreshed(Ok(doc)),
                                None => AppMessage::DocumentRefreshed(Err(
                                    "Document no longer available".to_string(),
                                )),
                            },
                            Err(e) => AppMessage::DocumentRefreshed(Err(e.to_string())),
                        };
                        let _ = tx.send(msg).await;
                    });
                }
                None => self.status = "Nothing to refresh".to_string(),
            },
        }
    }

    /// ID of the comment currently focused/highlighted on the task screen
    pub(super) fn selected_comment_id(&self) -> Option<String> {
        self.comments
            .get(self.comment_selected_index)
            .map(|c| c.id.clone())
    }

    /// Load comments for a task (top-level + replies)
    pub(super) fn load_comments(&mut self, task_id: String) {
        self.loading = true;
        self.status = "Loading comments...".to_string();

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
            // First, fetch top-level comments
            let top_level_result = client.get_task_comments(&task_id).await;

            match top_level_result {
                Ok(top_level_comments) => {
                    // For each top-level comment, fetch its replies
                    let mut all_comments = top_level_comments;

                    // Collect all reply fetches
                    let mut reply_futures = Vec::new();
                    for comment in &all_comments {
                        let comment_id = comment.id.clone();
                        let client_clone = client.clone();
                        reply_futures.push(async move {
                            let result = client_clone.get_comment_replies(&comment_id).await;
                            (comment_id, result)
                        });
                    }

                    // Wait for all replies to be fetched
                    let reply_results = futures::future::join_all(reply_futures).await;

                    // Add replies to the comments list with parent_id set
                    for (parent_id, reply_result) in reply_results {
                        if let Ok(replies) = reply_result {
                            for mut reply in replies {
                                reply.parent_id = Some(parent_id.clone());
                                all_comments.push(reply);
                            }
                        }
                    }

                    let msg = AppMessage::CommentsLoaded(Ok(all_comments));
                    let _ = tx.send(msg).await;
                }
                Err(e) => {
                    let msg = AppMessage::CommentsLoaded(Err(e.to_string()));
                    let _ = tx.send(msg).await;
                }
            }
        });
    }

    /// Create a new comment (top-level or reply)
    pub(super) fn create_comment(&mut self, task_id: String, text: String, parent_id: Option<String>) {
        self.loading = true;
        // Show appropriate status message based on whether this is a reply
        let is_reply = parent_id.is_some();
        self.status = if is_reply {
            "Saving reply...".to_string()
        } else {
            "Saving comment...".to_string()
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
        // Real @mentions need the structured block form: a `tag` block with
        // the user object is the only shape the web app turns into a link.
        // `comment_text` must then be empty, or ClickUp echoes the text twice.
        let blocks = crate::tui::mention::build_comment_blocks(&text, &self.mention_registry);
        let comment_text = if blocks.is_some() {
            String::new()
        } else {
            text.clone()
        };
        let request = CreateCommentRequest {
            comment_text,
            comment: blocks,
            assignee: None,
            assigned_commenter: None,
            parent_id: parent_id.clone(),
        };

        tokio::spawn(async move {
            // Use different endpoint based on whether this is a reply
            let result = if let Some(ref parent_comment_id) = parent_id {
                client
                    .create_comment_reply(parent_comment_id, &request)
                    .await
            } else {
                client.create_comment(&task_id, &request).await
            };

            let meta = CommentCreatedMessageMeta { is_reply, task_id };
            let msg = match result {
                Ok(comment) => AppMessage::CommentCreated(Ok(comment), meta),
                Err(e) => AppMessage::CommentCreated(Err(e.to_string()), meta),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Create a new task
    pub(super) fn create_task(&mut self, list_id: String) {
        self.loading = true;
        self.status = "Creating task...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        let name = self.task_name_input.clone();
        let description = if self.task_description_input.is_empty() {
            None
        } else {
            Some(self.task_description_input.clone())
        };

        let request = CreateTaskRequest {
            name,
            description,
            status: None,
            priority: None,
            assignees: None,
            due_date: None,
        };

        tokio::spawn(async move {
            let result = client.create_task(&list_id, &request).await;
            let msg = match result {
                Ok(task) => AppMessage::TaskCreated(Ok(task)),
                Err(e) => AppMessage::TaskCreated(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Open the delete confirmation for a specific task.
    pub(super) fn open_delete_confirmation(&mut self, task_id: Option<String>) {
        match task_id {
            Some(task_id) => {
                self.pending_delete_task_id = Some(task_id);
                self.dialog.show(DialogType::ConfirmDelete);
            }
            None => {
                self.status = "No task selected".to_string();
            }
        }
    }

    /// Delete the task captured when the confirmation dialog was opened.
    pub(super) fn delete_selected_task(&mut self) {
        let task_id = match self.pending_delete_task_id.take() {
            Some(task_id) => task_id,
            None => {
                // Fallback for callers that show the dialog without going through
                // open_delete_confirmation (e.g. older tests).
                let fallback = if self.screen == Screen::TaskDetail {
                    self.task_detail.task.as_ref().map(|task| task.id.clone())
                } else {
                    self.task_list.selected_task().map(|task| task.id.clone())
                };
                match fallback {
                    Some(task_id) => task_id,
                    None => {
                        self.status = "No task selected".to_string();
                        return;
                    }
                }
            }
        };

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        let task_id_clone = task_id.clone();
        self.status = format!("Deleting task {}...", task_id);
        tokio::spawn(async move {
            let result = client.delete_task(&task_id_clone).await;
            let msg = match result {
                Ok(()) => AppMessage::TaskDeleted(Ok(task_id_clone)),
                Err(e) => AppMessage::TaskDeleted(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Update an existing comment
    pub(super) fn update_comment(&mut self, comment_id: String, text: String) {
        self.loading = true;
        self.status = "Saving comment...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        // Editing re-sends plain text only: the update endpoint has no
        // verified support for the structured `comment` block array, so a
        // mention written here would save as literal "@Name" text.
        let request = UpdateCommentRequest {
            comment_text: Some(text),
            assigned: None,
            assignee: None,
            assigned_commenter: None,
        };
        tokio::spawn(async move {
            let result = client.update_comment(&comment_id, &request).await;
            let msg = match result {
                Ok(comment) => AppMessage::CommentUpdated(Ok(comment)),
                Err(e) => AppMessage::CommentUpdated(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    pub fn update_screen_title(&mut self) {
        self.screen_title = match &self.screen {
            Screen::Auth => generate_screen_title("Authentication"),
            Screen::Inbox => {
                let unread = self.inbox_unread_count();
                if unread > 0 {
                    generate_screen_title(&format!("Inbox ({})", unread))
                } else {
                    generate_screen_title("Inbox")
                }
            }
            Screen::Assigned => {
                if !self.status_filters.is_empty() {
                    generate_screen_title(&format!(
                        "Assigned to Me (Status: {})",
                        self.status_filters.join(", ")
                    ))
                } else if self.tasks.is_empty() {
                    generate_screen_title("Assigned to Me")
                } else {
                    generate_screen_title(&format!("Assigned to Me ({})", self.tasks.len()))
                }
            }
            Screen::Workspaces => generate_screen_title("Workspaces"),
            Screen::Spaces => {
                if let Some(workspace) = self
                    .current_workspace_id
                    .as_ref()
                    .and_then(|id| self.workspaces.iter().find(|workspace| &workspace.id == id))
                {
                    generate_screen_title(&workspace.name)
                } else {
                    generate_screen_title("Spaces")
                }
            }
            Screen::Folders => {
                if let Some(space) = self
                    .current_space_id
                    .as_ref()
                    .and_then(|id| self.spaces.iter().find(|space| &space.id == id))
                {
                    generate_screen_title(&space.name)
                } else {
                    generate_screen_title("Folders")
                }
            }
            Screen::Lists => {
                if let Some(folder) = self
                    .current_folder_id
                    .as_ref()
                    .and_then(|id| self.folders.iter().find(|folder| &folder.id == id))
                {
                    generate_screen_title(&folder.name)
                } else {
                    generate_screen_title("Lists")
                }
            }
            Screen::Tasks => {
                if let Some(list) = self
                    .current_list_id
                    .as_ref()
                    .and_then(|id| self.lists.iter().find(|list| &list.id == id))
                {
                    let base = format!("Tasks: {}", list.name);
                    // Filters compose, so both can be shown at once.
                    let mut title = base;
                    if self.assigned_filter_active {
                        title.push_str(" (Assigned to Me)");
                    }
                    if !self.status_filters.is_empty() {
                        title.push_str(&format!(
                            " (Status: {})",
                            self.status_filters.join(", ")
                        ));
                    }
                    generate_screen_title(&title)
                } else if !self.status_filters.is_empty() {
                    generate_screen_title(&format!(
                        "Tasks (Status: {})",
                        self.status_filters.join(", ")
                    ))
                } else {
                    if self.assigned_filter_active {
                        generate_screen_title("Tasks (Assigned to Me)")
                    } else {
                        generate_screen_title("Tasks")
                    }
                }
            }
            Screen::TaskDetail => {
                if let Some(task) = &self.task_detail.task {
                    generate_screen_title(&format!("Task: {}", task.name))
                } else {
                    generate_screen_title("Task Detail")
                }
            }
            Screen::Document => {
                if !self.document.title.is_empty() {
                    generate_screen_title(&format!("Doc: {}", self.document.title))
                } else {
                    generate_screen_title("Document")
                }
            }
        };
    }
}
