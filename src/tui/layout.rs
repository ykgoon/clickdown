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

/// Height ratio for task detail view: description panel gets 30%, comments get 70%
pub const TASK_DETAIL_DESCRIPTION_RATIO: u16 = 30;
pub const TASK_DETAIL_COMMENTS_RATIO: u16 = 70;

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

/// Split task detail area into description and comments panels with 3:7 ratio
/// Returns (description_area, comments_area)
pub fn split_task_detail(area: Rect) -> (Rect, Rect) {
    let chunks = ratatui::layout::Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(TASK_DETAIL_DESCRIPTION_RATIO),
            Constraint::Percentage(TASK_DETAIL_COMMENTS_RATIO),
        ])
        .split(area);

    (chunks[0], chunks[1])
}

/// Scroll state for tracking independent panel scrolling
#[derive(Debug, Clone, Default)]
pub struct ScrollState {
    /// Current scroll offset (number of lines scrolled from top)
    pub offset: usize,
    /// Maximum scroll offset (total content height - visible area)
    pub max_offset: usize,
    /// Whether scrolling is needed (content exceeds visible area)
    pub scrollable: bool,
}

impl ScrollState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update scroll state based on content and available space
    pub fn update(&mut self, content_height: usize, available_height: usize) {
        self.scrollable = content_height > available_height;
        self.max_offset = content_height.saturating_sub(available_height);
        
        // Clamp offset to valid range
        if self.offset > self.max_offset {
            self.offset = self.max_offset;
        }
    }

    /// Scroll up by one line
    pub fn scroll_up(&mut self) {
        if self.offset > 0 {
            self.offset = self.offset.saturating_sub(1);
        }
    }

    /// Scroll down by one line
    pub fn scroll_down(&mut self) {
        self.offset = (self.offset + 1).min(self.max_offset);
    }

    /// Scroll to a specific position
    pub fn scroll_to(&mut self, position: usize) {
        self.offset = position.min(self.max_offset);
    }

    /// Reset scroll position to top
    pub fn reset(&mut self) {
        self.offset = 0;
    }
}

/// Calculate scroll indicator position and size
/// Returns (start_row, height) relative to the area
pub fn calculate_scroll_indicator(area: Rect, content_height: usize, scroll_offset: usize) -> Option<(u16, u16)> {
    if content_height <= area.height as usize || area.height < 3 {
        return None;
    }

    let visible_height = area.height as usize;
    let max_offset = content_height.saturating_sub(visible_height);
    
    if max_offset == 0 {
        return None;
    }

    // Calculate thumb position and size
    // Thumb size is proportional to visible area / total content
    let thumb_size = ((visible_height * visible_height) as f64 / content_height as f64).ceil() as u16;
    let thumb_size = thumb_size.max(1).min(area.height.saturating_sub(2));
    
    // Calculate thumb start position
    let progress = scroll_offset as f64 / max_offset as f64;
    let available_track = area.height.saturating_sub(2); // Leave 1 row padding at top and bottom
    let thumb_start = (progress * (available_track - thumb_size) as f64) as u16 + 1;

    Some((thumb_start, thumb_size))
}

/// Render a scroll indicator on the right edge of the given area
pub fn render_scroll_indicator(frame: &mut Frame, area: Rect, content_height: usize, scroll_offset: usize) {
    if let Some((thumb_start, thumb_size)) = calculate_scroll_indicator(area, content_height, scroll_offset) {
        let right_edge_x = area.x + area.width - 1;
        
        // Draw thumb using block characters
        for y in 0..thumb_size {
            let draw_y = area.y + thumb_start + y;
            if draw_y < area.y + area.height {
                let pos = (right_edge_x, draw_y);
                // Use a simple character for the scroll thumb
                frame.buffer_mut().set_string(pos.0, pos.1, "│", Style::default().fg(Color::DarkGray));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_state_new() {
        let state = ScrollState::new();
        assert_eq!(state.offset, 0);
        assert_eq!(state.max_offset, 0);
        assert!(!state.scrollable);
    }

    #[test]
    fn test_scroll_state_update_scrollable() {
        let mut state = ScrollState::new();
        state.update(100, 50); // 100 lines of content, 50 visible
        assert!(state.scrollable);
        assert_eq!(state.max_offset, 50);
        assert_eq!(state.offset, 0);
    }

    #[test]
    fn test_scroll_state_update_not_scrollable() {
        let mut state = ScrollState::new();
        state.update(30, 50); // 30 lines of content, 50 visible
        assert!(!state.scrollable);
        assert_eq!(state.max_offset, 0);
    }

    #[test]
    fn test_scroll_state_scroll_up() {
        let mut state = ScrollState::new();
        state.update(100, 50);
        state.offset = 25;
        
        state.scroll_up();
        assert_eq!(state.offset, 24);
        
        // Scroll to top
        state.offset = 0;
        state.scroll_up();
        assert_eq!(state.offset, 0); // Should not go negative
    }

    #[test]
    fn test_scroll_state_scroll_down() {
        let mut state = ScrollState::new();
        state.update(100, 50); // max_offset = 50
        
        state.scroll_down();
        assert_eq!(state.offset, 1);
        
        // Scroll to bottom
        state.offset = 50;
        state.scroll_down();
        assert_eq!(state.offset, 50); // Should not exceed max
    }

    #[test]
    fn test_scroll_state_scroll_to() {
        let mut state = ScrollState::new();
        state.update(100, 50); // max_offset = 50
        
        state.scroll_to(25);
        assert_eq!(state.offset, 25);
        
        // Clamp to max
        state.scroll_to(100);
        assert_eq!(state.offset, 50);
    }

    #[test]
    fn test_scroll_state_reset() {
        let mut state = ScrollState::new();
        state.update(100, 50);
        state.offset = 25;
        
        state.reset();
        assert_eq!(state.offset, 0);
    }

    #[test]
    fn test_calculate_scroll_indicator_no_scroll_needed() {
        let area = Rect::new(0, 0, 50, 20);
        let result = calculate_scroll_indicator(area, 15, 0); // 15 lines, 20 visible
        assert!(result.is_none());
    }

    #[test]
    fn test_calculate_scroll_indicator_at_top() {
        let area = Rect::new(0, 0, 50, 20);
        let result = calculate_scroll_indicator(area, 100, 0);
        assert!(result.is_some());
        let (start, _size) = result.unwrap();
        assert_eq!(start, 1); // Should start near top
    }

    #[test]
    fn test_calculate_scroll_indicator_at_bottom() {
        let area = Rect::new(0, 0, 50, 20);
        let max_offset = 100 - 20; // 80
        let result = calculate_scroll_indicator(area, 100, max_offset);
        assert!(result.is_some());
        let (start, size) = result.unwrap();
        // Should be near the bottom
        assert!(start > 10);
    }

    #[test]
    fn test_calculate_scroll_indicator_small_area() {
        let area = Rect::new(0, 0, 50, 2); // Too small
        let result = calculate_scroll_indicator(area, 100, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_split_task_detail_ratio() {
        let area = Rect::new(0, 0, 100, 100);
        let (desc_area, comments_area) = split_task_detail(area);
        
        // Check heights are approximately 30% and 70%
        let total_height = desc_area.height + comments_area.height;
        let desc_ratio = desc_area.height as f64 / total_height as f64;
        
        // Allow some tolerance for rounding
        assert!(desc_ratio >= 0.25 && desc_ratio <= 0.35, "Description ratio should be ~30%, got {}", desc_ratio);
    }
}
