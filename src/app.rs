//! Main application structure using Elm architecture

use iced::{
    Element, Length, Subscription, Theme,
    widget::{Column, Container, Row, Text},
};
use anyhow::Result;
use std::sync::Arc;

use crate::api::{ClickUpClient, AuthManager};
use crate::config::ConfigManager;
use crate::models::{Workspace, ClickUpSpace, Folder, List, Task as ClickUpTask, TaskFilters, Document, Page};
use crate::ui::{
    sidebar, task_list, task_detail, auth_view, document_view,
};

/// Application state
pub struct ClickDown {
    /// Application state
    state: AppState,
    
    /// API client (initialized after auth)
    client: Option<Arc<ClickUpClient>>,
    
    /// Configuration manager
    config: ConfigManager,
    
    /// Loading state
    loading: bool,
    
    /// Error message
    error: Option<String>,
    
    /// Sidebar state
    sidebar: sidebar::State,
    
    /// Task list state
    task_list: task_list::State,

    /// Task detail panel state
    task_detail: Option<task_detail::State>,

    /// Document viewer state
    document_view: document_view::State,

    /// Authentication view state
    auth: auth_view::State,
}

/// Application state machine
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// Initializing
    Initializing,
    /// Authentication required
    Unauthenticated,
    /// Loading workspaces
    LoadingWorkspaces,
    /// Main application
    Main,
}

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    // Authentication
    TokenEntered(String),
    TokenSaved,
    AuthSuccess,
    AuthError(String),
    Logout,
    
    // Workspace/Navigation
    WorkspacesLoaded(Vec<Workspace>),
    WorkspaceSelected(Workspace),
    SpacesLoaded(Vec<ClickUpSpace>),
    SpaceSelected(ClickUpSpace),
    FoldersLoaded(Vec<Folder>),
    FolderSelected(Folder),
    ListsLoaded(Vec<List>),
    ListSelected(List),
    
    // Tasks
    TasksLoaded(Vec<ClickUpTask>),
    TaskSelected(ClickUpTask),
    TaskCreated(ClickUpTask),
    TaskUpdated(ClickUpTask),
    TaskDeleted(String),
    CreateTaskRequested,
    CloseTaskDetail,
    TaskNameChanged(String),
    SaveTask,
    DeleteTask(String),
    
    // UI
    ToggleSidebar,
    ToggleTheme,
    WindowResized(u32, u32),

    // Documents
    DocumentsLoaded(Vec<Document>),
    DocumentSelected(Document),
    PageSelected(Page),
    
    // Async operations
    Tick,
    None,
}

impl Default for ClickDown {
    fn default() -> Self {
        Self::new()
    }
}

