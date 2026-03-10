//! Input event handling for TUI

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Represents a TUI input event
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// A key was pressed
    Key(KeyEvent),
    /// Terminal was resized
    Resize,
    /// Quit requested
    Quit,
    /// No event
    None,
}

/// Check if quit was requested (Ctrl+Q only)
pub fn is_quit(key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('q') if key.modifiers == KeyModifiers::CONTROL => true,
        KeyCode::Char('Q') => true,
        _ => false,
    }
}
