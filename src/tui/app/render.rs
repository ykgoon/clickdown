//! Drawing the screen: layout, panes, and overlays.

use super::*;

impl TuiApp {

    pub(super) fn render(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        // Auto-clear URL copy status after 3 seconds
        if let Some(status_time) = self.url_copy_status_time {
            if status_time.elapsed() > std::time::Duration::from_secs(3) {
                self.url_copy_status = None;
                self.url_copy_status_time = None;
            }
        }

        terminal.draw(|frame: &mut Frame| {
            self.draw_ui(frame);
        })?;

        Ok(())
    }

    /// Draw the whole UI into `frame` (shared by the live loop and tests).
    pub fn draw_ui(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let layout = TuiLayout::new(area);

        // Render title bar
        layout.render_title(frame, &self.screen_title);

        // Check if terminal is too small
        if layout.too_small {
            layout.render_too_small_warning(frame);
            return;
        }

        // Render content area
        if self.sidebar.visible {
            let (sidebar_area, content_area) = layout.split_content(25);
            self.render_sidebar_content(frame, sidebar_area, content_area);
        } else {
            self.render_main_content(frame, layout.content_area);
        }

        // Render dialog if visible
        render_dialog(frame, &self.dialog, area);

        // Render assignee picker overlay if open
        if self.assignee_picker_open {
            render_assignee_picker(
                frame,
                area,
                &self.assignee_picker_members,
                &self.assignee_picker_selected,
                self.assignee_picker_cursor,
            );
        }

        // Render filter menu overlay if open
        if self.filter_menu_open {
            render_filter_menu(
                frame,
                area,
                self.assigned_filter_active,
                &self.status_filters,
                self.filter_menu_cursor,
            );
        }

        // Render status picker overlay if open
        if self.status_picker_open {
            let current_status = self.status_picker_task_id.as_ref().and_then(|task_id| {
                self.tasks
                    .iter()
                    .find(|t| &t.id == task_id)
                    .and_then(|t| t.status.as_ref().map(|s| s.status.as_str()))
            });
            render_status_picker(
                frame,
                area,
                &self.status_picker_statuses,
                self.status_picker_cursor,
                current_status,
                self.status_picker_mode == StatusPickerMode::FilterList,
                &self.status_picker_selected,
            );
        }

        // Render URL input dialog if open
        if self.url_input_open {
            self.render_url_input_dialog(frame, area);
        }

        // Render @mention picker overlay if open (issue 6)
        if self.mention_picker_open {
            render_mention_picker(
                frame,
                area,
                &self.mention_members,
                self.mention_cursor,
                &self.mention_query,
            );
        }

        // Render status line
        // Priority: error > navigating > url_copy_status > loading > regular status
        let status = if let Some(ref error) = self.error {
            error.clone()
        } else if self.navigating {
            format!("Loading... {}", self.navigating_level)
        } else if let Some(ref url_status) = self.url_copy_status {
            // Show URL copy status (takes priority over regular status)
            url_status.clone()
        } else if self.loading {
            "Loading...".to_string()
        } else {
            self.status.clone()
        };
        layout.render_status(frame, &status);

        // Render the command menu minibuffer LAST so it docks over the bottom
        // edge (including the status line), Doom echo-area style (issue 7).
        // Each level displays only the single key to press at that level.
        if self.command_menu.is_some() {
            let level = self.command_menu.clone().unwrap();
            render_command_menu(frame, area, &level, &self.screen);
        }
    }

    pub(super) fn render_sidebar_content(
        &mut self,
        frame: &mut Frame,
        sidebar_area: Rect,
        content_area: Rect,
    ) {
        render_sidebar(frame, &self.sidebar, sidebar_area);
        self.render_main_content(frame, content_area);
    }

