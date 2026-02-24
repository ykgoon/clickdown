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
    
    // Token input (partially masked: first 4 chars visible, rest masked)
    let input_display = if state.loading {
        "Loading...".to_string()
    } else if state.token_input.is_empty() {
        // Show placeholder when empty
        "[Type or paste your token here]".to_string()
    } else {
        // Show first 4 characters unmasked, rest as bullets
        let visible_chars = 4;
        let mut display = String::new();

        // Build display with cursor indicator in one pass
        let token_chars: Vec<char> = state.token_input.chars().collect();
        for i in 0..=token_chars.len() {
            // Add cursor indicator at cursor position
            if i == state.cursor_pos {
                display.push('█');  // Block cursor for better visibility
            }
            // Add character or bullet
            if i < token_chars.len() {
                if i < visible_chars {
                    display.push(token_chars[i]);
                } else {
                    display.push('•');
                }
            }
        }
        // Handle cursor at end
        if state.cursor_pos > token_chars.len() {
            display.push('█');
        }

        display
    };

    // Use bright white for better visibility
    let input = Paragraph::new(format!("Token: {}", input_display))
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    frame.render_widget(input, inner[2]);
    
    // Show token length for debugging
    let token_len_info = Paragraph::new(format!("({} characters)", state.token_input.len()))
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(token_len_info, inner[3]);
    
    // Error message
    if let Some(ref error) = state.error {
        let error_para = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red));
        frame.render_widget(error_para, inner[4]);
    }

    // Instructions
    let instructions = Paragraph::new("Press Enter to connect, Esc to cancel")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(instructions, inner[5]);
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
