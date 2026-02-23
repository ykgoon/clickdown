//! Login view component with username and password authentication

use iced::{
    Element, Length, Color,
    widget::{self, Button, Column, Container, Text, TextInput, Row},
};

use crate::app::Message;
use crate::ui::screen_id_overlay;

/// Login view state
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Username (email) input value
    pub username: String,

    /// Password input value
    pub password: String,

    /// Whether login is in progress
    pub logging_in: bool,

    /// Whether to show password (vs mask it)
    pub show_password: bool,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear credentials (used after successful login or logout)
    pub fn clear(&mut self) {
        self.username.clear();
        self.password.clear();
        self.logging_in = false;
        self.show_password = false;
    }
}

/// Render the login view
pub fn view<'a>(state: &'a State, error: Option<&'a str>) -> Element<'a, Message> {
    let title = Text::new("ClickDown")
        .size(48);

    let subtitle = Text::new("A fast ClickUp desktop client")
        .size(16)
        .color(Color::from_rgb(0.6, 0.6, 0.6));

    // Username field
    let username_label = Text::new("Email")
        .size(14);

    let username_input = TextInput::new(
        "you@example.com",
        &state.username,
    )
    .on_input(|v| Message::UsernameEntered(v))
    .padding(12)
    .size(14)
    .width(Length::Fixed(400.0));

    // Password field
    let password_label = Text::new("Password")
        .size(14);

    let password_input = TextInput::new(
        "••••••••",
        &state.password,
    )
    .secure(true)
    .on_input(|v| Message::PasswordEntered(v))
    .on_submit(Message::LoginRequested)
    .padding(12)
    .size(14)
    .width(Length::Fixed(400.0));

    // Show password toggle
    let show_password_checkbox = iced::widget::checkbox(
        "Show",
        state.show_password,
    )
    .on_toggle(|v| Message::ShowPasswordToggled(v))
    .size(14);

    let password_row = Row::new()
        .push(password_input)
        .push(widget::Space::with_width(Length::Fixed(12.0)))
        .push(show_password_checkbox)
        .spacing(12)
        .align_y(iced::alignment::Vertical::Center);

    // Login button
    let login_btn = Button::new(
        Text::new(if state.logging_in { "Logging in..." } else { "Login" })
    )
    .padding([12, 32])
    .width(Length::Fixed(400.0))
    .on_press_maybe(if state.logging_in { None } else { Some(Message::LoginRequested) });

    let mut content = Column::new()
        .push(title)
        .push(widget::Space::with_height(Length::Fixed(8.0)))
        .push(subtitle)
        .push(widget::Space::with_height(Length::Fixed(48.0)))
        .push(username_label)
        .push(widget::Space::with_height(Length::Fixed(8.0)))
        .push(username_input)
        .push(widget::Space::with_height(Length::Fixed(16.0)))
        .push(password_label)
        .push(widget::Space::with_height(Length::Fixed(8.0)))
        .push(password_row)
        .push(widget::Space::with_height(Length::Fixed(24.0)))
        .push(login_btn)
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
    let help_text = Text::new("Use your ClickUp account credentials to log in")
        .size(12)
        .color(Color::from_rgb(0.5, 0.5, 0.5));

    content = content
        .push(widget::Space::with_height(Length::Fixed(32.0)))
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
    screen_id_overlay::with_overlay(main_content.into(), "login")
}
