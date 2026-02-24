//! Layout components for TUI

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    text::{Line, Span},
};

/// Minimum terminal dimensions
pub const MIN_WIDTH: u16 = 80;
pub const MIN_HEIGHT: u16 = 24;

/// Layout structure for the application
#[derive(Debug, Clone)]
pub struct TuiLayout {
    /// Title bar area
    pub title_area: Rect,
    /// Main content area (sidebar + content)
    pub content_area: Rect,
    /// Status bar area
    pub status_area: Rect,
    /// Whether terminal is too small
    pub too_small: bool,
}

impl TuiLayout {
    /// Create a new layout based on terminal size
    pub fn new(area: Rect) -> Self {
        let too_small = area.width < MIN_WIDTH || area.height < MIN_HEIGHT;
        
        // Vertical layout: title (1) + content (flex) + status (3)
        let main_layout = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Title bar
                Constraint::Min(10),    // Content area
                Constraint::Length(3),  // Status bar
            ])
            .split(area);
        
        Self {
            title_area: main_layout[0],
            content_area: main_layout[1],
            status_area: main_layout[2],
            too_small,
        }
    }
    
    /// Split content area into sidebar and main content
    pub fn split_content(&self, sidebar_width: u16) -> (Rect, Rect) {
        let horizontal = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(sidebar_width),
                Constraint::Min(10),
            ])
            .split(self.content_area);
        
        (horizontal[0], horizontal[1])
    }
    
    /// Render title bar
    pub fn render_title(&self, frame: &mut Frame, title: &str) {
        let title_widget = Paragraph::new(Line::from(vec![
            Span::styled(title, Style::default().fg(Color::White).add_modifier(ratatui::style::Modifier::BOLD)),
        ]))
        .block(Block::default().borders(Borders::ALL).style(Style::default().bg(Color::Blue)));
        
        frame.render_widget(title_widget, self.title_area);
    }
    
    /// Render status bar with message and help hints
    pub fn render_status(&self, frame: &mut Frame, status: &str, hints: &str) {
        let status_text = Line::from(vec![
            Span::styled(status, Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::styled(hints, Style::default().fg(Color::DarkGray)),
        ]);
        
        let status_widget = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL).style(Style::default().bg(Color::Black)));
        
        frame.render_widget(status_widget, self.status_area);
    }
    
    /// Render "terminal too small" warning
    pub fn render_too_small_warning(&self, frame: &mut Frame) {
        let warning = Paragraph::new(vec![
            Line::from("Terminal too small!"),
            Line::from(format!("Minimum size: {}x{}", MIN_WIDTH, MIN_HEIGHT)),
            Line::from(format!("Current size: Please resize to at least {}x{}", MIN_WIDTH, MIN_HEIGHT)),
        ])
        .style(Style::default().fg(Color::Yellow));
        
        frame.render_widget(warning, self.content_area);
    }
}

/// Generate screen title based on current context
pub fn generate_screen_title(context: &str) -> String {
    format!("ClickDown - {}", context)
}
