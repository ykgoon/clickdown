//! Authentication widget

use ratatui::{
    Frame,
    layout::{Rect, Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Paragraph, Clear},
    text::{Line, Span},
};

/// Authentication state
#[derive(Debug, Clone)]
pub struct AuthState {
    pub token_input: String,
    pub cursor_pos: usize,
    pub error: Option<String>,
    pub loading: bool,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            token_input: String::new(),
            cursor_pos: 0,
            error: None,
            loading: false,
        }
    }
    
    pub fn add_char(&mut self, c: char) {
        self.token_input.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
    }
    
    pub fn remove_char(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.token_input.remove(self.cursor_pos);
        }
    }
    
    pub fn clear(&mut self) {
        self.token_input.clear();
        self.cursor_pos = 0;
        self.error = None;
    }
}

impl Default for AuthState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_auth(frame: &mut Frame, state: &AuthState, area: Rect) {
    // Center the auth box
    let auth_area = centered_rect(60, 40, area);
    
    frame.render_widget(Clear, auth_area);
    
    let block = Block::default()
        .title(" Authentication ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Blue));
    
    frame.render_widget(block, auth_area);
    
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(auth_area);
    
    // Title
    let title = Paragraph::new("Enter your ClickUp API Token")
        .style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(title, inner[0]);
    
    // Help text
    let help = Paragraph::new("Get your token from ClickUp Settings → Apps → ClickUp API")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, inner[1]);
    
    // Token input (masked)
    let masked: String = state.token_input.chars().map(|_| '•').collect();
    let input_display = if state.loading {
        "Loading...".to_string()
    } else {
        masked
    };
    
    let input = Paragraph::new(format!("Token: {}", input_display))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(input, inner[2]);
    
    // Error message
    if let Some(ref error) = state.error {
        let error_para = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red));
        frame.render_widget(error_para, inner[3]);
    }
    
    // Instructions
    let instructions = Paragraph::new("Press Enter to connect, Esc to cancel")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(instructions, inner[4]);
}

/// Create a centered rectangle
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

pub fn get_auth_hints() -> &'static str {
    "Enter: Connect | Esc: Cancel"
}
