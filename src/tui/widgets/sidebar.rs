//! Sidebar widget for workspace hierarchy navigation

use crate::tui::helpers::SelectableList;
use crate::tui::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem},
    Frame,
};

/// Sidebar item types
#[derive(Debug, Clone)]
pub enum SidebarItem {
    /// The Inbox entry pinned to the navigation root, with an unread badge.
    ///
    /// Like the Assigned entry it is a workspace-wide query, not a location in
    /// the hierarchy, so it only ever appears at the root level.
    Inbox {
        unread: usize,
    },
    /// The "Assigned to Me" entry pinned to the navigation root.
    ///
    /// It is not part of the workspace hierarchy: it is a shortcut to a
    /// workspace-wide query, so it only ever appears at the root level.
    AssignedToMe,
    Workspace {
        name: String,
        id: String,
    },
    Space {
        name: String,
        id: String,
    },
    Folder {
        name: String,
        id: String,
    },
    List {
        name: String,
        id: String,
    },
}

impl SidebarItem {
    /// Get the ID of this sidebar item
    pub fn id(&self) -> &str {
        match self {
            // Synthetic entry: this ID is never used to look anything up
            // upstream, only to keep the sidebar's selection logic uniform.
            SidebarItem::Inbox { .. } => "inbox",
            SidebarItem::AssignedToMe => "assigned-to-me",
            SidebarItem::Workspace { id, .. } => id,
            SidebarItem::Space { id, .. } => id,
            SidebarItem::Folder { id, .. } => id,
            SidebarItem::List { id, .. } => id,
        }
    }
}

/// Sidebar state
#[derive(Debug, Clone)]
pub struct SidebarState {
    list: SelectableList<SidebarItem>,
    /// Whether sidebar is visible
    pub visible: bool,
}

impl SidebarState {
    pub fn new() -> Self {
        Self {
            list: SelectableList::empty(),
            visible: true,
        }
    }

    /// Select the first item
    pub fn select_first(&mut self) {
        self.list.select_first();
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        self.list.select_next();
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        self.list.select_previous();
    }

    /// Get currently selected item
    pub fn selected_item(&self) -> Option<&SidebarItem> {
        self.list.selected()
    }

    /// Select item by ID, returns true if found
    pub fn select_by_id(&mut self, id: &str) -> bool {
        self.list.select_by(|item| item.id() == id)
    }

    /// Select item by index (public for testing)
    #[allow(dead_code)]
    pub fn select(&mut self, index: Option<usize>) {
        self.list.select(index);
    }

    /// Get sidebar items
    pub fn items(&self) -> &[SidebarItem] {
        self.list.items()
    }

    /// Get mutable sidebar items
    pub fn items_mut(&mut self) -> &mut Vec<SidebarItem> {
        self.list.items_mut()
    }

    /// Get the internal list state for rendering
    pub fn state(&self) -> &ratatui::widgets::ListState {
        self.list.state()
    }
}

impl Default for SidebarState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the sidebar
pub fn render_sidebar(frame: &mut Frame, state: &SidebarState, area: Rect) {
    let items: Vec<ListItem> = state
        .items()
        .iter()
        .map(|item| {
            // Each level of the sidebar lists exactly one kind of thing, so a
            // type prefix (WS/SP/FL/LI) only repeats what the screen already
            // says. Rows therefore start at the left edge; the sole exception
            // is the inbox, which appends its unread count when non-zero.
            let (name, name_style): (String, Style) = match item {
                SidebarItem::Inbox { unread } => {
                    let style = if *unread > 0 {
                        Style::default()
                            .fg(Theme::PRIMARY)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Theme::TEXT_DIM)
                    };
                    ("Inbox".to_string(), style)
                }
                // No marker: the label says what it is. Style alone sets the
                // two pinned entries apart from the hierarchy.
                SidebarItem::AssignedToMe => (
                    "Assigned to Me".to_string(),
                    Style::default()
                        .fg(Theme::WARNING)
                        .add_modifier(Modifier::BOLD),
                ),
                SidebarItem::Workspace { name, .. } => (
                    name.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                SidebarItem::Space { name, .. } => (name.clone(), Style::default()),
                SidebarItem::Folder { name, .. } => (name.clone(), Style::default()),
                SidebarItem::List { name, .. } => {
                    (name.clone(), Style::default().fg(Theme::PRIMARY))
                }
            };

            let mut spans = vec![Span::styled(name, name_style)];

            // Unread count trails the label so the name stays first and no
            // leading column is reserved for it.
            if let SidebarItem::Inbox { unread } = item {
                if *unread > 0 {
                    spans.push(Span::styled(
                        format!(" {}", unread),
                        Style::default().fg(Theme::PRIMARY),
                    ));
                }
            }

            let line = Line::from(spans);

            ListItem::new(line)
        })
        .collect();

    let sidebar = List::new(items)
        .block(crate::tui::layout::titled_block(" Navigation "))
        .highlight_style(
            Style::default()
                .bg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(sidebar, area, &mut state.state().clone());
}

/// Get help hints for sidebar
#[allow(dead_code)]
pub fn get_sidebar_hints() -> &'static str {
    "j/k: Navigate | Enter: Select | Tab: Toggle | SPC q: Quit"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_by_id_finds_matching_item() {
        let mut state = SidebarState::new();
        *state.items_mut() = vec![
            SidebarItem::Workspace {
                name: "WS1".to_string(),
                id: "ws-1".to_string(),
            },
            SidebarItem::Workspace {
                name: "WS2".to_string(),
                id: "ws-2".to_string(),
            },
            SidebarItem::Workspace {
                name: "WS3".to_string(),
                id: "ws-3".to_string(),
            },
        ];

        let found = state.select_by_id("ws-2");

        assert!(found, "Should find matching ID");
        assert_eq!(state.state().selected(), Some(1), "Should select index 1");
    }

    #[test]
    fn test_select_by_id_returns_false_for_non_existent_id() {
        let mut state = SidebarState::new();
        *state.items_mut() = vec![
            SidebarItem::Workspace {
                name: "WS1".to_string(),
                id: "ws-1".to_string(),
            },
            SidebarItem::Workspace {
                name: "WS2".to_string(),
                id: "ws-2".to_string(),
            },
        ];

        let found = state.select_by_id("ws-nonexistent");

        assert!(!found, "Should not find non-existent ID");
        assert_eq!(
            state.state().selected(),
            None,
            "Should not change selection"
        );
    }

    #[test]
    fn test_select_by_id_empty_items() {
        let mut state = SidebarState::new();

        let found = state.select_by_id("any-id");

        assert!(!found, "Should not find ID in empty list");
    }

    #[test]
    fn test_sidebar_item_id() {
        let workspace = SidebarItem::Workspace {
            name: "Test".to_string(),
            id: "ws-123".to_string(),
        };
        assert_eq!(workspace.id(), "ws-123");

        let space = SidebarItem::Space {
            name: "Test".to_string(),
            id: "sp-456".to_string(),
        };
        assert_eq!(space.id(), "sp-456");

        let folder = SidebarItem::Folder {
            name: "Test".to_string(),
            id: "fd-789".to_string(),
        };
        assert_eq!(folder.id(), "fd-789");

        let list = SidebarItem::List {
            name: "Test".to_string(),
            id: "lt-abc".to_string(),
        };
        assert_eq!(list.id(), "lt-abc");
    }
}
