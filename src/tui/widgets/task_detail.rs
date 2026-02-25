//! Task detail widget

use ratatui::{
    Frame,
    layout::{Rect, Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Paragraph, Wrap},
    text::{Line, Span},
};
use crate::models::Task;
use crate::tui::layout::ScrollState;

/// Task detail state
#[derive(Debug, Clone)]
pub struct TaskDetailState {
    pub task: Option<Task>,
    pub editing: bool,
    /// Scroll state for the description panel
    pub description_scroll: ScrollState,
}

impl TaskDetailState {
    pub fn new() -> Self {
        Self {
            task: None,
            editing: false,
            description_scroll: ScrollState::new(),
        }
    }
}

impl Default for TaskDetailState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_task_detail(frame: &mut Frame, state: &TaskDetailState, area: Rect) {
    let block = Block::default()
        .title(" Task Detail ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    frame.render_widget(&block, area);

    let inner_area = block.inner(area);
    
    // Split into task info and description with better ratio
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),  // Name
            Constraint::Length(1),  // Status
            Constraint::Length(1),  // Priority
            Constraint::Min(2),     // Description (flexible space)
        ])
        .split(inner_area);

    if let Some(task) = &state.task {
        frame.render_widget(
            Paragraph::new(format!("Name: {}", task.name)),
            inner[0],
        );

        let status = task.status.as_ref()
            .map(|s| s.status.as_str())
            .unwrap_or("None");
        frame.render_widget(
            Paragraph::new(format!("Status: {}", status)),
            inner[1],
        );

        let priority = task.priority.as_ref()
            .map(|p| p.priority.as_str())
            .unwrap_or("None");
        frame.render_widget(
            Paragraph::new(format!("Priority: {}", priority)),
            inner[2],
        );

        let desc = task.description.as_ref()
            .map(|d| d.as_text())
            .unwrap_or_else(|| "No description".to_string());

        // Calculate description content height for scroll state
        let available_height = inner[3].height as usize;
        let available_width = inner[3].width.saturating_sub(4) as usize; // Account for borders
        
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
                    .style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: true });
        
        // Render with scroll offset
        frame.render_widget(desc_paragraph, inner[3]);
        
        // Render scroll indicator if needed
        if scroll_state.scrollable {
            crate::tui::layout::render_scroll_indicator(
                frame,
                inner[3],
                content_height,
                scroll_state.offset,
            );
        }
    } else {
        frame.render_widget(
            Paragraph::new("No task selected"),
            inner[0],
        );
    }

    if state.editing {
        let edit_hint = Paragraph::new("Press Ctrl+S to save, Esc to cancel")
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(edit_hint, inner[3]);
    }
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

pub fn get_task_detail_hints() -> &'static str {
    "e: Edit | d: Delete | Ctrl+S: Save | Esc: Close"
}
