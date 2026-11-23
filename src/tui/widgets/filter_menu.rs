//! Filter menu widget - one entry point for everything that narrows a task list
//!
//! `SPC t f` opens this menu. It replaces the earlier per-criterion key
//! bindings (`SPC t f` toggling "Assigned to Me", `SPC t F` opening a status
//! picker) so there is a single, discoverable place to see which filters are
//! active and to remove them.
//!
//! Entries are a fixed, ordered list. Each one reports whether it is currently
//! active so the menu can show the live state instead of making the user
//! remember it.

use crate::tui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// One row in the filter menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMenuEntry {
    /// `f` - narrow the list to tasks assigned to the current user
    AssignedToMe,
    /// `s` - narrow the list to one status
    Status,
    /// `x` - drop every active filter
    ClearAll,
}

impl FilterMenuEntry {
    /// Every entry, in display order. Clear sits last so it cannot be hit by
    /// accident while reaching for a filter.
    pub const ALL: [FilterMenuEntry; 3] = [
        FilterMenuEntry::AssignedToMe,
        FilterMenuEntry::Status,
        FilterMenuEntry::ClearAll,
    ];

    /// Key shown for this entry and accepted by the handler.
    pub fn key(self) -> char {
        match self {
            FilterMenuEntry::AssignedToMe => 'f',
            FilterMenuEntry::Status => 's',
            FilterMenuEntry::ClearAll => 'x',
        }
    }

    /// Label shown in the menu.
    pub fn label(self) -> &'static str {
        match self {
            FilterMenuEntry::AssignedToMe => "Assigned to Me",
            FilterMenuEntry::Status => "By status",
            FilterMenuEntry::ClearAll => "Clear all filters",
        }
    }
}

/// Render the filter menu.
///
/// - `assigned_active`: whether the "Assigned to Me" filter is on
/// - `status`: the active status filter, if any
/// - `cursor`: highlighted entry
pub fn render_filter_menu(
    frame: &mut Frame,
    area: Rect,
    assigned_active: bool,
    status: Option<&str>,
    cursor: usize,
) {
    let entries = FilterMenuEntry::ALL;
    let overlay_width = 44.min(area.width.saturating_sub(4));
    let overlay_height = (entries.len() as u16 + 5).min(area.height.saturating_sub(4));

    let x = (area.width.saturating_sub(overlay_width)) / 2;
    let y = (area.height.saturating_sub(overlay_height)) / 2;
    let menu_area = Rect {
        x,
        y,
        width: overlay_width,
        height: overlay_height,
    };

    // Erase first: a background colour alone leaves glyphs underneath visible.
    frame.render_widget(Clear, menu_area);
    frame.render_widget(
        Block::default().style(Style::default().bg(Theme::BACKGROUND)),
        menu_area,
    );

    let block = Block::default()
        .title(" Filter ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Theme::BACKGROUND));
    frame.render_widget(&block, menu_area);
    let inner = block.inner(menu_area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let active = match entry {
                FilterMenuEntry::AssignedToMe => Some(assigned_active),
                FilterMenuEntry::Status => Some(status.is_some()),
                FilterMenuEntry::ClearAll => None,
            };

            let mut spans = vec![Span::styled(
                format!(" {:<3}", entry.key()),
                Style::default()
                    .fg(Theme::WARNING)
                    .add_modifier(Modifier::BOLD),
            )];

            let label_style = if active == Some(false) {
                // Unavailable on this screen, e.g. status on a view with no
                // list context.
                Style::default().fg(Theme::TEXT_DIM)
            } else {
                Style::default().fg(Theme::TEXT)
            };
            spans.push(Span::styled(entry.label(), label_style));

            if matches!(entry, FilterMenuEntry::Status) {
                if let Some(current) = status {
                    spans.push(Span::styled(
                        format!(": {}", current),
                        Style::default().fg(Theme::WARNING),
                    ));
                }
            } else {
                match active {
                    Some(true) => spans.push(Span::styled(
                        "  [on]",
                        Style::default().fg(Theme::SUCCESS),
                    )),
                    Some(false) => spans.push(Span::styled(
                        "  [off]",
                        Style::default().fg(Theme::TEXT_DIM),
                    )),
                    None => {}
                }
            }

            let style = if idx == cursor {
                Style::default()
                    .bg(Theme::SECONDARY)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(spans)).style(style)
        })
        .collect();

    frame.render_widget(List::new(items), layout[0]);
    frame.render_widget(
        Paragraph::new("j/k: navigate | Enter/letter: apply | Esc: close")
            .style(Style::default().fg(Theme::TEXT_DIM)),
        layout[1],
    );
}
