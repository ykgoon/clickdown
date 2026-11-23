//! Assignee, status, and multi-select filter management.

use super::*;

impl TuiApp {

    /// Open the status picker for a task.
    ///
    /// The ClickUp API does not expose list statuses through the endpoints we use
    /// here, so derive the options from statuses already observed on loaded tasks.
    /// This still lets users pick any status actually in use in the list.
    /// Remember every status name currently visible, so the filter menu can
    /// still offer the full set after a filter has replaced `tasks` with a
    /// narrow subset. Only ever grows.
    pub(super) fn remember_status_names(&mut self) {
        for task in self.tasks.iter() {
            let Some(status) = &task.status else { continue };
            if !self
                .known_statuses
                .iter()
                .any(|known| known.status.eq_ignore_ascii_case(&status.status))
            {
                self.known_statuses.push(status.clone());
            }
        }
    }

    /// Status options for the list filter, ordered by the list's configured
    /// order when known, else by first sighting.
    pub(super) fn status_filter_options(&self) -> Vec<crate::models::TaskStatus> {
        use crate::models::TaskStatus;

        let mut ordered: Vec<TaskStatus> = Vec::new();
        // Configured order from the active list, when we have it.
        if let Some(list_id) = self.current_list_id.clone() {
            ordered = self.list_statuses_for(&list_id);
        }

        // Without a single list to scope to (the Assigned view), every
        // configuration already fetched is a plausible filter value.
        if self.current_list_id.is_none() {
            for cached in self.cached_list_statuses.values() {
                for status in cached {
                    if !ordered
                        .iter()
                        .any(|existing| existing.status.eq_ignore_ascii_case(&status.status))
                    {
                        ordered.push(status.clone());
                    }
                }
            }
        }

        for known in &self.known_statuses {
            if !ordered
                .iter()
                .any(|existing| existing.status.eq_ignore_ascii_case(&known.status))
            {
                ordered.push(known.clone());
            }
        }

        // Nothing known at all: fall back to the generic ClickUp defaults so
        // the menu is never empty.
        if ordered.is_empty() {
            ordered = vec![
                TaskStatus {
                    id: None,
                    status: "To Do".to_string(),
                    color: None,
                    type_field: None,
                    orderindex: Some(0),
                    status_group: None,
                },
                TaskStatus {
                    id: None,
                    status: "In Progress".to_string(),
                    color: None,
                    type_field: None,
                    orderindex: Some(1),
                    status_group: None,
                },
                TaskStatus {
                    id: None,
                    status: "Done".to_string(),
                    color: None,
                    type_field: None,
                    orderindex: Some(2),
                    status_group: None,
                },
            ];
        }

        // Configured order, then alphabetical for anything without an index.
        // Without this the order follows the order statuses were first seen,
        // which depends on task load order rather than the workspace config.
        ordered.sort_by(|a, b| {
            a.orderindex
                .unwrap_or(u32::MAX)
                .cmp(&b.orderindex.unwrap_or(u32::MAX))
                .then_with(|| a.status.to_lowercase().cmp(&b.status.to_lowercase()))
        });
        ordered
    }

    /// Open the filter menu (`SPC t f`).
    pub fn open_filter_menu(&mut self) {
        // Every task-list screen can filter by status: a single list queries
        // `statuses[]`, and the Assigned view's workspace-wide query accepts
        // the same parameter.
        self.filter_can_status = matches!(self.screen, Screen::Tasks | Screen::Assigned);
        self.remember_status_names();
        self.filter_menu_open = true;
        self.filter_menu_cursor = 0;
        self.status = "Filter: j/k navigate, Enter select, x clears all, Esc closes".to_string();
    }

