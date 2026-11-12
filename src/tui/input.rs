//! Input event handling for TUI

use crossterm::event::KeyEvent;

/// Represents a TUI input event
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// A key was pressed
    Key(KeyEvent),
    /// Terminal was resized
    Resize,
    /// No event
    None,
}

// Quit is now the `SPC q` sequence: SPACE opens the transient SPACEmenu
// (only in normal mode -- text-input/insert contexts consume SPACE as text),
// and `q` at the menu root quits immediately (the session is saved on exit).
// There is deliberately no single-key quit and no confirmation dialog.
