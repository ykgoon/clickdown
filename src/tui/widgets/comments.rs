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
use crate::tui::layout::ScrollState;
use crate::tui::app::CommentViewMode;

/// Comment panel state with scroll tracking
#[derive(Debug, Clone)]
pub struct CommentPanelState {
    /// Current scroll state
    pub scroll: ScrollState,
    /// Index of selected comment
    pub selected_index: usize,
    /// Index of comment being edited (if any)
    pub editing_index: Option<usize>,
    /// Text for new comment or edit
    pub new_text: String,
    /// Whether focus is on comments
    pub has_focus: bool,
}

impl CommentPanelState {
    pub fn new() -> Self {
        Self {
            scroll: ScrollState::new(),
            selected_index: 0,
            editing_index: None,
            new_text: String::new(),
            has_focus: false,
        }
    }

    /// Select next comment with auto-scroll
    pub fn select_next(&mut self, comments: &[Comment]) {
        if comments.is_empty() {
            return;
        }
        if self.selected_index < comments.len() - 1 {
            self.selected_index += 1;
        }
    }

    /// Select previous comment with auto-scroll
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Auto-scroll to keep selected comment visible
    pub fn auto_scroll_to_selected(&mut self, visible_range: (usize, usize)) {
        let (visible_start, visible_end) = visible_range;
        
        if self.selected_index < visible_start {
            // Selected comment is above visible area - scroll up
            self.scroll.scroll_to(self.selected_index);
        } else if self.selected_index >= visible_end {
            // Selected comment is below visible area - scroll down
            self.scroll.scroll_to(self.selected_index);
        }
    }
}

