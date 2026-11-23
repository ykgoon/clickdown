//! Vim-style modal editing: INSERT-mode buffers, the SPACE command menu,
//! the @mention picker, and the per-field key handlers that feed them.

use super::*;

impl TuiApp {
    /// Check if any text input is currently active (comments, task creation, URL input, etc.)
    /// Used to guard global shortcuts like 'u' for URL copy
    pub fn is_text_input_active(&self) -> bool {
        self.url_input_open
            || self.status_picker_open
            || self.assignee_picker_open
            || self.task_creating
            || self.comment_editing_index.is_some()
            || !self.comment_new_text.is_empty()
            || self.insert_mode_target.is_some()
            || self.mention_picker_open
            || self.filter_menu_open
            || self.command_menu.is_some()
    }

    /// Handle text input when any text input field is active
    /// Delegates to the appropriate handler based on which input is active
    pub fn handle_text_input(&mut self, key: crossterm::event::KeyEvent) {
        if self.url_input_open {
            self.handle_url_input(key);
        } else if self.mention_picker_open {
            self.handle_mention_picker_input(key);
        } else if self.command_menu.is_some() {
            self.handle_command_menu_input(key);
        } else if self.insert_mode_target.is_some() {
            self.handle_insert_mode(key);
        } else if self.status_picker_open {
            self.handle_status_picker_input(key);
        } else if self.assignee_picker_open {
            self.handle_assignee_picker_input(key);
        } else if self.task_creating {
            self.handle_task_creation_input(key);
        } else if self.comment_editing_index.is_some() || !self.comment_new_text.is_empty() {
            self.handle_comment_input(key);
        }
    }

    // ==================== Vim-style modal input engine (issue 7) ====================

    /// Open the command menu overlay (SPACE, normal mode only)
    pub(super) fn open_command_menu(&mut self) {
        self.command_menu = Some(CommandMenuLevel::Root);
    }

    /// Current insert-mode buffer contents for the active target
    pub(super) fn insert_buffer(&self) -> String {
        match self.insert_mode_target.as_ref() {
            Some(InsertTarget::Description) => self.description_edit_buffer.clone(),
            Some(InsertTarget::Comment) => self.comment_new_text.clone(),
            None => String::new(),
        }
    }

    pub(super) fn insert_set_buffer(&mut self, value: String) {
        match self.insert_mode_target.as_ref() {
            Some(InsertTarget::Description) => self.description_edit_buffer = value,
            Some(InsertTarget::Comment) => self.comment_new_text = value,
            None => {}
        }
    }

    /// Cursor position (char index) for the active insert-mode buffer.
    pub(super) fn insert_cursor(&self) -> usize {
        match self.insert_mode_target.as_ref() {
            Some(InsertTarget::Description) => self.description_edit_cursor,
            Some(InsertTarget::Comment) => self.comment_cursor,
            _ => 0,
        }
    }

    pub(super) fn insert_set_cursor(&mut self, value: usize) {
        match self.insert_mode_target.as_ref() {
            Some(InsertTarget::Description) => self.description_edit_cursor = value,
            Some(InsertTarget::Comment) => self.comment_cursor = value,
            _ => {}
        }
    }

    /// Apply a plain text-editing key (cursor movement, word deletion, …)
    /// to the active insert-mode buffer. Returns true when consumed.
    pub(super) fn edit_insert_buffer(&mut self, key: &crossterm::event::KeyEvent) -> bool {
        match self.insert_mode_target.clone() {
            Some(InsertTarget::Description) => crate::tui::text_edit::apply_edit_key(
                &mut self.description_edit_buffer,
                &mut self.description_edit_cursor,
                key,
            ),
            Some(InsertTarget::Comment) => crate::tui::text_edit::apply_edit_key(
                &mut self.comment_new_text,
                &mut self.comment_cursor,
                key,
            ),
            _ => false,
        }
    }

