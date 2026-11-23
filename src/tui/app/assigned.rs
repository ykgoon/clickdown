//! Workspace-wide "Assigned to Me" paging.

use super::*;

impl TuiApp {

    /// Start (or restart) the workspace-wide assigned-tasks fetch from page 0.
    ///
    /// Pages are fetched one at a time: ClickUp caps a page at 100 tasks and
    /// does not report the total, so the next page is requested only when the
    /// previous one came back full. That keeps a small workload at one request.
    pub fn load_assigned_tasks(&mut self) {
        if self.current_user_id.is_none() {
            self.status = "Waiting for your user profile...".to_string();
            return;
        }
        let team_id = match self.current_team_id.clone() {
            Some(id) => id,
            None => {
                self.status = "No workspace available for assigned tasks".to_string();
                return;
            }
        };
        self.tasks.clear();
        self.assigned_loading = true;
        self.assigned_page_in_flight = None;
        self.loading = true;
        self.status = "Loading assigned tasks...".to_string();
        self.fetch_assigned_page(team_id, 0);
    }

    /// Request one page of workspace-wide assigned tasks.
    pub(super) fn fetch_assigned_page(&mut self, team_id: String, page: u32) {
        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.assigned_loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };
        let user_id = match self.current_user_id {
            Some(id) => id,
            None => {
                self.loading = false;
                self.assigned_loading = false;
                return;
            }
        };

        // Carry every active status filter into the query. The assigned
        // endpoint accepts a repeated `statuses[]`, so filtering here is a
        // server-side operation just like in a single list.
        let statuses = if self.status_filters.is_empty() {
            None
        } else {
            Some(self.status_filters.clone())
        };

        self.assigned_page_in_flight = Some(page);
        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client
                .get_team_tasks_with_assignee(&team_id, user_id, page, statuses)
                .await;
            let msg = match result {
                Ok(tasks) => AppMessage::AssignedTasksLoaded(Ok(tasks), page),
                Err(e) => AppMessage::AssignedTasksLoaded(Err(e.to_string()), page),
            };
            let _ = tx.send(msg).await;
        });
    }
}
