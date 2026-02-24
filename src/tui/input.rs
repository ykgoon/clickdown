//! Input event handling for TUI

use crossterm::event::{self, Event, KeyEvent, KeyCode, KeyModifiers};
use std::time::Duration;
use anyhow::Result;

/// Represents a TUI input event
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// A key was pressed
    Key(KeyEvent),
    /// Terminal was resized
    Resize(u16, u16),
    /// Tick event for rendering
    Tick,
    /// Quit requested
    Quit,
    /// No event
    None,
}

/// Poll for events with a timeout
pub fn poll(timeout: Duration) -> Result<bool> {
    Ok(event::poll(timeout)?)
}

/// Read the next event
pub fn read() -> Result<InputEvent> {
    let event = event::read()?;
    
    match event {
        Event::Key(key) => Ok(InputEvent::Key(key)),
        Event::Resize(width, height) => Ok(InputEvent::Resize(width, height)),
        _ => Ok(InputEvent::None),
    }
}

/// Check if quit was requested (Ctrl+Q only)
pub fn is_quit(key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('q') if key.modifiers == KeyModifiers::CONTROL => true,
        KeyCode::Char('Q') => true,
        _ => false,
    }
}

/// Check if escape was pressed
pub fn is_escape(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Esc)
}

/// Check if enter was pressed
pub fn is_enter(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Enter)
}
