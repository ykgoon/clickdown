//! Sidebar navigation component

use iced::{
    Element, Length, Color,
    widget::{self, Button, Column, Container, Scrollable, Text},
};

use crate::models::{Workspace, ClickUpSpace, Folder, List};
use crate::app::Message;
use crate::ui::screen_id_overlay;

/// Sidebar state
#[derive(Debug, Clone)]
pub struct State {
    /// All workspaces
    pub workspaces: Vec<Workspace>,

    /// Selected workspace
    pub selected_workspace: Option<Workspace>,

    /// All spaces in selected workspace
    pub spaces: Vec<ClickUpSpace>,

    /// Selected space
    pub selected_space: Option<ClickUpSpace>,

    /// All folders in selected space
    pub folders: Vec<Folder>,

    /// Selected folder
    pub selected_folder: Option<Folder>,

    /// All lists in selected folder/space
    pub lists: Vec<List>,

    /// Selected list
    pub selected_list: Option<List>,

    /// Sidebar collapsed state
    pub collapsed: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            workspaces: vec![],
            selected_workspace: None,
            spaces: vec![],
            selected_space: None,
            folders: vec![],
            selected_folder: None,
            lists: vec![],
            selected_list: None,
            collapsed: false,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the sidebar
pub fn view(state: &State) -> Element<'_, Message> {
    let width = if state.collapsed { 50.0 } else { 280.0 };

    let mut content = Column::new()
        .push(header(state))
        .push(widget::Space::with_height(Length::Fixed(16.0)))
        .push(workspaces_section(state))
        .push(widget::Space::with_height(Length::Fixed(16.0)));

    // Show navigation hierarchy if workspace is selected
    if state.selected_workspace.is_some() && !state.collapsed {
        content = content
            .push(spaces_section(state))
            .push(widget::Space::with_height(Length::Fixed(16.0)))
            .push(folders_section(state))
            .push(widget::Space::with_height(Length::Fixed(16.0)))
            .push(lists_section(state));
    }

    // Add logout button at bottom
    content = content
        .push(widget::Space::with_height(Length::Fill))
        .push(logout_button());

    let main_content = Container::new(Scrollable::new(content))
        .width(Length::Fixed(width))
        .height(Length::Fill);

    // Wrap with screen ID overlay
    screen_id_overlay::with_overlay(main_content.into(), "sidebar")
}

fn header(state: &State) -> Element<'_, Message> {
    let title = if state.collapsed {
        Text::new("CD").size(20)
    } else {
        Text::new("ClickDown").size(20)
    };

    Container::new(title)
        .padding(16)
        .into()
}

fn workspaces_section(state: &State) -> Element<'_, Message> {
    if state.collapsed {
        return widget::Space::with_height(Length::Fixed(0.0)).into();
    }

    let mut section = Column::new()
        .push(section_header("WORKSPACES"));

    for workspace in &state.workspaces {
        let is_selected = state.selected_workspace
            .as_ref()
            .map(|w| w.id == workspace.id)
            .unwrap_or(false);

        let label = if is_selected {
            format!("● {}", &workspace.name)
        } else {
            workspace.name.clone()
        };

        let btn = Button::new(Text::new(label))
            .padding([8, 16])
            .width(Length::Fill)
            .on_press(Message::WorkspaceSelected(workspace.clone()));

        section = section.push(btn);
    }

    section.into()
}

fn spaces_section(state: &State) -> Element<'_, Message> {
    if state.collapsed || state.spaces.is_empty() {
        return widget::Space::with_height(Length::Fixed(0.0)).into();
    }

    let mut section = Column::new()
        .push(section_header("SPACES"));

    for space in &state.spaces {
        let is_selected = state.selected_space
            .as_ref()
            .map(|s| s.id == space.id)
            .unwrap_or(false);

        let label = if is_selected {
            format!("● {}", &space.name)
        } else {
            space.name.clone()
        };

        let btn = Button::new(Text::new(label))
            .padding([8, 16])
            .width(Length::Fill)
            .on_press(Message::SpaceSelected(space.clone()));

        section = section.push(btn);
    }

    section.into()
}

fn folders_section(state: &State) -> Element<'_, Message> {
    if state.collapsed || state.folders.is_empty() {
        return widget::Space::with_height(Length::Fixed(0.0)).into();
    }

    let mut section = Column::new()
        .push(section_header("FOLDERS"));

    for folder in &state.folders {
        let is_selected = state.selected_folder
            .as_ref()
            .map(|f| f.id == folder.id)
            .unwrap_or(false);

        let label = if is_selected {
            format!("● 📁 {}", &folder.name)
        } else {
            format!("📁 {}", &folder.name)
        };

        let btn = Button::new(Text::new(label))
            .padding([8, 16])
            .width(Length::Fill)
            .on_press(Message::FolderSelected(folder.clone()));

        section = section.push(btn);
    }

    section.into()
}

fn lists_section(state: &State) -> Element<'_, Message> {
    if state.collapsed || state.lists.is_empty() {
        return widget::Space::with_height(Length::Fixed(0.0)).into();
    }

    let mut section = Column::new()
        .push(section_header("LISTS"));

    for list in &state.lists {
        let is_selected = state.selected_list
            .as_ref()
            .map(|l| l.id == list.id)
            .unwrap_or(false);

        let label = if is_selected {
            format!("● 📋 {}", &list.name)
        } else {
            format!("📋 {}", &list.name)
        };

        let btn = Button::new(Text::new(label))
            .padding([8, 16])
            .width(Length::Fill)
            .on_press(Message::ListSelected(list.clone()));

        section = section.push(btn);
    }

    section.into()
}

fn section_header(title: &str) -> Element<'_, Message> {
    Text::new(title)
        .size(12)
        .color(Color::from_rgb(0.5, 0.5, 0.5))
        .into()
}

fn logout_button() -> Element<'static, Message> {
    Button::new(Text::new("Logout"))
        .padding([8, 16])
        .width(Length::Fill)
        .on_press(Message::Logout)
        .into()
}
