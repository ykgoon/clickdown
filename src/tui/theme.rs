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
}