    pub(super) fn render_main_content(&mut self, frame: &mut Frame, area: Rect) {
        match self.screen {
            Screen::Auth => render_auth(frame, &self.auth_state, area),
            Screen::Inbox => {
                let read_ids = self.inbox_read_ids.clone();
                render_inbox(
                    frame,
                    &self.inbox_tasks,
                    self.inbox_selected,
                    &read_ids,
                    self.inbox_loading,
                    area,
                )
            }
            Screen::Tasks | Screen::Assigned => {
                self.task_list.set_viewport_height(area.height.saturating_sub(2) as usize);
                // Active filters are drawn in the frame title and a bar above
                // the rows, so what is applied is always visible instead of
                // being inferred from a shorter list.
                let assigned_active = self.assigned_filter_active;
                let statuses = self.status_filters.clone();
                // While pages are still arriving the list stays interactive;
                // the status line carries the progress.
                render_task_list(
                    frame,
                    &self.task_list,
                    area,
                    false,
                    assigned_active,
                    &statuses,
                )
            }
            Screen::TaskDetail => {
                // Expanded panel takes the full screen (issue 5)
                let (task_detail_area, comments_area) = match self.expanded_panel {
                    Some(ExpandedPanel::Description) => (area, Rect::default()),
                    Some(ExpandedPanel::Comments) => (Rect::default(), area),
                    None => match self.task_screen_focus() {
                        // TAB focus enlargement: the focused element grows to
                        // take the majority share of the task screen so more
                        // of its content is visible.
                        TaskScreenFocus::Description => split_task_detail_focus_description(area),
                        TaskScreenFocus::Comment(_) => split_task_detail_focus_comments(area),
                    },
                };

                if task_detail_area.height > 0 {
                    // Render task detail (30% normally, 65% when description
                    // is focused, full screen when expanded)
                    let description_edit = if self.insert_mode_target
                        == Some(InsertTarget::Description)
                    {
                        Some((
                            self.description_edit_buffer.as_str(),
                            self.description_edit_cursor,
                        ))
                    } else {
                        None
                    };
                    render_task_detail_with_editor(
                        frame,
                        &self.task_detail,
                        task_detail_area,
                        &self.task_name_input,
                        &self.task_description_input,
                        &self.task_creation_focus,
                        self.expanded_panel == Some(ExpandedPanel::Description),
                        self.expanded_panel == Some(ExpandedPanel::Description)
                            || (self.expanded_panel.is_none()
                                && self.task_screen_focus() == TaskScreenFocus::Description),
                        &mut self.description_scroll_offset,
                        Some(self.task_name_cursor),
                        Some(self.task_description_cursor),
                        description_edit,
                    );
                }

                if comments_area.height > 0 {
                    // Render comments (70% normally, 65% when a comment is
                    // focused, full screen when expanded)
                    render_comments(
                        frame,
                        &self.comments,
                        self.comment_selected_index,
                        self.comment_editing_index,
                        &self.comment_new_text,
                        self.comment_cursor,
                        self.comment_focus,
                        comments_area,
                        &self.comment_view_mode,
                        self.comment_pane_scroll,
                    );
                }
            }
            Screen::Document => render_document(frame, &self.document, area),
            _ => {
                use ratatui::widgets::Paragraph;
                let placeholder = Paragraph::new(format!("Navigate to see {}", self.screen_title));
                frame.render_widget(placeholder, area);
            }
        }
    }

