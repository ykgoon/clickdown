//! Task detail widget

use crate::models::Task;
use crate::tui::app::TaskCreationField;
use crate::tui::layout::ScrollState;
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Task detail state
#[derive(Debug, Clone)]
pub struct TaskDetailState {
    pub task: Option<Task>,
    pub editing: bool,
    pub creating: bool,
    /// Scroll state for the description panel
    pub description_scroll: ScrollState,
}

impl TaskDetailState {
    pub fn new() -> Self {
        Self {
            task: None,
            editing: false,
            creating: false,
            description_scroll: ScrollState::new(),
        }
    }
}

impl Default for TaskDetailState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_task_detail(
    frame: &mut Frame,
    state: &TaskDetailState,
    area: Rect,
    // Task creation form data (only used when state.creating is true)
    task_name_input: &str,
    task_description_input: &str,
    task_creation_focus: &TaskCreationField,
) {
    let mut manual_scroll = 0usize;
    render_task_detail_expanded(
        frame,
        state,
        area,
        task_name_input,
        task_description_input,
        task_creation_focus,
        false,
        true,
        &mut manual_scroll,
    )
}

/// Extended variant: renders the detail view with an optional full-screen
/// expanded description panel (issue 5), a focus highlight for the
/// description, and a manual scroll offset (clamped in place to the content).
#[allow(clippy::too_many_arguments)]
pub fn render_task_detail_expanded(
    frame: &mut Frame,
    state: &TaskDetailState,
    area: Rect,
    // Task creation form data (only used when state.creating is true)
    task_name_input: &str,
    task_description_input: &str,
    task_creation_focus: &TaskCreationField,
    expanded: bool,
    description_focused: bool,
    manual_scroll: &mut usize,
) {
    render_task_detail_with_editor(
        frame,
        state,
        area,
        task_name_input,
        task_description_input,
        task_creation_focus,
        expanded,
        description_focused,
        manual_scroll,
        None,
        None,
        None,
    );
}

