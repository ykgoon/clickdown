//! Task list widget

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, List, ListItem, ListState},
    text::{Line, Span},
};
use crate::models::Task;

/// Task list state
#[derive(Debug, Clone)]
pub struct TaskListState {
    pub selected: ListState,
    pub tasks: Vec<Task>,
}

impl TaskListState {
    pub fn new() -> Self {
        Self {
            selected: ListState::default(),
            tasks: Vec::new(),
        }
    }
    
    pub fn select_first(&mut self) {
        if !self.tasks.is_empty() {
            self.selected.select(Some(0));
        }
    }
    
    pub fn select_next(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        let i = match self.selected.selected() {
            Some(i) => if i >= self.tasks.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.selected.select(Some(i));
    }
    
    pub fn select_previous(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        let i = match self.selected.selected() {
            Some(i) => if i == 0 { self.tasks.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.selected.select(Some(i));
    }
    
    pub fn selected_task(&self) -> Option<&Task> {
        self.selected.selected().and_then(|i| self.tasks.get(i))
    }
}

impl Default for TaskListState {
    fn default() -> Self {
        Self::new()
    }
}

/// Get status color
fn get_status_color(status: &Option<crate::models::TaskStatus>) -> Color {
    match status {
        Some(s) => match s.status.to_lowercase().as_str() {
            "complete" | "done" => Color::Green,
            "in progress" => Color::Yellow,
            "todo" => Color::White,
            _ => Color::Gray,
        },
        None => Color::Gray,
    }
}

/// Get priority indicator
fn get_priority_indicator(priority: &Option<crate::models::Priority>) -> &'static str {
    match priority {
        Some(p) => match p.priority.as_str() {
            "urgent" => "⚡",
            "high" => "↑",
            "low" => "↓",
            _ => "•",
        },
        None => "•",
    }
}

pub fn render_task_list(frame: &mut Frame, state: &TaskListState, area: Rect) {
    let items: Vec<ListItem> = state.tasks.iter().map(|task| {
        let status_color = get_status_color(&task.status);
        let priority = get_priority_indicator(&task.priority);
        
        let status_str = task.status.as_ref()
            .map(|s| s.status.chars().next().unwrap_or(' ').to_uppercase().to_string())
            .unwrap_or_else(|| "-".to_string());
        
        ListItem::new(Line::from(vec![
            Span::styled(format!("[{}] ", priority), Style::default().fg(Color::Yellow)),
            Span::styled(status_str, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::raw(task.name.as_str()),
        ]))
    }).collect();
    
    let list = List::new(items)
        .block(Block::default()
            .title(" Tasks ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black)))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("▸ ");
    
    frame.render_stateful_widget(list, area, &mut state.selected.clone());
}

pub fn get_task_list_hints() -> &'static str {
    "j/k: Navigate | Enter: View | n: New | e: Edit | d: Delete"
}
