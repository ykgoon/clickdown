//! Introspection and mutation hooks used by integration tests.
//!
//! These drive the app from outside the crate; they are not part of the
//! binary's own behaviour.

use super::*;

impl TuiApp {
    /// Get the current user ID (for testing)
    pub fn current_user_id(&self) -> Option<i32> {
        self.current_user_id
    }

    /// Get the message sender for testing (sending async messages)
    pub fn message_tx_for_testing(&self) -> mpsc::Sender<AppMessage> {
        self.message_tx.clone().expect("message_tx should be set")
    }

    /// Set the assigned filter active state (for testing)
    pub fn set_assigned_filter_active(&mut self, active: bool) {
        self.assigned_filter_active = active;
    }

    /// Set the current list ID (for testing)
    pub fn set_current_list_id(&mut self, list_id: Option<String>) {
        self.current_list_id = list_id;
    }

    /// Set the current user ID (for testing)
    pub fn set_current_team_id_for_test(&mut self, team_id: String) {
        self.current_team_id = Some(team_id);
    }

    pub fn set_current_user_id(&mut self, user_id: Option<i32>) {
        self.current_user_id = user_id;
    }

    /// Check if loading is in progress (for testing)
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Get the current status message (for testing)
    pub fn status_message(&self) -> &str {
        &self.status
    }

    /// Get the current screen title (for testing)
    pub fn screen_title_for_test(&self) -> &str {
        &self.screen_title
    }

    /// Check if the assignee picker is open (for testing)
    pub fn is_assignee_picker_open(&self) -> bool {
        self.assignee_picker_open
    }

    /// Check whether the loading flag is set (for testing)
    pub fn loading_true_for_test(&self) -> bool {
        self.loading
    }

    /// Insert mode active? (for testing)
    pub fn insert_mode_active_for_test(&self) -> bool {
        self.insert_mode_target.is_some()
    }

    /// Description insert buffer (for testing)
    pub fn description_buffer_for_test(&self) -> String {
        self.description_edit_buffer.clone()
    }

    /// Command menu open? (for testing)
    pub fn command_menu_open_for_test(&self) -> bool {
        self.command_menu.is_some()
    }

    /// Current command menu level (for testing)
    pub fn command_menu_level_for_test(&self) -> Option<CommandMenuLevel> {
        self.command_menu.clone()
    }

    /// URL input dialog open? (for testing)
    pub fn url_input_open_for_test(&self) -> bool {
        self.url_input_open
    }

    /// Test-only: set the auth token input as if typed on the auth screen.
    #[doc(hidden)]
    pub fn set_auth_token_for_test(&mut self, token: &str) {
        self.auth_state.token_input = token.to_string();
        self.auth_state.cursor_pos = token.chars().count();
    }

    /// Test-only: whether the last API failure flagged the token as invalid.
    #[doc(hidden)]
    pub fn auth_invalid_for_test(&self) -> bool {
        self.auth_invalid
    }

    /// Test-only: current auth-screen error text, if any.
    #[doc(hidden)]
    pub fn auth_error_for_test(&self) -> Option<&str> {
        self.auth_state.error.as_deref()
    }

    /// Expanded panel (for testing)
    pub fn expanded_panel_for_test(&self) -> Option<ExpandedPanel> {
        self.expanded_panel
    }

    pub fn mention_picker_open_for_test(&self) -> bool {
        self.mention_picker_open
    }

    pub fn set_mention_members_for_test(&mut self, members: Vec<User>) {
        self.mention_members = members;
    }

    pub fn status_picker_statuses_for_test(&self) -> &[crate::models::TaskStatus] {
        &self.status_picker_statuses
    }

    pub fn status_picker_cursor_for_test(&self) -> usize {
        self.status_picker_cursor
    }

    pub fn set_status_picker_cursor_for_test(&mut self, cursor: usize) {
        self.status_picker_cursor = cursor;
    }

    pub fn status_picker_mode_for_test(&self) -> StatusPickerMode {
        self.status_picker_mode
    }

    pub fn status_picker_task_id_for_test(&self) -> Option<&String> {
        self.status_picker_task_id.as_ref()
    }

    pub fn status_filter_for_test(&self) -> Option<String> {
        self.status_filters.first().cloned()
    }

    /// All active status filters, in selection order
    pub fn status_filters_for_test(&self) -> Vec<String> {
        self.status_filters.clone()
    }