/// Same as [`render_task_detail_expanded`] but aware of the live text editors:
/// `description_edit` is `Some((buffer, cursor))` while the description is in
/// INSERT mode, and the task-creation cursors place the caret in the form.
#[allow(clippy::too_many_arguments)]
pub fn render_task_detail_with_editor(
    frame: &mut Frame,
    state: &TaskDetailState,
    area: Rect,
    task_name_input: &str,
    task_description_input: &str,
    task_creation_focus: &TaskCreationField,
    expanded: bool,
    description_focused: bool,
    manual_scroll: &mut usize,
    name_cursor: Option<usize>,
    description_input_cursor: Option<usize>,
    description_edit: Option<(&str, usize)>,
) {
    if state.creating {
        render_task_creation_form(
            frame,
            area,
            task_name_input,
            task_description_input,
            task_creation_focus,
            name_cursor,
            description_input_cursor,
        );
        return;
    }

    let block = Block::default()
        .title(" Task Detail ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Theme::BACKGROUND));

    frame.render_widget(&block, area);

    let inner_area = block.inner(area);

    // Split into task info and description with better ratio.
    // When the description is expanded (issue 5), the header rows collapse to
    // a single status line so the description is readable full-screen.
    let inner = if expanded {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // Status summary line
                Constraint::Min(2),    // Description (full screen)
            ])
            .split(inner_area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // Name
                Constraint::Length(1), // Status
                Constraint::Length(1), // Priority
                Constraint::Length(1), // Assignees
                Constraint::Min(2),    // Description (flexible space)
            ])
            .split(inner_area)
    };

    if let Some(task) = &state.task {
        if expanded {
            // Compact header: name + status on one line
            let status = task
                .status
                .as_ref()
                .map(|s| s.status.as_str())
                .unwrap_or("None");
            frame.render_widget(
                Paragraph::new(format!(
                    "{} [{}]  v: restore | j/k: scroll",
                    task.name, status
                )),
                inner[0],
            );
        } else {
            frame.render_widget(Paragraph::new(format!("Name: {}", task.name)), inner[0]);

            let status = task
                .status
                .as_ref()
                .map(|s| s.status.as_str())
                .unwrap_or("None");
            frame.render_widget(Paragraph::new(format!("Status: {}", status)), inner[1]);

            let priority = task
                .priority
                .as_ref()
                .map(|p| p.priority.as_str())
                .unwrap_or("None");
            frame.render_widget(Paragraph::new(format!("Priority: {}", priority)), inner[2]);

            // Render assignees
            let assignees_str = if task.assignees.is_empty() {
                "None".to_string()
            } else {
                task.assignees
                    .iter()
                    .map(|u| u.username.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            frame.render_widget(
                Paragraph::new(format!("Assignees: {}", assignees_str)),
                inner[3],
            );
        }

        let desc = task
            .description
            .as_ref()
            .map(|d| d.as_text())
            .unwrap_or_else(|| "No description".to_string());

        let desc_area = if expanded { inner[1] } else { inner[4] };

        // Calculate description content height for scroll state
        let available_height = desc_area.height as usize;
        let available_width = desc_area.width.saturating_sub(2) as usize; // Account for borders

        // Estimate content height by counting wrapped lines
        let content_height = estimate_wrapped_lines(&desc, available_width);

        // Update scroll state
        let mut scroll_state = state.description_scroll.clone();
        scroll_state.update(content_height, available_height);

        // The description is scrolled manually whenever it holds focus
        // (split or expanded view). The offset is clamped in place every
        // render, so it cannot drift past the end of the content.
        *manual_scroll = (*manual_scroll).min(scroll_state.max_offset);
        let effective_offset = *manual_scroll;

        let title = if expanded {
            " Description (expanded) "
        } else if description_focused && scroll_state.scrollable {
            // Discoverability: the pane can be scrolled with j/k while focused
            " Description (j/k scroll) "
        } else {
            " Description "
        };

        // Slight focus highlight: the focused pane keeps the bright primary
        // border with a bold title, while an unfocused pane dims to secondary
        // so the active element is obvious at a glance.
        let border_style = if description_focused {
            Style::default().fg(Theme::PRIMARY)
        } else {
            Style::default().fg(Theme::SECONDARY)
        };
        let title_style = if description_focused {
            border_style.add_modifier(Modifier::BOLD)
        } else {
            border_style
        };

        // Render description with text wrapping and scrolling. The text itself
        // brightens slightly when the pane holds focus.
        let text_style = if description_focused {
            Style::default().fg(Theme::TEXT)
        } else {
            Style::default().fg(Theme::TEXT_DIM)
        };

        // INSERT mode: show the live edit buffer with a visible caret that the
        // viewport follows, so long descriptions never hide the cursor.
        if let Some((buffer, cursor)) = description_edit {
            let edit_block = Block::default()
                .title(Span::styled(
                    " Description (INSERT — Ctrl+S save, Esc cancel) ",
                    Style::default()
                        .fg(Theme::WARNING)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Theme::WARNING));
            crate::tui::text_edit::render_editable(
                frame,
                desc_area,
                edit_block,
                buffer,
                cursor.min(buffer.chars().count()),
                Style::default().fg(Theme::TEXT),
            );
            return;
        }

        let desc_paragraph = Paragraph::new(desc)
            .block(
                Block::default()
                    .title(Span::styled(title, title_style))
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .style(text_style)
            .wrap(Wrap { trim: true })
            .scroll((effective_offset as u16, 0));

        frame.render_widget(desc_paragraph, desc_area);

        // Render scroll indicator if needed
        if scroll_state.scrollable {
            crate::tui::layout::render_scroll_indicator(
                frame,
                desc_area,
                content_height,
                effective_offset,
            );
        }
    } else {
        frame.render_widget(Paragraph::new("No task selected"), inner[0]);
    }

    if state.editing && !expanded {
        let edit_hint = Paragraph::new("Press Ctrl+S to save, Esc to cancel")
            .style(Style::default().fg(Theme::WARNING));
        frame.render_widget(edit_hint, inner[4]);
    }
}

/// Render the task creation form with name and description input fields.
///
/// When the cursors are supplied the fields render with a real terminal caret
/// and scroll to follow it; otherwise a block indicator is used (widget-only
/// snapshot tests call this without cursor state).
fn render_task_creation_form(
    frame: &mut Frame,
    area: Rect,
    name_input: &str,
    description_input: &str,
    focus: &TaskCreationField,
    name_cursor: Option<usize>,
    description_cursor: Option<usize>,
) {
    let block = Block::default()
        .title(" New Task ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Theme::BACKGROUND));

    frame.render_widget(&block, area);

    let inner_area = block.inner(area);

    // Layout: name, description, hint
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Name input (label + field + spacing)
            Constraint::Min(3),    // Description input (flexible)
            Constraint::Length(1), // Hint text
        ])
        .split(inner_area);

    let focused_style = Style::default()
        .fg(Theme::WARNING)
        .add_modifier(Modifier::BOLD);
    let idle_style = Style::default().fg(Theme::SECONDARY);
    let text_style = Style::default().fg(Theme::TEXT);

    // Name field
    let name_style = if *focus == TaskCreationField::Name {
        focused_style
    } else {
        idle_style
    };
    let name_block = Block::default()
        .title(" Name * ")
        .borders(Borders::ALL)
        .style(name_style);
    let name_area = name_block.inner(inner[0]);

    frame.render_widget(name_block, inner[0]);

    match (*focus == TaskCreationField::Name, name_cursor) {
        (true, Some(cursor)) => crate::tui::text_edit::render_editable_area(
            frame,
            name_area,
            name_input,
            cursor.min(name_input.chars().count()),
            text_style,
        ),
        (true, None) => {
            let mut spans: Vec<Span> = vec![Span::styled(name_input.to_string(), name_style)];
            spans.push(Span::styled("█", Style::default().fg(Theme::WARNING)));
            frame.render_widget(Paragraph::new(Line::from(spans)), name_area);
        }
        _ => frame.render_widget(Paragraph::new(name_input.to_string()), name_area),
    }

    // Description field
    let desc_style = if *focus == TaskCreationField::Description {
        focused_style
    } else {
        idle_style
    };
    let desc_block = Block::default()
        .title(" Description (optional) ")
        .borders(Borders::ALL)
        .style(desc_style);
    let desc_area = desc_block.inner(inner[1]);
    frame.render_widget(desc_block, inner[1]);

    if *focus == TaskCreationField::Description {
        if let Some(cursor) = description_cursor {
            crate::tui::text_edit::render_editable_area(
                frame,
                desc_area,
                description_input,
                cursor.min(description_input.chars().count()),
                text_style,
            );
        } else {
            let mut spans: Vec<Span> = description_input
                .lines()
                .flat_map(|line| {
                    vec![
                        Span::styled(line.to_string(), desc_style),
                        Span::styled("\n", Style::default()),
                    ]
                })
                .collect();
            spans.push(Span::styled("█", Style::default().fg(Theme::WARNING)));
            frame.render_widget(
                Paragraph::new(Line::from(spans)).wrap(Wrap { trim: false }),
                desc_area,
            );
        }
    } else {
        frame.render_widget(
            Paragraph::new(description_input.to_string()).wrap(Wrap { trim: false }),
            desc_area,
        );
    }

    // Hint text
    let hint = Paragraph::new("Tab: switch field | Ctrl+S: create | Esc: cancel")
        .style(Style::default().fg(Theme::WARNING));
    frame.render_widget(hint, inner[2]);
}

/// Estimate the number of lines after text wrapping
fn estimate_wrapped_lines(text: &str, available_width: usize) -> usize {
    if available_width == 0 {
        return text.len();
    }

    let mut lines = 0;
    for line in text.lines() {
        if line.is_empty() {
            lines += 1;
        } else {
            // Estimate wrapped lines for this line
            let wrapped = (line.len() + available_width - 1) / available_width;
            lines += wrapped.max(1);
        }
    }
    lines.max(1)
}

#[allow(dead_code)]
pub fn get_task_detail_hints() -> &'static str {
    "e: Edit | d: Delete | Ctrl+S: Save | Esc: Close"
}
