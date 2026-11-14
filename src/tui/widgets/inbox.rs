//! Inbox list widget.
//!
//! The inbox is not a `GroupedTaskList`: its entries come from the undocumented
//! inbox payload (a different shape), carry no status grouping, and show
//! attention-specific context (which list/folder, how many comments). It is
//! rendered as a flat list with the unread marker and comment count.
//!
//! Rows show `name` plus a dimmed location/comments suffix, so a row is
//! recognisable without opening it.

use crate::models::InboxTask;
use crate::tui::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
    Frame,
};

/// Render the inbox list.
///
/// `read_ids` decides which rows are dimmed: an entry the user has already seen
/// stays visible (so it is not lost) but no longer draws the eye.
pub fn render_inbox(
    frame: &mut Frame,
    tasks: &[InboxTask],
    selected: usize,
    read_ids: &std::collections::HashSet<String>,
    loading: bool,
    area: Rect,
) {
    if loading && tasks.is_empty() {
        let loading_text = Paragraph::new("Loading inbox...")
            .style(Style::default().fg(Theme::WARNING))
            .block(crate::tui::layout::titled_block(" Inbox "));
        frame.render_widget(loading_text, area);
        return;
    }

    if tasks.is_empty() {
        let empty = Paragraph::new("Inbox clear. Nothing needs your attention.")
            .style(Style::default().fg(Theme::SUCCESS))
            .block(crate::tui::layout::titled_block(" Inbox "));
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = tasks
        .iter()
        .map(|task| {
            let unread = !read_ids.contains(&task.id);
            let marker = if unread { "● " } else { "  " };

            // Location context: "Folder / List", whichever parts exist.
            let mut context: Vec<&str> = Vec::new();
            if let Some(folder) = task.folder_name() {
                context.push(folder);
            }
            if let Some(list) = task.list_name() {
                context.push(list);
            }
            let context = context.join(" / ");

            let mut spans = vec![
                Span::styled(
                    marker,
                    Style::default().fg(if unread {
                        Theme::PRIMARY
                    } else {
                        Theme::SECONDARY
                    }),
                ),
                Span::styled(
                    task.name.clone(),
                    if unread {
                        Style::default()
                            .fg(Theme::TEXT)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Theme::TEXT_DIM)
                    },
                ),
            ];

            if let Some(count) = task.comments_count.filter(|count| *count > 0) {
                spans.push(Span::styled(
                    format!("  {} comment{}", count, if count == 1 { "" } else { "s" }),
                    Style::default().fg(Theme::WARNING),
                ));
            }

            if !context.is_empty() {
                spans.push(Span::styled(
                    format!("  {}", context),
                    Style::default().fg(Theme::SECONDARY),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let mut state = ListState::default();
    if !tasks.is_empty() {
        state.select(Some(selected.min(tasks.len() - 1)));
    }

    let list = List::new(items)
        .block(crate::tui::layout::titled_block(" Inbox "))
        .highlight_style(
            Style::default()
                .bg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(list, area, &mut state);
}

/// Footer hint for the inbox view.
pub fn get_inbox_hints() -> &'static str {
    "j/k: Move | Enter: Open task | d: Dismiss | r: Reload | Esc: Back"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hints_mention_the_dismiss_key() {
        assert!(get_inbox_hints().contains("d: Dismiss"));
    }
}
