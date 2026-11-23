//! Moving between screens: hierarchy, inbox, and assigned views.

use super::*;

impl TuiApp {

    /// Navigate into the selected item (public for testing)
    /// Open the workspace-wide "Assigned to Me" view.
    ///
    /// Selecting the entry twice re-runs the query, which doubles as a
    /// cheap manual refresh from the root.
    pub(super) fn open_assigned_view(&mut self) {
        self.screen = Screen::Assigned;
        self.screen_title = generate_screen_title("Assigned to Me");
        // The hierarchy context no longer applies; clearing it keeps URL
        // copy and refresh from pointing at a list the user has left.
        self.current_list_id = None;
        self.load_assigned_tasks();
    }

    pub fn navigate_into(&mut self) {
        // Navigate based on current screen and selection
        // Clone the selected item to avoid borrow checker issues
        let selected_item = self.sidebar.selected_item().cloned();

        // Handle navigation based on current screen
        match &self.screen {
            Screen::Workspaces => {
                if let Some(SidebarItem::Inbox { .. }) = selected_item {
                    self.open_inbox_view();
                    return;
                }
                if let Some(SidebarItem::AssignedToMe) = selected_item {
                    self.open_assigned_view();
                    return;
                }
                if let Some(SidebarItem::Workspace { id, name }) = selected_item {
                    self.current_workspace_id = Some(id.clone());
                    self.current_space_id = None;
                    self.current_folder_id = None;
                    self.current_list_id = None;
                    self.load_spaces(id.clone());
                    self.screen = Screen::Spaces;
                    self.screen_title = generate_screen_title(&name);
                }
            }
            Screen::Spaces => {
                if let Some(SidebarItem::Space { id, name, .. }) = selected_item {
                    self.current_space_id = Some(id.clone());
                    self.current_folder_id = None;
                    self.current_list_id = None;
                    self.load_folders(id.clone());
                    self.screen = Screen::Folders;
                    self.screen_title = generate_screen_title(&name);
                }
            }
            Screen::Folders => {
                if let Some(SidebarItem::Folder { id, name, .. }) = selected_item {
                    self.current_folder_id = Some(id.clone());
                    self.current_list_id = None;
                    self.load_lists(id.clone());
                    self.screen = Screen::Lists;
                    self.screen_title = generate_screen_title(&name);
                }
            }
            Screen::Lists => {
                if let Some(SidebarItem::List { id, name, .. }) = selected_item {
                    self.current_list_id = Some(id.clone());
                    self.load_tasks(id.clone());
                    self.screen = Screen::Tasks;
                    self.screen_title = generate_screen_title(&format!("Tasks: {}", name));
                }
            }
            _ => {}
        }
    }

