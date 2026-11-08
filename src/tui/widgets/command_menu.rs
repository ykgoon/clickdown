//! SPACEmenu Doom-style minibuffer (issue 7) and @mention picker overlay (issue 6)
//!
//! The SPACEmenu is docked to the bottom edge of the screen (echo-area style).
//! Each menu level displays exactly the single key the user presses at that
//! level: the root lists group keys (`g t c y w h q`); selecting a group shows
//! that group's action keys (`n e s d ...`) with descriptions. Two-key chords
//! are never displayed — only the next key to press.

use crate::models::User;
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

// ============================================================================
// SPACEmenu minibuffer (Doom-style bottom-edge menu)
// ============================================================================

/// One selectable group in the root minibuffer row.
struct GroupEntry {
    label: &'static str,
    key: char,
}

/// Root minibuffer entries, in display order (matches the key handler in app.rs).
const ROOT_GROUPS: [GroupEntry; 7] = [
    GroupEntry { label: "Go", key: 'g' },
    GroupEntry { label: "Task", key: 't' },
    GroupEntry { label: "Comments", key: 'c' },
    GroupEntry { label: "Yank", key: 'y' },
    GroupEntry { label: "Window", key: 'w' },
    GroupEntry { label: "Help", key: 'h' },
    GroupEntry { label: "Quit", key: 'q' },
];

/// Color coding per group key (consistent across root and group levels).
fn group_color(key: char) -> Color {
    match key {
        'g' => Theme::PRIMARY,
        't' => Theme::WARNING,
        'c' => Theme::SUCCESS,
        'y' => Color::LightBlue,
        'w' => Color::Blue,
        'h' => Color::Magenta,
        'q' => Theme::ERROR,
        _ => Theme::TEXT,
    }
}

/// Action entries per group: (description, key). The key is the single key the
/// user presses at this level; the full chord is implicit (`SPC`, group, key).
fn group_actions(group: char) -> Option<(&'static str, Vec<(&'static str, char)>)> {
    match group {
        'g' => Some(("Go", vec![("Open URL", 'u')])),
        't' => Some((
            "Task",
            vec![
                ("New task", 'n'),
                ("Edit task / description", 'e'),
                ("Change status", 's'),
                ("Delete task", 'd'),
                ("Assign people", 'a'),
                ("Assigned-to-Me filter", 'f'),
                ("Expand/collapse description", 'v'),
            ],
        )),
        'c' => Some((
            "Comments",
            vec![
                ("New comment", 'n'),
                ("Edit selected comment", 'e'),
                ("Reply in thread", 'r'),
                ("Expand/collapse comments", 'v'),
            ],
        )),
        'y' => Some(("Yank", vec![("Copy task/element URL", 'u')])),
        'w' => Some((
            "Window",
            vec![("Toggle sidebar", 's'), ("Quit ClickDown", 'q')],
        )),
        _ => None,
    }
}

/// Rect anchored to the bottom edge of `area`: full width, `rows` tall.
fn minibuffer_rect(area: Rect, rows: u16) -> Rect {
    let rows = rows.min(area.height);
    Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(rows),
        width: area.width,
        height: rows,
    }
}

/// Paint the minibuffer background and return the padded inner rect.
fn minibuffer_frame(frame: &mut Frame, rect: Rect) -> Rect {
    frame.render_widget(Clear, rect);
    frame.render_widget(
        Block::default().style(Style::default().bg(Theme::BACKGROUND)),
        rect,
    );
    Rect {
        x: rect.x + 1,
        y: rect.y,
        width: rect.width.saturating_sub(2),
        height: rect.height,
    }
}

/// Full-width horizontal separator line.
fn separator_line(width: u16) -> Line<'static> {
    Line::from(Span::styled(
        "\u{2500}".repeat(width as usize),
        Style::default().fg(Theme::SECONDARY),
    ))
}

