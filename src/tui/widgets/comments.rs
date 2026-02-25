//! Comment list and form widgets

use chrono::{DateTime, Local};
use ratatui::{
    Frame,
    layout::{Rect, Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Paragraph},
    text::{Line, Span},
};
use crate::models::Comment;

/// Render comments section with list of comments and optional form
pub fn render_comments(
    frame: &mut Frame,
    comments: &[Comment],
    selected_index: usize,
    editing_index: Option<usize>,
    new_text: &str,
    comment_focus: bool,
    area: Rect,
) {
    // Check if area is too small to render
    if area.height < 5 || area.width < 20 {
        let block = Block::default()
            .title(" Comments ")
            .borders(Borders::ALL)
            .style(Style::default());
        frame.render_widget(block, area);
        
        if area.height >= 3 && area.width >= 15 {
            let msg = if comments.is_empty() {
                "No comments"
            } else {
                &format!("{} comments", comments.len())
            };
            let paragraph = Paragraph::new(msg)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(paragraph, area);
        }
        return;
    }

    let block = Block::default()
        .title(" Comments ")
        .borders(Borders::ALL)
        .style(Style::default());

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Split area into comments list and input form
    let has_input = editing_index.is_some() || !new_text.is_empty();

    let constraints = if has_input {
        vec![
            Constraint::Percentage(70),  // Comments list
            Constraint::Percentage(30),  // Input form
        ]
    } else {
        vec![Constraint::Percentage(100)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    // Render comments list
    render_comment_list(
        frame,
        comments,
        selected_index,
        editing_index,
        comment_focus,
        chunks[0],
    );
    
    // Render input form if editing or creating
    if has_input {
        let edit_label = if editing_index.is_some() {
            "Edit comment (Ctrl+S save, Esc cancel):"
        } else {
            "New comment (Ctrl+S save, Esc cancel):"
        };
        
        let input_text = if editing_index.is_some() || !new_text.is_empty() {
            new_text.to_string()
        } else {
            String::new()
        };
        
        let input_style = if comment_focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        
        let input = Paragraph::new(input_text)
            .style(input_style)
            .block(
                Block::default()
                    .title(edit_label)
                    .borders(Borders::ALL)
                    .style(Style::default().fg(if comment_focus { Color::Yellow } else { Color::DarkGray })),
            );

        frame.render_widget(input, chunks[1]);
    }
}

/// Render the list of comments
fn render_comment_list(
    frame: &mut Frame,
    comments: &[Comment],
    selected_index: usize,
    editing_index: Option<usize>,
    comment_focus: bool,
    area: Rect,
) {
    // Check if area is too small
    if area.height < 3 || area.width < 15 {
        return;
    }

    if comments.is_empty() {
        let empty_msg = Paragraph::new("No comments yet. Press 'n' to add one.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty_msg, area);
        return;
    }

    // Build comment lines
    let mut lines = Vec::new();

    // Calculate available width (accounting for borders)
    let available_width = area.width.saturating_sub(4) as usize; // 2 for borders, 2 for padding

    for (i, comment) in comments.iter().enumerate() {
        let is_selected = i == selected_index;
        let is_editing = editing_index == Some(i);

        // Skip rendering if this comment is being edited
        if is_editing {
            continue;
        }

        // Format author and date
        let author = comment.commenter.as_ref()
            .map(|c| c.username.as_str())
            .unwrap_or("Anonymous");

        let date_str = comment.created_at
            .map(|ts| format_timestamp(ts))
            .unwrap_or_else(|| "Unknown date".to_string());

        let edited = if comment.updated_at.is_some() && comment.updated_at != comment.created_at {
            " (edited)"
        } else {
            ""
        };

        // Header line: author and date
        let header_style = if is_selected && comment_focus {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        };

        let header = Line::from(vec![
            Span::styled(format!("{} - {}", author, date_str), header_style),
            Span::styled(edited, Style::default().fg(Color::DarkGray)),
        ]);
        lines.push(header);

        // Content lines with wrapping
        let content_style = if is_selected && comment_focus {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        // Wrap text to fit available width
        let wrapped_content = wrap_text(&comment.text, available_width);
        for line in wrapped_content {
            lines.push(Line::from(Span::styled(line, content_style)));
        }

        // Add spacing between comments
        lines.push(Line::from(""));
    }

    let comments_paragraph = Paragraph::new(lines);
    frame.render_widget(comments_paragraph, area);
    
    // Render selection highlight
    if comment_focus && !comments.is_empty() && selected_index < comments.len() {
        // Highlight is done via styling in the paragraph
    }
}

/// Format a Unix timestamp (milliseconds) to a readable date string
fn format_timestamp(ts: i64) -> String {
    // Convert milliseconds to seconds for chrono
    let secs = ts / 1000;
    
    // Try to convert to DateTime
    match DateTime::from_timestamp(secs, 0) {
        Some(dt) => {
            // Convert to local time and format
            let local_dt: DateTime<Local> = dt.into();
            local_dt.format("%b %d, %Y %H:%M").to_string()
        }
        None => "Unknown date".to_string(),
    }
}

/// Wrap text to fit within the given width
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    
    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + word.len() + 1 <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }
    
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    
    if lines.is_empty() {
        lines.push(String::new());
    }
    
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wrap_text() {
        let text = "This is a test comment with multiple words";
        let wrapped = wrap_text(text, 15);
        // Text wraps at word boundaries: "This is a test" | "comment with" | "multiple words"
        assert!(wrapped.len() >= 3);
        for line in &wrapped {
            assert!(line.len() <= 15);
        }
    }
    
    #[test]
    fn test_wrap_text_empty() {
        let wrapped = wrap_text("", 10);
        assert_eq!(wrapped.len(), 1);
        assert!(wrapped[0].is_empty());
    }
    
    #[test]
    fn test_format_timestamp() {
        // Test with a known timestamp: 1234567890000 ms = Feb 13, 2009
        let result = format_timestamp(1234567890000);
        assert!(result.contains("2009"));
        assert!(!result.starts_with("Day "));
    }

    #[test]
    fn test_format_timestamp_recent() {
        // Test with a more recent timestamp
        let result = format_timestamp(1700000000000); // Nov 2023
        assert!(result.contains("2023"));
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_timestamp_with_real_data() {
        // Test with various timestamp formats that ClickUp might return
        let result1 = format_timestamp(1234567890000); // Feb 13, 2009
        assert!(result1.contains("Feb"));
        assert!(result1.contains("2009"));
        
        let result2 = format_timestamp(1609459200000); // Jan 1, 2021
        assert!(result2.contains("2021"));
        
        // Test very old timestamp (near epoch)
        let result3 = format_timestamp(-1000);
        // Even negative timestamps get converted to a date near epoch
        assert!(!result3.is_empty());
    }
}
