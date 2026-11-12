//! Shared single/multi-line text editing primitives.
//!
//! Buffers are plain `String`s with a cursor stored as a **character index**
//! (not a byte index), which every widget already uses. The helpers here
//! implement readline/bash-like editing (word deletion, home/end, kill,
//! word movement) and a cursor-following renderer so the caret can never be
//! pushed outside the visible part of a text box.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Paragraph},
    Frame,
};

/// Convert a character index into a byte index, clamping to the string length.
pub fn char_to_byte(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(byte_index, _)| byte_index)
        .unwrap_or(text.len())
}

/// Number of characters in the buffer.
pub fn char_len(text: &str) -> usize {
    text.chars().count()
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Byte offsets of the line containing `cursor`.
fn line_bounds(text: &str, cursor: usize) -> (usize, usize) {
    let chars: Vec<char> = text.chars().collect();
    let cursor = cursor.min(chars.len());
    let mut start = 0usize;
    for (i, c) in chars.iter().enumerate().take(cursor) {
        if *c == '\n' {
            start = i + 1;
        }
    }
    let mut end = chars.len();
    for (i, c) in chars.iter().enumerate().skip(cursor) {
        if *c == '\n' {
            end = i;
            break;
        }
    }
    (start, end)
}

/// Insert a character at the cursor and advance it.
pub fn insert_char(text: &mut String, cursor: &mut usize, c: char) {
    let byte = char_to_byte(text, *cursor);
    text.insert(byte, c);
    *cursor += 1;
}

/// Insert a whole string (e.g. a clipboard paste) at the cursor.
pub fn insert_str(text: &mut String, cursor: &mut usize, s: &str) {
    let byte = char_to_byte(text, *cursor);
    text.insert_str(byte, s);
    *cursor += s.chars().count();
}

/// Remove the character before the cursor.
pub fn backspace(text: &mut String, cursor: &mut usize) {
    if *cursor == 0 {
        return;
    }
    let start = char_to_byte(text, *cursor - 1);
    let end = char_to_byte(text, *cursor);
    text.replace_range(start..end, "");
    *cursor -= 1;
}

/// Remove the character under the cursor (Delete key).
pub fn delete_forward(text: &mut String, cursor: &mut usize) {
    if *cursor >= char_len(text) {
        return;
    }
    let start = char_to_byte(text, *cursor);
    let end = char_to_byte(text, *cursor + 1);
    text.replace_range(start..end, "");
}

/// Remove a range of characters `start..end` (char indices).
fn delete_range(text: &mut String, cursor: &mut usize, start: usize, end: usize) {
    if start >= end {
        return;
    }
    let bstart = char_to_byte(text, start);
    let bend = char_to_byte(text, end);
    text.replace_range(bstart..bend, "");
    *cursor = start;
}

/// Delete the word before the cursor (Ctrl+W / Ctrl+Backspace / Alt+Backspace).
pub fn delete_word_back(text: &mut String, cursor: &mut usize) {
    if *cursor == 0 {
        return;
    }
    let chars: Vec<char> = text.chars().collect();
    let mut start = *cursor;
    // Skip trailing non-word characters (spaces, punctuation) first.
    while start > 0 && !is_word_char(chars[start - 1]) {
        start -= 1;
    }
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }
    delete_range(text, cursor, start, *cursor);
}

/// Delete the word after the cursor (Alt+D / Ctrl+Delete).
pub fn delete_word_forward(text: &mut String, cursor: &mut usize) {
    let chars: Vec<char> = text.chars().collect();
    if *cursor >= chars.len() {
        return;
    }
    let mut end = *cursor;
    while end < chars.len() && !is_word_char(chars[end]) {
        end += 1;
    }
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }
    delete_range(text, cursor, *cursor, end);
}

/// Move the cursor one word to the left.
pub fn move_word_left(text: &str, cursor: &mut usize) {
    let chars: Vec<char> = text.chars().collect();
    while *cursor > 0 && !is_word_char(chars[*cursor - 1]) {
        *cursor -= 1;
    }
    while *cursor > 0 && is_word_char(chars[*cursor - 1]) {
        *cursor -= 1;
    }
}

