//! Centralized theme constants for the TUI

use ratatui::style::Color;

pub struct Theme;

impl Theme {
    pub const PRIMARY: Color = Color::Cyan;
    pub const SECONDARY: Color = Color::DarkGray;
    pub const SUCCESS: Color = Color::Green;
    pub const WARNING: Color = Color::Yellow;
    pub const ERROR: Color = Color::Red;
    pub const TEXT: Color = Color::White;
    pub const TEXT_DIM: Color = Color::Gray;
    pub const BACKGROUND: Color = Color::Black;

    pub const TASK_STATUS_COMPLETE: Color = Color::Green;
    pub const TASK_STATUS_IN_PROGRESS: Color = Color::Yellow;
    pub const TASK_STATUS_TODO: Color = Color::White;
    pub const TASK_STATUS_OTHER: Color = Color::Gray;
}
