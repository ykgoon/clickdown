//! Authentication view component

use iced::{
    Element, Length, Color,
    widget::{self, Button, Column, Container, Text, TextInput},
};

use crate::app::Message;
use crate::ui::screen_id_overlay;

/// Authentication view state
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Token input value
    pub token_input: String,

    /// Whether token is being validated
    pub validating: bool,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Render the authentication view
pub fn view<'a>(state: &'a State, error: Option<&'a str>) -> Element<'a, Message> {
    let title = Text::new("ClickDown")
        .size(48);

    let subtitle = Text::new("A fast ClickUp desktop client")
        .size(16)
        .color(Color::from_rgb(0.6, 0.6, 0.6));

    let token_label = Text::new("Enter your ClickUp API Token")
        .size(14);

    let token_input = TextInput::new(
        "pk_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        &state.token_input,
    )
    .on_input(Message::TokenEntered)
    .padding(12)
    .size(14)
    .width(Length::Fixed(400.0));

    let connect_btn = Button::new(
        Text::new(if state.validating { "Connecting..." } else { "Connect" })
    )
    .padding([12, 32])
    .width(Length::Fixed(400.0));

    let mut content = Column::new()
        .push(title)
        .push(widget::Space::with_height(Length::Fixed(8.0)))
        .push(subtitle)
        .push(widget::Space::with_height(Length::Fixed(48.0)))
        .push(token_label)
        .push(widget::Space::with_height(Length::Fixed(8.0)))
        .push(token_input)
        .push(widget::Space::with_height(Length::Fixed(24.0)))
        .push(connect_btn)
        .align_x(iced::alignment::Horizontal::Center);

    // Show error if present
    if let Some(error_msg) = error {
        let error_text = Text::new(error_msg)
            .size(14)
            .color(Color::from_rgb(0.9, 0.3, 0.3));

        content = content
            .push(widget::Space::with_height(Length::Fixed(16.0)))
            .push(error_text);
    }

    // Add help text
    let help_text = Text::new("Get your token from ClickUp Settings → Apps → ClickUp API")
        .size(12)
        .color(Color::from_rgb(0.5, 0.5, 0.5));

    content = content
        .push(widget::Space::with_height(Length::Fixed(48.0)))
        .push(help_text);

    let main_content = Container::new(
        Column::new()
            .push(widget::Space::with_height(Length::Fill))
            .push(content)
            .push(widget::Space::with_height(Length::Fill))
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill);

    // Wrap with screen ID overlay
    screen_id_overlay::with_overlay(main_content.into(), "auth")
}