    /// Handle keys while insert mode is active.
    /// Ctrl+S saves and exits; Esc exits to normal mode without saving;
    /// @ opens the mention picker (issue 6); everything else is text and is
    /// handled by the shared readline-style editing engine.
    pub(super) fn handle_insert_mode(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyModifiers;
        match key.code {
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.commit_insert_mode();
                return;
            }
            KeyCode::Esc => {
                // Leave insert mode without saving (vim-style)
                let target = self.insert_mode_target.take();
                match target {
                    Some(InsertTarget::Description) => {
                        self.description_edit_buffer.clear();
                        self.description_edit_cursor = 0;
                        self.status = "Description edit cancelled".to_string();
                    }
                    Some(InsertTarget::Comment) => {
                        self.cancel_comment_editing();
                    }
                    _ => {}
                }
                return;
            }
            KeyCode::Tab => {
                // Only meaningful in task creation: switch field
                if self.task_creating {
                    self.handle_task_creation_input(key);
                }
                return;
            }
            KeyCode::Char('@') if !key.modifiers.intersects(
                KeyModifiers::CONTROL | KeyModifiers::ALT,
            ) => {
                // Insert the '@' then open the mention picker (issue 6)
                let mut buf = self.insert_buffer();
                let mut cursor = self.insert_cursor();
                crate::tui::text_edit::insert_char(&mut buf, &mut cursor, '@');
                self.insert_set_buffer(buf);
                self.insert_set_cursor(cursor);
                // Filter characters typed in the picker never reach the
                // buffer, so record where this @ landed.
                self.mention_at = Some(cursor.saturating_sub(1));
                self.open_mention_picker();
                return;
            }
            KeyCode::Enter => {
                let mut buf = self.insert_buffer();
                let mut cursor = self.insert_cursor();
                crate::tui::text_edit::insert_char(&mut buf, &mut cursor, '\n');
                self.insert_set_buffer(buf);
                self.insert_set_cursor(cursor);
                return;
            }
            _ => {}
        }

