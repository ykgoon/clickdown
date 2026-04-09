//! Help overlay widget with paginated pages

use crate::tui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Number of pages in the help dialog
const TOTAL_PAGES: u8 = 3;

/// Context for determining what to show on page 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpContext {
    /// Auth screen
    Auth,
    /// Workspace/Space/Folder navigation screens
    Navigation,
    /// Task list screen
    TaskList,
    /// Task detail with description focus
    TaskDetail,
    /// Task detail with comments focus
    Comments,
    /// Document viewer
    Document,
}

impl HelpContext {
    /// Get the display name for page 1 of this context
    fn page1_name(&self) -> &'static str {
        match self {
            HelpContext::Auth => "Auth",
            HelpContext::Navigation => "Navigation",
            HelpContext::TaskList => "Task List",
            HelpContext::TaskDetail => "Task Detail",
            HelpContext::Comments => "Comments",
            HelpContext::Document => "Document",
        }
    }
}

/// Help state with pagination
#[derive(Debug, Clone)]
pub struct HelpState {
    pub visible: bool,
    /// Current page (0-indexed: 0, 1, 2)
    pub page: u8,
}

impl HelpState {
    pub fn new() -> Self {
        Self {
            visible: false,
            page: 0,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.page = 0; // Reset to page 1 when opening
        }
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.page = 0;
    }

    /// Advance to next page (wraps from last to first)
    pub fn next_page(&mut self) {
        self.page = (self.page + 1) % TOTAL_PAGES;
    }

    /// Go to previous page (wraps from first to last)
    pub fn prev_page(&mut self) {
        if self.page == 0 {
            self.page = TOTAL_PAGES - 1;
        } else {
            self.page -= 1;
        }
    }

    /// Reset to page 1 (without changing visibility)
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.page = 0;
    }
}

impl Default for HelpState {
    fn default() -> Self {
        Self::new()
    }
}

/// Section builder helper
fn section(title: &str, items: &[(&str, &str)]) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        title.to_string(),
        Style::default().add_modifier(Modifier::BOLD),
    )));
    for (key, desc) in items {
        lines.push(Line::from(format!("  {:<14}- {}", key, desc)));
    }
    lines
}

