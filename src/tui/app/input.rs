//! Keyboard dispatch for each screen (normal mode).

use super::*;

impl TuiApp {

    pub(super) fn handle_input(&mut self) -> Result<Option<InputEvent>> {
        use crossterm::event;

        if event::poll(Duration::from_millis(16))? {
            let evt = event::read()?;

            // Handle dialog input — return the key event so update() can process it
            // This keeps dialog confirmation testable through the public update() method
            if self.dialog.is_visible() {
                if let event::Event::Key(key) = evt {
                    match key.code {
                        KeyCode::Left | KeyCode::Right => {
                            self.dialog.toggle();
                            return Ok(Some(InputEvent::None));
                        }
                        KeyCode::Enter | KeyCode::Esc => {
                            // Pass through to update() for handling
                            return Ok(Some(InputEvent::Key(key)));
                        }
                        _ => return Ok(Some(InputEvent::None)),
                    }
                }
            }

            // Convert to InputEvent
            match evt {
                event::Event::Key(key) => Ok(Some(InputEvent::Key(key))),
                event::Event::Resize(_, _) => Ok(Some(InputEvent::Resize)),
                _ => Ok(Some(InputEvent::None)),
            }
        } else {
            Ok(None)
        }
    }

    /// Process input event and update state (public for testing)
    pub fn update(&mut self, event: InputEvent) {
        // Mention picker is a modal overlay (issue 6): highest priority
        if self.mention_picker_open {
            if let InputEvent::Key(key) = event {
                self.handle_mention_picker_input(key);
            }
            return;
        }

        // Vim-style insert mode (issue 7): while active, all printable keys go
        // into the buffer; normal-mode commands are not interpreted.
        if self.insert_mode_target.is_some() {
            if let InputEvent::Key(key) = event {
                self.handle_insert_mode(key);
            }
            return;
        }

        // Command menu overlay (SPACE, issue 7)
        if self.command_menu.is_some() {
            if let InputEvent::Key(key) = event {
                self.handle_command_menu_input(key);
            }
            return;
        }

        if let InputEvent::Key(key) = event {
            // Handle chord completion: if leader is pending, check for matching second key
            if let Some(leader) = self.chord_leader_pending.take() {
                if leader == KeyCode::Char('g') && key.code == KeyCode::Char('u') {
                    self.open_url_input_dialog();
                    return;
                }
                // Non-matching second key: pass through to normal handling below
                // (the key variable still holds the original KeyEvent)
            } else if key.code == KeyCode::Char('g') && !self.is_text_input_active() {
                // Set leader pending and wait for second key (only when not in text input)
                self.chord_leader_pending = Some(KeyCode::Char('g'));
                return;
            }

            // Handle Esc to clear chord pending state
            if key.code == KeyCode::Esc {
                self.chord_leader_pending = None;
                // Fall through to normal Esc handling
            }

            // Handle URL input dialog (modal overlay)
            // MUST check this BEFORE the 'u' key handler, otherwise pasted 'u' characters
            // from URLs will trigger copy_url() instead of being inserted into the input
            if self.url_input_open {
                self.handle_url_input(key);
                return;
            }

            // Handle filter menu input (modal overlay)
            if self.filter_menu_open {
                self.handle_filter_menu_input(key);
                return;
            }

            // Handle status picker input (modal overlay)
            if self.status_picker_open {
                self.handle_status_picker_input(key);
                return;
            }

            // Handle dialog confirmation (Enter/Esc) — must be first, before ANY other handler
            // so dialog takes priority over text input, screen handlers, etc.
            if self.dialog.is_visible() {
                match key.code {
                    KeyCode::Enter => {
                        if self.dialog.confirmed()
                            && matches!(self.dialog.dialog_type, Some(DialogType::ConfirmDelete))
                        {
                            self.delete_selected_task();
                        }
                        self.dialog.hide();
                        return;
                    }
                    KeyCode::Esc => {
                        self.pending_delete_task_id = None;
                        self.dialog.hide();
                        return;
                    }
                    _ => {}
                }
            }

            // Handle text input — delegate to appropriate handler when any text input is active
            // This guards global shortcuts like 'u' from interfering with typing
            if self.is_text_input_active() {
                self.handle_text_input(key);
                return;
            }

            // SPACE opens the command menu (Spacemacs-style)
            if key.code == KeyCode::Char(' ') && self.screen != Screen::Auth {
                self.open_command_menu();
                return;
            }
        }

        match self.screen {
            Screen::Auth => self.update_auth(event),
            Screen::Workspaces | Screen::Spaces | Screen::Folders | Screen::Lists => {
                self.update_navigation(event)
            }
            Screen::Inbox => self.update_inbox(event),
            Screen::Tasks | Screen::Assigned => self.update_tasks(event),
            Screen::TaskDetail => self.update_task_detail(event),
            Screen::Document => self.update_document(event),
        }
    }

