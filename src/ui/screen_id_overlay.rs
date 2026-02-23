//! Screen ID overlay component
//!
//! Displays a small, unobtrusive screen ID in the bottom-left corner

use iced::{
    Element, Length, Color, Alignment,
    widget::{Container, Text, Stack},
};

use crate::screen_id::generate_screen_id;

/// Screen ID overlay state
#[derive(Debug, Clone)]
pub struct State {
    /// The screen identifier used for ID generation
    screen_name: String,
    /// The generated screen ID
    screen_id: String,
}

impl State {
    /// Create a new screen ID overlay state for a specific screen
    pub fn new(screen_name: &str) -> Self {
        let screen_id = generate_screen_id(screen_name);
        Self {
            screen_name: screen_name.to_string(),
            screen_id,
        }
    }

    /// Get the current screen ID
    pub fn id(&self) -> &str {
        &self.screen_id
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            screen_name: String::new(),
            screen_id: String::new(),
        }
    }
}

/// Render the screen ID overlay element
///
/// Creates a container with the screen ID displayed in small, low-contrast text.
pub fn overlay_element<'a, Message: 'a>(screen_id: &str) -> Element<'a, Message> {
    // Low contrast gray color for unobtrusive display
    let gray_color = Color::from_rgb(0.4, 0.4, 0.4);
    
    let id_text = Text::new(format!("ID: {}", screen_id))
        .size(11)
        .color(gray_color);

    Container::new(id_text)
        .padding(8)
        .into()
}

/// Wrap content with a screen ID overlay at the bottom-left
///
/// # Arguments
///
/// * `content` - The main content to wrap
/// * `screen_name` - Identifier for the screen (e.g., "auth", "task-list")
///
/// # Returns
///
/// A stack containing the content with the screen ID overlay
pub fn with_overlay<'a, Message: 'a>(
    content: Element<'a, Message>,
    screen_name: &str,
) -> Element<'a, Message> {
    let screen_id = generate_screen_id(screen_name);
    let overlay = overlay_element::<Message>(&screen_id);

    Stack::new()
        .push(content)
        .push(
            Container::new(overlay)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Start)
                .align_y(Alignment::End)
        )
        .into()
}
