//! Task detail widget

use ratatui::{
    Frame,
    layout::{Rect, Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Paragraph, Wrap},
    text::{Line, Span},
};
use crate::models::Task;

/// Task detail state
#[derive(Debug, Clone)]
pub struct TaskDetailState {
    pub task: Option<Task>,
    pub editing: bool,
}

impl TaskDetailState {
    pub fn new() -> Self {
        Self {
            task: None,
            editing: false,
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

    frame.render_widget(block, area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(3),  // Give description more space
        ])
        .split(area);

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
        
        // Render description with text wrapping
        let desc_paragraph = Paragraph::new(desc)
            .block(
                Block::default()
                    .title(" Description ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: true });
        
        frame.render_widget(desc_paragraph, inner[3]);
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

pub fn get_task_detail_hints() -> &'static str {
    "e: Edit | d: Delete | Ctrl+S: Save | Esc: Close"
}
