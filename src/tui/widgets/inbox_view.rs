//! Inbox view widget for displaying notifications

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::models::Notification;

/// State for the inbox notification list
pub struct InboxListState {
    pub list_state: ListState,
    pub notifications: Vec<Notification>,
}

impl InboxListState {
    pub fn new() -> Self {
        Self {
            list_state: ListState::default(),
            notifications: Vec::new(),
        }
    }

    pub fn set_notifications(&mut self, notifications: Vec<Notification>) {
        self.notifications = notifications;
        if self.notifications.is_empty() {
            self.list_state.select(None);
        } else if self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        } else if let Some(selected) = self.list_state.selected() {
            if selected >= self.notifications.len() {
                self.list_state
                    .select(Some(self.notifications.len().saturating_sub(1)));
            }
        }
    }

    pub fn select_next(&mut self) {
        if self.notifications.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.notifications.len() - 1 {
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
        if self.notifications.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.notifications.len() - 1
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

    pub fn selected_notification(&self) -> Option<&Notification> {
        self.list_state
            .selected()
            .and_then(|i| self.notifications.get(i))
    }
}

impl Default for InboxListState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the notification list
pub fn render_inbox_list(
    frame: &mut Frame,
    area: Rect,
    state: &mut InboxListState,
    _showing_detail: bool,
) {
    if state.notifications.is_empty() {
        // Empty state
        let empty_msg = Paragraph::new("📬 Inbox is empty - All caught up!")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Inbox"));
        frame.render_widget(empty_msg, area);
        return;
    }

    let items: Vec<ListItem> = state
        .notifications
        .iter()
        .enumerate()
        .map(|(i, notif)| {
            let is_selected = state.list_state.selected() == Some(i);

            // Format timestamp
            let time_str = notif
                .created_at
                .map(format_timestamp)
                .unwrap_or_else(|| "Unknown".to_string());

            // Truncate description for preview
            let desc_preview = if notif.description.is_empty() {
                String::new()
            } else {
                let truncated = if notif.description.len() > 60 {
                    format!("{}...", &notif.description[..57])
                } else {
                    notif.description.clone()
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
                    format!("[{}] ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(&notif.title, style),
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

/// Render notification detail panel
pub fn render_notification_detail(frame: &mut Frame, area: Rect, notification: &Notification) {
    let detail_block = Block::default()
        .borders(Borders::ALL)
        .title("Notification Details")
        .style(Style::default().bg(Color::DarkGray));

    let time_str = notification
        .created_at
        .map(format_timestamp)
        .unwrap_or_else(|| "Unknown".to_string());

    let content = format!(
        "Title: {}\n\nTime: {}\n\nDescription:\n{}",
        notification.title,
        time_str,
        if notification.description.is_empty() {
            "(No description)".to_string()
        } else {
            notification.description.clone()
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

    #[test]
    fn test_inbox_list_state_navigation() {
        let mut state = InboxListState::new();

        // Empty list
        state.select_next();
        assert_eq!(state.selected(), None);

        // Add notifications
        state.set_notifications(vec![
            Notification {
                id: "1".to_string(),
                workspace_id: "ws".to_string(),
                title: "First".to_string(),
                description: "".to_string(),
                created_at: Some(1000),
                read_at: None,
            },
            Notification {
                id: "2".to_string(),
                workspace_id: "ws".to_string(),
                title: "Second".to_string(),
                description: "".to_string(),
                created_at: Some(2000),
                read_at: None,
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