/// Move the cursor one word to the right.
pub fn move_word_right(text: &str, cursor: &mut usize) {
    let chars: Vec<char> = text.chars().collect();
    while *cursor < chars.len() && is_word_char(chars[*cursor]) {
        *cursor += 1;
    }
    while *cursor < chars.len() && !is_word_char(chars[*cursor]) {
        *cursor += 1;
    }
}

pub fn move_home(text: &str, cursor: &mut usize) {
    *cursor = line_bounds(text, *cursor).0;
}

pub fn move_end(text: &str, cursor: &mut usize) {
    *cursor = line_bounds(text, *cursor).1;
}

pub fn move_buffer_start(cursor: &mut usize) {
    *cursor = 0;
}

pub fn move_buffer_end(text: &str, cursor: &mut usize) {
    *cursor = char_len(text);
}

/// Move up/down one line, preserving the column as closely as possible.
pub fn move_vertical(text: &str, cursor: &mut usize, delta: isize) {
    let chars: Vec<char> = text.chars().collect();
    let cursor_now = *cursor;
    let (line_start, _) = line_bounds(text, cursor_now);
    let col = cursor_now - line_start;
    if delta < 0 {
        if line_start == 0 {
            return;
        }
        // Previous line spans [prev_start, line_start - 1) (excluding '\n').
        let prev_end = line_start - 1;
        let mut prev_start = 0usize;
        for (i, c) in chars.iter().enumerate().take(prev_end) {
            if *c == '\n' {
                prev_start = i + 1;
            }
        }
        *cursor = prev_start + col.min(prev_end.saturating_sub(prev_start));
    } else {
        let (_, line_end) = line_bounds(text, cursor_now);
        if line_end >= chars.len() {
            return;
        }
        let next_start = line_end + 1;
        let mut next_end = chars.len();
        for (i, c) in chars.iter().enumerate().skip(next_start) {
            if *c == '\n' {
                next_end = i;
                break;
            }
        }
        *cursor = next_start + col.min(next_end.saturating_sub(next_start));
    }
}

/// Kill from the cursor to the end of the current line (Ctrl+K).
pub fn kill_to_line_end(text: &mut String, cursor: &mut usize) {
    let (_, end) = line_bounds(text, *cursor);
    delete_range(text, cursor, *cursor, end);
}

/// Kill from the start of the current line to the cursor (Ctrl+U).
pub fn kill_to_line_start(text: &mut String, cursor: &mut usize) {
    let (start, _) = line_bounds(text, *cursor);
    delete_range(text, cursor, start, *cursor);
}

/// Apply a key event as a text-editing action.
///
/// Returns `true` when the key was consumed. Keys that belong to the widget
/// (Esc, Enter, Tab, Ctrl+S, …) return `false` so the caller keeps priority.
pub fn apply_edit_key(text: &mut String, cursor: &mut usize, key: &KeyEvent) -> bool {
    let mods = key.modifiers;
    let ctrl = mods.contains(KeyModifiers::CONTROL);
    let alt = mods.contains(KeyModifiers::ALT);

    match key.code {
        KeyCode::Char(c) => {
            if ctrl {
                match c.to_ascii_lowercase() {
                    'a' => move_home(text, cursor),
                    'e' => move_end(text, cursor),
                    'b' => move_char_left(text, cursor),
                    'f' => move_char_right(text, cursor),
                    'w' => delete_word_back(text, cursor),
                    'k' => kill_to_line_end(text, cursor),
                    'u' => kill_to_line_start(text, cursor),
                    'd' => delete_forward(text, cursor),
                    'h' => backspace(text, cursor),
                    _ => return false,
                }
                return true;
            }
            if alt {
                match c.to_ascii_lowercase() {
                    'b' => move_word_left(text, cursor),
                    'f' => move_word_right(text, cursor),
                    'd' => delete_word_forward(text, cursor),
                    '<' => move_buffer_start(cursor),
                    '>' => move_buffer_end(text, cursor),
                    _ => {}
                }
                return true;
            }
            if mods.contains(KeyModifiers::SUPER) {
                return true;
            }
            insert_char(text, cursor, c);
            true
        }
        KeyCode::Backspace => {
            if ctrl || alt {
                delete_word_back(text, cursor);
            } else {
                backspace(text, cursor);
            }
            true
        }
        KeyCode::Delete => {
            if ctrl || alt {
                delete_word_forward(text, cursor);
            } else {
                delete_forward(text, cursor);
            }
            true
        }
        KeyCode::Left => {
            if ctrl || alt {
                move_word_left(text, cursor);
            } else {
                move_char_left(text, cursor);
            }
            true
        }
        KeyCode::Right => {
            if ctrl || alt {
                move_word_right(text, cursor);
            } else {
                move_char_right(text, cursor);
            }
            true
        }
        KeyCode::Home => {
            if ctrl {
                move_buffer_start(cursor);
            } else {
                move_home(text, cursor);
            }
            true
        }
        KeyCode::End => {
            if ctrl {
                move_buffer_end(text, cursor);
            } else {
                move_end(text, cursor);
            }
            true
        }
        KeyCode::Up => {
            move_vertical(text, cursor, -1);
            true
        }
        KeyCode::Down => {
            move_vertical(text, cursor, 1);
            true
        }
        _ => false,
    }
}