/// Bottom hint line shared by both levels (no title — the menu is self-explanatory).
fn minibuffer_hint(group: Option<(&'static str, char)>, esc_verb: &str) -> Line<'static> {
    let verb = if group.is_some() { "an action key" } else { "a key" };
    Line::from(vec![
        Span::styled(
            format!("press {}", verb),
            Style::default().fg(Theme::SECONDARY),
        ),
        Span::styled(" \u{00b7} ", Style::default().fg(Theme::SECONDARY)),
        Span::styled(
            "Esc",
            Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}", esc_verb),
            Style::default().fg(Theme::SECONDARY),
        ),
    ])
}

/// Render the SPACEmenu as a Doom-style minibuffer docked to the bottom edge.
///
/// Each level shows exactly the key(s) the user can press at that level:
/// Root lists group keys; a group level lists that group's action keys.
pub fn render_command_menu(
    frame: &mut Frame,
    area: Rect,
    level: &crate::tui::app::CommandMenuLevel,
) {
    match level {
        crate::tui::app::CommandMenuLevel::Root => render_minibuffer_root(frame, area),
        crate::tui::app::CommandMenuLevel::Group(group) => {
            render_minibuffer_group(frame, area, *group)
        }
    }
}

/// Root level: one row of color-coded group columns (label over key),
/// a separator, then the hint line.
fn render_minibuffer_root(frame: &mut Frame, area: Rect) {
    let rect = minibuffer_rect(area, 4); // labels + keys, separator, hint
    let inner = minibuffer_frame(frame, rect);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // group labels + keys
            Constraint::Length(1), // separator
            Constraint::Length(1), // hint
        ])
        .split(inner);

    let n = ROOT_GROUPS.len() as u32;
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints((0..n).map(|_| Constraint::Ratio(1, n)))
        .split(rows[0]);

    for (i, entry) in ROOT_GROUPS.iter().enumerate() {
        let widget = Paragraph::new(vec![
            Line::from(Span::styled(
                entry.label.to_string(),
                Style::default()
                    .fg(group_color(entry.key))
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                entry.key.to_string(),
                Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD),
            )),
        ]);
        frame.render_widget(widget, cols[i]);
    }

    frame.render_widget(Paragraph::new(separator_line(inner.width)), rows[1]);
    frame.render_widget(Paragraph::new(minibuffer_hint(None, "close")), rows[2]);
}

/// Group level: group name line, then one action per row
/// (single key + description), separator, hint line.
fn render_minibuffer_group(frame: &mut Frame, area: Rect, group: char) {
    let Some((label, actions)) = group_actions(group) else {
        return; // unknown group: nothing to render
    };

    let rows_needed = (actions.len() as u16) + 3; // label + actions + separator + hint
    let rect = minibuffer_rect(area, rows_needed);
    let inner = minibuffer_frame(frame, rect);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(1),                      // group label
            Constraint::Length(actions.len() as u16),   // action rows
            Constraint::Length(1),                      // separator
            Constraint::Length(1),                      // hint
        ])
        .split(inner);

    let label_line = Line::from(Span::styled(
        format!(" {}", label),
        Style::default()
            .fg(group_color(group))
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(label_line), rows[0]);

    let entry_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints((0..actions.len()).map(|_| Constraint::Length(1)))
        .split(rows[1]);

    for (i, (desc, key)) in actions.iter().enumerate() {
        let line = Line::from(vec![
            Span::styled(
                format!(" {:<4}", key),
                Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                desc.to_string(),
                Style::default().fg(Theme::TEXT),
            ),
        ]);
        frame.render_widget(Paragraph::new(line), entry_rows[i]);
    }

    frame.render_widget(Paragraph::new(separator_line(inner.width)), rows[2]);
    frame.render_widget(
        Paragraph::new(minibuffer_hint(Some((label, group)), "back")),
        rows[3],
    );
}

// ============================================================================
// @mention picker overlay (issue 6)
// ============================================================================

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
