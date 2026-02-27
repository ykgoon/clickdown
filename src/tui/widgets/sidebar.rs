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
