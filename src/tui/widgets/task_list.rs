//! Task list widget

use crate::models::Task;
use crate::tui::helpers::SelectableList;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Task list state
#[derive(Debug, Clone)]
pub struct TaskListState {
    list: SelectableList<Task>,
}

impl TaskListState {
    pub fn new() -> Self {
        Self {
            list: SelectableList::empty(),
        }
    }

    pub fn select_first(&mut self) {
        self.list.select_first();
    }

    pub fn select_next(&mut self) {
        self.list.select_next();
    }

    pub fn select_previous(&mut self) {
        self.list.select_previous();
    }

    pub fn selected_task(&self) -> Option<&Task> {
        self.list.selected()
    }

    /// Select task by index
    pub fn select(&mut self, index: Option<usize>) {
        self.list.select(index);
    }

    /// Get tasks
    pub fn tasks(&self) -> &[Task] {
        self.list.items()
    }

    /// Get mutable tasks
    pub fn tasks_mut(&mut self) -> &mut Vec<Task> {
        self.list.items_mut()
    }

    /// Get the internal list state for rendering
    pub fn state(&self) -> &ratatui::widgets::ListState {
        self.list.state()
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

pub fn render_task_list(frame: &mut Frame, state: &TaskListState, area: Rect, loading: bool) {
    // Show loading indicator if loading
    if loading {
        use ratatui::widgets::Paragraph;
        let loading_text = Paragraph::new("Loading assigned tasks...")
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .title(" Tasks ")
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Black)),
            );
        frame.render_widget(loading_text, area);
        return;
    }

    let items: Vec<ListItem> = state
        .tasks()
        .iter()
        .map(|task| {
            let status_color = get_status_color(&task.status);
            let priority = get_priority_indicator(&task.priority);

            let status_str = task
                .status
                .as_ref()
                .map(|s| {
                    s.status
                        .chars()
                        .next()
                        .unwrap_or(' ')
                        .to_uppercase()
                        .to_string()
                })
                .unwrap_or_else(|| "-".to_string());

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("[{}] ", priority),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    status_str,
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::raw(task.name.as_str()),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Tasks ")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(list, area, &mut state.state().clone());
}

pub fn get_task_list_hints() -> &'static str {
    "j/k: Navigate | Enter: View | n: New | e: Edit | d: Delete"
}