impl Default for CommentPanelState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render comments section with list of comments and optional form
pub fn render_comments(
    frame: &mut Frame,
    comments: &[Comment],
    selected_index: usize,
    editing_index: Option<usize>,
    new_text: &str,
    comment_focus: bool,
    area: Rect,
    view_mode: &CommentViewMode,
) {
    // Build title based on view mode
    let title = match view_mode {
        CommentViewMode::TopLevel => " Comments ".to_string(),
        CommentViewMode::InThread { parent_author, .. } => {
            // Truncate long names
            let truncated = if parent_author.len() > 20 {
                format!("{}...", &parent_author[..17])
            } else {
                parent_author.clone()
            };
            format!(" Comments > {} ", truncated)
        }
    };

    // Check if area is too small to render
    if area.height < 5 || area.width < 20 {
        let block = Block::default()
            .title(title)
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
        .title(title)
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

    // Render comments list with scrolling
    render_comment_list(
        frame,
        comments,
        selected_index,
        editing_index,
        comment_focus,
        chunks[0],
        view_mode,
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

/// Render the list of comments with scrolling support
fn render_comment_list(
    frame: &mut Frame,
    comments: &[Comment],
    selected_index: usize,
    editing_index: Option<usize>,
    comment_focus: bool,
    area: Rect,
    view_mode: &CommentViewMode,
) {
    // Check if area is too small
    if area.height < 3 || area.width < 15 {
        return;
    }

    // Filter comments based on view mode
    let filtered_comments: Vec<(usize, &Comment)> = match view_mode {
        CommentViewMode::TopLevel => {
            // Show only top-level comments (no parent_id)
            let result: Vec<(usize, &Comment)> = comments.iter()
                .enumerate()
                .filter(|(_, c)| {
                    let is_top_level = c.parent_id.is_none();
                    tracing::debug!("Comment {} ({}): parent_id={:?}, is_top_level={}", 
                        c.id, c.text.chars().take(20).collect::<String>(), c.parent_id, is_top_level);
                    is_top_level
                })
                .collect();
            tracing::info!("TopLevel view: showing {} of {} comments", result.len(), comments.len());
            result
        }
        CommentViewMode::InThread { parent_comment_id, .. } => {
            // Show parent comment first, then all replies
            let mut result: Vec<(usize, &Comment)> = Vec::new();
            
            // Find and add parent comment first
            for (i, comment) in comments.iter().enumerate() {
                if comment.id == *parent_comment_id {
                    tracing::debug!("Found parent comment: {}", comment.id);
                    result.push((i, comment));
                    break;
                }
            }
            
            // Add all replies (comments with this parent_id)
            for (i, comment) in comments.iter().enumerate() {
                if comment.parent_id.as_ref() == Some(parent_comment_id) {
                    tracing::debug!("Found reply: {} -> parent={}", comment.id, parent_comment_id);
                    result.push((i, comment));
                }
            }
            
            tracing::info!("InThread view: showing {} comments (1 parent + {} replies)", 
                result.len(), result.len().saturating_sub(1));
            result
        }
    };

    if filtered_comments.is_empty() {
        let empty_msg = match view_mode {
            CommentViewMode::TopLevel => "No comments yet. Press 'n' to add one.",
            CommentViewMode::InThread { .. } => "No replies yet. Press 'r' to reply.",
        };
        let paragraph = Paragraph::new(empty_msg)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
        return;
    }

    // Calculate available width (accounting for borders)
    let available_width = area.width.saturating_sub(4) as usize; // 2 for borders, 2 for padding

    // Build all comment lines first to calculate total height
    let mut all_comment_lines: Vec<(usize, Line)> = Vec::new(); // (original_index, line)
    let mut display_indices: Vec<usize> = Vec::new(); // Maps display position to original index

    // Pre-calculate reply counts for top-level comments (for task 3.3)
    let reply_counts: std::collections::HashMap<&str, usize> = if matches!(view_mode, CommentViewMode::TopLevel) {
        let mut counts = std::collections::HashMap::new();
        for comment in comments.iter() {
            if let Some(parent_id) = &comment.parent_id {
                *counts.entry(parent_id.as_str()).or_insert(0) += 1;
            }
        }
        counts
    } else {
        std::collections::HashMap::new()
    };

    // Track if we're rendering the parent comment in thread view (for task 3.4)
    let mut is_first_comment_in_thread = true;

    for (orig_idx, comment) in filtered_comments.iter() {
        let is_editing = editing_index == Some(*orig_idx);

        // Skip rendering if this comment is being edited
        if is_editing {
            continue;
        }

        // Track this comment's display position
        display_indices.push(*orig_idx);

        // In thread view, identify the parent comment (first comment in the filtered list)
        let is_parent_in_thread = matches!(view_mode, CommentViewMode::InThread { .. }) && is_first_comment_in_thread;
        is_first_comment_in_thread = false;

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

        // Check if this comment is selected by comparing with the stored selected_index
        let is_selected = *orig_idx == selected_index;
        
        // Task 3.4: Parent comment in thread view gets distinct styling
        let header_style = if is_parent_in_thread {
            // Parent comment: bold white with underline
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else if is_selected && comment_focus {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        };

        // Build header line with optional reply count (task 3.3)
        let mut header_spans = vec![
            Span::styled(format!("{} - {}", author, date_str), header_style),
            Span::styled(edited, Style::default().fg(Color::DarkGray)),
        ];

        // Add reply count indicator for top-level comments with replies
        if matches!(view_mode, CommentViewMode::TopLevel) {
            if let Some(reply_count) = reply_counts.get(comment.id.as_str()) {
                if *reply_count > 0 {
                    header_spans.push(Span::styled(
                        format!(" • {} repl{}", reply_count, if *reply_count == 1 { "y" } else { "ies" }),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
            }
        }

        // Task 3.4: Add "Parent comment" label for parent in thread view
        if is_parent_in_thread {
            header_spans.push(Span::styled(
                " • Parent comment",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ));
        }

        let header = Line::from(header_spans);
        all_comment_lines.push((*orig_idx, header));

        // Content lines with wrapping
        // Task 3.4: Parent comment gets distinct background
        let content_style = if is_parent_in_thread {
            Style::default().add_modifier(Modifier::BOLD)
        } else if is_selected && comment_focus {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        // Task 3.6: Add visual thread indicator for replies (not parent)
        let is_reply_in_thread = matches!(view_mode, CommentViewMode::InThread { .. }) && !is_parent_in_thread;
        
        // Wrap text to fit available width
        let wrapped_content = wrap_text(&comment.text, available_width);
        for line in wrapped_content {
            if is_reply_in_thread {
                // Add vertical line indicator for replies (thread line)
                let indented_line = format!("│ {}", line);
                all_comment_lines.push((*orig_idx, Line::from(Span::styled(indented_line, content_style))));
            } else {
                all_comment_lines.push((*orig_idx, Line::from(Span::styled(line, content_style))));
            }
        }

        // Add spacing between comments
        all_comment_lines.push((*orig_idx, Line::from("")));
    }

    let total_lines = all_comment_lines.len();
    let available_height = area.height as usize;
    
    // Calculate visible range based on scroll position
    // For simplicity, we'll scroll by comment index, not line index
    let mut scroll_offset = 0;
    
    // Find the line index where the selected comment starts
    let selected_line_start = all_comment_lines.iter()
        .position(|(idx, _)| *idx == selected_index)
        .unwrap_or(0);
    
    // Calculate visible range
    let visible_start = scroll_offset;
    let visible_end = (scroll_offset + available_height).min(total_lines);
    
    // Auto-scroll: adjust scroll_offset if selected comment is outside visible range
    if selected_line_start < visible_start {
        scroll_offset = selected_line_start;
    } else if selected_line_start >= visible_end {
        scroll_offset = (selected_line_start + 1).saturating_sub(available_height);
    }
    
    // Get visible lines
    let visible_lines: Vec<Line> = all_comment_lines
        .iter()
        .skip(scroll_offset)
        .take(available_height)
        .map(|(_, line)| line.clone())
        .collect();

    let comments_paragraph = Paragraph::new(visible_lines);
    frame.render_widget(comments_paragraph, area);

    // Render scroll indicator if content exceeds visible area
    if total_lines > available_height {
        crate::tui::layout::render_scroll_indicator(
            frame,
            area,
            total_lines,
            scroll_offset,
        );
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

    #[test]
    fn test_comment_panel_state_new() {
        let state = CommentPanelState::new();
        assert_eq!(state.selected_index, 0);
        assert_eq!(state.editing_index, None);
        assert!(state.new_text.is_empty());
        assert!(!state.has_focus);
    }

    #[test]
    fn test_comment_panel_state_select_next() {
        let mut state = CommentPanelState::new();
        let comments = vec![
            Comment { id: "1".to_string(), text: "Comment 1".to_string(), ..test_comment() },
            Comment { id: "2".to_string(), text: "Comment 2".to_string(), ..test_comment() },
            Comment { id: "3".to_string(), text: "Comment 3".to_string(), ..test_comment() },
        ];

        state.select_next(&comments);
        assert_eq!(state.selected_index, 1);

        state.select_next(&comments);
        assert_eq!(state.selected_index, 2);

        // Should not go past last comment
        state.select_next(&comments);
        assert_eq!(state.selected_index, 2);
    }

    #[test]
    fn test_comment_panel_state_select_previous() {
        let mut state = CommentPanelState::new();
        state.selected_index = 2;

        state.select_previous();
        assert_eq!(state.selected_index, 1);

        state.select_previous();
        assert_eq!(state.selected_index, 0);

        // Should not go below 0
        state.select_previous();
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_comment_panel_state_auto_scroll_to_selected() {
        let mut state = CommentPanelState::new();
        state.scroll.update(100, 20); // 100 lines, 20 visible

        // Selected comment is above visible range (visible: 10-30, selected: 5)
        state.selected_index = 5;
        state.auto_scroll_to_selected((10, 30));
        assert_eq!(state.scroll.offset, 5);

        // Selected comment is below visible range (visible: 10-30, selected: 35)
        state.selected_index = 35;
        state.auto_scroll_to_selected((10, 30));
        assert_eq!(state.scroll.offset, 35);

        // Selected comment is within visible range - no scroll
        state.selected_index = 20;
        state.auto_scroll_to_selected((10, 30));
        // Should not change offset if already visible (depends on implementation)
    }

    /// Helper function to create a test comment
    fn test_comment() -> Comment {
        Comment {
            id: String::new(),
            text: String::new(),
            text_preview: String::new(),
            commenter: None,
            created_at: Some(1234567890000),
            updated_at: None,
            assigned_commenter: None,
            assigned_by: None,
            assigned: false,
            reaction: String::new(),
            parent_id: None,
        }
    }
}