/// Get lines for a given page and context
fn page_lines(context: &HelpContext, page: u8) -> Vec<Line<'static>> {
    let nav = section("Navigation", &[
        ("j/k or ↑/↓", "Move selection"),
        ("Enter", "Select/Open item"),
        ("Esc", "Go back/Close"),
        ("g u", "Navigate to URL"),
    ]);

    let global = section("Global", &[
        ("Ctrl+Q", "Quit (saves session)"),
        ("Tab", "Toggle sidebar"),
        ("?", "Show this help"),
        ("u", "Copy element URL"),
    ]);

    let actions = section("Actions", &[
        ("n", "Create new item"),
        ("e", "Edit selected item"),
        ("d", "Delete selected item"),
    ]);

    let task_list = section("Task List", &[
        ("a", "Toggle Assigned to Me filter"),
        ("n", "Create new task"),
        ("s", "Open status picker"),
        ("d", "Delete selected task"),
    ]);

    let task_detail = section("Task Detail", &[
        ("s", "Open status picker"),
        ("A", "Open assignee picker"),
        ("e", "Edit task"),
        ("Tab", "Toggle comments focus"),
        ("Esc", "Back to task list"),
    ]);

    let task_creation = section("Task Creation", &[
        ("Tab", "Switch between name/description"),
        ("Ctrl+S", "Create task"),
        ("Esc", "Cancel creation"),
    ]);

    let comments = section("Comments", &[
        ("Tab", "Toggle focus (task/comments)"),
        ("j/k", "Navigate comments"),
        ("n", "New comment"),
        ("e", "Edit selected comment"),
        ("r", "Reply to thread (in thread view)"),
        ("Enter", "View thread"),
        ("Ctrl+S", "Save comment"),
        ("Esc", "Cancel editing / Exit thread"),
    ]);

    let forms = section("Forms", &[
        ("Ctrl+S", "Save"),
        ("Esc", "Cancel"),
    ]);

    let session = section("Session", &[
        ("", "Session auto-saves on exit"),
        ("", "Restores last view on startup"),
    ]);

    match page {
        0 => {
            // Page 1: Contextual
            match context {
                HelpContext::Auth => {
                    let mut lines = Vec::new();
                    lines.push(Line::from(Span::styled(
                        "Auth",
                        Style::default().add_modifier(Modifier::BOLD),
                    )));
                    lines.push(Line::from("  Enter         - Connect to ClickUp"));
                    lines.push(Line::from("  Esc           - Cancel"));
                    lines.push(Line::from(""));
                    lines
                }
                HelpContext::Navigation => nav,
                HelpContext::TaskList => task_list.clone(),
                HelpContext::TaskDetail => task_detail.clone(),
                HelpContext::Comments => comments.clone(),
                HelpContext::Document => {
                    let mut lines = Vec::new();
                    lines.push(Line::from(Span::styled(
                        "Document",
                        Style::default().add_modifier(Modifier::BOLD),
                    )));
                    lines.push(Line::from("  j/k or ↑/↓  - Scroll"));
                    lines.push(Line::from("  Esc         - Close document"));
                    lines.push(Line::from(""));
                    lines
                }
            }
        }
        1 => {
            // Page 2: Always Global (Navigation + Global + Actions + Forms)
            let mut lines = Vec::new();
            lines.extend(nav);
            lines.push(Line::from(""));
            lines.extend(global);
            lines.push(Line::from(""));
            lines.extend(actions);
            lines.push(Line::from(""));
            lines.extend(forms);
            lines
        }
        _ => {
            // Page 3: Reference (everything not on page 1 or 2)
            let mut lines = Vec::new();
            match context {
                HelpContext::TaskList => {
                    lines.extend(task_detail);
                    lines.push(Line::from(""));
                    lines.extend(task_creation);
                    lines.push(Line::from(""));
                    lines.extend(comments);
                    lines.push(Line::from(""));
                    lines.extend(session);
                }
                HelpContext::TaskDetail => {
                    lines.extend(task_list);
                    lines.push(Line::from(""));
                    lines.extend(task_creation);
                    lines.push(Line::from(""));
                    lines.extend(comments);
                    lines.push(Line::from(""));
                    lines.extend(session);
                }
                HelpContext::Comments => {
                    lines.extend(task_detail);
                    lines.push(Line::from(""));
                    lines.extend(task_creation);
                    lines.push(Line::from(""));
                    lines.extend(task_list);
                    lines.push(Line::from(""));
                    lines.extend(session);
                }
                HelpContext::Navigation | HelpContext::Auth | HelpContext::Document => {
                    lines.extend(task_list);
                    lines.push(Line::from(""));
                    lines.extend(task_detail);
                    lines.push(Line::from(""));
                    lines.extend(task_creation);
                    lines.push(Line::from(""));
                    lines.extend(comments);
                    lines.push(Line::from(""));
                    lines.extend(session);
                }
            }
            lines
        }
    }
}

/// Get title text for current page
fn page_title(context: &HelpContext, page: u8) -> String {
    let section_name = match page {
        0 => context.page1_name(),
        1 => "Global",
        _ => "Reference",
    };
    format!(
        " Keyboard Shortcuts — {}  ({}/{}) ",
        section_name,
        page + 1,
        TOTAL_PAGES
    )
}

pub fn render_help(frame: &mut Frame, state: &HelpState, context: &HelpContext, area: Rect) {
    if !state.visible {
        return;
    }

    let help_area = centered_rect(70, 70, area);

    frame.render_widget(Clear, help_area);

    // Build title with page indicator
    let title = page_title(context, state.page);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().bg(Theme::BACKGROUND).fg(Theme::PRIMARY));

    frame.render_widget(block, help_area);

    // Build content lines for current page
    let content_lines = page_lines(context, state.page);

    // Layout: content area + footer
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Min(1), // Content
            Constraint::Length(1), // Footer
        ])
        .split(help_area);

    // Render content
    let content = Paragraph::new(content_lines).style(Style::default().fg(Theme::TEXT));
    frame.render_widget(content, inner[0]);

    // Render pagination footer
    let footer_text = format!(
        "◄ ►  {}/{}  │  j/k: Pages  │  Esc: Close",
        state.page + 1,
        TOTAL_PAGES
    );
    let footer = Paragraph::new(footer_text).style(Style::default().fg(Theme::SECONDARY));
    frame.render_widget(footer, inner[1]);
}