    /// Navigate back to previous screen (public for testing)
    pub fn navigate_back(&mut self) {
        match self.screen {
            Screen::Auth => {}       // Can't go back from auth
            Screen::Workspaces => {} // Can't go back from workspaces
            Screen::Inbox => {
                // Back to the root with the inbox entry still selected.
                self.screen = Screen::Workspaces;
                self.screen_title = generate_screen_title("Workspaces");
                *self.sidebar.items_mut() = self.root_sidebar_items();
                if !self.sidebar.select_by_id("inbox") {
                    self.sidebar.select_first();
                }
                self.status = "Back to workspaces".to_string();
            }
            Screen::Assigned => {
                // Return to the navigation root with the entry still selected.
                self.screen = Screen::Workspaces;
                self.screen_title = generate_screen_title("Workspaces");
                *self.sidebar.items_mut() = self.root_sidebar_items();
                if !self.sidebar.select_by_id("assigned-to-me") {
                    self.sidebar.select_first();
                }
                self.status = "Back to workspaces".to_string();
            }
            Screen::Spaces => {
                // Navigate back to Workspaces
                self.current_space_id = None;
                self.current_folder_id = None;
                self.current_list_id = None;

                // Repopulate with the full root sidebar (pinned Inbox and
                // Assigned entries included). Building this from the
                // workspace list alone silently dropped both shortcuts.
                *self.sidebar.items_mut() = self.root_sidebar_items();

                // Restore selection using current_workspace_id
                if let Some(ref workspace_id) = self.current_workspace_id {
                    if !self.sidebar.select_by_id(workspace_id) {
                        // Workspace not found (e.g., was deleted), fallback to first
                        self.sidebar.select_first();
                        self.status = "Saved workspace not found, showing workspaces".to_string();
                    }
                } else {
                    self.sidebar.select_first();
                }

                self.screen = Screen::Workspaces;
                self.screen_title = generate_screen_title("Workspaces");
            }
            Screen::Folders => {
                // Navigate back to Spaces
                self.current_folder_id = None;
                self.current_list_id = None;

                // Repopulate sidebar with spaces
                let mut items = Vec::new();
                items.extend(self.spaces.iter().map(|s| SidebarItem::Space {
                    name: s.name.clone(),
                    id: s.id.clone(),
                }));
                *self.sidebar.items_mut() = items;

                // Restore selection using current_space_id
                if let Some(ref space_id) = self.current_space_id {
                    if !self.sidebar.select_by_id(space_id) {
                        // Space not found, fallback to first
                        self.sidebar.select_first();
                        self.status = "Saved space not found, showing spaces".to_string();
                    }
                } else {
                    self.sidebar.select_first();
                }

                self.screen = Screen::Spaces;
                // Update title to show workspace name
                if let Some(ref workspace_id) = self.current_workspace_id {
                    if let Some(workspace) = self.workspaces.iter().find(|w| &w.id == workspace_id)
                    {
                        self.screen_title = generate_screen_title(&workspace.name);
                    } else {
                        self.screen_title = generate_screen_title("Workspaces");
                    }
                } else {
                    self.screen_title = generate_screen_title("Workspaces");
                }
            }
            Screen::Lists => {
                // Navigate back to Folders
                self.current_list_id = None;

                // Repopulate sidebar with folders
                let mut items = Vec::new();
                items.extend(self.folders.iter().map(|f| SidebarItem::Folder {
                    name: f.name.clone(),
                    id: f.id.clone(),
                }));
                *self.sidebar.items_mut() = items;

                // Restore selection using current_folder_id
                if let Some(ref folder_id) = self.current_folder_id {
                    if !self.sidebar.select_by_id(folder_id) {
                        // Folder not found, fallback to first
                        self.sidebar.select_first();
                        self.status = "Saved folder not found, showing folders".to_string();
                    }
                } else {
                    self.sidebar.select_first();
                }

                self.screen = Screen::Folders;
                // Update title to show space name
                if let Some(ref space_id) = self.current_space_id {
                    if let Some(space) = self.spaces.iter().find(|s| &s.id == space_id) {
                        self.screen_title = generate_screen_title(&space.name);
                    } else {
                        self.screen_title = generate_screen_title("Spaces");
                    }
                } else {
                    self.screen_title = generate_screen_title("Spaces");
                }
            }
            Screen::Tasks => {
                // Navigate back to Lists
                // Capture the list id before clearing: the Lists screen shows
                // the folder's lists, so sidebar focus must return to the list
                // the user just came from. Previously the id was cleared before
                // being read, so focus always fell back to the top.
                let restore_list_id = self.current_list_id.take();

                // Repopulate sidebar with lists
                let mut items = Vec::new();
                items.extend(self.lists.iter().map(|l| SidebarItem::List {
                    name: l.name.clone(),
                    id: l.id.clone(),
                }));
                *self.sidebar.items_mut() = items;

                // Restore selection to the previously opened list
                if let Some(ref list_id) = restore_list_id {
                    if !self.sidebar.select_by_id(list_id) {
                        // List not found, fallback to first
                        self.sidebar.select_first();
                        self.status = "Saved list not found, showing lists".to_string();
                    }
                } else {
                    self.sidebar.select_first();
                }

                self.screen = Screen::Lists;
                // Update title to show folder name
                if let Some(ref folder_id) = self.current_folder_id {
                    if let Some(folder) = self.folders.iter().find(|f| &f.id == folder_id) {
                        self.screen_title = generate_screen_title(&folder.name);
                    } else {
                        self.screen_title = generate_screen_title("Folders");
                    }
                } else {
                    self.screen_title = generate_screen_title("Folders");
                }
            }
            Screen::TaskDetail => {
                // Return to the list the task was opened from. The Assigned
                // view is workspace-wide, so going back there is right when
                // that is where the user came from.
                self.screen = self.task_origin_screen.clone();
                self.update_screen_title();
            }
            Screen::Document => {
                self.screen = Screen::Tasks;
                self.update_screen_title();
            }
        }
    }

    /// Build the navigation-root sidebar: the pinned "Assigned to Me" entry
    /// followed by every workspace.
    ///
    /// The Assigned entry is deliberately only added here: it is a shortcut to
    /// a workspace-wide query, so it makes no sense inside the hierarchy.
    pub(super) fn root_sidebar_items(&self) -> Vec<SidebarItem> {
        let mut items = vec![
            SidebarItem::Inbox {
                unread: self.inbox_unread_count(),
            },
            SidebarItem::AssignedToMe,
        ];
        items.extend(self.workspaces.iter().map(|w| SidebarItem::Workspace {
            name: w.name.clone(),
            id: w.id.clone(),
        }));
        items
    }

    /// Number of inbox entries the user has not seen yet.
    ///
    /// Counts entries that are neither read nor dismissed, which is what the
    /// sidebar badge shows.
    pub fn inbox_unread_count(&self) -> usize {
        self.inbox_tasks
            .iter()
            .filter(|task| !self.inbox_read_ids.contains(&task.id))
            .count()
    }
}
