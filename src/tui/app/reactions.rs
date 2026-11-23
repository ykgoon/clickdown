//! Comment reactions and clipboard actions.

use super::*;

impl TuiApp {

    /// Toggle the 👍 reaction on the selected comment (`SPC c u`).
    ///
    /// Thumbs-up is the one reaction this client exposes; pressing it again
    /// removes the reaction, so the binding behaves as a toggle. The
    /// optimistic local flip keeps the UI responsive, and the API result
    /// either confirms it or rolls it back.
    pub fn toggle_comment_thumbs_up(&mut self) {
        if self.screen != Screen::TaskDetail {
            self.status = "Comments live in task detail".to_string();
            return;
        }

        let comment_id = match self.selected_comment().map(|c| c.id.clone()) {
            Some(id) => id,
            None => {
                self.status = "Select a comment to react to".to_string();
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

        // Decide direction from what is on screen, then flip it optimistically.
        let adding = !self
            .selected_comment()
            .map(|c| c.has_reaction(THUMBS_UP_REACTION))
            .unwrap_or(false);
        self.apply_reaction_locally(&comment_id, THUMBS_UP_REACTION, adding);

        self.status = if adding {
            "Adding 👍...".to_string()
        } else {
            "Removing 👍...".to_string()
        };

        let tx = self.message_tx.clone().unwrap();
        let reaction = THUMBS_UP_REACTION.to_string();
        tokio::spawn(async move {
            let result = if adding {
                client.add_comment_reaction(&comment_id, &reaction).await
            } else {
                client.remove_comment_reaction(&comment_id, &reaction).await
            };
            let msg = AppMessage::CommentReactionUpdated(
                result.map_err(|e| e.to_string()),
                comment_id,
                reaction,
                adding,
            );
            let _ = tx.send(msg).await;
        });
    }

    /// Add or remove a reaction in the loaded comment, without an API call
    pub(super) fn apply_reaction_locally(&mut self, comment_id: &str, reaction: &str, adding: bool) {
        let reaction = crate::models::comment::canonical_reaction_name(reaction);
        if let Some(comment) = self.comments.iter_mut().find(|c| c.id == comment_id) {
            let present = comment.has_reaction(&reaction);
            if adding && !present {
                comment.reactions.push(reaction.clone());
            } else if !adding && present {
                comment
                    .reactions
                    .retain(|r| !crate::models::comment::canonical_reaction_name(r)
                        .eq_ignore_ascii_case(&reaction));
            }
        }
    }

    /// Copy the content of the focused element (`SPC y c`).
    ///
    /// On the task detail screen the element under TAB focus decides what is
    /// copied: the task description, or the selected comment. Everywhere else
    /// the screen's primary content is copied, so the binding is never dead.
    pub fn copy_focused_content(&mut self) {
        let (text, label) = match self.focused_content_to_copy() {
            Some(pair) => pair,
            None => {
                let message = self.copy_unavailable_reason();
                self.set_copy_status(message);
                return;
            }
        };

        match self.clipboard.copy_text(&text) {
            Ok(()) => {
                let chars = text.chars().count();
                let first_line = text.lines().next().unwrap_or("").trim();
                let preview = if chars == 0 {
                    String::new()
                } else {
                    format!(": {}", crate::utils::truncate_chars(first_line, 40))
                };
                self.set_copy_status(&format!("Copied {} ({} chars){}", label, chars, preview));
            }
            Err(e) => self.set_copy_status(&format!("Failed to copy {}: {}", label, e)),
        }
    }

    /// The text and a short label for the focused element, if there is one.
    ///
    /// This is the single source of truth for `SPC y c`: the command copies
    /// whatever this returns, and tests assert against it directly.
    pub(super) fn focused_content_to_copy(&self) -> Option<(String, &'static str)> {
        match self.screen {
            Screen::TaskDetail => match self.task_screen_focus() {
                TaskScreenFocus::Comment(_) => self
                    .selected_comment()
                    .map(|comment| (crate::utils::comment_text_to_plain(&comment.text), "comment")),
                TaskScreenFocus::Description => self
                    .description_text()
                    .filter(|text| !text.trim().is_empty())
                    .map(|text| (text, "description")),
            },
            Screen::Document => {
                let content = self.document.content.clone();
                if content.trim().is_empty() {
                    None
                } else {
                    Some((content, "document"))
                }
            }
            _ => None,
        }
    }

    /// Why nothing is copyable on the current screen, for the status bar
    pub(super) fn copy_unavailable_reason(&self) -> &'static str {
        match self.screen {
            Screen::TaskDetail => match self.task_screen_focus() {
                TaskScreenFocus::Comment(_) => "No comment selected",
                TaskScreenFocus::Description => "No description to copy",
            },
            Screen::Document => "No document content to copy",
            _ => "Nothing to copy on this screen",
        }
    }

    /// Description of the task currently shown, as editable text
    pub(super) fn description_text(&self) -> Option<String> {
        self.task_detail
            .task
            .as_ref()
            .and_then(|task| task.description.as_ref().map(|d| d.as_text()))
    }

    /// The comment highlighted on the task screen, in the current view
    pub(super) fn selected_comment(&self) -> Option<&crate::models::Comment> {
        let index = self.comment_selected_index;
        // Only copy a comment that is actually visible in the current view, so
        // a stale index left over from another thread cannot copy the wrong one.
        if !self.comment_indices_in_view().contains(&index) {
            return None;
        }
        self.comments.get(index)
    }

    /// Show a transient message in the status bar (shared with URL copy)
    pub(super) fn set_copy_status(&mut self, message: &str) {
        self.url_copy_status = Some(message.to_string());
        self.url_copy_status_time = Some(std::time::Instant::now());
    }

    /// Copy URL for the current context to clipboard
    pub(super) fn copy_url(&mut self) {
        tracing::debug!("copy_url called, screen: {:?}", self.screen);

        // Helper to get ID from sidebar item
        fn get_sidebar_id(item: &SidebarItem) -> &str {
            match item {
                // The Inbox and Assigned entries have no upstream object of
                // their own; the URL comes from the selected item instead.
                SidebarItem::Inbox { .. } => "",
                SidebarItem::AssignedToMe => "",
                SidebarItem::Workspace { id, .. } => id,
                SidebarItem::Space { id, .. } => id,
                SidebarItem::Folder { id, .. } => id,
                SidebarItem::List { id, .. } => id,
            }
        }

        let url_result = match self.screen {
            Screen::Auth => {
                self.url_copy_status = Some("URL copy not available on auth screen".to_string());
                return;
            }
            Screen::Workspaces => {
                if let Some(ws) = self.sidebar.selected_item() {
                    ClickUpUrlGenerator::workspace_url(get_sidebar_id(ws))
                } else {
                    self.url_copy_status = Some("No workspace selected".to_string());
                    return;
                }
            }
            Screen::Spaces => {
                if let Some(space) = self.sidebar.selected_item() {
                    // Use tracked workspace context
                    if let Some(ref ws_id) = self.current_workspace_id {
                        ClickUpUrlGenerator::space_url(ws_id, get_sidebar_id(space))
                    } else {
                        self.url_copy_status = Some("Missing workspace context".to_string());
                        return;
                    }
                } else {
                    self.url_copy_status = Some("No space selected".to_string());
                    return;
                }
            }
            Screen::Folders => {
                if let Some(folder) = self.sidebar.selected_item() {
                    // Use tracked workspace context
                    if let Some(ref ws_id) = self.current_workspace_id {
                        ClickUpUrlGenerator::folder_url(ws_id, get_sidebar_id(folder))
                    } else {
                        self.url_copy_status = Some("Missing workspace context".to_string());
                        return;
                    }
                } else {
                    self.url_copy_status = Some("No folder selected".to_string());
                    return;
                }
            }
            Screen::Lists => {
                if let Some(list) = self.sidebar.selected_item() {
                    // Use tracked workspace context
                    if let Some(ref ws_id) = self.current_workspace_id {
                        ClickUpUrlGenerator::list_url(ws_id, get_sidebar_id(list))
                    } else {
                        self.url_copy_status = Some("Missing workspace context".to_string());
                        return;
                    }
                } else {
                    self.url_copy_status = Some("No list selected".to_string());
                    return;
                }
            }
            Screen::Inbox => {
                // Inbox entries are tasks; use the highlighted one.
                if let Some(task) = self.selected_inbox_task() {
                    ClickUpUrlGenerator::task_url("", "", &task.id)
                } else {
                    self.url_copy_status = Some("No inbox item selected".to_string());
                    return;
                }
            }
            Screen::Tasks | Screen::Assigned => {
                if let Some(task) = self.task_list.selected_task() {
                    // Short-form task URL: only need task ID
                    ClickUpUrlGenerator::task_url("", "", &task.id)
                } else {
                    self.url_copy_status = Some("No task selected".to_string());
                    return;
                }
            }
            Screen::TaskDetail => {
                // Check if comment focus is active and a comment is selected
                if self.comment_focus
                    && !self.comments.is_empty()
                    && self.comment_selected_index < self.comments.len()
                {
                    // Copy selected comment URL
                    let comment = &self.comments[self.comment_selected_index];
                    if let Some(task) = &self.task_detail.task {
                        ClickUpUrlGenerator::comment_url("", "", &task.id, &comment.id)
                    } else {
                        self.url_copy_status = Some("No task selected".to_string());
                        return;
                    }
                } else {
                    // Copy task URL
                    if let Some(task) = &self.task_detail.task {
                        ClickUpUrlGenerator::task_url("", "", &task.id)
                    } else {
                        self.url_copy_status = Some("No task selected".to_string());
                        return;
                    }
                }
            }
            Screen::Document => {
                if let Some(doc) = self.documents.first() {
                    // Short-form document URL: only need doc ID
                    ClickUpUrlGenerator::document_url("", &doc.id)
                } else {
                    self.url_copy_status = Some("No document selected".to_string());
                    return;
                }
            }
        };

        // Handle URL generation result
        let url = match url_result {
            Ok(url) => {
                tracing::debug!("Generated URL: {}", url);
                url
            }
            Err(e) => {
                self.url_copy_status = Some(format!("URL error: {}", e));
                return;
            }
        };

        // Copy to clipboard
        match self.clipboard.copy_text(&url) {
            Ok(()) => {
                // Show success with truncated URL
                let truncated = if url.len() > 60 {
                    format!("{}...", &url[..57])
                } else {
                    url.clone()
                };
                self.url_copy_status = Some(format!("Copied: {}", truncated));
                self.url_copy_status_time = Some(std::time::Instant::now());
            }
            Err(e) => {
                self.url_copy_status = Some(format!("Failed to copy URL: {}", e));
                self.url_copy_status_time = Some(std::time::Instant::now());
            }
        }
    }
}