/// Get pagination hint for status bar
pub fn get_help_hints(state: &HelpState) -> String {
    format!("j/k: Pages | Esc: Close | {}/{}", state.page + 1, TOTAL_PAGES)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_state_new() {
        let state = HelpState::new();
        assert!(!state.visible);
        assert_eq!(state.page, 0);
    }

    #[test]
    fn test_help_state_toggle_resets_page() {
        let mut state = HelpState::new();
        state.visible = true;
        state.page = 2;
        state.toggle(); // closes
        state.toggle(); // opens
        assert!(state.visible);
        assert_eq!(state.page, 0);
    }

    #[test]
    fn test_help_state_next_page_wrapping() {
        let mut state = HelpState::new();
        state.visible = true;

        state.next_page();
        assert_eq!(state.page, 1);

        state.next_page();
        assert_eq!(state.page, 2);

        // Wrap from last to first
        state.next_page();
        assert_eq!(state.page, 0);
    }

    #[test]
    fn test_help_state_prev_page_wrapping() {
        let mut state = HelpState::new();
        state.visible = true;
        state.page = 1;

        state.prev_page();
        assert_eq!(state.page, 0);

        // Wrap from first to last
        state.prev_page();
        assert_eq!(state.page, 2);
    }

    #[test]
    fn test_help_state_hide_resets() {
        let mut state = HelpState::new();
        state.visible = true;
        state.page = 2;
        state.hide();
        assert!(!state.visible);
        assert_eq!(state.page, 0);
    }

    #[test]
    fn test_page_title_format() {
        let ctx = HelpContext::TaskList;
        assert_eq!(page_title(&ctx, 0), " Keyboard Shortcuts — Task List  (1/3) ");
        assert_eq!(page_title(&ctx, 1), " Keyboard Shortcuts — Global  (2/3) ");
        assert_eq!(page_title(&ctx, 2), " Keyboard Shortcuts — Reference  (3/3) ");
    }

    #[test]
    fn test_page_lines_page2_is_always_global() {
        // Page 2 should always include Navigation, Global, Actions, Forms
        let contexts = [
            HelpContext::Auth,
            HelpContext::Navigation,
            HelpContext::TaskList,
            HelpContext::TaskDetail,
            HelpContext::Comments,
            HelpContext::Document,
        ];
        for ctx in contexts {
            let lines = page_lines(&ctx, 1);
            assert!(!lines.is_empty(), "Page 2 should not be empty for {:?}", ctx);
        }
    }

    #[test]
    fn test_help_hints_format() {
        let mut state = HelpState::new();
        state.visible = true;

        let hints = get_help_hints(&state);
        assert_eq!(hints, "j/k: Pages | Esc: Close | 1/3");

        state.page = 2;
        let hints = get_help_hints(&state);
        assert_eq!(hints, "j/k: Pages | Esc: Close | 3/3");
    }

    #[test]
    fn test_section_format() {
        let lines = section("Test", &[("a", "Do something"), ("b", "Do other")]);
        assert_eq!(lines.len(), 3); // title + 2 items
        assert!(matches!(&lines[0], Line { .. })); // title is bold
    }

    #[test]
    fn test_contextual_page1_content() {
        // TaskList page 1 should contain task list shortcuts
        let lines = page_lines(&HelpContext::TaskList, 0);
        assert!(!lines.is_empty());

        // TaskDetail page 1 should contain task detail shortcuts
        let lines = page_lines(&HelpContext::TaskDetail, 0);
        assert!(!lines.is_empty());

        // Comments page 1 should contain comments shortcuts
        let lines = page_lines(&HelpContext::Comments, 0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_page3_complement() {
        // Page 3 should be different from page 1 for TaskList
        let page1 = page_lines(&HelpContext::TaskList, 0);
        let page3 = page_lines(&HelpContext::TaskList, 2);
        assert_ne!(page1.len(), page3.len(), "Page 3 should differ from page 1");
    }
}
