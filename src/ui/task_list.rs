//! Task list view component

use iced::{
    Element, Length, Color,
    widget::{Button, Column, Container, Row, Scrollable, Space, Text},
};
use chrono::{DateTime, Utc};

use crate::models::{List, Task};
use crate::app::Message;

/// Task list state
#[derive(Debug, Clone)]
pub struct State {
    /// Selected list
    pub selected_list: Option<List>,

    /// Tasks in the selected list
    pub tasks: Vec<Task>,

    /// Current page for pagination
    pub page: u32,

    /// Search query
    pub search_query: String,

    /// Filter by status
    pub filter_status: Option<String>,

    /// Sort order
    pub sort_by: SortOrder,
}

/// Sort order for tasks
#[derive(Debug, Clone, Copy, Default)]
pub enum SortOrder {
    #[default]
    CreatedDesc,
    CreatedAsc,
    DueDateAsc,
    DueDateDesc,
    NameAsc,
    NameDesc,
}

impl State {
    pub fn new() -> Self {
        Self {
            selected_list: None,
            tasks: vec![],
            page: 0,
            search_query: String::new(),
            filter_status: None,
            sort_by: SortOrder::CreatedDesc,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the task list view
pub fn view(state: &State, loading: bool) -> Element<'_, Message> {
    let list_name = state.selected_list
        .as_ref()
        .map(|l| l.name.as_str())
        .unwrap_or("Select a List");

    let mut content = Column::new()
        .push(header(list_name))
        .push(Space::with_height(Length::Fixed(16.0)));

    if loading {
        content = content.push(loading_view());
    } else if state.tasks.is_empty() {
        content = content.push(empty_view());
    } else {
        content = content.push(tasks_list(state));
    }

    Container::new(Scrollable::new(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(24)
        .into()
}

fn header(list_name: &str) -> Element<'_, Message> {
    let title = Text::new(list_name)
        .size(24);

    let new_task_btn = Button::new(Text::new("+ New Task"))
        .padding([8, 16])
        .on_press(Message::CreateTaskRequested);

    Row::new()
        .push(title)
        .push(Space::with_width(Length::Fill))
        .push(new_task_btn)
        .into()
}

fn loading_view() -> Element<'static, Message> {
    Container::new(
        Column::new()
            .push(Text::new("Loading tasks...").size(16))
            .align_x(iced::alignment::Horizontal::Center)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

fn empty_view() -> Element<'static, Message> {
    Container::new(
        Column::new()
            .push(Text::new("No tasks yet").size(18))
            .push(Text::new("Click '+ New Task' to create one").size(14))
            .align_x(iced::alignment::Horizontal::Center)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

fn tasks_list(state: &State) -> Element<'_, Message> {
    let mut column = Column::new();

    for task in &state.tasks {
        column = column.push(task_row(task));
    }

    column.into()
}

fn task_row(task: &Task) -> Element<'_, Message> {
    // Status indicator
    let _status_color = task.status
        .as_ref()
        .and_then(|s| s.color.as_ref())
        .map(|c| parse_color(c))
        .flatten()
        .unwrap_or(Color::from_rgb(0.4, 0.4, 0.4));

    let status_indicator = Container::new(Space::with_width(Length::Fixed(4.0)))
        .width(Length::Fixed(4.0))
        .height(Length::Fixed(24.0));

    // Task name
    let task_name = Text::new(&task.name)
        .size(14);

    // Priority badge
    let priority = task.priority
        .as_ref()
        .map(|p| priority_badge(&p.priority))
        .unwrap_or_else(|| Space::with_width(Length::Fixed(0.0)).into());

    // Due date
    let due_date = task.due_date
        .map(format_due_date)
        .map(|d| Text::new(d).size(12).color(Color::from_rgb(0.6, 0.6, 0.6)))
        .map(Element::from)
        .unwrap_or_else(|| Space::with_width(Length::Fixed(0.0)).into());

    Row::new()
        .push(status_indicator)
        .push(Space::with_width(Length::Fixed(12.0)))
        .push(task_name)
        .push(Space::with_width(Length::Fill))
        .push(priority)
        .push(Space::with_width(Length::Fixed(16.0)))
        .push(due_date)
        .push(Space::with_width(Length::Fixed(16.0)))
        .into()
}

fn priority_badge(priority: &str) -> Element<'_, Message> {
    let (text, _color) = match priority.to_lowercase().as_str() {
        "urgent" | "highest" => ("🔥 Urgent", Color::from_rgb(0.9, 0.2, 0.2)),
        "high" => ("⬆ High", Color::from_rgb(0.9, 0.5, 0.2)),
        "normal" | "medium" => ("➡ Normal", Color::from_rgb(0.5, 0.5, 0.5)),
        "low" => ("⬇ Low", Color::from_rgb(0.3, 0.6, 0.3)),
        "lowest" => ("⬇ Lowest", Color::from_rgb(0.4, 0.7, 0.4)),
        _ => (priority, Color::from_rgb(0.5, 0.5, 0.5)),
    };

    Text::new(text).size(12).into()
}

fn format_due_date(timestamp: i64) -> String {
    let dt = DateTime::from_timestamp(timestamp / 1000, 0)
        .unwrap_or_else(Utc::now);

    dt.format("%b %d").to_string()
}

fn parse_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;

    Some(Color::from_rgb(r, g, b))
}