pub fn move_char_left(_text: &str, cursor: &mut usize) {
    *cursor = cursor.saturating_sub(1);
}

pub fn move_char_right(text: &str, cursor: &mut usize) {
    if *cursor < char_len(text) {
        *cursor += 1;
    }
}

/// A buffer laid out in visual rows, with the cursor position resolved.
pub struct WrappedText {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
}

/// Wrap `text` at `width` columns (hard wrap, honouring explicit newlines)
/// and locate `cursor` inside the resulting visual grid.
///
/// A cursor sitting exactly on a soft-wrap boundary is placed at the start of
/// the following visual row, which is where the caret visually belongs.
pub fn wrap_with_cursor(text: &str, width: usize, cursor: usize) -> WrappedText {
    let width = width.max(1);
    let mut lines: Vec<String> = vec![String::new()];
    let mut row = 0usize;
    let mut col = 0usize;
    let mut cursor_pos: Option<(usize, usize)> = None;

    for (index, c) in text.chars().enumerate() {
        if index == cursor {
            cursor_pos = Some((row, col));
        }
        if c == '\n' {
            lines.push(String::new());
            row += 1;
            col = 0;
            continue;
        }
        if col >= width {
            lines.push(String::new());
            row += 1;
            col = 0;
            // The caret belongs here if it was pending at the wrap point.
            if index == cursor {
                cursor_pos = Some((row, col));
            }
        }
        if let Some(line) = lines.get_mut(row) {
            line.push(c);
        }
        col += 1;
    }
    if cursor_pos.is_none() {
        // Caret at the end of the buffer: keep it at the end of the last
        // visual row instead of opening an empty trailing row.
        cursor_pos = Some((row, col.min(width)));
    }

    WrappedText {
        lines,
        cursor_row: cursor_pos.map(|p| p.0).unwrap_or(0),
        cursor_col: cursor_pos.map(|p| p.1).unwrap_or(0),
    }
}

/// Render an editable buffer inside `area`, scrolling so the caret is visible.
///
/// The block is drawn first; the caret is placed with the terminal cursor so
/// it is always visible regardless of how long the buffer grows. `scroll_out`
/// receives the first visible visual row, which lets callers keep their own
/// scroll bookkeeping in sync.
pub fn render_editable(
    frame: &mut Frame,
    area: Rect,
    block: Block,
    text: &str,
    cursor: usize,
    style: Style,
) {
    let inner = block.inner(area);
    frame.render_widget(block, area);
    render_editable_area(frame, inner, text, cursor, style);
}

/// Render an editable buffer into an already-bordered `inner` area.
pub fn render_editable_area(
    frame: &mut Frame,
    inner: Rect,
    text: &str,
    cursor: usize,
    style: Style,
) {
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let wrapped = wrap_with_cursor(text, inner.width as usize, cursor);
    let visible = inner.height as usize;

    // Minimal follow-scroll: keep the caret row at the bottom edge once the
    // buffer overflows, then scroll back up when the caret moves up.
    let mut first_row = if wrapped.cursor_row >= visible {
        wrapped.cursor_row + 1 - visible
    } else {
        0
    };
    // If the caret sits alone on a trailing empty line, keep one line of
    // context above it so the box never looks empty while typing.
    let caret_line_empty = wrapped
        .lines
        .get(wrapped.cursor_row)
        .map(|line| line.is_empty())
        .unwrap_or(true);
    if caret_line_empty && first_row > 0 {
        first_row -= 1;
    }

    let lines: Vec<Line> = wrapped
        .lines
        .iter()
        .skip(first_row)
        .map(|line| Line::from(line.clone()))
        .collect();

    frame.render_widget(Paragraph::new(lines).style(style), inner);

    let cursor_y = inner.y + (wrapped.cursor_row - first_row) as u16;
    let cursor_x = inner.x + (wrapped.cursor_col as u16).min(inner.width.saturating_sub(1));
    frame.set_cursor_position((cursor_x, cursor_y));
}