impl ClickDown {
    pub fn new() -> Self {
        let config = ConfigManager::new().unwrap_or_default();
        let auth_manager = AuthManager::new().unwrap_or_default();
        
        let state = if auth_manager.has_token() {
            AppState::Initializing
        } else {
            AppState::Unauthenticated
        };
        
        Self {
            state,
            client: None,
            config,
            loading: false,
            error: None,
            sidebar: sidebar::State::new(),
            task_list: task_list::State::new(),
            task_detail: None,
            document_view: document_view::State::new(),
            auth: auth_view::State::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::TokenEntered(token) => {
                // Initialize client with token
                let client = Arc::new(ClickUpClient::new(token.clone()));
                self.client = Some(client.clone());
                
                // Save token
                if let Ok(auth) = AuthManager::new() {
                    let _ = auth.save_token(&token);
                }
                
                self.state = AppState::LoadingWorkspaces;
                self.loading = true;
                
                return iced::Task::perform(
                    async move {
                        client.get_workspaces().await
                    },
                    |result| match result {
                        Ok(workspaces) => Message::WorkspacesLoaded(workspaces),
                        Err(e) => Message::AuthError(e.to_string()),
                    },
                );
            }
            
            Message::TokenSaved => {
                self.state = AppState::Main;
                self.loading = false;
            }
            
            Message::AuthSuccess => {
                self.state = AppState::Main;
                self.loading = false;
            }
            
            Message::AuthError(error) => {
                self.error = Some(error);
                self.loading = false;
                self.state = AppState::Unauthenticated;
            }
            
            Message::Logout => {
                self.client = None;
                self.sidebar = sidebar::State::new();
                self.task_list = task_list::State::new();
                self.task_detail = None;
                self.state = AppState::Unauthenticated;
                
                if let Ok(auth) = AuthManager::new() {
                    let _ = auth.clear_token();
                }
            }
            
            Message::WorkspacesLoaded(workspaces) => {
                self.sidebar.workspaces = workspaces;
                self.loading = false;
                
                if let Some(ws) = self.sidebar.workspaces.first().cloned() {
                    self.sidebar.selected_workspace = Some(ws.clone());
                    return self.load_spaces(ws.id);
                }
            }
            
            Message::WorkspaceSelected(workspace) => {
                self.sidebar.selected_workspace = Some(workspace.clone());
                self.sidebar.selected_space = None;
                self.sidebar.selected_folder = None;
                self.sidebar.selected_list = None;
                self.task_list.tasks.clear();
                return self.load_spaces(workspace.id);
            }
            
            Message::SpacesLoaded(spaces) => {
                self.sidebar.spaces = spaces;
            }
            
            Message::SpaceSelected(space) => {
                self.sidebar.selected_space = Some(space.clone());
                self.sidebar.selected_folder = None;
                self.sidebar.selected_list = None;
                self.task_list.tasks.clear();
                return self.load_folders(space.id);
            }
            
            Message::FoldersLoaded(folders) => {
                self.sidebar.folders = folders;
            }
            
            Message::FolderSelected(folder) => {
                self.sidebar.selected_folder = Some(folder.clone());
                self.sidebar.selected_list = None;
                self.task_list.tasks.clear();
                return self.load_lists_in_folder(folder.id);
            }
            
            Message::ListsLoaded(lists) => {
                self.sidebar.lists = lists;
            }
            
            Message::ListSelected(list) => {
                self.sidebar.selected_list = Some(list.clone());
                self.task_list.selected_list = Some(list.clone());
                return self.load_tasks(list.id);
            }
            
            Message::TasksLoaded(tasks) => {
                self.task_list.tasks = tasks;
                self.loading = false;
            }
            
            Message::TaskSelected(task) => {
                self.task_detail = Some(task_detail::State::new(task));
            }
            
            Message::TaskCreated(task) => {
                self.task_list.tasks.insert(0, task);
                self.task_detail = None;
            }
            
            Message::TaskUpdated(updated_task) => {
                if let Some(pos) = self.task_list.tasks.iter()
                    .position(|t| t.id == updated_task.id) 
                {
                    self.task_list.tasks[pos] = updated_task;
                }
            }
            
            Message::TaskDeleted(task_id) => {
                self.task_list.tasks.retain(|t| t.id != task_id);
                self.task_detail = None;
            }
            
            Message::CreateTaskRequested => {
                if let Some(ref list) = self.task_list.selected_list {
                    self.task_detail = Some(task_detail::State::new_for_create(list.id.clone()));
                }
            }
            
            Message::CloseTaskDetail => {
                self.task_detail = None;
            }

            Message::TaskNameChanged(name) => {
                if let Some(ref mut detail) = self.task_detail {
                    detail.edited_name = name;
                }
            }

            Message::SaveTask => {
                if let Some(ref detail) = self.task_detail {
                    let client = self.client.clone();
                    let task_detail = detail.clone();
                    
                    if detail.creating {
                        // Create new task
                        return iced::Task::perform(
                            async move {
                                if let Some(_client) = client {
                                    // For now, just create a minimal task with the name
                                    // In a real implementation, you'd use the CreateTaskRequest struct
                                    let list_id = task_detail.list_id.unwrap_or_default();
                                    // This would call client.create_task() - stubbed for now
                                    tracing::info!("Would create task '{}' in list {}", task_detail.edited_name, list_id);
                                }
                                Message::CloseTaskDetail
                            },
                            |msg| msg,
                        );
                    } else {
                        // Update existing task
                        return iced::Task::perform(
                            async move {
                                if let Some(_client) = client {
                                    // This would call client.update_task() - stubbed for now
                                    tracing::info!("Would update task '{}'", task_detail.edited_name);
                                }
                                Message::CloseTaskDetail
                            },
                            |msg| msg,
                        );
                    }
                }
            }

            Message::DeleteTask(task_id) => {
                if let Some(_client) = self.client.clone() {
                    let task_id_clone = task_id.clone();
                    return iced::Task::perform(
                        async move {
                            // This would call client.delete_task() - stubbed for now
                            tracing::info!("Would delete task {}", task_id_clone);
                            Message::TaskDeleted(task_id)
                        },
                        |msg| msg,
                    );
                }
            }
            
            Message::ToggleSidebar => {
                self.sidebar.collapsed = !self.sidebar.collapsed;
            }
            
            Message::ToggleTheme => {
                // Theme toggling would go here
            }
            
            Message::WindowResized(width, height) => {
                self.config.get_mut().window_width = width;
                self.config.get_mut().window_height = height;
                let _ = self.config.save();
            }

            Message::DocumentsLoaded(docs) => {
                // Store documents in sidebar or show in a list
                tracing::info!("Loaded {} documents", docs.len());
            }

            Message::DocumentSelected(doc) => {
                self.document_view = document_view::State::with_document(doc);
            }

            Message::PageSelected(page) => {
                self.document_view.current_page = Some(page);
                if let Some(content) = &self.document_view.current_page {
                    self.document_view.rendered_content = document_view::render_markdown(
                        &content.content_markdown.clone().unwrap_or_default()
                    );
                }
            }

            Message::Tick => {
                // Periodic tick for any background operations
            }

            Message::None => {}
        }
        
        iced::Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Could add periodic refresh here
        Subscription::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.state {
            AppState::Unauthenticated => {
                auth_view::view(&self.auth, self.error.as_deref())
            }
            AppState::Initializing | AppState::LoadingWorkspaces => {
                loading_view("Loading ClickUp...")
            }
            AppState::Main => {
                self.main_view()
            }
        }
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }
    
