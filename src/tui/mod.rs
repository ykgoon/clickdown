//! TUI module for terminal-based user interface

pub mod terminal;
pub mod app;
pub mod widgets;
pub mod layout;
pub mod input;

pub use app::TuiApp;
pub use layout::TuiLayout;
pub use input::InputEvent;