/// Render a single-line editable field inside `area` (no block of its own),
/// scrolling horizontally so the caret stays visible.
pub fn render_editable_line(
    frame: &mut Frame,
    area: Rect,
    text: &str,
    cursor: usize,
    style: Style,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let width = area.width as usize;
    let chars: Vec<char> = text.chars().collect();
    let cursor = cursor.min(chars.len());

    // Horizontal window: keep the caret inside the visible slice.
    let start = if cursor >= width {
        cursor + 1 - width
    } else {
        0
    };
    let visible: String = chars.iter().skip(start).take(width).collect();

    frame.render_widget(Paragraph::new(visible).style(style), area);
    let cursor_x = area.x + (cursor - start) as u16;
    let cursor_x = cursor_x.min(area.x + area.width.saturating_sub(1));
    frame.set_cursor_position((cursor_x, area.y));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, mods)
    }

    #[test]
    fn word_deletion_matches_readline() {
        let mut text = "hello brave world".to_string();
        let mut cursor = text.chars().count();
        apply_edit_key(&mut text, &mut cursor, &key(KeyCode::Backspace, KeyModifiers::CONTROL));
        assert_eq!(text, "hello brave ");
        apply_edit_key(&mut text, &mut cursor, &key(KeyCode::Backspace, KeyModifiers::CONTROL));
        assert_eq!(text, "hello ");
    }

    #[test]
    fn home_and_end_move_within_line() {
        let mut text = "abc\ndefg\nhi".to_string();
        let mut cursor = 6; // inside "defg" (d=4,e=5,f=6,g=7,\n=8)
        apply_edit_key(&mut text, &mut cursor, &key(KeyCode::Home, KeyModifiers::NONE));
        assert_eq!(cursor, 4);
        apply_edit_key(&mut text, &mut cursor, &key(KeyCode::End, KeyModifiers::NONE));
        assert_eq!(cursor, 8);
        apply_edit_key(&mut text, &mut cursor, &key(KeyCode::Home, KeyModifiers::CONTROL));
        assert_eq!(cursor, 0);
    }

    #[test]
    fn kill_to_line_start_and_end() {
        let mut text = "first line\nsecond".to_string();
        let mut cursor = "first line\nsec".chars().count();
        apply_edit_key(&mut text, &mut cursor, &key(KeyCode::Char('u'), KeyModifiers::CONTROL));
        assert_eq!(text, "first line\nond");
        apply_edit_key(&mut text, &mut cursor, &key(KeyCode::Char('k'), KeyModifiers::CONTROL));
        assert_eq!(text, "first line\n");
    }

    #[test]
    fn cursor_wraps_to_next_visual_row() {
        // Caret exactly on a soft-wrap boundary belongs to the next row.
        let wrapped = wrap_with_cursor("abcdef", 3, 3);
        assert_eq!(wrapped.lines, vec!["abc", "def"]);
        assert_eq!(wrapped.cursor_row, 1);
        assert_eq!(wrapped.cursor_col, 0);

        // Caret at the very end stays on the last row.
        let end = wrap_with_cursor("abcdef", 3, 6);
        assert_eq!(end.lines, vec!["abc", "def"]);
        assert_eq!(end.cursor_row, 1);
        assert_eq!(end.cursor_col, 3);
    }

    #[test]
    fn vertical_movement_preserves_column() {
        let mut text = "abcdef\nxy\nlonger line".to_string();
        let mut cursor = 4; // 'e' in first line
        apply_edit_key(&mut text, &mut cursor, &key(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(cursor, 9); // clamped to the end of "xy"
        apply_edit_key(&mut text, &mut cursor, &key(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(cursor, 12); // column 2 on the third line
    }
}