    /// Keyboard handling for the filter menu overlay.
    pub(super) fn handle_filter_menu_input(&mut self, key: crossterm::event::KeyEvent) {
        use crate::tui::widgets::FilterMenuEntry;

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.filter_menu_cursor + 1 < FilterMenuEntry::ALL.len() {
                    self.filter_menu_cursor += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.filter_menu_cursor = self.filter_menu_cursor.saturating_sub(1);
            }
            KeyCode::Enter => self.apply_filter_menu_selection(),
            // Direct letters, as advertised in the menu itself.
            KeyCode::Char(c) => {
                if let Some(entry) = FilterMenuEntry::ALL
                    .iter()
                    .find(|entry| entry.key() == c)
                    .copied()
                {
                    self.activate_filter_entry(entry);
                }
            }
            KeyCode::Esc => self.close_filter_menu(),
            _ => {}
        }
    }

    /// Close the filter menu without changing anything.
    pub fn close_filter_menu(&mut self) {
        self.filter_menu_open = false;
        self.status = "Filter menu closed".to_string();
    }

    /// Activate the highlighted filter menu entry.
    pub fn apply_filter_menu_selection(&mut self) {
        let entry = crate::tui::widgets::FilterMenuEntry::ALL
            .get(self.filter_menu_cursor)
            .copied();
        if let Some(entry) = entry {
            self.activate_filter_entry(entry);
        }
    }

    /// Activate one filter entry by key or by menu selection.
    pub fn activate_filter_entry(&mut self, entry: crate::tui::widgets::FilterMenuEntry) {
        use crate::tui::widgets::FilterMenuEntry;

        match entry {
            FilterMenuEntry::AssignedToMe => self.toggle_assigned_filter(),
            FilterMenuEntry::Status => {
                self.filter_menu_open = false;
                self.open_status_filter_picker();
            }
            FilterMenuEntry::ClearAll => self.clear_all_filters(),
        }
    }

    /// Toggle the "Assigned to Me" narrow, reloading from the source so the
    /// result is not compounded on top of an earlier filtered response.
    pub(super) fn toggle_assigned_filter(&mut self) {
        let turning_on = !self.assigned_filter_active;
        self.assigned_filter_active = turning_on;
        self.filter_menu_open = false;
        self.update_screen_title();
        self.reload_current_task_list();
        self.status = if turning_on {
            "Filtering: Assigned to Me".to_string()
        } else {
            "Assigned to Me filter off".to_string()
        };
    }

    /// Drop every active filter and reload.
    pub fn clear_all_filters(&mut self) {
        let had_any = !self.status_filters.is_empty() || self.assigned_filter_active;
        self.status_filters.clear();
        self.assigned_filter_active = false;
        self.filter_menu_open = false;
        self.update_screen_title();
        self.reload_current_task_list();
        self.status = if had_any {
            "All filters cleared".to_string()
        } else {
            "No filters were active".to_string()
        };
    }

    /// Apply a saved query-style screen (`Assigned to Me`) once the data it
    /// needs exists. Safe to call repeatedly; it no-ops until the user id and
    /// a workspace are known, and clears the request when it succeeds.
    pub(super) fn try_restore_query_screen(&mut self) {
        if self.restored_screen != Some(Screen::Assigned) {
            return;
        }
        if self.current_user_id.is_none() || self.current_team_id.is_none() {
            // Needs the profile (assignee) and a workspace (team) first.
            return;
        }
        self.restored_screen = None;
        self.restoring_session = false;
        self.open_assigned_view();
        self.status = "Restored to Assigned to Me".to_string();
    }

    /// Whether any filter is currently narrowing the list.
    pub fn has_active_filters(&self) -> bool {
        !self.status_filters.is_empty() || self.assigned_filter_active
    }

    /// Reload the visible task list from its source (never from the already
    /// filtered rows), which is what makes filters removable.
    ///
    /// Before this existed, turning a filter off re-issued a request that
    /// still carried the other filter's parameters, so a filter could never
    /// be undone without leaving the screen.
    pub(super) fn reload_current_task_list(&mut self) {
        if self.screen == Screen::Assigned {
            self.load_assigned_tasks();
            return;
        }
        let Some(list_id) = self.current_list_id.clone() else {
            return;
        };
        if self.assigned_filter_active {
            self.load_tasks_with_assigned_filter(list_id);
        } else {
            self.load_tasks(list_id);
        }
    }

    /// Statuses configured for one list, from cache first, then the
    /// hierarchy if it carries this list's configuration.
    pub(super) fn list_statuses_for(&self, list_id: &str) -> Vec<crate::models::TaskStatus> {
        if let Some(cached) = self.cached_list_statuses.get(list_id) {
            if !cached.is_empty() {
                return cached.clone();
            }
        }
        self.lists
            .iter()
            .find(|list| list.id == list_id)
            .map(|list| list.statuses.clone())
            .unwrap_or_default()
    }

    /// Status options for a picker.
    ///
    /// Pickers must not derive their options from the currently displayed
    /// tasks alone: when the task list is filtered the result set holds a
    /// single status, which collapsed the change-status menu to one entry.
    ///
    /// When changing `task`, the task's own list configuration is
    /// authoritative, because ClickUp rejects a status that does not belong
    /// to the task's list; other lists are never offered. Only when that
    /// configuration is unknown does this widen to everything observed, so
    /// the menu is never a single entry while the real config is fetched.
    pub(super) fn collect_known_statuses(
        &self,
        task: Option<&crate::models::Task>,
    ) -> Vec<crate::models::TaskStatus> {
        use crate::models::TaskStatus;

        fn add(status: &TaskStatus, statuses: &mut Vec<TaskStatus>) {
            if !statuses
                .iter()
                .any(|existing| existing.status.eq_ignore_ascii_case(&status.status))
            {
                statuses.push(status.clone());
            }
        }

        fn sort(statuses: &mut [TaskStatus]) {
            statuses.sort_by(|a, b| {
                a.orderindex
                    .unwrap_or(u32::MAX)
                    .cmp(&b.orderindex.unwrap_or(u32::MAX))
                    .then_with(|| a.status.to_lowercase().cmp(&b.status.to_lowercase()))
            });
        }

        // Changing a task: use its list's configuration when we know it.
        if let Some(task) = task {
            let list_id = task
                .list
                .as_ref()
                .map(|list| list.id.clone())
                .or_else(|| self.current_list_id.clone());
            if let Some(list_id) = list_id {
                let mut scoped = self.list_statuses_for(&list_id);
                if !scoped.is_empty() {
                    // The task's present status must always be selectable.
                    if let Some(status) = &task.status {
                        add(status, &mut scoped);
                    }
                    sort(&mut scoped);
                    return scoped;
                }
            }
        }

        // Unknown configuration: widen to everything this app has seen.
        //
        // `known_statuses` is included because `tasks` holds the *filtered*
        // rows: once a status filter is active the loaded tasks are a single
        // status, so deriving the menu from them alone would collapse it to
        // one entry. The registry only ever grows and survives filtering.
        let mut statuses: Vec<TaskStatus> = Vec::new();
        for list in &self.lists {
            for status in &list.statuses {
                add(status, &mut statuses);
            }
        }
        for cached in self.cached_list_statuses.values() {
            for status in cached {
                add(status, &mut statuses);
            }
        }
        for known in &self.known_statuses {
            add(known, &mut statuses);
        }
        for loaded_task in self.tasks.iter() {
            if let Some(status) = &loaded_task.status {
                add(status, &mut statuses);
            }
        }
        if let Some(task) = task {
            if let Some(status) = &task.status {
                add(status, &mut statuses);
            }
        }

        // Last-resort defaults when nothing at all is known.
        if statuses.is_empty() {
            statuses = vec![
                TaskStatus {
                    id: None,
                    status: "To Do".to_string(),
                    color: Some("#8794a6".to_string()),
                    type_field: None,
                    orderindex: Some(0),
                    status_group: Some("todo".to_string()),
                },
                TaskStatus {
                    id: None,
                    status: "In Progress".to_string(),
                    color: Some("#4f46de".to_string()),
                    type_field: None,
                    orderindex: Some(1),
                    status_group: Some("in_progress".to_string()),
                },
                TaskStatus {
                    id: None,
                    status: "Done".to_string(),
                    color: Some("#0f4a58".to_string()),
                    type_field: None,
                    orderindex: Some(2),
                    status_group: Some("done".to_string()),
                },
            ];
        }

        sort(&mut statuses);
        statuses
    }

    /// List whose status configuration should be fetched for `task`, if any.
    ///
    /// The Assigned view has no navigation context, so fall back to the
    /// task's own list reference; otherwise the picker could never learn the
    /// full configuration for a task opened from a workspace-wide view.
    pub(super) fn status_source_list_id(&self, task: Option<&crate::models::Task>) -> Option<String> {
        self.current_list_id.clone().or_else(|| {
            task.and_then(|task| task.list.as_ref().map(|list| list.id.clone()))
        })
    }

    pub(super) fn open_status_picker(&mut self, task: crate::models::Task) {
        // Changing a task's status, not filtering the list
        self.status_picker_mode = StatusPickerMode::ChangeTask;

        // Store the task ID we're changing status for
        self.status_picker_task_id = Some(task.id.clone());

        // Store original status for potential rollback
        self.status_picker_original_status = task.status.clone();

        // Full known set, never just what the filtered list is showing.
        let statuses = self.collect_known_statuses(Some(&task));

        self.status_picker_cursor = task
            .status
            .as_ref()
            .and_then(|current| {
                statuses
                    .iter()
                    .position(|status| status.status.eq_ignore_ascii_case(&current.status))
            })
            .unwrap_or(0);
        self.status_picker_statuses = statuses;
        self.status_picker_open = true;

        // If we only know a subset of the list's statuses, fetch the full
        // configuration in the background and refresh the picker when it lands.
        // The Assigned view has no navigation context, so this resolves the
        // task's own list instead of silently skipping the fetch.
        if let Some(list_id) = self.status_source_list_id(Some(&task)) {
            if !self.cached_list_statuses.contains_key(&list_id) {
                if let Some(client) = self.client.clone() {
                    let tx = self.message_tx.clone().unwrap();
                    self.loading = true;
                    self.status = "Loading statuses...".to_string();
                    tokio::spawn(async move {
                        let result = client.get_list(&list_id).await;
                        let statuses = result.map(|l| l.statuses);
                        let msg = AppMessage::ListStatusesLoaded(
                            list_id,
                            statuses.map_err(|e| e.to_string()),
                        );
                        let _ = tx.send(msg).await;
                    });
                }
            }
        }
        self.status = "Select new status (j/k navigate, Enter select, Esc cancel)".to_string();
    }

    /// Open the status-filter picker for the current task list
    ///
    /// Seeds from `collect_known_statuses`, so a list already filtered down
    /// to one status still offers every status the workspace has shown. The
    /// full configuration is fetched in the background when only a subset is
    /// known. Selecting a status re-queries the list with `statuses[]` so
    /// filtering uses ClickUp's server-side filter.
    pub fn open_status_filter_picker(&mut self) {
        // Filtering the list, not changing a task. Clearing the task id is
        // what stops Enter from calling save_status() on the task the user
        // picked earlier with `SPC t s`.
        self.status_picker_mode = StatusPickerMode::FilterList;
        self.status_picker_task_id = None;
        self.status_picker_original_status = None;

        // Durable registry first: the visible rows may already be filtered
        // down to the status we are trying to change or remove.
        self.remember_status_names();
        let statuses = self.status_filter_options();

        // Preselect the currently active filters so the picker opens showing
        // exactly what is in force, and re-toggling them off is obvious.
        self.status_picker_selected = statuses
            .iter()
            .filter(|status| {
                self.status_filters
                    .iter()
                    .any(|active| active.eq_ignore_ascii_case(&status.status))
            })
            .map(|status| status.status.clone())
            .collect();
        self.status_picker_cursor = self
            .status_filters
            .first()
            .and_then(|current| {
                statuses
                    .iter()
                    .position(|status| status.status.eq_ignore_ascii_case(current))
            })
            .unwrap_or(0);
        self.status_picker_statuses = statuses;
        self.status_picker_open = true;

        {
            if let Some(list_id) = &self.current_list_id {
                if !self.cached_list_statuses.contains_key(list_id) {
                    if let Some(client) = self.client.clone() {
                        let list_id = list_id.clone();
                        let tx = self.message_tx.clone().unwrap();
                        self.loading = true;
                        self.status = "Loading statuses...".to_string();
                        tokio::spawn(async move {
                            let result = client.get_list(&list_id).await;
                            let statuses = result.map(|l| l.statuses);
                            let msg = AppMessage::ListStatusesLoaded(
                                list_id,
                                statuses.map_err(|e| e.to_string()),
                            );
                            let _ = tx.send(msg).await;
                        });
                    }
                }
            }
        }
        self.status =
            "Filter by status (j/k navigate, Space toggle, Enter apply, Esc close)".to_string();
    }

    /// Apply the statuses selected in the picker.
    ///
    /// Selecting none means "no status filter", which is how a status filter
    /// is removed from inside the picker.
    pub fn apply_status_filter(&mut self) {
        if self.status_picker_statuses.is_empty() {
            self.status_picker_open = false;
            self.status = "No statuses available".to_string();
            return;
        }

        // Forgiving default: Enter on a highlighted status with nothing
        // ticked applies that single status, so the common case stays one
        // keypress. Removing every status filter is the menu's `x`.
        if self.status_picker_selected.is_empty() {
            if let Some(status) = self.status_picker_statuses.get(self.status_picker_cursor) {
                self.status_picker_selected.push(status.status.clone());
            }
        }

        // Keep the picker's own order (configured order) rather than the
        // order the user happened to toggle them in.
        self.status_filters = self
            .status_picker_statuses
            .iter()
            .filter(|status| {
                self.status_picker_selected
                    .iter()
                    .any(|chosen| chosen.eq_ignore_ascii_case(&status.status))
            })
            .map(|status| status.status.clone())
            .collect();

        self.status_picker_open = false;
        self.update_screen_title();
        self.reload_current_task_list();

        self.status = if self.status_filters.is_empty() {
            "Status filters cleared".to_string()
        } else {
            format!("Filtering: {}", self.status_filters.join(", "))
        };
    }

    /// Handle keyboard input for status picker
    pub fn handle_status_picker_input(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.status_picker_cursor < self.status_picker_statuses.len().saturating_sub(1) {
                    self.status_picker_cursor += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.status_picker_cursor > 0 {
                    self.status_picker_cursor -= 1;
                }
            }
            // Space toggles the highlighted status in filter mode, so several
            // statuses can be shown at once.
            KeyCode::Char(' ') if self.status_picker_mode == StatusPickerMode::FilterList => {
                if let Some(status) = self.status_picker_statuses.get(self.status_picker_cursor) {
                    let name = status.status.clone();
                    if let Some(pos) = self
                        .status_picker_selected
                        .iter()
                        .position(|chosen| chosen.eq_ignore_ascii_case(&name))
                    {
                        self.status_picker_selected.remove(pos);
                    } else {
                        self.status_picker_selected.push(name);
                    }
                }
            }
            KeyCode::Enter => {
                if self.status_picker_mode == StatusPickerMode::FilterList {
                    self.apply_status_filter();
                } else {
                    self.save_status();
                }
            }
            KeyCode::Esc => {
                if self.status_picker_mode == StatusPickerMode::FilterList {
                    // Esc only closes the picker. It must not silently drop
                    // the active filter: removing filters is an explicit
                    // action ("Clear all filters" in the menu), otherwise Esc
                    // looks like a cancel but changes what is on screen.
                    self.status_picker_open = false;
                    self.status = "Filter unchanged".to_string();
                } else {
                    self.status_picker_open = false;
                    self.status = "Status change cancelled".to_string();
                }
            }
            _ => {}
        }
    }

    /// Save the selected status to the task
    pub(super) fn save_status(&mut self) {
        let task_id = match &self.status_picker_task_id {
            Some(id) => id.clone(),
            None => {
                self.status_picker_open = false;
                self.status = "No task selected".to_string();
                return;
            }
        };

        let new_status = match self.status_picker_statuses.get(self.status_picker_cursor) {
            Some(status) => status.clone(),
            None => {
                self.status_picker_open = false;
                self.status = "No status selected".to_string();
                return;
            }
        };

        // Optimistic UI update: update the task in self.tasks and rebuild grouped list
        for task in &mut self.tasks {
            if task.id == task_id {
                task.status = Some(new_status.clone());
                break;
            }
        }
        if let Some(detail_task) = &mut self.task_detail.task {
            if detail_task.id == task_id {
                detail_task.status = Some(new_status.clone());
            }
        }
        self.rebuild_task_list();

        // Make API call
        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.status_picker_open = false;
                self.status = "Not authenticated".to_string();
                return;
            }
        };

        self.loading = true;
        self.status = "Updating status...".to_string();

        let task_id_clone = task_id.clone();
        let new_status_clone = new_status.status.clone();
        let tx = self.message_tx.clone().unwrap();

        use crate::models::UpdateTaskRequest;
        let update = UpdateTaskRequest {
            name: None,
            description: None,
            markdown_description: None,
            status: Some(new_status_clone),
            priority: None,
            assignees: None,
            due_date: None,
        };

        tokio::spawn(async move {
            let result = client.update_task(&task_id_clone, &update).await;
            let msg = match result {
                Ok(task) => AppMessage::TaskStatusUpdated(Ok(task)),
                Err(e) => AppMessage::TaskStatusUpdated(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });

        self.status_picker_open = false;
    }
}
