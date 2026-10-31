//! SPACEmenu command overlay (issue 7) and @mention picker overlay (issue 6)

use crate::models::User;
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Render the SPACEmenu command overlay as a centered popup.
///
/// `level` selects the root menu or a command group (`g`, `t`, `w`).
pub fn render_command_menu(frame: &mut Frame, area: Rect, level: &crate::tui::app::CommandMenuLevel) {
    let popup = centered(area, 50, 14);
    frame.render_widget(Clear, popup);

    let level_char = match level {
        crate::tui::app::CommandMenuLevel::Root => ' ',
        crate::tui::app::CommandMenuLevel::Group(c) => *c,
    };
    let (title, lines): (&str, Vec<Line>) = match level_char {
        'g' => (
            " SPACEmenu: Go (g) ",
            vec![
                Line::from(Span::styled("g u", Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled("  Navigate to URL", Style::default().fg(Theme::TEXT))),
                Line::from(""),
                Line::from(Span::styled("Esc", Style::default().fg(Theme::SECONDARY))),
                Line::from(Span::styled("  Close menu", Style::default().fg(Theme::SECONDARY))),
            ],
        ),
        't' => (
            " SPACEmenu: Task (t) ",
            vec![
                Line::from(Span::styled("t n", Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled("  New task", Style::default().fg(Theme::TEXT))),
                Line::from(Span::styled("t s", Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled("  Change status", Style::default().fg(Theme::TEXT))),
                Line::from(Span::styled("t d", Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled("  Delete task", Style::default().fg(Theme::TEXT))),
                Line::from(""),
                Line::from(Span::styled("Esc", Style::default().fg(Theme::SECONDARY))),
                Line::from(Span::styled("  Close menu", Style::default().fg(Theme::SECONDARY))),
            ],
        ),
        'w' => (
            " SPACEmenu: Window (w) ",
            vec![
                Line::from(Span::styled("w q", Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled("  Quit ClickDown", Style::default().fg(Theme::TEXT))),
                Line::from(""),
                Line::from(Span::styled("Esc", Style::default().fg(Theme::SECONDARY))),
                Line::from(Span::styled("  Close menu", Style::default().fg(Theme::SECONDARY))),
            ],
        ),
        _ => (
            " SPACEmenu ",
            vec![
                Line::from(Span::styled("SPC g", Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled("  Go to... (u: URL)", Style::default().fg(Theme::TEXT))),
                Line::from(Span::styled("SPC t", Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled("  Task actions (n/s/d)", Style::default().fg(Theme::TEXT))),
                Line::from(Span::styled("SPC w", Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled("  Window actions (q: quit)", Style::default().fg(Theme::TEXT))),
                Line::from(""),
                Line::from(Span::styled("Esc", Style::default().fg(Theme::SECONDARY))),
                Line::from(Span::styled("  Close menu", Style::default().fg(Theme::SECONDARY))),
            ],
        ),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().bg(Theme::BACKGROUND));
    frame.render_widget(block, popup);
    let inner = Rect {
        x: popup.x + 1,
        y: popup.y + 1,
        width: popup.width.saturating_sub(2),
        height: popup.height.saturating_sub(2),
    };
    let text = Paragraph::new(lines);
    frame.render_widget(text, inner);
}

/// Render the @mention picker overlay: filtered member list + query line.
pub fn render_mention_picker(
    frame: &mut Frame,
    area: Rect,
    members: &[User],
    cursor: usize,
    query: &str,
) {
    let popup = centered(area, 46, 14);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Mention @ ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Theme::BACKGROUND));
    frame.render_widget(block, popup);

    let inner = Rect {
        x: popup.x + 1,
        y: popup.y + 1,
        width: popup.width.saturating_sub(2),
        height: popup.height.saturating_sub(2),
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    // Filter members by the typed query, same rule as insert_selected_mention
    let q = query.to_lowercase();
    let filtered: Vec<&User> = members
        .iter()
        .filter(|m| m.username.to_lowercase().contains(&q))
        .collect();

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(idx, member)| {
            let is_cursor = idx == cursor;
            let color = member
                .color
                .as_ref()
                .and_then(|c| parse_hex_color(c))
                .unwrap_or(Theme::TEXT);
            let line = Line::from(vec![
                Span::styled(
                    format!("{} @{}", if is_cursor { ">" } else { " " }, member.username),
                    Style::default().fg(color),
                ),
                Span::styled(
                    member
                        .email
                        .as_ref()
                        .map(|e| format!("  ({})", e))
                        .unwrap_or_default(),
                    Style::default().fg(Theme::SECONDARY),
                ),
            ]);
            let style = if is_cursor {
                Style::default().bg(Theme::SECONDARY).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect();

    frame.render_widget(List::new(items), layout[0]);

    let hint = if filtered.is_empty() {
        Paragraph::new(format!("No member matches '{}'", query))
            .style(Style::default().fg(Theme::WARNING))
    } else {
        Paragraph::new(format!("filter: {} | Enter: mention | Esc: cancel", query))
            .style(Style::default().fg(Theme::SECONDARY))
    };
    frame.render_widget(hint, layout[1]);
}

/// Create a centered rectangle within `area`
fn centered(area: Rect, percent_x: u16, height: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x.min(100)) / 2),
            Constraint::Percentage(percent_x.min(100)),
            Constraint::Percentage((100 - percent_x.min(100)) / 2),
        ])
        .split(vertical[1]);
    horizontal[1]
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