    pub fn set_status_filters_for_test(&mut self, statuses: Vec<String>) {
        self.status_filters = statuses;
    }

    pub fn filter_menu_open_for_test(&self) -> bool {
        self.filter_menu_open
    }

    pub fn filter_menu_cursor_for_test(&self) -> usize {
        self.filter_menu_cursor
    }

    pub fn filter_can_status_for_test(&self) -> bool {
        self.filter_can_status
    }

    pub fn assigned_filter_active_for_test(&self) -> bool {
        self.assigned_filter_active
    }

    pub fn has_active_filters_for_test(&self) -> bool {
        self.has_active_filters()
    }

    pub fn status_for_test(&self) -> String {
        self.status.clone()
    }

    pub fn status_picker_open_for_test(&self) -> bool {
        self.status_picker_open
    }

    pub fn remember_status_names_for_test(&mut self) {
        self.remember_status_names();
    }

    pub fn status_filter_options_for_test(&self) -> Vec<crate::models::TaskStatus> {
        self.status_filter_options()
    }

    pub fn set_status_filter_for_test(&mut self, status: Option<String>) {
        self.status_filters = status.into_iter().collect();
    }

    /// Seed the Assigned view. Rows live in `tasks`, which is the one place
    /// the list renders from; this rebuilds the grouped list and registers the
    /// statuses, exactly as the paged loader does.
    pub fn set_assigned_tasks_for_test(&mut self, tasks: Vec<crate::models::Task>) {
        self.tasks = tasks;
        self.remember_status_names();
        self.rebuild_task_list();
    }

    pub fn open_status_picker_for_test(&mut self, task: crate::models::Task) {
        self.open_status_picker(task);
    }

    pub fn set_task_detail_task_for_test(&mut self, task: crate::models::Task) {
        self.task_detail.task = Some(task);
    }

    pub fn open_mention_picker_for_test(&mut self) {
        self.open_mention_picker();
    }

    pub fn mention_members_for_test(&self) -> &[crate::models::User] {
        &self.mention_members
    }

    pub fn is_mention_picker_open(&self) -> bool {
        self.mention_picker_open
    }

    /// Push a message through the real channel so `process_async_messages`
    /// handles it exactly as it would at runtime. Tests must not re-implement
    /// the handler, or they would assert on a copy of the logic.
    pub(super) fn deliver_for_test(&mut self, msg: AppMessage) {
        self.message_tx
            .as_ref()
            .expect("message channel should exist")
            .try_send(msg)
            .expect("test channel should accept a message");
    }

    /// Enter restore mode for a saved screen name, as `restore_session_state`
    /// does after reading the cache. Separated so tests do not have to write
    /// and read a session through SQLite: each app instance owns its own
    /// temporary database, so a cross-instance round trip cannot work.
    pub fn restore_saved_screen_for_test(&mut self, screen_name: &str) {
        self.restoring_session = true;
        self.restored_screen = match screen_name {
            "Assigned" => Some(Screen::Assigned),
            _ => None,
        };
    }

    /// Deliver a synthetic `WorkspacesLoaded`, as startup does.
    pub fn simulate_workspaces_loaded_for_test(&mut self, workspaces: Vec<crate::models::Workspace>) {
        self.deliver_for_test(AppMessage::WorkspacesLoaded(Ok(workspaces)));
    }

    /// Deliver a synthetic `CurrentUserLoaded`, as startup does.
    pub fn simulate_user_loaded_for_test(&mut self, user: crate::models::User) {
        self.deliver_for_test(AppMessage::CurrentUserLoaded(Ok(user)));
    }

    /// Render the whole UI to a string (for asserting on what the user sees).
    pub fn render_to_string_for_test(&mut self, width: u16, height: u16) -> String {
        use ratatui::{backend::TestBackend, Terminal};

        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| self.draw_ui(frame))
            .expect("draw should succeed");