    pub(super) fn update_auth(&mut self, event: InputEvent) {
        if let InputEvent::Key(key) = event {
            match key.code {
                KeyCode::Esc => {
                    self.auth_state.clear();
                }
                KeyCode::Enter if !self.auth_state.loading => {
                    self.authenticate();
                }
                KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Handle paste (Ctrl+V or Ctrl+Shift+V)
                    match Clipboard::new() {
                        Ok(mut clipboard) => {
                            match clipboard.get_text() {
                                Ok(text) => {
                                    // Insert clipboard content at cursor position
                                    crate::tui::text_edit::insert_str(
                                        &mut self.auth_state.token_input,
                                        &mut self.auth_state.cursor_pos,
                                        &text,
                                    );
                                    self.status = "Pasted from clipboard".to_string();
                                }
                                Err(_) => {
                                    self.status =
                                        "Paste failed: could not read clipboard".to_string();
                                }
                            }
                        }
                        Err(_) => {
                            self.status = "Paste failed: clipboard unavailable".to_string();
                        }
                    }
                }
                KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.auth_state.cursor_pos = 0;
                }
                KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.auth_state.cursor_pos = self.auth_state.token_input.chars().count();
                }
                _ => {
                    // Typing, cursor movement, Ctrl+W, Home/End, …
                    crate::tui::text_edit::apply_edit_key(
                        &mut self.auth_state.token_input,
                        &mut self.auth_state.cursor_pos,
                        &key,
                    );
                }
            }
        }
    }

    pub(super) fn update_navigation(&mut self, event: InputEvent) {
        if let InputEvent::Key(key) = event {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.sidebar.select_next();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.sidebar.select_previous();
                }
                KeyCode::Enter => {
                    self.navigate_into();
                }
                KeyCode::Esc => {
                    self.navigate_back();
                }
                KeyCode::Tab => {
                    self.sidebar.visible = !self.sidebar.visible;
                }
                _ => {}
            }
        }
    }

    /// Key handling for the inbox view.
    ///
    /// j/k move and Enter opens the task. Dismissing is deliberately *not* a
    /// bare key: it is `SPC i d`, so a stray keypress cannot hide an item from
    /// the inbox with no undo.
    pub(super) fn update_inbox(&mut self, event: InputEvent) {
        if let InputEvent::Key(key) = event {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    if self.inbox_selected + 1 < self.inbox_tasks.len() {
                        self.inbox_selected += 1;
                        self.mark_inbox_entry_read_current();
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if self.inbox_selected > 0 {
                        self.inbox_selected -= 1;
                        self.mark_inbox_entry_read_current();
                    }
                }
                KeyCode::Enter => self.open_selected_inbox_task(),
                KeyCode::Esc => self.navigate_back(),
                KeyCode::Char('r') => self.load_inbox(),
                _ => {}
            }
        }
    }

    /// Mark the highlighted entry read (moving the cursor counts as seeing it).
    pub(super) fn mark_inbox_entry_read_current(&mut self) {
        if let Some(task_id) = self.selected_inbox_task().map(|task| task.id.clone()) {
            self.mark_inbox_entry_read(&task_id);
        }
    }

    pub(super) fn update_tasks(&mut self, event: InputEvent) {
        if let InputEvent::Key(key) = event {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.task_list.select_next();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.task_list.select_previous();
                }
                KeyCode::PageDown => {
                    self.task_list.select_next_page();
                }
                KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.task_list.select_next_page();
                }
                KeyCode::PageUp => {
                    self.task_list.select_previous_page();
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.task_list.select_previous_page();
                }
                KeyCode::Enter => {
                    if let Some(task) = self.task_list.selected_task().cloned() {
                        self.task_detail.task = Some(task.clone());
                        self.reset_task_detail_view();
                        self.task_origin_screen = self.screen.clone();
                        self.screen = Screen::TaskDetail;
                        self.update_screen_title();
                        // Load comments for this task
                        self.load_comments(task.id.clone());
                    }
                }
                KeyCode::Esc => {
                    // From the root-level Assigned view, Esc returns to the
                    // workspace list; from a list's tasks it steps back too.
                    self.navigate_back();
                }
                KeyCode::Tab => {
                    // Focus status group of selected task / fold-unfold focused group
                    self.tab_task_list_focus();
                }
                _ => {}
            }
        }
    }

    pub(super) fn update_task_detail(&mut self, event: InputEvent) {
        // Handle assignee picker input first (takes priority)
        if self.assignee_picker_open {
            if let InputEvent::Key(key) = event {
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
                        // Toggle selection of current member
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
                        // Save assignees
                        self.save_assignees();
                    }
                    KeyCode::Esc => {
                        // Cancel
                        self.assignee_picker_open = false;
                        self.status = "Assignment cancelled".to_string();
                    }
                    _ => {}
                }
            }
            return;
        }

        // Handle task creation form input (takes priority over comment editing)
        if self.task_creating {
            if let InputEvent::Key(key) = event {
                match key.code {
                    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Validate and create task
                        if self.task_name_input.trim().is_empty() {
                            self.status = "Task name is required".to_string();
                            return;
                        }
                        if let Some(list_id) = &self.current_list_id {
                            self.create_task(list_id.clone());
                        } else {
                            self.status = "No list selected".to_string();
                        }
                        return;
                    }
                    KeyCode::Esc => {
                        // Cancel task creation
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
                    KeyCode::Tab => {
                        // Toggle focus between name and description
                        self.task_creation_focus = match self.task_creation_focus {
                            TaskCreationField::Name => TaskCreationField::Description,
                            TaskCreationField::Description => TaskCreationField::Name,
                        };
                        return;
                    }
                    _ => {
                        // Text editing (typing, cursor movement, word deletion, …)
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
                        return;
                    }
                }
            }
            return;
        }

        if let InputEvent::Key(key) = event {
            // Handle comment editing input
            if self.comment_editing_index.is_some() || !self.comment_new_text.is_empty() {
                match key.code {
                    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Save comment
                        if let Some(edit_idx) = self.comment_editing_index {
                            if edit_idx == usize::MAX {
                                // Creating new comment or reply (sentinel value)
                                if let Some(_task) = &self.task_detail.task {
                                    let text = self.comment_new_text.clone();
                                    // Validate text is not empty or whitespace-only
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

                                    // Determine parent_id based on view mode
                                    let parent_id = match &self.comment_view_mode {
                                        CommentViewMode::InThread {
                                            parent_comment_id, ..
                                        } => Some(parent_comment_id.clone()),
                                        CommentViewMode::TopLevel => None,
                                    };

                                    self.create_comment(task_id, text, parent_id);
                                }
                            } else {
                                // Update existing comment at index edit_idx
                                if let Some(_task) = &self.task_detail.task {
                                    let text = self.comment_new_text.clone();
                                    // Validate text is not empty or whitespace-only
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
                                            return;
                                        }
                                    }
                                }
                            }
                        } else if !self.comment_new_text.is_empty() {
                            // Fallback: Create new comment or reply (when editing_index is None but text exists)
                            if let Some(_task) = &self.task_detail.task {
                                let text = self.comment_new_text.clone();
                                // Validate text is not empty or whitespace-only
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

                                // Determine parent_id based on view mode
                                let parent_id = match &self.comment_view_mode {
                                    CommentViewMode::InThread {
                                        parent_comment_id, ..
                                    } => Some(parent_comment_id.clone()),
                                    CommentViewMode::TopLevel => None,
                                };

                                self.create_comment(task_id, text, parent_id);
                            }
                        }
                        return;
                    }
                    KeyCode::Esc => {
                        // Cancel editing
                        self.comment_new_text.clear();
                        self.comment_cursor = 0;
                        self.comment_editing_index = None;
                        self.status =
                            if matches!(self.comment_view_mode, CommentViewMode::InThread { .. }) {
                                "Reply cancelled".to_string()
                            } else {
                                "Comment editing cancelled".to_string()
                            };
                        return;
                    }
                    _ => {
                        // Text editing (typing, cursor movement, word deletion, …)
                        crate::tui::text_edit::apply_edit_key(
                            &mut self.comment_new_text,
                            &mut self.comment_cursor,
                            &key,
                        );
                        return;
                    }
                }
            }

            // Handle normal task detail and comment navigation
            match key.code {
                KeyCode::Esc => {
                    // Collapse an expanded panel first (issue 5)
                    if self.expanded_panel.is_some() {
                        self.expanded_panel = None;
                        self.status = "Panel restored".to_string();
                        return;
                    }
                    // Exit thread view if in thread, otherwise go back to tasks
                    if matches!(self.comment_view_mode, CommentViewMode::InThread { .. }) {
                        // Exit thread view and return to top-level
                        self.comment_view_mode = CommentViewMode::TopLevel;
                        self.comment_selected_index = self.comment_previous_selection.unwrap_or(0);
                        self.comment_previous_selection = None;
                        self.status = "Back to top-level comments".to_string();
                    } else if self.comment_editing_index.is_some()
                        || !self.comment_new_text.is_empty()
                    {
                        // Cancel comment editing
                        self.comment_new_text.clear();
                        self.comment_cursor = 0;
                        self.comment_editing_index = None;
                        self.status = "Comment editing cancelled".to_string();
                    } else {
                        // Exit task detail view, returning to whichever list
                        // the task was opened from (a list, or the
                        // workspace-wide Assigned view).
                        self.task_detail.editing = false;
                        self.screen = self.task_origin_screen.clone();
                        self.update_screen_title();
                    }
                }
                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Save task - not yet implemented
                    self.task_detail.editing = false;
                    self.status = "Save task - coming soon".to_string();
                }
                // Comment navigation
                KeyCode::Tab => {
                    // TAB focus traversal: Description → comment 1 → comment 2 → …
                    self.tab_task_screen_focus();
                }
                KeyCode::Up => {
                    self.scroll_task_screen_focus(-1);
                }
                KeyCode::Down => {
                    self.scroll_task_screen_focus(1);
                }
                KeyCode::PageDown => {
                    // Page down: scroll the focused element a page.
                    self.scroll_task_screen_focus(20);
                }
                KeyCode::PageUp => {
                    // Page up: scroll the focused element a page.
                    self.scroll_task_screen_focus(-20);
                }
                KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.scroll_task_screen_focus(20);
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.scroll_task_screen_focus(-20);
                }
                KeyCode::Char('j') if self.comment_focus => {
                    self.move_comment_selection(1);
                }
                KeyCode::Char('k') if self.comment_focus => {
                    self.move_comment_selection(-1);
                }
                // j/k scroll the description whenever it holds focus (split or
                // expanded layout), matching the Up/Down behaviour above. They
                // were previously dead keys outside the expanded panel.
                KeyCode::Char('j') => {
                    self.scroll_task_screen_focus(1);
                }
                KeyCode::Char('k') => {
                    self.scroll_task_screen_focus(-1);
                }
                KeyCode::Enter if self.comment_focus => {
                    // Enter thread view when a top-level comment is selected.
                    // Selection is remapped to the parent's index inside the
                    // filtered thread view, so the same value is reused.
                    let in_thread =
                        matches!(self.comment_view_mode, CommentViewMode::InThread { .. });
                    let selected = self
                        .comments
                        .get(self.comment_selected_index)
                        .filter(|comment| comment.parent_id.is_none());
                    if let Some(comment) = selected.filter(|_| !in_thread) {
                        let author = comment
                            .commenter
                            .as_ref()
                            .map(|c| c.username.clone())
                            .unwrap_or_else(|| "Unknown".to_string());
                        self.comment_view_mode = CommentViewMode::InThread {
                            parent_comment_id: comment.id.clone(),
                            parent_author: author,
                        };
                        self.comment_previous_selection = Some(self.comment_selected_index);
                        self.status = "Viewing thread. Press Esc to go back".to_string();
                    }
                }
                // Modal entry keys: 'n' starts a new comment (INPUT mode) when
                // comment focus is on; 'e' enters description INPUT mode otherwise.
                KeyCode::Char('n') if self.comment_focus => {
                    self.start_comment_compose(ComposeKind::New);
                }
                KeyCode::Char('e') if !self.comment_focus => {
                    if let Some(task) = self.task_detail.task.clone() {
                        self.description_edit_buffer = task
                            .description
                            .as_ref()
                            .map(|d| d.as_text())
                            .unwrap_or_default();
                        self.description_edit_cursor =
                            self.description_edit_buffer.chars().count();
                        self.insert_mode_target = Some(InsertTarget::Description);
                        self.status =
                            "INSERT mode: edit description, Ctrl+S save, Esc normal mode".to_string();
                    } else {
                        self.status = "No task selected".to_string();
                    }
                }
                _ => {}
            }
        }
    }

    /// Indices of comments visible in the current comment view, in display order.
    pub(super) fn comment_indices_in_view(&self) -> Vec<usize> {
        match &self.comment_view_mode {
            CommentViewMode::TopLevel => self
                .comments
                .iter()
                .enumerate()
                .filter(|(_, comment)| comment.parent_id.is_none())
                .map(|(index, _)| index)
                .collect(),
            CommentViewMode::InThread {
                parent_comment_id, ..
            } => {
                let mut indices = Vec::new();
                if let Some(parent_index) = self
                    .comments
                    .iter()
                    .position(|comment| &comment.id == parent_comment_id)
                {
                    indices.push(parent_index);
                }
                indices.extend(
                    self.comments
                        .iter()
                        .enumerate()
                        .filter(|(_, comment)| {
                            comment.parent_id.as_ref() == Some(parent_comment_id)
                        })
                        .map(|(index, _)| index),
                );
                indices
            }
        }
    }

    /// Move the comment selection within the currently visible comments.
    pub(super) fn move_comment_selection(&mut self, delta: isize) {
        let indices = self.comment_indices_in_view();
        if indices.is_empty() {
            return;
        }

        let current_position = indices
            .iter()
            .position(|index| *index == self.comment_selected_index);
        let next_position = match current_position {
            Some(position) => {
                (position as isize + delta).rem_euclid(indices.len() as isize) as usize
            }
            None => 0,
        };
        self.comment_selected_index = indices[next_position];
    }

    pub(super) fn update_document(&mut self, event: InputEvent) {
        if let InputEvent::Key(key) = event {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.document.scroll_down();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.document.scroll_up();
                }
                KeyCode::PageDown => {
                    // Page down: roughly a screenful of lines
                    for _ in 0..10 {
                        self.document.scroll_down();
                    }
                }
                KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    for _ in 0..10 {
                        self.document.scroll_down();
                    }
                }
                KeyCode::PageUp => {
                    // Page up: roughly a screenful of lines
                    for _ in 0..10 {
                        self.document.scroll_up();
                    }
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    for _ in 0..10 {
                        self.document.scroll_up();
                    }
                }
                KeyCode::Esc => {
                    self.navigate_back();
                }
                _ => {}
            }
        }
    }
}
