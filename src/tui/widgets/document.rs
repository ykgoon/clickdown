//! Document view widget

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    text::Line,
};
use pulldown_cmark::{Parser, Event, Tag, TagEnd};

/// Document state
#[derive(Debug, Clone)]
pub struct DocumentState {
    pub title: String,
    pub content: String,
    pub scroll_offset: usize,
}

impl DocumentState {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            content: String::new(),
            scroll_offset: 0,
        }
    }
}

impl Default for DocumentState {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple Markdown to plain text conversion for terminal
fn markdown_to_text(md: &str) -> String {
    let parser = Parser::new(md);
    let mut result = String::new();
    
    for event in parser {
        match event {
            Event::Text(text) => result.push_str(&text),
            Event::Code(code) => result.push_str(&format!("`{}`", code)),
            Event::SoftBreak | Event::HardBreak => result.push('\n'),
            Event::End(TagEnd::Paragraph) => result.push_str("\n\n"),
            Event::End(TagEnd::Heading(_)) => result.push_str("\n\n"),
            _ => {}
        }
    }
    
    result
}

pub fn render_document(frame: &mut Frame, state: &DocumentState, area: Rect) {
    let block = Block::default()
        .title(format!(" {} ", state.title))
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    
    frame.render_widget(block, area);
    
    let inner_area = Rect::new(
        area.x + 1,
        area.y + 1,
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    );
    
    let text = markdown_to_text(&state.content);
    let lines: Vec<Line> = text.lines().map(|l| Line::from(l.to_string())).collect();
    
    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(Color::White));
    
    frame.render_widget(paragraph, inner_area);
}

pub fn get_document_hints() -> &'static str {
    "j/k: Scroll | Esc: Close"
}
