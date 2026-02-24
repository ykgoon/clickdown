//! Help overlay widget

use ratatui::{
    Frame,
    layout::{Rect, Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Paragraph, Clear},
    text::{Line, Span},
};

/// Help state
#[derive(Debug, Clone)]
pub struct HelpState {
    pub visible: bool,
}

impl HelpState {
    pub fn new() -> Self {
        Self { visible: false }
    }
    
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
    
    pub fn hide(&mut self) {
        self.visible = false;
    }
}

impl Default for HelpState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_help(frame: &mut Frame, state: &HelpState, area: Rect) {
    if !state.visible {
        return;
    }
    
    let help_area = centered_rect(70, 70, area);
    
    frame.render_widget(Clear, help_area);
    
    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Cyan));
    
    frame.render_widget(block, help_area);
    
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(help_area);
    
    let nav = Paragraph::new(vec![
        Line::from(Span::styled("Navigation:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k or ↑/↓  - Move selection"),
        Line::from("  Enter       - Select/Open item"),
        Line::from("  Esc         - Go back/Close"),
    ]);
    
    let global = Paragraph::new(vec![
        Line::from(Span::styled("Global:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  q           - Quit"),
        Line::from("  Tab         - Toggle sidebar"),
        Line::from("  ?           - Show this help"),
    ]);
    
    let actions = Paragraph::new(vec![
        Line::from(Span::styled("Actions:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  n           - Create new item"),
        Line::from("  e           - Edit selected item"),
        Line::from("  d           - Delete selected item"),
    ]);
    
    let forms = Paragraph::new(vec![
        Line::from(Span::styled("Forms:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+S      - Save"),
        Line::from("  Esc         - Cancel"),
    ]);
    
    frame.render_widget(nav, inner[0]);
    frame.render_widget(global, inner[1]);
    frame.render_widget(actions, inner[2]);
    frame.render_widget(forms, inner[3]);
    
    let close_hint = Paragraph::new("Press any key to close")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(close_hint, inner[8]);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
