//! Confirmation dialog widget

use ratatui::{
    Frame,
    layout::{Rect, Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Paragraph, Clear},
    text::{Line, Span},
};

/// Dialog types
#[derive(Debug, Clone)]
pub enum DialogType {
    ConfirmDelete,
    ConfirmQuit,
    Custom(String),
}

impl DialogType {
    pub fn message(&self) -> &str {
        match self {
            DialogType::ConfirmDelete => "Are you sure you want to delete this item?",
            DialogType::ConfirmQuit => "Are you sure you want to quit?",
            DialogType::Custom(msg) => msg,
        }
    }
}

/// Dialog state
#[derive(Debug, Clone)]
pub struct DialogState {
    pub dialog_type: Option<DialogType>,
    pub selected: bool, // false = No, true = Yes
}

impl DialogState {
    pub fn new() -> Self {
        Self {
            dialog_type: None,
            selected: false,
        }
    }
    
    pub fn show(&mut self, dialog_type: DialogType) {
        self.dialog_type = Some(dialog_type);
        self.selected = false;
    }
    
    pub fn hide(&mut self) {
        self.dialog_type = None;
        self.selected = false;
    }
    
    pub fn toggle(&mut self) {
        self.selected = !self.selected;
    }
    
    pub fn is_visible(&self) -> bool {
        self.dialog_type.is_some()
    }
    
    pub fn confirmed(&self) -> bool {
        self.selected
    }
}

impl Default for DialogState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_dialog(frame: &mut Frame, state: &DialogState, area: Rect) {
    if !state.is_visible() {
        return;
    }
    
    let dialog_type = match &state.dialog_type {
        Some(d) => d,
        None => return,
    };
    
    // Center the dialog
    let dialog_area = centered_rect(50, 30, area);
    
    frame.render_widget(Clear, dialog_area);
    
    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Yellow));
    
    frame.render_widget(block, dialog_area);
    
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(dialog_area);
    
    // Message
    let msg = Paragraph::new(dialog_type.message())
        .style(Style::default().fg(Color::White));
    frame.render_widget(msg, inner[0]);
    
    // Buttons
    let yes_style = if state.selected {
        Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red)
    };
    
    let no_style = if !state.selected {
        Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };
    
    let buttons = Line::from(vec![
        Span::styled("  No  ", no_style),
        Span::raw("  "),
        Span::styled("  Yes  ", yes_style),
    ]);
    
    let buttons_para = Paragraph::new(buttons);
    frame.render_widget(buttons_para, inner[1]);
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

pub fn get_dialog_hints() -> &'static str {
    "←/→: Select | Enter: Confirm | Esc: Cancel"
}