        self.edit_insert_buffer(&key);
    }

    /// Cancel comment editing (normal-mode Esc path)
    pub(super) fn cancel_comment_editing(&mut self) {
        self.comment_new_text.clear();
        self.comment_cursor = 0;
        self.comment_editing_index = None;
        self.status = if matches!(self.comment_view_mode, CommentViewMode::InThread { .. }) {
            "Reply cancelled".to_string()
        } else {
            "Comment editing cancelled".to_string()
        };
    }

    /// Commit the active insert-mode buffer (Ctrl+S semantics)
    pub(super) fn commit_insert_mode(&mut self) {
        match self.insert_mode_target.take() {
            Some(InsertTarget::Description) => {
                self.save_description_edit();
            }
            Some(InsertTarget::Comment) => {
                // Reuse the existing comment save path
                self.save_comment_from_buffer();
            }
            None => {}
        }
    }

    /// Save the description edit buffer to the task
    pub(super) fn save_description_edit(&mut self) {
        let task = match &self.task_detail.task {
            Some(t) => t.clone(),
            None => {
                self.description_edit_buffer.clear();
                self.description_edit_cursor = 0;
                self.status = "No task selected".to_string();
                return;
            }
        };
        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.description_edit_buffer.clear();
                self.description_edit_cursor = 0;
                self.status = "Not authenticated".to_string();
                return;
            }
        };

        let new_text = self.description_edit_buffer.clone();
        if new_text.trim().is_empty() {
            self.description_edit_buffer.clear();
            self.description_edit_cursor = 0;
            self.status = "Description cannot be empty".to_string();
            return;
        }

        let task_id = task.id.clone();
        let tx = self.message_tx.clone().unwrap();
        use crate::models::UpdateTaskRequest;
        // Descriptions only carry real user tags in markdown form:
        // `[@Name](#user_mention#<id>)`. Plain text would show up as literal
        // "@Name" in the ClickUp web app.
        let (description, markdown_description) =
            if crate::tui::mention::contains_mention(&new_text, &self.mention_registry) {
                let md = crate::tui::mention::to_markdown(&new_text, &self.mention_registry);
                (None, Some(md))
            } else {
                (Some(new_text), None)
            };
        let update = UpdateTaskRequest {
            name: None,
            description,
            markdown_description,
            status: None,
            priority: None,
            assignees: None,
            due_date: None,
        };

        self.loading = true;
        self.status = "Saving description...".to_string();
        tokio::spawn(async move {
            let result = client.update_task(&task_id, &update).await;
            let msg = match result {
                Ok(task) => AppMessage::TaskDescriptionUpdated(Ok(task)),
                Err(e) => AppMessage::TaskDescriptionUpdated(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Save the comment buffer (create, reply, or edit) — extracted from the
    /// pre-existing Ctrl+S handler so insert mode can reuse it.
    pub(super) fn save_comment_from_buffer(&mut self) {
        let text = self.comment_new_text.clone();
        if text.trim().is_empty() {
            self.status = if matches!(self.comment_view_mode, CommentViewMode::InThread { .. }) {
                "Reply cannot be empty".to_string()
            } else {
                "Comment cannot be empty".to_string()
            };
            return;
        }
        if let Some(_task) = &self.task_detail.task {
            let task_id = _task.id.clone();
            let parent_id = match &self.comment_view_mode {
                CommentViewMode::InThread {
                    parent_comment_id, ..
                } => Some(parent_comment_id.clone()),
                CommentViewMode::TopLevel => None,
            };
            if let Some(edit_idx) = self.comment_editing_index {
                if edit_idx == usize::MAX {
                    self.create_comment(task_id, text, parent_id);
                } else if let Some(comment_id) =
                    self.comments.get(edit_idx).map(|c| c.id.clone())
                {
                    self.update_comment(comment_id, text);
                } else {
                    self.comment_new_text.clear();
                    self.comment_cursor = 0;
                    self.comment_editing_index = None;
                    self.status = "Comment no longer exists".to_string();
                }
            } else {
                self.create_comment(task_id, text, parent_id);
            }
        }
    }

    // ==================== Command menu overlay (SPACE, issue 7) ====================

    /// Toggle an expanded panel (menu `t v` / `c v`).
/// If the same panel is already expanded it is restored; otherwise it replaces any current expansion.
pub fn toggle_expanded_panel(&mut self, panel: ExpandedPanel) {
    if self.expanded_panel == Some(panel) {
        self.expanded_panel = None;
        self.status = "Panel restored".to_string();
    } else {
        self.expanded_panel = Some(panel);
        self.status = if panel == ExpandedPanel::Description {
            "Description expanded (v restore, j/k scroll)".to_string()
        } else {
            "Comments expanded (v restore, j/k scroll)".to_string()
        };
    }
}

    /// Reset per-task view state when a task is opened in the detail screen:
    /// description scrolled back to the top, no expansion, description focused.
    /// Without this the scroll offset leaks across tasks, so a short
    /// description could open already scrolled past its end.
    pub(super) fn reset_task_detail_view(&mut self) {
        self.description_scroll_offset = 0;
        self.expanded_panel = None;
        self.comment_focus = false;
        self.comment_pane_scroll = 0;
    }

    /// Current TAB-focus element on the task screen.
    ///
    /// Legacy `comment_focus` bool remains the source of truth for the
    /// form/comments pane split; this derives the per-element focus from it
    /// plus the comment selection.
    pub(super) fn task_screen_focus(&self) -> TaskScreenFocus {
        if self.comment_focus && !self.comment_indices_in_view().is_empty() {
            TaskScreenFocus::Comment(self.comment_selected_index)
        } else {
            TaskScreenFocus::Description
        }
    }

    /// Advance TAB focus on the task screen:
    /// Description → first comment → next comment → … → back to Description.
    pub(super) fn tab_task_screen_focus(&mut self) {
        let indices = self.comment_indices_in_view();
        if indices.is_empty() {
            // No comments to focus; stay on the description.
            self.comment_focus = false;
            self.status = "Focus: Description (no comments)".to_string();
            return;
        }
        if !self.comment_focus {
            // Description → first comment
            self.comment_focus = true;
            self.comment_selected_index = indices[0];
            self.comment_pane_scroll = 0;
            self.status = "Focus: comment 1 (TAB next, ↑/↓ scroll)".to_string();
            return;
        }
        // Currently on a comment: advance to the next one, or wrap to description.
        let position = indices
            .iter()
            .position(|index| *index == self.comment_selected_index);
        match position {
            Some(position) if position + 1 < indices.len() => {
                self.comment_selected_index = indices[position + 1];
                self.comment_pane_scroll = 0;
                self.status = format!(
                    "Focus: comment {} (TAB next, ↑/↓ scroll)",
                    position + 2
                );
            }
            _ => {
                // Last comment (or stale selection): wrap to description.
                self.comment_focus = false;
                self.comment_pane_scroll = 0;
                self.status = "Focus: Description".to_string();
            }
        }
    }

    /// Scroll the focused task-screen element (description or focused comment
    /// pane) by `delta` lines. When the comment pane is focused this scrolls
    /// the whole comments pane so the focused comment stays visible.
    pub(super) fn scroll_task_screen_focus(&mut self, delta: isize) {
        // An expanded description panel owns the scroll keys: it fills the
        // screen, so nothing else can be the scroll target.
        if self.expanded_panel == Some(ExpandedPanel::Description) {
            if delta > 0 {
                self.description_scroll_down_n(delta as usize);
            } else {
                self.description_scroll_up_n((-delta) as usize);
            }
            return;
        }
        // Likewise an expanded comments panel owns the scroll keys.
        if self.expanded_panel == Some(ExpandedPanel::Comments) {
            let new_offset = if delta > 0 {
                self.comment_pane_scroll.saturating_add(delta as usize)
            } else {
                self.comment_pane_scroll.saturating_sub((-delta) as usize)
            };
            self.comment_pane_scroll = new_offset;
            return;
        }
        match self.task_screen_focus() {
            TaskScreenFocus::Description => {
                // The description scrolls with j/k, ↑/↓, PgUp/PgDn and
                // Ctrl+F/Ctrl+U whenever it holds focus, in both layouts.
                if delta > 0 {
                    self.description_scroll_down_n(delta as usize);
                } else {
                    self.description_scroll_up_n((-delta) as usize);
                }
            }
            TaskScreenFocus::Comment(_) => {
                let new_offset = if delta > 0 {
                    self.comment_pane_scroll.saturating_add(delta as usize)
                } else {
                    self.comment_pane_scroll.saturating_sub((-delta) as usize)
                };
                self.comment_pane_scroll = new_offset;
            }
        }
    }

    /// Scroll the description down by n lines.
    pub(super) fn description_scroll_down_n(&mut self, n: usize) {
        self.description_scroll_offset = self.description_scroll_offset.saturating_add(n);
    }

    /// Scroll the description up by n lines.
    pub(super) fn description_scroll_up_n(&mut self, n: usize) {
        self.description_scroll_offset = self.description_scroll_offset.saturating_sub(n);
    }

/// Start composing a comment/reply in INSERT mode (press SPACE, then `c n/e/r`).
pub(super) fn start_comment_compose(&mut self, kind: ComposeKind) {
    if self.screen != Screen::TaskDetail {
        self.status = "Comments live in task detail".to_string();
        return;
    }
    match kind {
        ComposeKind::New => {
            if !self.comment_focus {
                self.comment_focus = true;
            }
            self.comment_new_text.clear();
            self.comment_cursor = 0;
            // usize::MAX is a sentinel value indicating "new comment" mode
            self.comment_editing_index = Some(usize::MAX);
            self.insert_mode_target = Some(InsertTarget::Comment);
            self.status =
                "INSERT mode: type comment, Ctrl+S save, Esc to normal mode".to_string();
        }
        ComposeKind::Edit => {
            if !self.comment_focus {
                self.status = "Tab to comments, then press SPACE, 'c e' to edit".to_string();
                return;
            }
            if self.comments.is_empty()
                || self.comment_selected_index >= self.comments.len()
            {
                self.status = "No comment selected to edit".to_string();
                return;
            }
            let comment = &self.comments[self.comment_selected_index];
            self.comment_new_text = comment.text.clone();
            self.comment_cursor = self.comment_new_text.chars().count();
            self.comment_editing_index = Some(self.comment_selected_index);
            self.insert_mode_target = Some(InsertTarget::Comment);
            self.status =
                "INSERT mode: edit comment, Ctrl+S save, Esc normal mode".to_string();
        }
        ComposeKind::Reply => {
            if !matches!(self.comment_view_mode, CommentViewMode::InThread { .. }) {
                self.status =
                    "Enter a thread first (Enter on a comment), then press SPACE, 'c r'".to_string();
                return;
            }
            self.comment_new_text.clear();
            self.comment_cursor = 0;
            self.comment_editing_index = Some(usize::MAX);
            self.insert_mode_target = Some(InsertTarget::Comment);
            let author = match &self.comment_view_mode {
                CommentViewMode::InThread { parent_author, .. } => parent_author.clone(),
                _ => "Unknown".to_string(),
            };
            self.status = format!("INSERT reply to {}: Ctrl+S save, Esc normal mode", author);
        }
    }
}

pub(super) fn handle_command_menu_input(&mut self, key: crossterm::event::KeyEvent) {
        let level = match &self.command_menu {
            Some(l) => l.clone(),
            None => return,
        };
        match level {
            CommandMenuLevel::Root => match key.code {
                KeyCode::Esc => {
                    self.command_menu = None;
                    self.status = "Command menu closed".to_string();
                }
                KeyCode::Char(c) => {
                    match c {
                        'g' => self.command_menu = Some(CommandMenuLevel::Group('g')),
                        't' => self.command_menu = Some(CommandMenuLevel::Group('t')),
                        'c' => self.command_menu = Some(CommandMenuLevel::Group('c')),
                        'y' => self.command_menu = Some(CommandMenuLevel::Group('y')),
                        'w' => self.command_menu = Some(CommandMenuLevel::Group('w')),
                        // The Inbox group only exists on the inbox screen; the
                        // menu is contextual, so guard the key as well as the
                        // rendered entries.
                        'i' if self.screen == Screen::Inbox => {
                            self.command_menu = Some(CommandMenuLevel::Group('i'))
                        }
                        'r' => {
                            // `SPC r` -- re-query ClickUp for what is on screen
                            self.command_menu = None;
                            self.refresh_current_view();
                        }
                        'q' => {
                            // `SPC q` -- quit immediately (session is saved on exit)
                            self.command_menu = None;
                            self.quit();
                        }
                        _ => {
                            self.command_menu = None;
                            self.status = format!("No command group '{}'", c);
                        }
                    }
                }
                _ => {}
            },
            CommandMenuLevel::Group(group) => {
                let g = group;
                // Esc at a group level walks back to the root menu (Doom-style
                // menu hierarchy); any other key closes the menu first.
                if key.code == KeyCode::Esc {
                    self.command_menu = Some(CommandMenuLevel::Root);
                    return;
                }
                // Consume the key and close the menu regardless
                self.command_menu = None;
                match (g, key.code) {
                    ('g', KeyCode::Char('u')) => self.open_url_input_dialog(),
                    ('t', KeyCode::Char('n')) => {
                        // New task from tasks screen
                        self.task_name_input.clear();
                        self.task_name_cursor = 0;
                        self.task_description_input.clear();
                        self.task_description_cursor = 0;
                        self.task_creating = true;
                        self.task_creation_focus = TaskCreationField::Name;
                        self.task_detail.task = None;
                        self.task_detail.creating = true;
                        self.task_detail.editing = false;
                        // The comments pane below the form renders
                        // `self.comments`; without this reset it would show
                        // the comments of the task the user just left.
                        self.comments.clear();
                        self.comment_selected_index = 0;
                        self.comment_editing_index = None;
                        self.comment_new_text.clear();
                        self.comment_cursor = 0;
                        self.comment_focus = false;
                        self.comment_view_mode = CommentViewMode::TopLevel;
                        self.comment_pane_scroll = 0;
                        self.screen = Screen::TaskDetail;
                        self.screen_title = generate_screen_title("New Task");
                        self.status = "Enter task name (Ctrl+S to create, Esc to cancel)".to_string();
                    }
                    ('t', KeyCode::Char('d')) => {
                        // Delete current task
                        if self.screen == Screen::Tasks {
                            let task_id = self.task_list.selected_task().map(|t| t.id.clone());
                            self.open_delete_confirmation(task_id);
                        } else if self.screen == Screen::TaskDetail {
                            let task_id =
                                self.task_detail.task.as_ref().map(|t| t.id.clone());
                            self.open_delete_confirmation(task_id);
                        }
                    }
                    ('t', KeyCode::Char('s')) => {
                        // Status picker for current task
                        if self.screen == Screen::Tasks {
                            if let Some(task) = self.task_list.selected_task().cloned() {
                                self.open_status_picker(task);
                            }
                        } else if self.screen == Screen::TaskDetail {
                            if let Some(task) = &self.task_detail.task {
                                self.open_status_picker(task.clone());
                            }
                        }
                    }
                    ('t', KeyCode::Char('e')) => {
                        // Edit: tasks screen opens the task editor; task detail
                        // enters INSERT mode on the description
                        if self.screen == Screen::TaskDetail {
                            self.description_edit_buffer = self
                                .task_detail
                                .task
                                .as_ref()
                                .and_then(|t| t.description.as_ref().map(|d| d.as_text()))
                                .unwrap_or_default();
                            self.description_edit_cursor =
                                self.description_edit_buffer.chars().count();
                            self.insert_mode_target = Some(InsertTarget::Description);
                            self.status = "INSERT mode: edit description, Ctrl+S save, Esc normal mode".to_string();
                            return;
                        }
                        if self.screen == Screen::Tasks {
                            if let Some(task) = self.task_list.selected_task().cloned() {
                                self.task_detail.task = Some(task.clone());
                                self.reset_task_detail_view();
                                self.task_detail.editing = true;
                                self.task_detail.creating = false;
                                self.comments.clear();
                                self.comment_selected_index = 0;
                                self.comment_editing_index = None;
                                self.comment_new_text.clear();
                                self.comment_cursor = 0;
                                self.comment_focus = false;
                                self.comment_view_mode = CommentViewMode::TopLevel;
                                self.screen = Screen::TaskDetail;
                                self.update_screen_title();
                                self.load_comments(task.id);
                            }
                        }
                    }
                    ('t', KeyCode::Char('a')) => {
                        // Assign people to the current task
                        if self.screen == Screen::TaskDetail {
                            self.open_assignee_picker_flow();
                        } else {
                            self.status = "Assign works from task detail".to_string();
                        }
                    }
                    ('t', KeyCode::Char('f')) => {
                        // One entry point for every task-list filter
                        // (assigned-to-me, by status, clear all).
                        if self.screen == Screen::Tasks || self.screen == Screen::Assigned {
                            self.open_filter_menu();
                        } else {
                            self.status = "Filters work from the task list".to_string();
                        }
                    }
                    ('t', KeyCode::Char('v')) => {
                        // Expand/collapse the description panel
                        if self.screen == Screen::TaskDetail {
                            self.toggle_expanded_panel(ExpandedPanel::Description);
                        }
                    }
                    // `SPC i d` -- dismiss the highlighted inbox item. This is
                    // the only way to dismiss: there is no bare `d` binding, so
                    // a stray keypress cannot hide an item by accident.
                    ('i', KeyCode::Char('d')) => {
                        if self.screen == Screen::Inbox {
                            self.dismiss_selected_inbox_task();
                        } else {
                            self.status = "Dismiss works in the inbox".to_string();
                        }
                    }
                    ('c', KeyCode::Char('n')) => self.start_comment_compose(ComposeKind::New),
                    ('c', KeyCode::Char('u')) => self.toggle_comment_thumbs_up(),
                    ('c', KeyCode::Char('e')) => self.start_comment_compose(ComposeKind::Edit),
                    ('c', KeyCode::Char('r')) => self.start_comment_compose(ComposeKind::Reply),
                    ('c', KeyCode::Char('v')) => {
                        // Expand/collapse the comments panel
                        if self.screen == Screen::TaskDetail {
                            self.toggle_expanded_panel(ExpandedPanel::Comments);
                        }
                    }
                    ('y', KeyCode::Char('u')) => {
                        tracing::debug!("Copy URL via command menu (SPC y u)");
                        self.copy_url();
                    }
                    ('y', KeyCode::Char('c')) => {
                        tracing::debug!("Copy focused content via command menu (SPC y c)");
                        self.copy_focused_content();
                    }
                    ('w', KeyCode::Char('s')) => {
                        // Toggle sidebar (also on screens where Tab folds groups)
                        self.sidebar.visible = !self.sidebar.visible;
                        self.status = if self.sidebar.visible {
                            "Sidebar shown".to_string()
                        } else {
                            "Sidebar hidden".to_string()
                        };
                    }
                    _ => {
                        self.status = format!("No command '{}{}'", g,
                            match key.code {
                                KeyCode::Char(c) => c.to_string(),
                                _ => format!("{:?}", key.code),
                            });
                    }
                }
            }
        }
    }

    // ==================== @mention picker (issue 6) ====================

    /// Open the mention picker; fetch members if not cached
    pub(super) fn open_mention_picker(&mut self) {
        // Remember where to return
        self.mention_prev_target = self.insert_mode_target.take();
        self.mention_cursor = 0;
        self.mention_query.clear();
        self.mention_picker_open = true;

        // A task opened from the workspace-wide Assigned view has no
        // `current_list_id` (the hierarchy context is gone), but the task
        // itself carries its list. Resolve from either so the picker is never
        // silently empty.
        let list_id = match self.resolve_task_list_id() {
            Some(id) => id,
            None => {
                self.status = "No list context for @mentions on this task".to_string();
                return;
            }
        };

        let have_members_for_this_list =
            !self.mention_members.is_empty() && self.mention_members_list_id.as_deref() == Some(list_id.as_str());

        // Reuse anything already fetched for this list.
        if let Some(cached) = self.cached_list_members.get(&list_id) {
            if !have_members_for_this_list {
                self.mention_members = cached.clone();
                self.mention_members_list_id = Some(list_id);
                self.seed_mention_registry();
            }
            return;
        }

        if have_members_for_this_list {
            return;
        }

        // Members came from another list: drop them so the picker cannot
        // offer people who cannot see this task.
        if self.mention_members_list_id.as_deref() != Some(list_id.as_str()) {
            self.mention_members.clear();
        }
        self.mention_members_list_id = Some(list_id.clone());

        let client = match &self.client {
            Some(c) => c.clone(),
            None => return,
        };
        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client.get_list_members(&list_id).await;
            let msg = match result {
                Ok(members) => AppMessage::MentionMembersLoaded(Ok(members)),
                Err(e) => AppMessage::MentionMembersLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// List id to fetch @mention members from.
    ///
    /// Prefers the navigation context, then the task's own list reference
    /// (the Assigned view clears the former), then the list a restored
    /// session pointed at.
    /// List id for operations that act on the open task (assignees,
    /// @mention members).
    ///
    /// A task opened from the workspace-wide Assigned view has no
    /// `current_list_id` because the hierarchy context is gone, but the task
    /// itself carries its list. Without the fallback, every such task refused
    /// to assign with "list context not available".
    pub(super) fn resolve_task_list_id(&self) -> Option<String> {
        if let Some(id) = &self.current_list_id {
            return Some(id.clone());
        }
        if let Some(task) = &self.task_detail.task {
            if let Some(list) = &task.list {
                return Some(list.id.clone());
            }
        }
        if let Some(id) = &self.restored_list_id {
            return Some(id.clone());
        }
        None
    }

    /// Register every known member as a mention target.
    pub(super) fn seed_mention_registry(&mut self) {
        for member in &self.mention_members {
            if let Some(entry) = crate::tui::mention::MentionEntry::from_user(member) {
                if !self.mention_registry.iter().any(|existing| existing.id == entry.id) {
                    self.mention_registry.push(entry);
                }
            }
        }
    }

    /// Handle keys while the mention picker overlay is open
    pub(super) fn handle_mention_picker_input(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyModifiers;
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.mention_cursor + 1 < self.mention_members.len() {
                    self.mention_cursor += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.mention_cursor = self.mention_cursor.saturating_sub(1);
            }
            KeyCode::Esc => {
                // Close without inserting; return to insert mode
                self.mention_picker_open = false;
                self.insert_mode_target = self.mention_prev_target.take();
                self.status = "Mention cancelled".to_string();
            }
            KeyCode::Enter => {
                self.insert_selected_mention();
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+S saves the buffer and closes everything
                self.mention_picker_open = false;
                self.insert_mode_target = self.mention_prev_target.take();
                self.commit_insert_mode();
            }
            KeyCode::Char(c) => {
                // Type-to-filter within the picker
                self.mention_query.push(c);
                self.mention_cursor = 0;
            }
            KeyCode::Backspace => {
                self.mention_query.pop();
            }
            _ => {}
        }
    }

    /// Insert the selected mention into the underlying buffer
    pub(super) fn insert_selected_mention(&mut self) {
        // Filtered member list, matching the rendered order
        let query = self.mention_query.to_lowercase();
        let candidates: Vec<&User> = self
            .mention_members
            .iter()
            .filter(|m| m.username.to_lowercase().contains(&query))
            .collect();

        if let Some(member) = candidates.get(self.mention_cursor) {
            let username = member.username.clone();
            // Splice over the '@' that opened the picker. Filter text typed
            // in the picker only ever went to `mention_query`, never to the
            // buffer, so no query suffix has to be stripped here.
            let buf = match self.mention_prev_target.as_ref() {
                Some(InsertTarget::Description) => self.description_edit_buffer.clone(),
                Some(InsertTarget::Comment) => self.comment_new_text.clone(),
                _ => String::new(),
            };
            let buf_len = buf.chars().count();
            let at = self.mention_at.unwrap_or(buf_len).min(buf_len);
            let before: String = buf.chars().take(at).collect();
            let after: String = buf.chars().skip(at + 1).collect();
            let insertion = format!("@{} ", username);
            let new_buf = format!("{before}{insertion}{after}");
            let new_cursor = before.chars().count() + insertion.chars().count();

            // Remember who this is, so saving can emit a real ClickUp tag.
            if let Some(entry) = crate::tui::mention::MentionEntry::from_user(member) {
                if !self
                    .mention_registry
                    .iter()
                    .any(|existing| existing.id == entry.id)
                {
                    self.mention_registry.push(entry);
                }
            }

            match self.mention_prev_target.as_ref() {
                Some(InsertTarget::Description) => {
                    self.description_edit_buffer = new_buf;
                    self.description_edit_cursor = new_cursor;
                }
                Some(InsertTarget::Comment) => {
                    self.comment_new_text = new_buf;
                    self.comment_cursor = new_cursor;
                }
                _ => {}
            }
            self.status = format!("Mentioned @{}", username);
        } else {
            self.status = "No matching member".to_string();
        }

        self.mention_at = None;
        // Return to insert mode
        self.mention_picker_open = false;
        self.insert_mode_target = self.mention_prev_target.take();
    }

    /// Handle keyboard input within the assignee picker
    pub(super) fn handle_assignee_picker_input(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        match key.code {
            KeyCode::Char('j') => {
                if self.assignee_picker_cursor
                    < self.assignee_picker_members.len().saturating_sub(1)
                {
                    self.assignee_picker_cursor += 1;
                }
            }
            KeyCode::Char('k') => {
                if self.assignee_picker_cursor > 0 {
                    self.assignee_picker_cursor -= 1;
                }
            }
            KeyCode::Char(' ') => {
                if let Some(member) = self
                    .assignee_picker_members
                    .get(self.assignee_picker_cursor)
                {
                    if self.assignee_picker_selected.contains(&member.id) {
                        self.assignee_picker_selected.remove(&member.id);
                    } else {
                        self.assignee_picker_selected.insert(member.id);
                    }
                }
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_assignees();
            }
            KeyCode::Esc => {
                self.assignee_picker_open = false;
                self.status = "Assignment cancelled".to_string();
            }
            _ => {}
        }
    }

    /// Handle keyboard input within task creation form
    pub(super) fn handle_task_creation_input(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        match key.code {
            KeyCode::Tab => {
                self.task_creation_focus = match self.task_creation_focus {
                    TaskCreationField::Name => TaskCreationField::Description,
                    TaskCreationField::Description => TaskCreationField::Name,
                };
                self.status = format!(
                    "Focus: {}",
                    match self.task_creation_focus {
                        TaskCreationField::Name => "Task Name",
                        TaskCreationField::Description => "Description",
                    }
                );
                return;
            }
            KeyCode::Esc => {
                self.task_name_input.clear();
                self.task_name_cursor = 0;
                self.task_description_input.clear();
                self.task_description_cursor = 0;
                self.task_creating = false;
                self.task_detail.creating = false;
                self.screen = Screen::Tasks;
                self.update_screen_title();
                self.status = "Task creation cancelled".to_string();
                return;
            }
            KeyCode::Enter => {
                if matches!(self.task_creation_focus, TaskCreationField::Name) {
                    self.task_creation_focus = TaskCreationField::Description;
                    self.status = "Focus: Description".to_string();
                }
                return;
            }
            _ => {}
        }
        if let KeyCode::Char('s') = key.code {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                if self.task_name_input.trim().is_empty() {
                    self.status = "Task name is required".to_string();
                    return;
                }
                if let Some(list_id) = &self.current_list_id {
                    self.create_task(list_id.clone());
                }
                return;
            }
        }
        match self.task_creation_focus {
            TaskCreationField::Name => {
                crate::tui::text_edit::apply_edit_key(
                    &mut self.task_name_input,
                    &mut self.task_name_cursor,
                    &key,
                );
            }
            TaskCreationField::Description => {
                crate::tui::text_edit::apply_edit_key(
                    &mut self.task_description_input,
                    &mut self.task_description_cursor,
                    &key,
                );
            }
        }
    }

    /// Handle keyboard input within comment editing
    pub(super) fn handle_comment_input(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        match key.code {
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(edit_idx) = self.comment_editing_index {
                    if edit_idx == usize::MAX {
                        if let Some(_task) = &self.task_detail.task {
                            let text = self.comment_new_text.clone();
                            if text.trim().is_empty() {
                                self.status = if matches!(
                                    self.comment_view_mode,
                                    CommentViewMode::InThread { .. }
                                ) {
                                    "Reply cannot be empty".to_string()
                                } else {
                                    "Comment cannot be empty".to_string()
                                };
                                return;
                            }
                            let task_id = _task.id.clone();
                            let parent_id = match &self.comment_view_mode {
                                CommentViewMode::InThread {
                                    parent_comment_id, ..
                                } => Some(parent_comment_id.clone()),
                                CommentViewMode::TopLevel => None,
                            };
                            self.create_comment(task_id, text, parent_id);
                        }
                    } else {
                        if let Some(_task) = &self.task_detail.task {
                            let text = self.comment_new_text.clone();
                            if text.trim().is_empty() {
                                self.status = "Comment cannot be empty".to_string();
                                return;
                            }
                            match self
                                .comments
                                .get(edit_idx)
                                .map(|comment| comment.id.clone())
                            {
                                Some(comment_id) => self.update_comment(comment_id, text),
                                None => {
                                    self.comment_new_text.clear();
                                    self.comment_cursor = 0;
                                    self.comment_editing_index = None;
                                    self.status = "Comment no longer exists".to_string();
                                }
                            }
                        }
                    }
                } else if !self.comment_new_text.is_empty() {
                    if let Some(_task) = &self.task_detail.task {
                        let text = self.comment_new_text.clone();
                        if text.trim().is_empty() {
                            self.status = if matches!(
                                self.comment_view_mode,
                                CommentViewMode::InThread { .. }
                            ) {
                                "Reply cannot be empty".to_string()
                            } else {
                                "Comment cannot be empty".to_string()
                            };
                            return;
                        }
                        let task_id = _task.id.clone();
                        let parent_id = match &self.comment_view_mode {
                            CommentViewMode::InThread {
                                parent_comment_id, ..
                            } => Some(parent_comment_id.clone()),
                            CommentViewMode::TopLevel => None,
                        };
                        self.create_comment(task_id, text, parent_id);
                    }
                }
            }
            KeyCode::Esc => {
                self.comment_new_text.clear();
                self.comment_cursor = 0;
                self.comment_editing_index = None;
                self.status = if matches!(self.comment_view_mode, CommentViewMode::InThread { .. })
                {
                    "Reply cancelled".to_string()
                } else {
                    "Comment editing cancelled".to_string()
                };
            }
            _ => {
                // Text editing (typing, cursor movement, word deletion, …)
                crate::tui::text_edit::apply_edit_key(
                    &mut self.comment_new_text,
                    &mut self.comment_cursor,
                    &key,
                );
            }
        }
    }
}
