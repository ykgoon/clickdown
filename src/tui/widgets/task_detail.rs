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
    if state.creating {
        render_task_creation_form(
            frame,
            area,
            task_name_input,
            task_description_input,
            task_creation_focus,
        );
        return;
    }

    let block = Block::default()
        .title(" Task Detail ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Theme::BACKGROUND));

    frame.render_widget(&block, area);

    let inner_area = block.inner(area);

    // Split into task info and description with better ratio
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // Name
            Constraint::Length(1), // Status
            Constraint::Length(1), // Priority
            Constraint::Length(1), // Assignees
            Constraint::Min(2),    // Description (flexible space)
        ])
        .split(inner_area);

    if let Some(task) = &state.task {
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

        let desc = task
            .description
            .as_ref()
            .map(|d| d.as_text())
            .unwrap_or_else(|| "No description".to_string());

        // Calculate description content height for scroll state
        let available_height = inner[4].height as usize;
        let available_width = inner[4].width.saturating_sub(4) as usize; // Account for borders

        // Estimate content height by counting wrapped lines
        let content_height = estimate_wrapped_lines(&desc, available_width);

        // Update scroll state
        let mut scroll_state = state.description_scroll.clone();
        scroll_state.update(content_height, available_height);

        // Render description with text wrapping and scrolling
        let desc_paragraph = Paragraph::new(desc)
            .block(
                Block::default()
                    .title(" Description ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Theme::PRIMARY)),
            )
            .wrap(Wrap { trim: true });

        // Render with scroll offset
        frame.render_widget(desc_paragraph, inner[4]);

        // Render scroll indicator if needed
        if scroll_state.scrollable {
            crate::tui::layout::render_scroll_indicator(
                frame,
                inner[4],
                content_height,
                scroll_state.offset,
            );
        }
    } else {
        frame.render_widget(Paragraph::new("No task selected"), inner[0]);
    }

    if state.editing {
        let edit_hint = Paragraph::new("Press Ctrl+S to save, Esc to cancel")
            .style(Style::default().fg(Theme::WARNING));
        frame.render_widget(edit_hint, inner[4]);
    }
}

/// Render the task creation form with name and description input fields
fn render_task_creation_form(
    frame: &mut Frame,
    area: Rect,
    name_input: &str,
    description_input: &str,
    focus: &TaskCreationField,
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

    // Name field
    let name_style = if *focus == TaskCreationField::Name {
        Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::SECONDARY)
    };
    let name_block = Block::default()
        .title(" Name * ")
        .borders(Borders::ALL)
        .style(name_style);
    let name_area = name_block.inner(inner[0]);
    frame.render_widget(name_block, inner[0]);

    // Display cursor indicator for focused field
    let cursor_indicator = if *focus == TaskCreationField::Name { "█" } else { "" };
    let name_content = if name_input.is_empty() {
        Line::from(Span::styled(
            format!("{}{}", cursor_indicator, if *focus == TaskCreationField::Name { "" } else { "" }),
            Style::default().fg(Theme::SECONDARY),
        ))
    } else {
        Line::from(vec![
            Span::styled(name_input.to_string(), name_style),
            Span::styled(cursor_indicator.to_string(), Style::default().fg(Theme::WARNING)),
        ])
    };
    frame.render_widget(Paragraph::new(name_content), name_area);

    // Description field
    let desc_style = if *focus == TaskCreationField::Description {
        Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::SECONDARY)
    };
    let desc_block = Block::default()
        .title(" Description (optional) ")
        .borders(Borders::ALL)
        .style(desc_style);
    let desc_area = desc_block.inner(inner[1]);
    frame.render_widget(desc_block, inner[1]);

    let cursor_indicator = if *focus == TaskCreationField::Description { "█" } else { "" };
    let desc_content = if description_input.is_empty() {
        Line::from(Span::styled(
            format!("{}{}", cursor_indicator, if *focus == TaskCreationField::Description { "" } else { "" }),
            Style::default().fg(Theme::SECONDARY),
        ))
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
        // Add cursor at end
        spans.push(Span::styled(
            cursor_indicator.to_string(),
            Style::default().fg(Theme::WARNING),
        ));
        Line::from(spans)
    };
    frame.render_widget(Paragraph::new(desc_content).wrap(Wrap { trim: false }), desc_area);

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
