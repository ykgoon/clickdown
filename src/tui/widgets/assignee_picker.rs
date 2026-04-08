//! Assignee picker widget - overlay dialog for selecting task assignees

use crate::models::User;
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render the assignee picker as an overlay
///
/// Parameters:
/// - `frame`: The rendering frame
/// - `area`: The total screen area (used to center the overlay)
/// - `members`: All members who can access the list
/// - `selected_ids`: Currently assigned user IDs
/// - `cursor`: Current cursor position
pub fn render_assignee_picker(
    frame: &mut Frame,
    area: Rect,
    members: &[User],
    selected_ids: &std::collections::HashSet<i64>,
    cursor: usize,
) {
    // Calculate overlay dimensions
    let overlay_width = 60.min(area.width.saturating_sub(4));
    let overlay_height = (members.len() as u16 + 4).min(area.height.saturating_sub(4));
    let overlay_height = overlay_height.max(8); // Minimum height

    // Center the overlay
    let x = (area.width.saturating_sub(overlay_width)) / 2;
    let y = (area.height.saturating_sub(overlay_height)) / 2;

    let picker_area = Rect {
        x,
        y,
        width: overlay_width,
        height: overlay_height,
    };

    // Render dimmed background behind overlay
    let dim_block = Block::default().style(Style::default().bg(Theme::SECONDARY));

    // Clear the overlay area first
    frame.render_widget(dim_block, picker_area);

    // Build the overlay content
    let block = Block::default()
        .title(" Select Assignees ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Theme::BACKGROUND));

    frame.render_widget(&block, picker_area);

    let inner = block.inner(picker_area);

    // Split into member list and hint
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Member list
            Constraint::Length(1), // Hint line
        ])
        .split(inner);

    // Render members as a list
    let items: Vec<ListItem> = members
        .iter()
        .enumerate()
        .map(|(idx, member)| {
            let is_selected = selected_ids.contains(&member.id);
            let is_cursor = idx == cursor;

            let checkbox = if is_selected { "[x]" } else { "[ ]" };

            let checkbox_style = if is_selected {
                Style::default()
                    .fg(Theme::SUCCESS)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Theme::TEXT_DIM)
            };

            let mut spans = vec![
                Span::styled(format!("{} ", checkbox), checkbox_style),
                Span::styled(
                    &member.username,
                    Style::default()
                        .fg(Theme::TEXT)
                        .add_modifier(Modifier::BOLD),
                ),
            ];

            if let Some(email) = &member.email {
                if !email.is_empty() {
                    spans.push(Span::styled(
                        format!(" <{}>", email),
                        Style::default().fg(Theme::SECONDARY),
                    ));
                }
            }

            if let Some(initials) = &member.initials {
                if !initials.is_empty() {
                    spans.push(Span::styled(
                        format!(" ({})", initials),
                        Style::default().fg(Theme::PRIMARY),
                    ));
                }
            }

            let line = Line::from(spans);

            let item_style = if is_cursor {
                Style::default()
                    .bg(Theme::SECONDARY)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(line).style(item_style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, layout[0]);

    // Render hint line
    let hint = Paragraph::new("Space: toggle | j/k: navigate | Ctrl+S: save | Esc: cancel")
        .style(Style::default().fg(Theme::WARNING));
    frame.render_widget(hint, layout[1]);
}
