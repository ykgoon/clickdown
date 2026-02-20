//! Reusable UI components

use iced::{
    Length, Color,
    widget::{self, Button, Text, TextInput},
};

/// Create a primary button
pub fn button<'a, Message>(label: &'a str) -> Button<'a, Message>
where
    Message: 'static + Clone,
{
    Button::new(Text::new(label))
        .padding([8, 16])
}

/// Create a secondary styled button
pub fn button_secondary<'a, Message>(label: &'a str) -> Button<'a, Message>
where
    Message: 'static + Clone,
{
    Button::new(Text::new(label))
        .padding([8, 16])
}

/// Create a danger styled button
pub fn button_danger<'a, Message>(label: &'a str) -> Button<'a, Message>
where
    Message: 'static + Clone,
{
    Button::new(Text::new(label))
        .padding([8, 16])
}

/// Create a text input field
pub fn input<'a, Message>(placeholder: &'a str, value: &'a str) -> TextInput<'a, Message>
where
    Message: 'static + Clone,
{
    TextInput::new(placeholder, value)
        .padding(8)
        .width(Length::Fill)
}

/// Parse a hex color string to iced::Color
pub fn parse_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;

    Some(Color::from_rgb(r, g, b))
}
