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

/// Truncate text to at most `max_chars` characters without splitting UTF-8 sequences.
pub fn truncate_chars(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_chars_multibyte() {
        assert_eq!(truncate_chars("日本語のテキスト", 3), "日本語");
        assert_eq!(truncate_chars("a😀bc", 2), "a😀");
        assert_eq!(truncate_chars("short", 10), "short");
    }
}
