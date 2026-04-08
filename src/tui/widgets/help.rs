//! Help overlay widget

use crate::tui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
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
        .style(Style::default().bg(Theme::BACKGROUND).fg(Theme::PRIMARY));

    frame.render_widget(block, help_area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1),  // title
            Constraint::Length(4),  // nav (title + 3 items)
            Constraint::Length(1),  // spacer
            Constraint::Length(5),  // global (title + 4 items)
            Constraint::Length(1),  // spacer
            Constraint::Length(4),  // actions (title + 3 items)
            Constraint::Length(1),  // spacer
            Constraint::Length(5),  // task list (title + 4 items)
            Constraint::Length(1),  // spacer
            Constraint::Length(5),  // task detail (title + 4 items)
            Constraint::Length(1),  // spacer
            Constraint::Length(10), // comments (title + 9 items)
            Constraint::Length(1),  // spacer
            Constraint::Length(3),  // forms (title + 2 items)
            Constraint::Length(1),  // spacer
            Constraint::Length(3),  // session (title + 2 items)
            Constraint::Min(1),     // close hint
        ])
        .split(help_area);

    let nav = Paragraph::new(vec![
        Line::from(Span::styled(
            "Navigation:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  j/k or ↑/↓  - Move selection"),
        Line::from("  Enter       - Select/Open item"),
        Line::from("  Esc         - Go back/Close"),
    ]);

    let global = Paragraph::new(vec![
        Line::from(Span::styled(
            "Global:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Ctrl+Q      - Quit (saves session)"),
        Line::from("  Tab         - Toggle sidebar"),
        Line::from("  ?           - Show this help"),
        Line::from("  u           - Copy element URL"),
    ]);

    let actions = Paragraph::new(vec![
        Line::from(Span::styled(
            "Actions:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  n           - Create new item"),
        Line::from("  e           - Edit selected item"),
        Line::from("  d           - Delete selected item"),
    ]);

    let assigned = Paragraph::new(vec![
        Line::from(Span::styled(
            "Task List:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  a           - Toggle Assigned to Me filter"),
        Line::from("  n           - Create new task"),
        Line::from("  d           - Delete selected task"),
    ]);

    let task_detail = Paragraph::new(vec![
        Line::from(Span::styled(
            "Task Detail:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  A           - Open assignee picker"),
        Line::from("  e           - Edit task"),
        Line::from("  Tab         - Toggle comments focus"),
        Line::from("  Esc         - Back to task list"),
    ]);

    let comments = Paragraph::new(vec![
        Line::from(Span::styled(
            "Comments (Task Detail):",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Tab         - Toggle focus (task/comments)"),
        Line::from("  j/k         - Navigate comments"),
        Line::from("  n           - New comment"),
        Line::from("  e           - Edit selected comment"),
        Line::from("  r           - Reply to thread (in thread view)"),
        Line::from("  Enter       - View thread"),
        Line::from("  Ctrl+S      - Save comment"),
        Line::from("  Esc         - Cancel editing / Exit thread"),
    ]);

    let forms = Paragraph::new(vec![
        Line::from(Span::styled(
            "Forms:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Ctrl+S      - Save"),
        Line::from("  Esc         - Cancel"),
    ]);

    let session = Paragraph::new(vec![
        Line::from(Span::styled(
            "Session:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Session auto-saves on exit"),
        Line::from("  Restores last view on startup"),
    ]);

    // Render sections at correct indices (odd indices are content, even are spacers)
    frame.render_widget(nav, inner[1]);
    frame.render_widget(global, inner[3]);
    frame.render_widget(actions, inner[5]);
    frame.render_widget(assigned, inner[7]);
    frame.render_widget(task_detail, inner[9]);
    frame.render_widget(comments, inner[11]);
    frame.render_widget(forms, inner[13]);
    frame.render_widget(session, inner[15]);

    let close_hint =
        Paragraph::new("Press any key to close").style(Style::default().fg(Theme::SECONDARY));
    frame.render_widget(close_hint, inner[16]);
}

/// Get help hint for status bar
#[allow(dead_code)]
pub fn get_help_hints() -> &'static str {
    "? - Help"
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
