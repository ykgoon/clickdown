//! Task detail panel component

use iced::{
    Element, Length,
    widget::{Button, Column, Container, Row, Scrollable, Space, Text, TextInput},
};
use chrono::DateTime;

use crate::models::Task;
use crate::app::Message;
use crate::ui::screen_id_overlay;

/// Task detail panel state
#[derive(Debug, Clone)]
pub struct State {
    /// The task being viewed/edited
    pub task: Task,

    /// Whether we're creating a new task
    pub creating: bool,

    /// List ID for new tasks
    pub list_id: Option<String>,

    /// Edited name
    pub edited_name: String,

    /// Edited description
    pub edited_description: String,

    /// Selected status
    pub selected_status: Option<String>,

    /// Selected priority
    pub selected_priority: Option<String>,
}

impl State {
    pub fn new(task: Task) -> Self {
        Self {
            edited_name: task.name.clone(),
            edited_description: task.description.clone().unwrap_or_default(),
            selected_status: task.status.as_ref().map(|s| s.status.clone()),
            selected_priority: task.priority.as_ref().map(|p| p.priority.clone()),
            task,
            creating: false,
            list_id: None,
        }
    }

    pub fn new_for_create(list_id: String) -> Self {
        Self {
            task: Task {
                id: String::new(),
                name: String::new(),
                description: None,
                status: None,
                orderindex: None,
                content: None,
                created_at: None,
                updated_at: None,
                closed_at: None,
                creator: None,
                assignees: vec![],
                checklists: vec![],
                tags: vec![],
                parent: None,
                priority: None,
                due_date: None,
                start_date: None,
                points: None,
                custom_fields: vec![],
                attachments: vec![],
                list: None,
                folder: None,
                space: None,
                url: None,
                time_estimate: None,
                time_spent: None,
            },
            edited_name: String::new(),
            edited_description: String::new(),
            selected_status: None,
            selected_priority: None,
            creating: true,
            list_id: Some(list_id),
        }
    }
}

/// Render the task detail panel
pub fn view(state: &State) -> Element<'_, Message> {
    let mut content = Column::new()
        .push(header(state))
        .push(Space::with_height(Length::Fixed(24.0)))
        .push(name_field(state))
        .push(Space::with_height(Length::Fixed(16.0)))
        .push(properties_row(state))
        .push(Space::with_height(Length::Fixed(24.0)))
        .push(description_section(state));

    // Add action buttons for existing tasks
    if !state.creating {
        content = content
            .push(Space::with_height(Length::Fixed(24.0)))
            .push(action_buttons(state));
    }

    let main_content = Container::new(Scrollable::new(content))
        .width(Length::Fixed(450.0))
        .height(Length::Fill);

    // Wrap with screen ID overlay
    screen_id_overlay::with_overlay(main_content.into(), "task-detail")
}

fn header(state: &State) -> Element<'_, Message> {
    let title = if state.creating {
        Text::new("New Task").size(20)
    } else {
        Text::new("Task Details").size(20)
    };

    let close_btn = Button::new(Text::new("✕"))
        .padding([8, 12])
        .on_press(Message::CloseTaskDetail);

    Row::new()
        .push(title)
        .push(Space::with_width(Length::Fill))
        .push(close_btn)
        .into()
}

fn name_field(state: &State) -> Element<'_, Message> {
    Column::new()
        .push(Text::new("Task Name").size(14))
        .push(Space::with_height(Length::Fixed(8.0)))
        .push(
            TextInput::new("Enter task name...", &state.edited_name)
                .padding(12)
                .size(14)
                .on_input(Message::TaskNameChanged)
        )
        .into()
}

fn properties_row(state: &State) -> Element<'_, Message> {
    Row::new()
        .push(status_dropdown(state))
        .push(Space::with_width(Length::Fixed(16.0)))
        .push(priority_dropdown(state))
        .push(Space::with_width(Length::Fixed(16.0)))
        .push(due_date_field(state))
        .into()
}

fn status_dropdown(state: &State) -> Element<'_, Message> {
    let status = state.selected_status.as_deref().unwrap_or("Not Set");

    Column::new()
        .push(Text::new("Status").size(12))
        .push(Space::with_height(Length::Fixed(4.0)))
        .push(
            Button::new(Text::new(status))
                .padding([8, 12])
        )
        .into()
}

fn priority_dropdown(state: &State) -> Element<'_, Message> {
    let priority = state.selected_priority.as_deref().unwrap_or("None");

    Column::new()
        .push(Text::new("Priority").size(12))
        .push(Space::with_height(Length::Fixed(4.0)))
        .push(
            Button::new(Text::new(priority))
                .padding([8, 12])
        )
        .into()
}

fn due_date_field(state: &State) -> Element<'_, Message> {
    let due_text = state.task.due_date
        .map(|ts| {
            DateTime::from_timestamp(ts / 1000, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_default()
        })
        .unwrap_or_else(|| "Not set".to_string());

    Column::new()
        .push(Text::new("Due Date").size(12))
        .push(Space::with_height(Length::Fixed(4.0)))
        .push(
            Button::new(Text::new(due_text))
                .padding([8, 12])
        )
        .into()
}

fn description_section(state: &State) -> Element<'_, Message> {
    Column::new()
        .push(Text::new("Description").size(14))
        .push(Space::with_height(Length::Fixed(8.0)))
        .push(
            Container::new(
                Scrollable::new(
                    Text::new(&state.edited_description)
                        .size(14)
                )
            )
            .height(Length::Fixed(200.0))
            .padding(12)
        )
        .into()
}

fn action_buttons(state: &State) -> Element<'_, Message> {
    let save_btn = Button::new(
        Row::new()
            .push(Text::new(if state.creating { "Create" } else { "Save" }))
    )
    .padding([12, 24])
    .width(Length::Fill)
    .on_press(Message::SaveTask);

    let delete_btn = if !state.creating {
        Button::new(Text::new("Delete"))
            .padding([12, 24])
            .on_press(Message::DeleteTask(state.task.id.clone()))
    } else {
        Button::new(Text::new("Cancel"))
            .padding([12, 24])
            .on_press(Message::CloseTaskDetail)
    };

    Row::new()
        .push(save_btn)
        .push(Space::with_width(Length::Fixed(12.0)))
        .push(delete_btn)
        .into()
}