        let buffer = terminal.backend().buffer();
        let mut out = String::new();
        for y in 0..height {
            for x in 0..width {
                out.push_str(buffer[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    pub fn tasks_for_test(&self) -> &[Task] {
        &self.tasks
    }

    /// Seed the row store. Mirrors a list load: the status registry is kept
    /// growing, so a later status filter does not collapse the picker to the
    /// statuses that survived filtering.
    pub fn set_tasks_for_test(&mut self, tasks: Vec<Task>) {
        self.tasks = tasks;
        self.remember_status_names();
    }

    pub fn set_cached_list_statuses_for_test(&mut self, list_id: &str, statuses: Vec<crate::models::TaskStatus>) {
        self.cached_list_statuses
            .insert(list_id.to_string(), statuses);
    }

    /// Set the cached list members (for testing)
    pub fn set_cached_list_members(&mut self, list_id: &str, members: Vec<User>) {
        self.cached_list_members
            .insert(list_id.to_string(), members);
    }

    /// Check if URL input dialog is open (for testing)
    pub fn is_url_input_open(&self) -> bool {
        self.url_input_open
    }

    /// Get the URL input text (for testing)
    pub fn url_input_text(&self) -> &str {
        &self.url_input_text
    }

    /// Whether the app has been asked to quit (for testing)
    pub fn is_quitting_for_test(&self) -> bool {
        self.state == AppState::Quitting
    }

    /// Check if comment editing is active (for testing)
    pub fn is_comment_editing_active(&self) -> bool {
        self.comment_editing_index.is_some()
    }

    /// Check if task creation is active (for testing)
    pub fn is_task_creating(&self) -> bool {
        self.task_creating
    }

    /// Get the comment new text (for testing)
    pub fn comment_new_text(&self) -> &str {
        &self.comment_new_text
    }

    /// Get the task name input (for testing)
    pub fn task_name_input(&self) -> &str {
        &self.task_name_input
    }

    /// Get the task description input (for testing)
    pub fn task_description_input(&self) -> &str {
        &self.task_description_input
    }

    /// Get the task creation focus field (for testing)
    pub fn task_creation_focus(&self) -> &TaskCreationField {
        &self.task_creation_focus
    }

    /// Set task creating state (for testing)
    pub fn set_task_creating(&mut self, creating: bool) {
        self.task_creating = creating;
        self.task_detail.creating = creating;
    }

    /// Set task creation focus (for testing)
    pub fn set_task_creation_focus(&mut self, focus: TaskCreationField) {
        self.task_creation_focus = focus;
    }

    /// Set task detail task (for testing)
    pub fn set_task_detail_task(&mut self, task: Task) {
        self.task_detail.task = Some(task);
    }

    /// Set comments (for testing)
    pub fn set_comments(&mut self, comments: Vec<crate::models::Comment>) {
        self.comments = comments;
    }

    /// Read the reactions recorded on a comment (for testing)
    pub fn reactions_for_test(&self, comment_id: &str) -> Vec<String> {
        self.comments
            .iter()
            .find(|c| c.id == comment_id)
            .map(|c| c.reactions.clone())
            .unwrap_or_default()
    }

    /// Get the loaded comments (for testing)
    pub fn comments_for_test(&self) -> &[crate::models::Comment] {
        &self.comments
    }

    /// Get the selected comment index (for testing)
    pub fn comment_selected_index_for_test(&self) -> usize {
        self.comment_selected_index
    }

    /// Set comment focus (for testing)
    pub fn set_comment_focus(&mut self, focus: bool) {
        self.comment_focus = focus;
    }

    /// Set the highlighted comment index (for testing)
    pub fn set_comment_selected_index_for_test(&mut self, index: usize) {
        self.comment_selected_index = index;
    }

    /// Transient copy feedback shown in the status bar (for testing)
    pub fn copy_status_for_test(&self) -> Option<String> {
        self.url_copy_status.clone()
    }

    /// The text `SPC y c` would put on the clipboard (for testing)
    pub fn focused_copy_text_for_test(&self) -> Option<String> {
        self.focused_content_to_copy().map(|(text, _)| text)
    }

    /// Get the description pane scroll offset (for testing)
    pub fn description_scroll_offset_for_test(&self) -> usize {
        self.description_scroll_offset
    }

    /// Set the description pane scroll offset (for testing)
    pub fn set_description_scroll_offset_for_test(&mut self, offset: usize) {
        self.description_scroll_offset = offset;
    }

    /// Get the URL input error message (for testing)
    pub fn url_input_error(&self) -> Option<&str> {
        self.url_input_error.as_deref()
    }

    /// Get the task list for testing
    pub fn task_list_for_test(&self) -> &crate::tui::widgets::GroupedTaskList {
        &self.task_list
    }

    /// Get mutable access to tasks for testing
    pub fn tasks_mut_for_test(&mut self) -> &mut Vec<Task> {
        &mut self.tasks
    }

    /// Get mutable access to task list for testing
    pub fn task_list_mut_for_test(&mut self) -> &mut crate::tui::widgets::GroupedTaskList {
        &mut self.task_list
    }

    /// Get mutable access to dialog for testing
    pub fn dialog_mut_for_test(&mut self) -> &mut crate::tui::widgets::DialogState {
        &mut self.dialog
    }

    /// Set screen for testing
    pub fn set_screen_for_test(&mut self, screen: Screen) {
        self.screen = screen;
    }

    /// Current screen (for testing)
    pub fn screen_for_test(&self) -> Screen {
        self.screen.clone()
    }

    /// Check if dialog is visible (for testing)
    pub fn is_dialog_visible(&self) -> bool {
        self.dialog.is_visible()
    }

    /// Check if dialog is confirmed (for testing)
    pub fn is_dialog_confirmed(&self) -> bool {
        self.dialog.confirmed()
    }

    /// Get task count (for testing)
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
    /// Get the current screen (public for testing)
    pub fn screen(&self) -> Screen {
        self.screen.clone()
    }

    /// Inbox entries currently listed (for testing)
    pub fn inbox_names_for_test(&self) -> &[crate::models::InboxTask] {
        &self.inbox_tasks
    }

    /// Number of inbox entries (for testing)
    pub fn inbox_len_for_test(&self) -> usize {
        self.inbox_tasks.len()
    }

    /// Open the inbox view directly, bypassing the root sidebar entry.
    ///
    /// The entry itself is hidden while `INBOX_VISIBLE` is `false`, but the
    /// screen behind it still ships and is still tested, so tests need a way in
    /// that does not depend on the (deliberately absent) sidebar row.
    pub fn open_inbox_for_test(&mut self) {
        self.open_inbox_view();
    }

    /// Sidebar access for tests that render it directly.
    pub fn sidebar(&mut self) -> &mut SidebarState {
        &mut self.sidebar
    }

    /// Task list access for tests that render it directly.
    pub fn task_list(&mut self) -> &mut GroupedTaskList {
        &mut self.task_list
    }

    /// TAB on the task list: focus the selected task's status header, then
    /// fold/unfold that header on the next press.
    pub(super) fn tab_task_list_focus(&mut self) {
        if let Some(group_name) = self.task_list.select_group_header_of_selected_task() {
            self.status = format!("Status group focused: {}", group_name);
            return;
        }
        match self.task_list.toggle_fold_of_selected_header() {
            Some((name, count, folded)) => {
                let suffix = if count == 1 { "" } else { "s" };
                self.status = if folded {
                    format!("Folded {} — {} task{} hidden", name, count, suffix)
                } else {
                    format!("Unfolded {} — {} task{} visible", name, count, suffix)
                };
            }
            None => {
                self.status = "No status group focused".to_string();
            }
        }
    }

    /// Rebuild the grouped list from `tasks`, preserving fold state and selection.
    pub(super) fn rebuild_task_list(&mut self) {
        let selected_id = self.task_list.selected_task().map(|t| t.id.clone());
        self.task_list = GroupedTaskList::from_tasks_with_folds(
            self.tasks.clone(),
            self.task_list.folded_groups().clone(),
        );
        if let Some(ref id) = selected_id {
            if !self
                .task_list
                .rows()
                .iter()
                .any(|r| matches!(r, crate::tui::widgets::ListRow::Task(t) if &t.id == id))
            {
                // Selected task no longer exists, select first
                self.task_list.select_first();
            }
        }
    }

    /// Public wrapper for testing
    pub fn rebuild_task_list_for_test(&mut self) {
        self.rebuild_task_list();
    }

    /// Get task detail (public for testing)
    pub fn task_detail(&mut self) -> &mut TaskDetailState {
        &mut self.task_detail
    }

    /// Get status message (public for testing)
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Get cache manager (public for testing)
    pub fn cache(&mut self) -> &mut crate::cache::CacheManager {
        &mut self.cache
    }

    /// Get current workspace ID (public for testing)
    pub fn current_workspace_id(&self) -> Option<&String> {
        self.current_workspace_id.as_ref()
    }

    /// Whether the status picker overlay is open (public for testing)
    pub fn is_status_picker_open(&self) -> bool {
        self.status_picker_open
    }

    /// Set screen directly (public for testing)
    pub fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
    }
}