    /// Load spaces for a workspace
    fn load_spaces(&self, team_id: String) -> iced::Task<Message> {
        let client = self.client.clone().unwrap();
        iced::Task::perform(
            async move {
                client.get_spaces(&team_id).await
            },
            |result| match result {
                Ok(spaces) => Message::SpacesLoaded(spaces),
                Err(e) => Message::AuthError(e.to_string()),
            },
        )
    }
    
    /// Load folders for a space
    fn load_folders(&self, space_id: String) -> iced::Task<Message> {
        let client = self.client.clone().unwrap();
        iced::Task::perform(
            async move {
                client.get_folders(&space_id).await
            },
            |result| match result {
                Ok(folders) => Message::FoldersLoaded(folders),
                Err(e) => Message::AuthError(e.to_string()),
            },
        )
    }
    
    /// Load lists in a folder
    fn load_lists_in_folder(&self, folder_id: String) -> iced::Task<Message> {
        let client = self.client.clone().unwrap();
        iced::Task::perform(
            async move {
                client.get_lists_in_folder(&folder_id, None).await
            },
            |result| match result {
                Ok(lists) => Message::ListsLoaded(lists),
                Err(e) => Message::AuthError(e.to_string()),
            },
        )
    }
    
    /// Load tasks in a list
    fn load_tasks(&self, list_id: String) -> iced::Task<Message> {
        let client = self.client.clone().unwrap();
        let filters = TaskFilters::default();
        iced::Task::perform(
            async move {
                client.get_tasks(&list_id, &filters).await
            },
            |result| match result {
                Ok(tasks) => Message::TasksLoaded(tasks),
                Err(e) => Message::AuthError(e.to_string()),
            },
        )
    }
    
    /// Main application view
    fn main_view(&self) -> Element<'_, Message> {
        let sidebar_view = sidebar::view(&self.sidebar);
        
        // Show document view if a document is selected, otherwise show task list
        let main_content = if self.document_view.document.is_some() {
            document_view::view(&self.document_view)
        } else {
            task_list::view(&self.task_list, self.loading)
        };

        let content = Row::new()
            .push(sidebar_view)
            .push(main_content);

        // Add task detail panel if open
        let content = if let Some(ref detail_state) = self.task_detail {
            Row::new()
                .push(content)
                .push(task_detail::view(detail_state))
        } else {
            Row::new().push(content)
        };

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn loading_view<'a, Message: 'static>(text: &'a str) -> Element<'a, Message> {
    Container::new(
        Column::new()
            .push(Text::new(text).size(16))
            .align_x(iced::alignment::Horizontal::Center)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// Run the application
pub fn run() -> Result<()> {
    iced::run(
        "ClickDown",
        ClickDown::update,
        ClickDown::view,
    )?;

    Ok(())
}
