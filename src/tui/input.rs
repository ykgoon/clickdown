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
// and `q` at the menu root requests quit via the ConfirmQuit dialog.
// There is deliberately no single-key quit.