    /// Render the URL input dialog as a centered modal overlay
    pub(super) fn render_url_input_dialog(&self, frame: &mut Frame, area: Rect) {
        use ratatui::{
            style::{Modifier, Style},
            text::Span,
            widgets::{Block, Borders, Clear, Paragraph},
        };

        // Center the dialog: 60% width, ~12 rows tall
        let popup_layout = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Percentage(35),
                ratatui::layout::Constraint::Length(12),
                ratatui::layout::Constraint::Percentage(35),
            ])
            .split(area);

        let dialog_area = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Percentage(20),
                ratatui::layout::Constraint::Percentage(60),
                ratatui::layout::Constraint::Percentage(20),
            ])
            .split(popup_layout[1])[1];

        // Clear underlying content
        frame.render_widget(Clear, dialog_area);

        // Dialog border and background
        let block = Block::default()
            .title(" Navigate to URL ")
            .borders(Borders::ALL)
            .style(
                Style::default()
                    .bg(ratatui::style::Color::Rgb(30, 30, 46))
                    .fg(ratatui::style::Color::Rgb(137, 180, 250)),
            );
        frame.render_widget(block, dialog_area);

        let inner = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(2)
            .constraints([
                ratatui::layout::Constraint::Length(1), // prompt
                ratatui::layout::Constraint::Length(1), // spacer
                ratatui::layout::Constraint::Length(3), // input field
                ratatui::layout::Constraint::Length(2), // error message
                ratatui::layout::Constraint::Length(1), // spacer
                ratatui::layout::Constraint::Length(1), // hints
            ])
            .split(dialog_area);

        // Prompt
        let prompt = Paragraph::new("Enter a ClickUp URL:").style(
            Style::default()
                .fg(ratatui::style::Color::Rgb(205, 214, 244))
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(prompt, inner[0]);

        // Input field: a live caret that scrolls horizontally with the text
        let input_rect = inner[2];
        let input_style = Style::default().fg(ratatui::style::Color::Rgb(205, 214, 244));
        if self.url_input_text.is_empty() {
            frame.render_widget(
                Paragraph::new(Span::styled(
                    "https://app.clickup.com/...",
                    Style::default().fg(ratatui::style::Color::DarkGray),
                )),
                input_rect,
            );
        } else {
            crate::tui::text_edit::render_editable_line(
                frame,
                input_rect,
                &self.url_input_text,
                self.url_input_cursor,
                input_style,
            );
        }

        // Error message (if any)
        if let Some(ref error) = self.url_input_error {
            let error_para = Paragraph::new(Span::styled(
                format!("⚠ {}", error),
                Style::default().fg(ratatui::style::Color::Rgb(243, 139, 168)),
            ));
            frame.render_widget(error_para, inner[3]);
        }

        // Hints
        let hints = Paragraph::new("Enter: Navigate | Esc: Cancel | Ctrl+V: Paste")
            .style(Style::default().fg(ratatui::style::Color::DarkGray));
        frame.render_widget(hints, inner[5]);
    }

    /// Open the URL input dialog
    pub(super) fn open_url_input_dialog(&mut self) {
        // Guard: must be authenticated
        if self.state == AppState::Unauthenticated {
            self.status = "Please authenticate first".to_string();
            return;
        }
        self.url_input_open = true;
        self.url_input_text = String::new();
        self.url_input_error = None;
        self.url_input_cursor = 0;
    }

    /// Handle URL input dialog submission
    pub(super) fn submit_url_input(&mut self) {
        if self.url_input_text.trim().is_empty() {
            self.url_input_error = Some("Please enter a URL".to_string());
            return;
        }

        let url = self.url_input_text.clone();
        self.close_url_input_dialog();

        // Parse the URL
        use crate::utils::UrlParser;
        match UrlParser::parse(&url) {
            Ok(parsed) => {
                self.navigate_from_parsed_url(parsed);
            }
            Err(e) => {
                // Reopen dialog with error and restore the URL text
                self.url_input_open = true;
                self.url_input_text = url.clone();
                let error_msg = format!("Unrecognized ClickUp URL format: {}", e);
                self.url_input_error = Some(error_msg);
                self.url_input_cursor = url.chars().count();
            }
        }
    }

    /// Close the URL input dialog
    pub(super) fn close_url_input_dialog(&mut self) {
        self.url_input_open = false;
        self.url_input_text.clear();
        self.url_input_error = None;
        self.url_input_cursor = 0;
    }

    /// Handle keyboard input within the URL input dialog
    pub fn handle_url_input(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        match key.code {
            KeyCode::Esc => {
                self.close_url_input_dialog();
            }
            KeyCode::Enter => {
                self.submit_url_input();
            }
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Paste from clipboard
                use arboard::Clipboard;
                if let Ok(mut clipboard) = Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        crate::tui::text_edit::insert_str(
                            &mut self.url_input_text,
                            &mut self.url_input_cursor,
                            &text,
                        );
                        self.url_input_error = None;
                    }
                }
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.url_input_cursor = 0;
            }
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.url_input_cursor = self.url_input_text.chars().count();
            }
            _ => {
                // Typing, cursor movement, word deletion, Home/End, …
                if crate::tui::text_edit::apply_edit_key(
                    &mut self.url_input_text,
                    &mut self.url_input_cursor,
                    &key,
                ) {
                    self.url_input_error = None;
                }
            }
        }
    }
}
