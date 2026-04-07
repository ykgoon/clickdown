//! Sidebar widget for workspace hierarchy navigation

use crate::tui::helpers::SelectableList;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Sidebar item types
#[derive(Debug, Clone)]
pub enum SidebarItem {
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
            let (type_label, name, name_style) = match item {
                SidebarItem::Workspace { name, .. } => (
                    "WS",
                    format!("{}", name),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                SidebarItem::Space { name, .. } => ("SP", format!("{}", name), Style::default()),
                SidebarItem::Folder { name, .. } => ("FL", format!("{}", name), Style::default()),
                SidebarItem::List { name, .. } => {
                    ("LI", format!("{}", name), Style::default().fg(Color::Cyan))
                }
            };

            let line = Line::from(vec![
                Span::styled(type_label, Style::default().fg(Color::DarkGray)),
                Span::raw(" "),
                Span::styled(name, name_style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let sidebar = List::new(items)
        .block(
            Block::default()
                .title(" Navigation ")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(sidebar, area, &mut state.state().clone());
}

/// Get help hints for sidebar
#[allow(dead_code)]
pub fn get_sidebar_hints() -> &'static str {
    "j/k: Navigate | Enter: Select | Tab: Toggle | Ctrl+Q: Quit"
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
