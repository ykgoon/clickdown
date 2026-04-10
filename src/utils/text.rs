//! Text formatting utilities for ClickDown

use chrono::{DateTime, Local};

/// Format a Unix timestamp (milliseconds) to a readable date string
pub fn format_timestamp(ts: i64) -> String {
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
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + word.len() < width {
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
