//! Inbox loading, read/dismiss tracking, and view entry.

use super::*;

/// Whether the Inbox entry is offered at the navigation root.
///
/// Currently `false`: the inbox entries are not useful yet, so the entry is
/// hidden rather than shown empty or misleading. Everything behind it (screen,
/// fetch, read/dismiss tracking, the contextual menu group) is still wired up
/// and tested, so flipping this back to `true` restores the feature without
/// touching the navigation code.
pub const INBOX_VISIBLE: bool = false;

impl TuiApp {

    /// Start (or restart) the inbox fetch for the current workspace.
    ///
    /// Treats failure as "inbox unavailable": the endpoint is undocumented, so
    /// a rejection must not look like an application error.
    pub(super) fn load_inbox(&mut self) {
        let team_id = match self.current_team_id.clone() {
            Some(id) => id,
            None => {
                self.status = "No workspace available for inbox".to_string();
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

        self.inbox_loading = true;

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client.get_inbox(&team_id).await;
            let msg = match result {
                Ok(inbox) => AppMessage::InboxLoaded(Ok(inbox)),
                Err(e) => AppMessage::InboxLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Apply the loaded inbox: drop dismissed entries, order by recency.
    pub(super) fn apply_inbox(&mut self, inbox: crate::models::Inbox) {
        // Locally dismissed entries are hidden; everything else is shown.
        let mut tasks: Vec<crate::models::InboxTask> = inbox
            .tasks
            .into_iter()
            .filter(|task| !self.inbox_dismissed_ids.contains(&task.id))
            .collect();

        // The API's own order is not by recency, so sort explicitly: most
        // recent activity first, matching the "what happened last" intention.
        tasks.sort_by(|a, b| {
            b.updated_at
                .unwrap_or(i64::MIN)
                .cmp(&a.updated_at.unwrap_or(i64::MIN))
                .then_with(|| a.id.cmp(&b.id))
        });

        self.inbox_tasks = tasks;
        if self.inbox_selected >= self.inbox_tasks.len() {
            self.inbox_selected = self.inbox_tasks.len().saturating_sub(1);
        }
    }

    /// The inbox entry currently highlighted.
    pub(super) fn selected_inbox_task(&self) -> Option<&crate::models::InboxTask> {
        self.inbox_tasks.get(self.inbox_selected)
    }

    /// Open the highlighted inbox entry in the task detail view.
    pub(super) fn open_selected_inbox_task(&mut self) {
        let task_id = match self.selected_inbox_task() {
            Some(task) => task.id.clone(),
            None => {
                self.status = "No inbox item selected".to_string();
                return;
            }
        };
        // Opening counts as reading, so the badge clears as the user works
        // through the list.
        self.mark_inbox_entry_read(&task_id);
        self.task_origin_screen = Screen::Inbox;
        let team_id = self.current_team_id.clone();
        self.navigate_to_task(team_id, task_id);
    }

    /// Record an inbox entry as read (locally).
    pub(super) fn mark_inbox_entry_read(&mut self, task_id: &str) {
        if self.inbox_read_ids.contains(task_id) {
            return;
        }
        self.inbox_read_ids.insert(task_id.to_string());
        if let Some(team_id) = self.current_team_id.clone() {
            let ids = vec![task_id.to_string()];
            if let Err(e) = self.cache.mark_inbox_read(&team_id, &ids, false) {
                tracing::warn!("Failed to persist inbox read state: {}", e);
            }
        }
        // The badge lives in the sidebar; refresh it.
        *self.sidebar.items_mut() = self.root_sidebar_items();
        self.sync_sidebar_selection_to_screen();
    }

    /// Dismiss the highlighted inbox entry (locally).
    ///
    /// The inbox endpoint is read-only — every dismiss route returns 404 — so
    /// dismissal is remembered in the cache and the entry is filtered out of
    /// future loads. It does not mark anything read in ClickUp itself.
    pub(super) fn dismiss_selected_inbox_task(&mut self) {
        let task_id = match self.selected_inbox_task() {
            Some(task) => task.id.clone(),
            None => {
                self.status = "No inbox item selected".to_string();
                return;
            }
        };

        self.inbox_dismissed_ids.insert(task_id.clone());
        if let Some(team_id) = self.current_team_id.clone() {
            let ids = vec![task_id.clone()];
            if let Err(e) = self.cache.mark_inbox_read(&team_id, &ids, true) {
                tracing::warn!("Failed to persist inbox dismissal: {}", e);
            }
        }

        self.inbox_tasks.retain(|task| task.id != task_id);
        if self.inbox_selected >= self.inbox_tasks.len() {
            self.inbox_selected = self.inbox_tasks.len().saturating_sub(1);
        }

        *self.sidebar.items_mut() = self.root_sidebar_items();
        self.sync_sidebar_selection_to_screen();
        self.status = format!(
            "Dismissed ({} left)",
            self.inbox_tasks.len()
        );
    }

    /// Keep the sidebar cursor on the entry for the current screen.
    pub(super) fn sync_sidebar_selection_to_screen(&mut self) {
        let wanted = match self.screen {
            Screen::Inbox => "inbox",
            Screen::Assigned => "assigned-to-me",
            Screen::Workspaces => "inbox",
            _ => return,
        };
        if self.screen == Screen::Workspaces {
            return; // selection belongs to the user while browsing the root
        }
        self.sidebar.select_by_id(wanted);
    }

    /// Enter the inbox view from the navigation root.
    pub(super) fn open_inbox_view(&mut self) {
        self.screen = Screen::Inbox;
        self.screen_title = generate_screen_title("Inbox");
        self.current_list_id = None;
        // Restore locally tracked state for this workspace, then fetch.
        if let Some(team_id) = self.current_team_id.clone() {
            match self.cache.load_inbox_state(&team_id) {
                Ok((read, dismissed)) => {
                    self.inbox_read_ids = read;
                    self.inbox_dismissed_ids = dismissed;
                }
                Err(e) => tracing::warn!("Failed to load inbox state: {}", e),
            }
        }
        self.inbox_loading = true;
        self.status = "Loading inbox...".to_string();
        self.load_inbox();
    }
}
