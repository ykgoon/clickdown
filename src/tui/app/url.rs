//! URL input: parsing, validation, and navigation from a pasted link.

use super::*;

impl TuiApp {

    /// Navigate to a resource based on a parsed URL
    pub(super) fn navigate_from_parsed_url(&mut self, parsed: crate::utils::ParsedUrl) {
        use crate::utils::ParsedUrl;
        match parsed {
            ParsedUrl::Workspace { workspace_id } => {
                self.navigate_to_workspace(workspace_id);
            }
            ParsedUrl::Space {
                workspace_id,
                space_id,
            } => {
                self.navigate_to_space(workspace_id, space_id);
            }
            ParsedUrl::Folder {
                workspace_id,
                folder_id,
            } => {
                self.navigate_to_folder(workspace_id, folder_id);
            }
            ParsedUrl::List {
                workspace_id,
                list_id,
            } => {
                self.navigate_to_list(workspace_id, list_id);
            }
            ParsedUrl::Task { team_id, task_id } => {
                self.navigate_to_task(team_id, task_id);
            }
            ParsedUrl::Comment {
                team_id,
                task_id,
                comment_id,
            } => {
                self.navigate_to_comment(team_id, task_id, comment_id);
            }
            ParsedUrl::Document { doc_id } => {
                self.navigate_to_document(doc_id);
            }
        }
    }

    // --- URL-based navigation implementations ---

    /// Navigate to a workspace by ID
    pub(super) fn navigate_to_workspace(&mut self, workspace_id: String) {
        self.navigating = true;
        self.navigating_level = "workspace".to_string();

        // Check if already at target
        if self.screen == Screen::Workspaces {
            if let Some(selected) = self.sidebar.selected_item() {
                use crate::tui::widgets::SidebarItem;
                let id = match selected {
                    SidebarItem::Workspace { id, .. } => id.clone(),
                    _ => String::new(),
                };
                if id == workspace_id {
                    self.navigating = false;
                    self.navigating_level.clear();
                    self.status = "Already viewing this workspace".to_string();
                    return;
                }
            }
        }

        // Navigate to workspaces screen and select the workspace
        self.screen = Screen::Workspaces;
        self.current_workspace_id = None;
        self.current_space_id = None;
        self.current_folder_id = None;
        self.current_list_id = None;
        // The root sidebar is not the bare workspace list: it starts with the
        // pinned Inbox and Assigned entries, so the root has to be rebuilt and
        // the target selected by ID rather than by workspace index.
        *self.sidebar.items_mut() = self.root_sidebar_items();
        self.sidebar.select_first();

        // Find and select the target workspace in the list
        for ws in self.workspaces.iter() {
            if ws.id == workspace_id {
                self.sidebar.select_by_id(&workspace_id);
                self.current_workspace_id = Some(workspace_id);
                self.navigating = false;
                self.navigating_level.clear();
                self.status = format!("Navigated to workspace: {}", ws.name);
                return;
            }
        }

        // Workspace not found in current list
        self.navigating = false;
        self.navigating_level.clear();
        self.status = "Workspace not found in current workspace list".to_string();
    }

    /// Navigate to a space by workspace + space ID
    pub(super) fn navigate_to_space(&mut self, workspace_id: String, space_id: String) {
        self.navigating = true;
        self.navigating_level = "space".to_string();

        // First ensure we're in the right workspace
        self.current_workspace_id = Some(workspace_id.clone());
        self.screen = Screen::Spaces;
        self.current_space_id = None;
        self.current_folder_id = None;
        self.current_list_id = None;
        self.sidebar.select_first();

        // Find target space
        for (i, space) in self.spaces.iter().enumerate() {
            if space.id == space_id {
                self.sidebar.select(Some(i));
                self.current_space_id = Some(space_id);
                self.navigating = false;
                self.navigating_level.clear();
                self.status = format!("Navigated to space: {}", space.name);
                return;
            }
        }

        // Space not found — need to load spaces first
        self.load_spaces(workspace_id);
        self.status = "Loading spaces...".to_string();
    }

