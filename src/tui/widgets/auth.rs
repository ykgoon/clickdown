//! Authentication widget

use crate::tui::layout::centered_rect;
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
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
        .style(Style::default().bg(Theme::PRIMARY));

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
        .style(Style::default().fg(Theme::SECONDARY));
    frame.render_widget(help, inner[1]);

    // Token input (partially masked: first 4 chars visible, rest masked).
    // The masked string is rendered through the shared single-line editor so
    // the caret is always visible even for very long tokens.
    let line_area = inner[2];
    let style = Style::default()
        .fg(Theme::TEXT)
        .add_modifier(Modifier::BOLD);
    let prefix = "Token: ";

    if state.loading {
        frame.render_widget(
            Paragraph::new(format!("{}Loading...", prefix)).style(style),
            line_area,
        );
    } else if state.token_input.is_empty() {
        frame.render_widget(
            Paragraph::new(format!("{}[Type or paste your token here]", prefix))
                .style(Style::default().fg(Theme::SECONDARY)),
            line_area,
        );
        // Caret sits right after "Token: " on the empty field.
        let prefix_len = prefix.chars().count() as u16;
        frame.set_cursor_position((
            (line_area.x + prefix_len).min(line_area.x + line_area.width.saturating_sub(1)),
            line_area.y,
        ));
    } else {
        let token_chars: Vec<char> = state.token_input.chars().collect();
        let cursor = state.cursor_pos.min(token_chars.len());

        // Insert a block caret so the position stays visible even where the
        // hardware cursor is not drawn (and in widget snapshots).
        let mut masked = String::new();
        for (i, c) in token_chars.iter().enumerate() {
            if i == cursor {
                masked.push('█');
            }
            masked.push(if i < 4 { *c } else { '•' });
        }
        if cursor >= token_chars.len() {
            masked.push('█');
        }

        let prefix_len = prefix.chars().count();
        let caret_in_full = prefix_len + cursor + usize::from(cursor < token_chars.len());
        let full = format!("{}{}", prefix, masked);

        // Horizontal window that keeps the caret in view.
        let width = line_area.width as usize;
        let chars: Vec<char> = full.chars().collect();
        let start = if caret_in_full >= width {
            caret_in_full + 1 - width
        } else {
            0
        };
        let visible: String = chars.iter().skip(start).take(width).collect();
        frame.render_widget(Paragraph::new(visible).style(style), line_area);

        let cursor_x = line_area.x + (caret_in_full - start) as u16;
        frame.set_cursor_position((
            cursor_x.min(line_area.x + line_area.width.saturating_sub(1)),
            line_area.y,
        ));
    }

    // Show token length for debugging
    let token_len_info = Paragraph::new(format!("({} characters)", state.token_input.len()))
        .style(Style::default().fg(Theme::TEXT_DIM));
    frame.render_widget(token_len_info, inner[3]);

    // Error message
    if let Some(ref error) = state.error {
        let error_para = Paragraph::new(error.as_str()).style(Style::default().fg(Theme::ERROR));
        frame.render_widget(error_para, inner[4]);
    }

    // Instructions
    let instructions = Paragraph::new("Press Enter to connect, Esc to cancel")
        .style(Style::default().fg(Theme::SECONDARY));
    frame.render_widget(instructions, inner[5]);
}

#[allow(dead_code)]
pub fn get_auth_hints() -> &'static str {
    "Enter: Connect | Esc: Cancel"
}
