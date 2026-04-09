//! Status picker widget - overlay dialog for selecting task status

use crate::models::TaskStatus;
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render the status picker as an overlay
///
/// Parameters:
/// - `frame`: The rendering frame
/// - `area`: The total screen area (used to center the overlay)
/// - `statuses`: Available statuses to choose from
/// - `cursor`: Current cursor position
/// - `current_status`: The task's current status (to highlight)
pub fn render_status_picker(
    frame: &mut Frame,
    area: Rect,
    statuses: &[TaskStatus],
    cursor: usize,
    current_status: Option<&str>,
) {
    // Calculate overlay dimensions
    let overlay_width = 40.min(area.width.saturating_sub(4));
    let overlay_height = (statuses.len() as u16 + 4).min(area.height.saturating_sub(4));
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
        .title(" Change Status ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Theme::BACKGROUND));

    frame.render_widget(&block, picker_area);

    let inner = block.inner(picker_area);

    // Split into status list and hint
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Status list
            Constraint::Length(1), // Hint line
        ])
        .split(inner);

    // Render statuses as a list
    let items: Vec<ListItem> = statuses
        .iter()
        .enumerate()
        .map(|(idx, status)| {
            let is_cursor = idx == cursor;
            let is_current = current_status
                .map(|c| c.eq_ignore_ascii_case(&status.status))
                .unwrap_or(false);

            // Parse color from hex string
            let status_color = status
                .color
                .as_ref()
                .and_then(|c| parse_hex_color(c))
                .unwrap_or(Theme::TEXT);

            let status_indicator = if is_current { "*" } else { " " };

            let mut spans = vec![
                Span::styled(
                    format!("{} ", status_indicator),
                    Style::default().fg(Theme::TEXT_DIM),
                ),
                Span::styled(
                    &status.status,
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ];

            // Add status group indicator
            if let Some(group) = &status.status_group {
                spans.push(Span::styled(
                    format!(" [{}]", group),
                    Style::default().fg(Theme::SECONDARY),
                ));
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
    let hint = Paragraph::new("j/k: navigate | Enter: select | Esc: cancel")
        .style(Style::default().fg(Theme::WARNING));
    frame.render_widget(hint, layout[1]);
}

/// Parse a hex color string to a ratatui Color
fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color::Rgb(r, g, b))
    } else {
        None
    }
}