    /// Navigate to a folder by workspace + folder ID
    pub(super) fn navigate_to_folder(&mut self, workspace_id: String, folder_id: String) {
        self.navigating = true;
        self.navigating_level = "folder".to_string();

        self.current_workspace_id = Some(workspace_id.clone());
        self.current_space_id = None;
        self.current_folder_id = None;
        self.current_list_id = None;
        self.screen = Screen::Folders;
        self.sidebar.select_first();

        for (i, folder) in self.folders.iter().enumerate() {
            if folder.id == folder_id {
                self.sidebar.select(Some(i));
                self.current_folder_id = Some(folder_id);
                self.navigating = false;
                self.navigating_level.clear();
                self.status = format!("Navigated to folder: {}", folder.name);
                return;
            }
        }

        self.navigating = false;
        self.navigating_level.clear();
        self.status = "Folder not found in current folder list".to_string();
    }

    /// Navigate to a list by workspace + list ID
    pub(super) fn navigate_to_list(&mut self, workspace_id: String, list_id: String) {
        self.navigating = true;
        self.navigating_level = "list".to_string();

        self.current_workspace_id = Some(workspace_id);
        self.current_space_id = None;
        self.current_folder_id = None;
        self.current_list_id = None;
        self.screen = Screen::Lists;
        self.sidebar.select_first();

        for (i, list) in self.lists.iter().enumerate() {
            if list.id == list_id {
                self.sidebar.select(Some(i));
                self.current_list_id = Some(list_id);
                self.navigating = false;
                self.navigating_level.clear();
                self.status = format!("Navigated to list: {}", list.name);
                return;
            }
        }

        self.navigating = false;
        self.navigating_level.clear();
        self.status = "List not found in current list".to_string();
    }

    /// Navigate to a task by task ID (short-form URL)
    pub(super) fn navigate_to_task(&mut self, team_id: Option<String>, task_id: String) {
        self.navigating = true;
        self.navigating_level = "task".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.navigating = false;
                self.navigating_level.clear();
                self.status = "Not authenticated".to_string();
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        let target_task_id = task_id.clone();
        let target_team_id = team_id;
        let prev_screen = self.screen.clone();
        tokio::spawn(async move {
            // Fetch task from API to get its context
            let result = client
                .get_task_with_context(&target_task_id, target_team_id.as_deref())
                .await;
            let msg = match result {
                Ok(task) => AppMessage::TaskFetchedForNavigation(Ok(task), prev_screen),
                Err(e) => AppMessage::TaskFetchedForNavigation(Err(e.to_string()), prev_screen),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Navigate to a comment by task ID + comment ID
    pub(super) fn navigate_to_comment(&mut self, team_id: Option<String>, task_id: String, comment_id: String) {
        // First navigate to the task, then find the comment
        self.navigating = true;
        self.navigating_level = "comment".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.navigating = false;
                self.navigating_level.clear();
                self.status = "Not authenticated".to_string();
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        let target_task_id = task_id.clone();
        let target_team_id = team_id;
        let target_comment_id = comment_id.clone();
        let prev_screen = self.screen.clone();
        tokio::spawn(async move {
            let result = client
                .get_task_with_context(&target_task_id, target_team_id.as_deref())
                .await;
            let msg = match result {
                Ok(task) => AppMessage::CommentFetchedForNavigation(
                    Ok(task),
                    target_comment_id,
                    prev_screen,
                ),
                Err(e) => AppMessage::CommentFetchedForNavigation(
                    Err(e.to_string()),
                    target_comment_id,
                    prev_screen,
                ),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Navigate to a document by doc ID
    pub(super) fn navigate_to_document(&mut self, doc_id: String) {
        self.navigating = true;
        self.navigating_level = "document".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.navigating = false;
                self.navigating_level.clear();
                self.status = "Not authenticated".to_string();
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        let target_doc_id = doc_id.clone();
        let prev_screen = self.screen.clone();
        tokio::spawn(async move {
            // Search for the document by ID using filters
            use crate::models::DocumentFilters;
            let filters = DocumentFilters::default();
            let result = client.search_docs(&filters).await;
            match result {
                Ok(docs) => {
                    // Find the matching document
                    let doc = docs.iter().find(|d| d.id == target_doc_id);
                    if let Some(doc) = doc.cloned() {
                        let _ = tx
                            .send(AppMessage::DocumentFetchedForNavigation(
                                Ok(doc),
                                prev_screen,
                            ))
                            .await;
                    } else {
                        let _ = tx
                            .send(AppMessage::DocumentFetchedForNavigation(
                                Err("Document not found in search results".to_string()),
                                prev_screen,
                            ))
                            .await;
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(AppMessage::DocumentFetchedForNavigation(
                            Err(e.to_string()),
                            prev_screen,
                        ))
                        .await;
                }
            }
        });
    }
}
