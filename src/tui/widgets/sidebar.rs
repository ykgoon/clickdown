//! Sidebar widget for workspace hierarchy navigation

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, List, ListItem, ListState},
    text::{Line, Span},
};

/// Sidebar state
#[derive(Debug, Clone)]
pub struct SidebarState {
    /// Current selection index
    pub selected: ListState,
    /// Items to display
    pub items: Vec<SidebarItem>,
    /// Whether sidebar is visible
    pub visible: bool,
    /// Scroll offset
    pub scroll_offset: usize,
}

/// Sidebar item types
#[derive(Debug, Clone)]
pub enum SidebarItem {
    Workspace { name: String, id: String },
    Space { name: String, id: String, indent: usize },
    Folder { name: String, id: String, indent: usize },
    List { name: String, id: String, indent: usize },
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

impl SidebarState {
    pub fn new() -> Self {
        Self {
            selected: ListState::default(),
            items: Vec::new(),
            visible: true,
            scroll_offset: 0,
        }
    }
    
    /// Select the first item
    pub fn select_first(&mut self) {
        if !self.items.is_empty() {
            self.selected.select(Some(0));
        }
    }
    
    /// Move selection down
    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.selected.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected.select(Some(i));
    }
    
    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.selected.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected.select(Some(i));
    }
    
    /// Get currently selected item
    pub fn selected_item(&self) -> Option<&SidebarItem> {
        self.selected.selected().and_then(|i| self.items.get(i))
    }

    /// Select item by ID, returns true if found
    pub fn select_by_id(&mut self, id: &str) -> bool {
        for (i, item) in self.items.iter().enumerate() {
            if item.id() == id {
                self.selected.select(Some(i));
                return true;
            }
        }
        false
    }
}

impl Default for SidebarState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the sidebar
pub fn render_sidebar(frame: &mut Frame, state: &SidebarState, area: Rect) {
    let items: Vec<ListItem> = state.items.iter().map(|item| {
        let (prefix, content) = match item {
            SidebarItem::Workspace { name, .. } => {
                ("", Span::styled(name, Style::default().add_modifier(Modifier::BOLD)))
            }
            SidebarItem::Space { name, .. } => {
                ("  ", Span::raw(name.as_str()))
            }
            SidebarItem::Folder { name, .. } => {
                ("    ", Span::raw(name.as_str()))
            }
            SidebarItem::List { name, .. } => {
                ("      ", Span::styled(name, Style::default().fg(Color::Cyan)))
            }
        };
        
        ListItem::new(Line::from(vec![
            Span::raw(prefix),
            content,
        ]))
    }).collect();
    
    let sidebar = List::new(items)
        .block(Block::default()
            .title(" Navigation ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black)))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("▸ ");
    
    frame.render_stateful_widget(sidebar, area, &mut state.selected.clone());
}

/// Get help hints for sidebar
pub fn get_sidebar_hints() -> &'static str {
    "j/k: Navigate | Enter: Select | Tab: Toggle | Ctrl+Q: Quit"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_by_id_finds_matching_item() {
        let mut state = SidebarState::new();
        state.items = vec![
            SidebarItem::Workspace { name: "WS1".to_string(), id: "ws-1".to_string() },
            SidebarItem::Workspace { name: "WS2".to_string(), id: "ws-2".to_string() },
            SidebarItem::Workspace { name: "WS3".to_string(), id: "ws-3".to_string() },
        ];

        let found = state.select_by_id("ws-2");

        assert!(found, "Should find matching ID");
        assert_eq!(state.selected.selected(), Some(1), "Should select index 1");
    }

    #[test]
    fn test_select_by_id_returns_false_for_non_existent_id() {
        let mut state = SidebarState::new();
        state.items = vec![
            SidebarItem::Workspace { name: "WS1".to_string(), id: "ws-1".to_string() },
            SidebarItem::Workspace { name: "WS2".to_string(), id: "ws-2".to_string() },
        ];

        let found = state.select_by_id("ws-nonexistent");

        assert!(!found, "Should not find non-existent ID");
        assert_eq!(state.selected.selected(), None, "Should not change selection");
    }

    #[test]
    fn test_select_by_id_empty_items() {
        let mut state = SidebarState::new();
        state.items = vec![];

        let found = state.select_by_id("any-id");

        assert!(!found, "Should not find ID in empty list");
    }

    #[test]
    fn test_sidebar_item_id() {
        let workspace = SidebarItem::Workspace { name: "Test".to_string(), id: "ws-123".to_string() };
        assert_eq!(workspace.id(), "ws-123");

        let space = SidebarItem::Space { name: "Test".to_string(), id: "sp-456".to_string(), indent: 1 };
        assert_eq!(space.id(), "sp-456");

        let folder = SidebarItem::Folder { name: "Test".to_string(), id: "fd-789".to_string(), indent: 2 };
        assert_eq!(folder.id(), "fd-789");

        let list = SidebarItem::List { name: "Test".to_string(), id: "lt-abc".to_string(), indent: 3 };
        assert_eq!(list.id(), "lt-abc");
    }
}
