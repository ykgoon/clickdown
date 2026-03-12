//! Inbox view widget for displaying activity feed

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::models::InboxActivity;

/// State for the inbox activity list
#[derive(Clone)]
pub struct InboxListState {
    pub list_state: ListState,
    pub activities: Vec<InboxActivity>,
}

impl InboxListState {
    pub fn new() -> Self {
        Self {
            list_state: ListState::default(),
            activities: Vec::new(),
        }
    }

    pub fn set_activities(&mut self, activities: Vec<InboxActivity>) {
        self.activities = activities;
        if self.activities.is_empty() {
            self.list_state.select(None);
        } else if self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        } else if let Some(selected) = self.list_state.selected() {
            if selected >= self.activities.len() {
                self.list_state
                    .select(Some(self.activities.len().saturating_sub(1)));
            }
        }
    }

    /// Legacy method for backward compatibility (maps to set_activities)
    #[allow(dead_code)]
    pub fn set_notifications(&mut self, activities: Vec<InboxActivity>) {
        self.set_activities(activities);
    }

    pub fn select_next(&mut self) {
        if self.activities.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.activities.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_previous(&mut self) {
        if self.activities.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.activities.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    #[allow(dead_code)]
    pub fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }

    pub fn selected_activity(&self) -> Option<&InboxActivity> {
        self.list_state
            .selected()
            .and_then(|i| self.activities.get(i))
    }

    /// Legacy method for backward compatibility
    #[allow(dead_code)]
    pub fn selected_notification(&self) -> Option<&InboxActivity> {
        self.selected_activity()
    }

    /// Get activities (public for testing)
    #[allow(dead_code)]
    pub fn activities(&self) -> &Vec<InboxActivity> {
        &self.activities
    }

    /// Legacy method for backward compatibility
    #[allow(dead_code)]
    pub fn notifications(&self) -> &Vec<InboxActivity> {
        &self.activities
    }

    /// Select an activity by index (public for testing)
    #[allow(dead_code)]
    pub fn select(&mut self, index: Option<usize>) {
        self.list_state.select(index);
    }
}

impl Default for InboxListState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the activity list
pub fn render_inbox_list(
    frame: &mut Frame,
    area: Rect,
    state: &mut InboxListState,
    _showing_detail: bool,
) {
    if state.activities.is_empty() {
        // Empty state
        let empty_msg = Paragraph::new("📬 Inbox is empty - All caught up!")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Inbox"));
        frame.render_widget(empty_msg, area);
        return;
    }

    let items: Vec<ListItem> = state
        .activities
        .iter()
        .enumerate()
        .map(|(i, activity)| {
            let is_selected = state.list_state.selected() == Some(i);

            // Get activity type icon
            let type_icon = activity.icon();

            // Format timestamp
            let time_str = format_timestamp(activity.timestamp);

            // Truncate description for preview
            let desc_preview = if activity.description.is_empty() {
                String::new()
            } else {
                let truncated = if activity.description.len() > 60 {
                    format!("{}...", &activity.description[..57])
                } else {
                    activity.description.clone()
                };
                format!(" - {}", truncated)
            };

            let style = if is_selected {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", type_icon),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(&activity.title, style),
                Span::styled(desc_preview, Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(" ({})", time_str),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Inbox"))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut state.list_state);
}

/// Render activity detail panel
pub fn render_notification_detail(frame: &mut Frame, area: Rect, activity: &InboxActivity) {
    let detail_block = Block::default()
        .borders(Borders::ALL)
        .title("Activity Details")
        .style(Style::default().bg(Color::DarkGray));

    let time_str = format_timestamp(activity.timestamp);

    let content = format!(
        "Type: {} {}\n\nTime: {}\n\nTitle:\n{}\n\nDescription:\n{}",
        activity.icon(),
        activity.activity_type.label(),
        time_str,
        activity.title,
        if activity.description.is_empty() {
            "(No description)".to_string()
        } else {
            activity.description.clone()
        }
    );

    let detail = Paragraph::new(content)
        .block(detail_block)
        .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(Clear, area);
    frame.render_widget(detail, area);
}

/// Format a Unix timestamp (milliseconds) to a readable date string
fn format_timestamp(ts: i64) -> String {
    use chrono::{DateTime, Local};

    let secs = ts / 1000;
    match DateTime::from_timestamp(secs, 0) {
        Some(dt) => {
            let local_dt: DateTime<Local> = dt.into();
            local_dt.format("%b %d, %Y %H:%M").to_string()
        }
        None => "Unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ActivityType;

    #[test]
    fn test_inbox_list_state_navigation() {
        let mut state = InboxListState::new();

        // Empty list
        state.select_next();
        assert_eq!(state.selected(), None);

        // Add activities
        state.set_activities(vec![
            InboxActivity {
                id: "1".to_string(),
                activity_type: ActivityType::Assignment,
                title: "First".to_string(),
                description: "".to_string(),
                timestamp: 1000,
                task_id: None,
                comment_id: None,
                workspace_id: "ws".to_string(),
                task_name: String::new(),
                previous_status: None,
                new_status: None,
                due_date: None,
            },
            InboxActivity {
                id: "2".to_string(),
                activity_type: ActivityType::Comment,
                title: "Second".to_string(),
                description: "".to_string(),
                timestamp: 2000,
                task_id: None,
                comment_id: None,
                workspace_id: "ws".to_string(),
                task_name: String::new(),
                previous_status: None,
                new_status: None,
                due_date: None,
            },
        ]);

        assert_eq!(state.selected(), Some(0));

        state.select_next();
        assert_eq!(state.selected(), Some(1));

        state.select_next();
        assert_eq!(state.selected(), Some(0)); // Wraps around

        state.select_previous();
        assert_eq!(state.selected(), Some(1));
    }
}
