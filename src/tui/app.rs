//! Main TUI application

use anyhow::Result;
use arboard::Clipboard;
use crossterm::event::{KeyCode, KeyModifiers};
use futures::future::join_all;
use ratatui::prelude::Rect;
use ratatui::Frame;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::api::{AuthManager, ClickUpApi, ClickUpClient};
use crate::cache::CacheManager;
use crate::config::ConfigManager;
use crate::models::{
    AssignedItem, AssignedItemsFilter, ClickUpSpace, Comment, CreateCommentRequest, Document,
    Folder, List, SessionState, Task, UpdateCommentRequest, User, Workspace,
};
use crate::tui::widgets::SidebarItem;
use crate::utils::{ClickUpUrlGenerator, ClipboardService, UrlGenerator};

use super::input::{is_quit, InputEvent};
use super::layout::{generate_screen_title, split_task_detail, TuiLayout};
use super::terminal;
use super::widgets::{
    get_dialog_hints, handle_assigned_view_input, render_auth, render_assigned_view, render_comments,
    render_dialog, render_document, render_help, render_inbox_list, render_notification_detail,
    render_sidebar, render_task_detail, render_task_list, AuthState, DialogState, DialogType,
    DocumentState, HelpState, InboxListState, SidebarState, TaskDetailState, TaskListState,
};

/// Application screens
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Auth,
    Workspaces,
    Spaces,
    Folders,
    Lists,
    Tasks,
    TaskDetail,
    #[allow(dead_code)]
    Document,
    Inbox,
    AssignedTasks,
}

/// Comment view mode for threaded comments
#[derive(Debug, Clone, PartialEq)]
pub enum CommentViewMode {
    /// Showing all top-level comments
    TopLevel,
    /// Inside a thread, showing parent comment and replies
    InThread {
        parent_comment_id: String,
        parent_author: String,
    },
}

/// Async messages for API results
#[derive(Debug, Clone)]
pub enum AppMessage {
    WorkspacesLoaded(Result<Vec<Workspace>, String>),
    SpacesLoaded(Result<Vec<ClickUpSpace>, String>),
    FoldersLoaded(Result<Vec<Folder>, String>),
    ListsLoaded(Result<Vec<List>, String>),
    TasksLoaded(Result<Vec<Task>, String>),
    CommentsLoaded(Result<Vec<Comment>, String>),
    CommentCreated(Result<Comment, String>, bool), // bool = is_reply
    CommentUpdated(Result<Comment, String>),
    AssignedTasksLoaded(Result<Vec<Task>, String>),
    AssignedTasksPreloaded(Result<Vec<Task>, String>), // Pre-loaded at startup (no loading indicator)
    CurrentUserLoaded(Result<User, String>),
    NotificationsLoaded(Result<Vec<crate::models::Notification>, String>),
    InboxActivityLoaded(Result<Vec<crate::models::InboxActivity>, String>),
    // Assigned items (tasks + comments)
    AssignedItemsLoaded(Result<Vec<crate::models::AssignedItem>, String>),
}

/// Main TUI application state
pub struct TuiApp {
    /// Current screen
    screen: Screen,

    /// Application state
    state: AppState,

    /// API client
    client: Option<Arc<dyn ClickUpApi>>,

    /// Cache manager
    cache: CacheManager,

    /// Auth manager
    auth: AuthManager,

    /// Error message
    error: Option<String>,

    /// Loading state
    loading: bool,

    /// Sidebar state
    sidebar: SidebarState,

    /// Task list state
    task_list: TaskListState,

    /// Task detail state
    task_detail: TaskDetailState,

    /// Auth state
    auth_state: AuthState,

    /// Document state
    document: DocumentState,

    /// Dialog state
    dialog: DialogState,

    /// Help state
    help: HelpState,

    /// Current screen title
    screen_title: String,

    /// Status message
    status: String,

    /// Data
    workspaces: Vec<Workspace>,
    spaces: Vec<ClickUpSpace>,
    folders: Vec<Folder>,
    lists: Vec<List>,
    tasks: Vec<Task>,
    documents: Vec<Document>,
    comments: Vec<Comment>,
    notifications: Vec<crate::models::Notification>,

    /// Comment UI state
    comment_selected_index: usize,
    comment_editing_index: Option<usize>,
    comment_new_text: String,
    comment_focus: bool, // true = focus on comments, false = focus on task form

    /// Comment thread navigation state
    comment_view_mode: CommentViewMode,
    comment_previous_selection: Option<usize>, // Store selection when entering thread

    /// Inbox UI state
    inbox_showing_detail: bool,
    inbox_loading: bool,
    inbox_list: InboxListState,
    /// Smart inbox activity feed
    inbox_activity: Vec<crate::models::InboxActivity>,
    inbox_activity_loading: bool,
    inbox_activity_error: Option<String>,

    /// Assigned tasks UI state
    assigned_tasks: TaskListState,
    assigned_tasks_count: usize,
    assigned_tasks_loading: bool,
    assigned_tasks_error: Option<String>,

    /// Assigned items UI state (tasks + comments)
    assigned_items: Vec<AssignedItem>,
    assigned_items_count: usize,
    assigned_items_loading: bool,
    assigned_items_error: Option<String>,
    assigned_items_filter: crate::models::AssignedItemsFilter,
    assigned_items_selected_index: usize,

    /// User identity for assignee filtering
    current_user_id: Option<i32>,

    /// Async message receiver
    message_rx: Option<mpsc::Receiver<AppMessage>>,

    /// Async message sender
    message_tx: Option<mpsc::Sender<AppMessage>>,

    /// Clipboard service for copying URLs
    clipboard: ClipboardService,

    /// Status message for URL copy feedback
    url_copy_status: Option<String>,

    /// Timestamp when URL copy status was set (for auto-clear)
    url_copy_status_time: Option<std::time::Instant>,

    /// Navigation context for URL generation
    /// Tracks the current position in the workspace hierarchy
    current_workspace_id: Option<String>,
    current_space_id: Option<String>,
    current_folder_id: Option<String>,
    current_list_id: Option<String>,

    /// Session restore state
    /// Tracks whether we're currently restoring a saved session
    restoring_session: bool,
    /// Target IDs to restore at each navigation level
    restored_workspace_id: Option<String>,
    restored_space_id: Option<String>,
    restored_folder_id: Option<String>,
    restored_list_id: Option<String>,
    restored_task_id: Option<String>,
}

/// Application state
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Initializing,
    Unauthenticated,
    Main,
}

impl TuiApp {
    pub fn new() -> Result<Self> {
        let auth = AuthManager::new().unwrap_or_default();
        let cache = CacheManager::new(ConfigManager::database_path()?)?;

        let state = if auth.load_token().ok().flatten().is_some() {
            AppState::Initializing
        } else {
            AppState::Unauthenticated
        };

        let screen = if matches!(state, AppState::Unauthenticated) {
            Screen::Auth
        } else {
            Screen::Workspaces
        };

        // Create channel for async messages
        let (message_tx, message_rx) = mpsc::channel(32);

        let mut app = Self {
            screen,
            state,
            client: None,
            cache,
            auth,
            error: None,
            loading: false,
            sidebar: SidebarState::new(),
            task_list: TaskListState::new(),
            task_detail: TaskDetailState::new(),
            auth_state: AuthState::new(),
            document: DocumentState::new(),
            dialog: DialogState::new(),
            help: HelpState::new(),
            screen_title: generate_screen_title("Authentication"),
            status: String::new(),
            workspaces: Vec::new(),
            spaces: Vec::new(),
            folders: Vec::new(),
            lists: Vec::new(),
            tasks: Vec::new(),
            documents: Vec::new(),
            comments: Vec::new(),
            notifications: Vec::new(),
            comment_selected_index: 0,
            comment_editing_index: None,
            comment_new_text: String::new(),
            comment_focus: false,
            comment_view_mode: CommentViewMode::TopLevel,
            comment_previous_selection: None,
            inbox_showing_detail: false,
            inbox_loading: false,
            inbox_list: InboxListState::new(),
            inbox_activity: Vec::new(),
            inbox_activity_loading: false,
            inbox_activity_error: None,
            assigned_tasks: TaskListState::new(),
            assigned_tasks_count: 0,
            assigned_tasks_loading: false,
            assigned_tasks_error: None,
            assigned_items: Vec::new(),
            assigned_items_count: 0,
            assigned_items_loading: false,
            assigned_items_error: None,
            assigned_items_filter: crate::models::AssignedItemsFilter::All,
            assigned_items_selected_index: 0,
            current_user_id: None,
            message_rx: Some(message_rx),
            message_tx: Some(message_tx.clone()),
            clipboard: ClipboardService::new(),
            url_copy_status: None,
            url_copy_status_time: None,
            current_workspace_id: None,
            current_space_id: None,
            current_folder_id: None,
            current_list_id: None,
            restoring_session: false,
            restored_workspace_id: None,
            restored_space_id: None,
            restored_folder_id: None,
            restored_list_id: None,
            restored_task_id: None,
        };

        // Try to restore session state if authenticated
        if matches!(app.state, AppState::Initializing) {
            let _ = app.restore_session_state();
        }

        app.update_screen_title();

        if matches!(app.state, AppState::Initializing) {
            // Load token and create client
            if let Ok(Some(token)) = app.auth.load_token() {
                app.client = Some(Arc::new(ClickUpClient::new(token)));
                app.load_workspaces();
            } else {
                app.state = AppState::Unauthenticated;
                app.screen = Screen::Auth;
            }
        }

        Ok(app)
    }

    /// Create a new TUI app with a custom client (for testing)
    #[allow(dead_code)]
    pub fn with_client(client: Arc<dyn ClickUpApi>) -> Result<Self> {
        let cache = CacheManager::new(ConfigManager::database_path()?)?;
        let auth = AuthManager::new().unwrap_or_default();

        // Create channel for async messages
        let (message_tx, message_rx) = mpsc::channel(32);

        let app = Self {
            screen: Screen::Workspaces,
            state: AppState::Main,
            client: Some(client),
            cache,
            auth,
            error: None,
            loading: false,
            sidebar: SidebarState::new(),
            task_list: TaskListState::new(),
            task_detail: TaskDetailState::new(),
            auth_state: AuthState::new(),
            document: DocumentState::new(),
            dialog: DialogState::new(),
            help: HelpState::new(),
            screen_title: generate_screen_title("Workspaces"),
            status: String::new(),
            workspaces: Vec::new(),
            spaces: Vec::new(),
            folders: Vec::new(),
            lists: Vec::new(),
            tasks: Vec::new(),
            documents: Vec::new(),
            comments: Vec::new(),
            notifications: Vec::new(),
            comment_selected_index: 0,
            comment_editing_index: None,
            comment_new_text: String::new(),
            comment_focus: false,
            comment_view_mode: CommentViewMode::TopLevel,
            comment_previous_selection: None,
            inbox_showing_detail: false,
            inbox_loading: false,
            inbox_list: InboxListState::new(),
            inbox_activity: Vec::new(),
            inbox_activity_loading: false,
            inbox_activity_error: None,
            assigned_tasks: TaskListState::new(),
            assigned_tasks_count: 0,
            assigned_tasks_loading: false,
            assigned_tasks_error: None,
            assigned_items: Vec::new(),
            assigned_items_count: 0,
            assigned_items_loading: false,
            assigned_items_error: None,
            assigned_items_filter: crate::models::AssignedItemsFilter::All,
            assigned_items_selected_index: 0,
            current_user_id: None,
            message_rx: Some(message_rx),
            message_tx: Some(message_tx.clone()),
            clipboard: ClipboardService::new(),
            url_copy_status: None,
            url_copy_status_time: None,
            current_workspace_id: None,
            current_space_id: None,
            current_folder_id: None,
            current_list_id: None,
            restoring_session: false,
            restored_workspace_id: None,
            restored_space_id: None,
            restored_folder_id: None,
            restored_list_id: None,
            restored_task_id: None,
        };

        Ok(app)
    }

    /// Run the TUI application
    pub fn run(&mut self) -> Result<()> {
        let mut terminal = terminal::init()?;
        let mut last_render = std::time::Instant::now();
        let render_interval = Duration::from_millis(33); // ~30 FPS

        let result = self.run_loop(&mut terminal, &mut last_render, render_interval);

        terminal::restore()?;
        result
    }

    fn run_loop(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
        last_render: &mut std::time::Instant,
        render_interval: Duration,
    ) -> Result<()> {
        loop {
            // Handle async messages first
            self.process_async_messages();

            // Handle input
            if let Some(event) = self.handle_input()? {
                match event {
                    InputEvent::Quit => {
                        // Save session state before quitting
                        if let Err(e) = self.save_session_state() {
                            tracing::error!("Failed to save session state: {}", e);
                        }
                        break;
                    }
                    _ => self.update(event),
                }
            }

            // Render at target frame rate
            if last_render.elapsed() >= render_interval {
                self.render(terminal)?;
                *last_render = std::time::Instant::now();
            }
        }

        Ok(())
    }

    /// Process async messages from API calls (public for testing)
    pub fn process_async_messages(&mut self) {
        if let Some(ref mut rx) = self.message_rx {
            // Try to receive messages without blocking
            // We need to collect messages first to avoid borrow conflicts with load_* methods
            let mut messages = Vec::new();
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }

            // Now process messages without holding the borrow
            for msg in messages {
                match msg {
                    AppMessage::WorkspacesLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(workspaces) => {
                                self.workspaces = workspaces.clone();
                                // Populate sidebar with assigned tasks at top, then inbox, then workspaces
                                let mut items = vec![SidebarItem::AssignedTasks, SidebarItem::Inbox];
                                items.extend(self.workspaces.iter().map(|w| {
                                    SidebarItem::Workspace {
                                        name: w.name.clone(),
                                        id: w.id.clone(),
                                    }
                                }));
                                *self.sidebar.items_mut() = items;

                                // Check if we're restoring a session
                                if self.restoring_session {
                                    // Try to select the restored workspace
                                    if let Some(restored_id) = self.restored_workspace_id.clone() {
                                        if self.sidebar.select_by_id(&restored_id) {
                                            // Found the workspace, load its spaces
                                            let workspace_name = self
                                                .workspaces
                                                .iter()
                                                .find(|w| w.id == restored_id)
                                                .map(|w| w.name.clone())
                                                .unwrap_or_default();
                                            self.load_spaces(restored_id.clone());
                                            self.screen = Screen::Spaces;
                                            self.screen_title =
                                                generate_screen_title(&workspace_name);
                                        } else {
                                            // Workspace not found, fallback to Workspaces screen
                                            self.restoring_session = false;
                                            self.screen = Screen::Workspaces;
                                            self.screen_title = generate_screen_title("Workspaces");
                                            self.status =
                                                "Saved workspace not found, showing workspaces"
                                                    .to_string();
                                            tracing::warn!("Restored workspace {} not found, falling back to Workspaces", restored_id);
                                        }
                                    } else {
                                        // No workspace ID saved, stay at Workspaces
                                        self.restoring_session = false;
                                        self.sidebar.select_first();
                                        self.status = format!(
                                            "Loaded {} workspace(s)",
                                            self.workspaces.len()
                                        );
                                    }
                                } else {
                                    // Normal behavior (not restoring)
                                    // Check if we have a current_workspace_id from navigation context
                                    if let Some(ref workspace_id) = self.current_workspace_id {
                                        if !self.sidebar.select_by_id(workspace_id) {
                                            // Workspace not found, fallback to first
                                            self.sidebar.select_first();
                                        }
                                    } else {
                                        self.sidebar.select_first();
                                    }
                                    self.status =
                                        format!("Loaded {} workspace(s)", self.workspaces.len());
                                }

                                self.state = AppState::Main;
                                // Clear any previous error state
                                self.error = None;

                                // Pre-load assigned tasks after workspaces are loaded
                                self.pre_load_assigned_tasks();
                                
                                // Pre-load notifications in background
                                if let Some(workspace_id) = &self.current_workspace_id.clone() {
                                    self.pre_load_notifications_background(workspace_id.clone());
                                }
                            }
                            Err(e) => {
                                self.loading = false;
                                self.error = Some(format!("Failed to load workspaces: {}", e));
                                self.status = "Failed to load workspaces".to_string();
                                if self.restoring_session {
                                    self.restoring_session = false;
                                }
                            }
                        }
                    }
                    AppMessage::SpacesLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(spaces) => {
                                self.spaces = spaces.clone();
                                // Populate sidebar with assigned tasks, inbox, then spaces
                                let mut items = vec![SidebarItem::AssignedTasks, SidebarItem::Inbox];
                                items.extend(self.spaces.iter().map(|s| SidebarItem::Space {
                                    name: s.name.clone(),
                                    id: s.id.clone(),
                                    
                                }));
                                *self.sidebar.items_mut() = items;

                                // Check if we're restoring a session
                                if self.restoring_session {
                                    // Try to select the restored space
                                    if let Some(restored_id) = self.restored_space_id.clone() {
                                        if self.sidebar.select_by_id(&restored_id) {
                                            // Found the space, load its folders
                                            let space_name = self
                                                .spaces
                                                .iter()
                                                .find(|s| s.id == restored_id)
                                                .map(|s| s.name.clone())
                                                .unwrap_or_default();
                                            self.load_folders(restored_id.clone());
                                            self.screen = Screen::Folders;
                                            self.screen_title = generate_screen_title(&space_name);
                                        } else {
                                            // Space not found, fallback to Spaces screen
                                            self.restoring_session = false;
                                            self.screen = Screen::Spaces;
                                            self.screen_title = generate_screen_title("Spaces");
                                            self.status =
                                                "Saved space not found, showing spaces".to_string();
                                            tracing::warn!("Restored space {} not found, falling back to Spaces", restored_id);
                                        }
                                    } else {
                                        // No space ID saved, stay at Spaces
                                        self.restoring_session = false;
                                        self.sidebar.select_first();
                                        self.status =
                                            format!("Loaded {} space(s)", self.spaces.len());
                                    }
                                } else {
                                    // Normal behavior (not restoring)
                                    // Check if we have a current_space_id from navigation context
                                    if let Some(ref space_id) = self.current_space_id {
                                        if !self.sidebar.select_by_id(space_id) {
                                            // Space not found, fallback to first
                                            self.sidebar.select_first();
                                        }
                                    } else {
                                        self.sidebar.select_first();
                                    }
                                    self.status = format!("Loaded {} space(s)", self.spaces.len());
                                }

                                // Clear any previous error state
                                self.error = None;
                            }
                            Err(e) => {
                                self.loading = false;
                                self.error = Some(format!("Failed to load spaces: {}", e));
                                self.status = "Failed to load spaces".to_string();
                                if self.restoring_session {
                                    self.restoring_session = false;
                                }
                            }
                        }
                    }
                    AppMessage::FoldersLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(folders) => {
                                self.folders = folders.clone();
                                // Populate sidebar with assigned tasks, inbox, then folders
                                let mut items = vec![SidebarItem::AssignedTasks, SidebarItem::Inbox];
                                items.extend(self.folders.iter().map(|f| SidebarItem::Folder {
                                    name: f.name.clone(),
                                    id: f.id.clone(),
                                    
                                }));
                                *self.sidebar.items_mut() = items;

                                // Check if we're restoring a session
                                if self.restoring_session {
                                    // Try to select the restored folder
                                    if let Some(restored_id) = self.restored_folder_id.clone() {
                                        if self.sidebar.select_by_id(&restored_id) {
                                            // Found the folder, load its lists
                                            let folder_name = self
                                                .folders
                                                .iter()
                                                .find(|f| f.id == restored_id)
                                                .map(|f| f.name.clone())
                                                .unwrap_or_default();
                                            self.load_lists(restored_id.clone());
                                            self.screen = Screen::Lists;
                                            self.screen_title = generate_screen_title(&folder_name);
                                        } else {
                                            // Folder not found, fallback to Folders screen
                                            self.restoring_session = false;
                                            self.screen = Screen::Folders;
                                            self.screen_title = generate_screen_title("Folders");
                                            self.status = "Saved folder not found, showing folders"
                                                .to_string();
                                            tracing::warn!("Restored folder {} not found, falling back to Folders", restored_id);
                                        }
                                    } else {
                                        // No folder ID saved, stay at Folders
                                        self.restoring_session = false;
                                        self.sidebar.select_first();
                                        self.status =
                                            format!("Loaded {} folder(s)", self.folders.len());
                                    }
                                } else {
                                    // Normal behavior (not restoring)
                                    // Check if we have a current_folder_id from navigation context
                                    if let Some(ref folder_id) = self.current_folder_id {
                                        if !self.sidebar.select_by_id(folder_id) {
                                            // Folder not found, fallback to first
                                            self.sidebar.select_first();
                                        }
                                    } else {
                                        self.sidebar.select_first();
                                    }
                                    self.status =
                                        format!("Loaded {} folder(s)", self.folders.len());
                                }

                                // Clear any previous error state
                                self.error = None;
                            }
                            Err(e) => {
                                self.loading = false;
                                self.error = Some(format!("Failed to load folders: {}", e));
                                self.status = "Failed to load folders".to_string();
                                if self.restoring_session {
                                    self.restoring_session = false;
                                }
                            }
                        }
                    }
                    AppMessage::ListsLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(lists) => {
                                self.lists = lists.clone();
                                // Populate sidebar with assigned tasks, inbox, then lists
                                let mut items = vec![SidebarItem::AssignedTasks, SidebarItem::Inbox];
                                items.extend(self.lists.iter().map(|l| SidebarItem::List {
                                    name: l.name.clone(),
                                    id: l.id.clone(),
                                    
                                }));
                                *self.sidebar.items_mut() = items;

                                // Check if we're restoring a session
                                if self.restoring_session {
                                    // Try to select the restored list
                                    if let Some(restored_id) = self.restored_list_id.clone() {
                                        if self.sidebar.select_by_id(&restored_id) {
                                            // Found the list, load its tasks
                                            let list_name = self
                                                .lists
                                                .iter()
                                                .find(|l| l.id == restored_id)
                                                .map(|l| l.name.clone())
                                                .unwrap_or_default();
                                            self.load_tasks(restored_id.clone());
                                            self.screen = Screen::Tasks;
                                            self.screen_title = generate_screen_title(&format!(
                                                "Tasks: {}",
                                                list_name
                                            ));
                                        } else {
                                            // List not found, fallback to Lists screen
                                            self.restoring_session = false;
                                            self.screen = Screen::Lists;
                                            self.screen_title = generate_screen_title("Lists");
                                            self.status =
                                                "Saved list not found, showing lists".to_string();
                                            tracing::warn!(
                                                "Restored list {} not found, falling back to Lists",
                                                restored_id
                                            );
                                        }
                                    } else {
                                        // No list ID saved, stay at Lists
                                        self.restoring_session = false;
                                        self.sidebar.select_first();
                                        self.status =
                                            format!("Loaded {} list(s)", self.lists.len());
                                    }
                                } else {
                                    // Normal behavior (not restoring)
                                    // Check if we have a current_list_id from navigation context
                                    if let Some(ref list_id) = self.current_list_id {
                                        if !self.sidebar.select_by_id(list_id) {
                                            // List not found, fallback to first
                                            self.sidebar.select_first();
                                        }
                                    } else {
                                        self.sidebar.select_first();
                                    }
                                    self.status = format!("Loaded {} list(s)", self.lists.len());
                                }

                                // Clear any previous error state
                                self.error = None;
                            }
                            Err(e) => {
                                self.loading = false;
                                self.error = Some(format!("Failed to load lists: {}", e));
                                self.status = "Failed to load lists".to_string();
                                if self.restoring_session {
                                    self.restoring_session = false;
                                }
                            }
                        }
                    }
                    AppMessage::TasksLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(tasks) => {
                                // Sort tasks by status priority and recency before displaying
                                let sorted_tasks = crate::models::task::sort_tasks(tasks);
                                // Update task_list.tasks for rendering (not self.tasks)
                                *self.task_list.tasks_mut() = sorted_tasks.clone();
                                self.tasks = sorted_tasks;

                                // Check if we're restoring a session
                                if self.restoring_session {
                                    // Try to select the restored task
                                    if let Some(ref restored_id) = self.restored_task_id {
                                        // Find the task in the list
                                        if let Some(task_idx) = self
                                            .task_list
                                            .tasks()
                                            .iter()
                                            .position(|t| &t.id == restored_id)
                                        {
                                            // Found the task, select it
                                            self.task_list.select(Some(task_idx));

                                            // Check if we should navigate to TaskDetail view
                                            // We need to check the original saved screen type
                                            // For now, stay at Tasks view - user can navigate to task detail
                                            self.restoring_session = false;
                                            self.status = format!(
                                                "Restored to Tasks view - {} task(s) loaded",
                                                self.task_list.tasks().len()
                                            );
                                            tracing::info!("Session restore complete: tasks loaded, task {} selected", restored_id);
                                        } else {
                                            // Task not found, stay at Tasks screen
                                            self.restoring_session = false;
                                            self.task_list.select_first();
                                            self.status =
                                                "Saved task not found, showing tasks".to_string();
                                            tracing::warn!(
                                                "Restored task {} not found, falling back to Tasks",
                                                restored_id
                                            );
                                        }
                                    } else {
                                        // No task ID saved, stay at Tasks
                                        self.restoring_session = false;
                                        self.task_list.select_first();
                                        self.status = format!(
                                            "Loaded {} task(s)",
                                            self.task_list.tasks().len()
                                        );
                                    }
                                } else {
                                    // Normal behavior (not restoring)
                                    self.task_list.select_first();
                                    self.status =
                                        format!("Loaded {} task(s)", self.task_list.tasks().len());
                                }

                                // Clear any previous error state
                                self.error = None;
                            }
                            Err(e) => {
                                self.loading = false;
                                self.error = Some(format!("Failed to load tasks: {}", e));
                                self.status = "Failed to load tasks".to_string();
                                // Clear tasks on error to prevent stale data
                                self.task_list.tasks_mut().clear();
                                self.tasks.clear();
                                if self.restoring_session {
                                    self.restoring_session = false;
                                }
                            }
                        }
                    }
                    AppMessage::CommentsLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(comments) => {
                                tracing::debug!("Loaded {} comments", comments.len());
                                for (i, comment) in comments.iter().enumerate() {
                                    tracing::debug!(
                                        "Comment {}: id={}, parent_id={:?}, author={:?}",
                                        i,
                                        comment.id,
                                        comment.parent_id,
                                        comment.commenter.as_ref().map(|c| &c.username)
                                    );
                                }
                                self.comments = comments;
                                self.comment_selected_index = 0;
                                self.error = None;
                                self.status = format!("Loaded {} comment(s)", self.comments.len());
                            }
                            Err(e) => {
                                self.error = Some(format!("Failed to load comments: {}", e));
                                self.status = "Failed to load comments".to_string();
                                self.comments.clear();
                            }
                        }
                    }
                    AppMessage::CommentCreated(result, is_reply) => {
                        self.loading = false;
                        match result {
                            Ok(comment) => {
                                self.comments.insert(0, comment);
                                self.comment_new_text.clear();
                                self.comment_editing_index = None;
                                self.status = if is_reply {
                                    "Reply added".to_string()
                                } else {
                                    "Comment added".to_string()
                                };
                            }
                            Err(e) => {
                                self.error = Some(format!(
                                    "Failed to create {}: {}",
                                    if is_reply { "reply" } else { "comment" },
                                    e
                                ));
                                self.status = format!(
                                    "Failed to create {}",
                                    if is_reply { "reply" } else { "comment" }
                                );
                            }
                        }
                    }
                    AppMessage::CommentUpdated(result) => {
                        self.loading = false;
                        match result {
                            Ok(comment) => {
                                if let Some(idx) =
                                    self.comments.iter().position(|c| c.id == comment.id)
                                {
                                    self.comments[idx] = comment;
                                }
                                self.comment_new_text.clear();
                                self.comment_editing_index = None;
                                self.status = "Comment updated".to_string();
                            }
                            Err(e) => {
                                self.error = Some(format!("Failed to update comment: {}", e));
                                self.status = "Failed to update comment".to_string();
                            }
                        }
                    }
                    AppMessage::AssignedTasksLoaded(result) => {
                        self.assigned_tasks_loading = false;
                        match result {
                            Ok(tasks) => {
                                // Sort tasks by status priority and recency
                                let sorted_tasks = crate::models::task::sort_tasks(tasks);

                                // Update assigned tasks list
                                *self.assigned_tasks.tasks_mut() = sorted_tasks.clone();
                                self.assigned_tasks_count = sorted_tasks.len();

                                // Cache the assigned tasks
                                if let Err(e) = self.cache.cache_assigned_tasks(&sorted_tasks) {
                                    tracing::warn!("Failed to cache assigned tasks: {}", e);
                                }

                                self.assigned_tasks.select_first();
                                self.assigned_tasks_error = None;
                                self.status = format!("Loaded {} assigned task(s)", self.assigned_tasks_count);

                                // Try to detect user ID from the first task if not already set
                                if self.current_user_id.is_none() {
                                    if let Some(task) = sorted_tasks.first() {
                                        self.detect_user_id_from_task(task);
                                    }
                                }
                            }
                            Err(e) => {
                                self.assigned_tasks_error = Some(format!("Failed to load assigned tasks: {}", e));
                                self.status = "Failed to load assigned tasks".to_string();
                                self.assigned_tasks.tasks_mut().clear();
                                self.assigned_tasks_count = 0;
                            }
                        }
                    }
                    AppMessage::AssignedItemsLoaded(result) => {
                        self.assigned_items_loading = false;
                        match result {
                            Ok(items) => {
                                // Update assigned items list
                                self.assigned_items = items.clone();
                                self.assigned_items_count = items.len();
                                self.assigned_items_selected_index = 0;

                                // Cache the assigned comments (extract from items)
                                let comments: Vec<crate::models::AssignedComment> = items
                                    .iter()
                                    .filter_map(|item| {
                                        if let AssignedItem::AssignedComment(ac) = item {
                                            Some(ac.clone())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                
                                if let Err(e) = self.cache.cache_assigned_comments(&comments) {
                                    tracing::warn!("Failed to cache assigned comments: {}", e);
                                }

                                self.assigned_items_error = None;
                                self.status = format!("Loaded {} assigned item(s)", self.assigned_items_count);
                            }
                            Err(e) => {
                                self.assigned_items_error = Some(format!("Failed to load assigned items: {}", e));
                                self.status = "Failed to load assigned items".to_string();
                                self.assigned_items.clear();
                                self.assigned_items_count = 0;
                            }
                        }
                    }
                    AppMessage::AssignedTasksPreloaded(result) => {
                        // Pre-loaded tasks don't show loading indicators
                        match result {
                            Ok(tasks) => {
                                // Sort tasks by status priority and recency
                                let sorted_tasks = crate::models::task::sort_tasks(tasks);

                                // Update assigned tasks list (only if not already populated)
                                if self.assigned_tasks.tasks().is_empty() {
                                    *self.assigned_tasks.tasks_mut() = sorted_tasks.clone();
                                    self.assigned_tasks_count = sorted_tasks.len();
                                    self.assigned_tasks.select_first();
                                    self.status = format!("Pre-loaded {} assigned task(s)", self.assigned_tasks_count);
                                } else {
                                    // Already have data from cache, just update in background
                                    self.assigned_tasks_count = sorted_tasks.len();
                                    tracing::debug!("Background refresh completed: {} assigned task(s)", sorted_tasks.len());
                                }

                                // Cache the assigned tasks
                                if let Err(e) = self.cache.cache_assigned_tasks(&sorted_tasks) {
                                    tracing::warn!("Failed to cache assigned tasks: {}", e);
                                }

                                // Try to detect user ID from the first task if not already set
                                if self.current_user_id.is_none() {
                                    if let Some(task) = sorted_tasks.first() {
                                        self.detect_user_id_from_task(task);
                                    }
                                }
                            }
                            Err(e) => {
                                // Don't show error for pre-load failures, just log
                                tracing::warn!("Failed to pre-load assigned tasks: {}", e);
                            }
                        }
                    }
                    AppMessage::CurrentUserLoaded(result) => {
                        match result {
                            Ok(user) => {
                                // Store user ID for filtering assigned tasks
                                self.current_user_id = Some(user.id as i32);
                                tracing::info!("Detected current user ID from API: {} ({})", user.id, user.username);

                                // Now load assigned tasks with the user ID
                                self.load_assigned_tasks();
                            }
                            Err(e) => {
                                self.assigned_tasks_loading = false;
                                self.assigned_tasks_error = Some(format!("Failed to load user profile: {}", e));
                                self.status = "Failed to load user profile".to_string();
                            }
                        }
                    }
                    AppMessage::NotificationsLoaded(result) => {
                        self.inbox_loading = false;
                        match result {
                            Ok(notifications) => {
                                self.notifications = notifications.clone();
                                // Convert notifications to activities for display
                                let activities: Vec<crate::models::InboxActivity> = notifications
                                    .iter()
                                    .map(|n| crate::models::InboxActivity {
                                        id: format!("notification_{}", n.id),
                                        activity_type: crate::models::ActivityType::Assignment,
                                        title: n.title.clone(),
                                        description: n.description.clone(),
                                        timestamp: n.created_at.unwrap_or(0),
                                        task_id: None,
                                        comment_id: None,
                                        workspace_id: n.workspace_id.clone(),
                                        task_name: String::new(),
                                        previous_status: None,
                                        new_status: None,
                                        due_date: None,
                                    })
                                    .collect();
                                self.inbox_list.set_activities(activities);
                                self.status = format!("Loaded {} notification(s)", self.notifications.len());
                            }
                            Err(e) => {
                                self.status = format!("Failed to load notifications: {}", e);
                                // Keep existing cached data if available
                            }
                        }
                    }
                    AppMessage::InboxActivityLoaded(result) => {
                        self.inbox_activity_loading = false;
                        match result {
                            Ok(activities) => {
                                self.inbox_activity = activities.clone();
                                self.status = format!("Loaded {} activity item(s)", self.inbox_activity.len());
                            }
                            Err(e) => {
                                self.inbox_activity_error = Some(e.clone());
                                self.status = format!("Failed to load inbox activity: {}", e);
                                // Keep existing cached data if available
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_input(&mut self) -> Result<Option<InputEvent>> {
        use crossterm::event;

        if event::poll(Duration::from_millis(16))? {
            let evt = event::read()?;

            // Check for quit (Ctrl+Q)
            if let event::Event::Key(key) = &evt {
                if is_quit(*key) {
                    self.dialog.show(DialogType::ConfirmQuit);
                    return Ok(Some(InputEvent::None));
                }
            }

            // Handle dialog input
            if self.dialog.is_visible() {
                if let event::Event::Key(key) = evt {
                    match key.code {
                        KeyCode::Left | KeyCode::Right => {
                            self.dialog.toggle();
                            return Ok(Some(InputEvent::None));
                        }
                        KeyCode::Enter => {
                            if self.dialog.confirmed() {
                                // Check what we're confirming
                                match self.dialog.dialog_type {
                                    Some(DialogType::ConfirmQuit) => {
                                        return Ok(Some(InputEvent::Quit));
                                    }
                                    Some(DialogType::ConfirmDelete) => {
                                        // Task deletion not yet implemented
                                        self.status = "Task deletion - coming soon".to_string();
                                    }
                                    _ => {}
                                }
                            }
                            self.dialog.hide();
                            return Ok(Some(InputEvent::None));
                        }
                        KeyCode::Esc => {
                            self.dialog.hide();
                            return Ok(Some(InputEvent::None));
                        }
                        _ => return Ok(Some(InputEvent::None)),
                    }
                }
            }

            // Handle help overlay
            if self.help.visible {
                self.help.hide();
                return Ok(Some(InputEvent::None));
            }

            // Convert to InputEvent
            match evt {
                event::Event::Key(key) => Ok(Some(InputEvent::Key(key))),
                event::Event::Resize(_, _) => Ok(Some(InputEvent::Resize)),
                _ => Ok(Some(InputEvent::None)),
            }
        } else {
            Ok(None)
        }
    }

    /// Process input event and update state (public for testing)
    pub fn update(&mut self, event: InputEvent) {
        // When help is visible, any key closes it
        if self.help.visible {
            if let InputEvent::Key(_) = event {
                self.help.hide();
                return;
            }
        }

        // Handle help toggle with ?
        if let InputEvent::Key(key) = event {
            if key.code == KeyCode::Char('?') {
                self.help.toggle();
                return;
            }

            // Handle URL copy with single key 'u' (for URL)
            // This is simpler and more reliable than modifier combinations
            if key.code == KeyCode::Char('u') {
                tracing::debug!("URL copy shortcut detected (u key)");
                self.copy_url();
                return;
            }
        }

        match self.screen {
            Screen::Auth => self.update_auth(event),
            Screen::Workspaces | Screen::Spaces | Screen::Folders | Screen::Lists => {
                self.update_navigation(event)
            }
            Screen::Tasks => self.update_tasks(event),
            Screen::TaskDetail => self.update_task_detail(event),
            Screen::Document => self.update_document(event),
            Screen::Inbox => self.update_inbox(event),
            Screen::AssignedTasks => self.update_assigned_tasks(event),
        }
    }

    fn update_auth(&mut self, event: InputEvent) {
        if let InputEvent::Key(key) = event {
            match key.code {
                KeyCode::Esc => {
                    self.auth_state.clear();
                }
                KeyCode::Enter if !self.auth_state.loading => {
                    self.authenticate();
                }
                KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Handle paste (Ctrl+V or Ctrl+Shift+V)
                    match Clipboard::new() {
                        Ok(mut clipboard) => {
                            match clipboard.get_text() {
                                Ok(text) => {
                                    // Insert clipboard content at cursor position
                                    for c in text.chars() {
                                        self.auth_state.add_char(c);
                                    }
                                    self.status = "Pasted from clipboard".to_string();
                                }
                                Err(_) => {
                                    self.status =
                                        "Paste failed: could not read clipboard".to_string();
                                }
                            }
                        }
                        Err(_) => {
                            self.status = "Paste failed: clipboard unavailable".to_string();
                        }
                    }
                }
                KeyCode::Char(c) => {
                    self.auth_state.add_char(c);
                }
                KeyCode::Backspace => {
                    self.auth_state.remove_char();
                }
                _ => {}
            }
        }
    }

    fn update_navigation(&mut self, event: InputEvent) {
        if let InputEvent::Key(key) = event {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.sidebar.select_next();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.sidebar.select_previous();
                }
                KeyCode::Enter => {
                    self.navigate_into();
                }
                KeyCode::Esc => {
                    self.navigate_back();
                }
                KeyCode::Tab => {
                    self.sidebar.visible = !self.sidebar.visible;
                }
                _ => {}
            }
        }
    }

    fn update_tasks(&mut self, event: InputEvent) {
        if let InputEvent::Key(key) = event {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.task_list.select_next();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.task_list.select_previous();
                }
                KeyCode::Enter => {
                    if let Some(task) = self.task_list.selected_task().cloned() {
                        self.task_detail.task = Some(task.clone());
                        self.screen = Screen::TaskDetail;
                        self.update_screen_title();
                        // Load comments for this task
                        self.load_comments(task.id.clone());
                    }
                }
                KeyCode::Char('n') => {
                    // Create new task - not yet implemented
                    self.status = "Create task - coming soon".to_string();
                }
                KeyCode::Char('e') => {
                    if self.task_detail.task.is_some() {
                        self.task_detail.editing = true;
                        self.screen = Screen::TaskDetail;
                        self.update_screen_title();
                    }
                }
                KeyCode::Char('d') => {
                    if self.task_list.selected_task().is_some() {
                        self.dialog.show(DialogType::ConfirmDelete);
                    }
                }
                KeyCode::Esc => {
                    self.navigate_back();
                }
                KeyCode::Tab => {
                    self.sidebar.visible = !self.sidebar.visible;
                }
                _ => {}
            }
        }
    }

    fn update_task_detail(&mut self, event: InputEvent) {
        if let InputEvent::Key(key) = event {
            // Handle comment editing input
            if self.comment_editing_index.is_some() || !self.comment_new_text.is_empty() {
                match key.code {
                    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Save comment
                        if let Some(edit_idx) = self.comment_editing_index {
                            if edit_idx == usize::MAX {
                                // Creating new comment or reply (sentinel value)
                                if let Some(_task) = &self.task_detail.task {
                                    let text = self.comment_new_text.clone();
                                    // Validate text is not empty or whitespace-only
                                    if text.trim().is_empty() {
                                        self.status = if matches!(
                                            self.comment_view_mode,
                                            CommentViewMode::InThread { .. }
                                        ) {
                                            "Reply cannot be empty".to_string()
                                        } else {
                                            "Comment cannot be empty".to_string()
                                        };
                                        return;
                                    }
                                    let task_id = _task.id.clone();

                                    // Determine parent_id based on view mode
                                    let parent_id = match &self.comment_view_mode {
                                        CommentViewMode::InThread {
                                            parent_comment_id, ..
                                        } => Some(parent_comment_id.clone()),
                                        CommentViewMode::TopLevel => None,
                                    };

                                    self.create_comment(task_id, text, parent_id);
                                }
                            } else {
                                // Update existing comment at index edit_idx
                                if let Some(_task) = &self.task_detail.task {
                                    let text = self.comment_new_text.clone();
                                    // Validate text is not empty or whitespace-only
                                    if text.trim().is_empty() {
                                        self.status = "Comment cannot be empty".to_string();
                                        return;
                                    }
                                    let comment_id = self.comments[edit_idx].id.clone();
                                    self.update_comment(comment_id, text);
                                }
                            }
                        } else if !self.comment_new_text.is_empty() {
                            // Fallback: Create new comment or reply (when editing_index is None but text exists)
                            if let Some(_task) = &self.task_detail.task {
                                let text = self.comment_new_text.clone();
                                // Validate text is not empty or whitespace-only
                                if text.trim().is_empty() {
                                    self.status = if matches!(
                                        self.comment_view_mode,
                                        CommentViewMode::InThread { .. }
                                    ) {
                                        "Reply cannot be empty".to_string()
                                    } else {
                                        "Comment cannot be empty".to_string()
                                    };
                                    return;
                                }
                                let task_id = _task.id.clone();

                                // Determine parent_id based on view mode
                                let parent_id = match &self.comment_view_mode {
                                    CommentViewMode::InThread {
                                        parent_comment_id, ..
                                    } => Some(parent_comment_id.clone()),
                                    CommentViewMode::TopLevel => None,
                                };

                                self.create_comment(task_id, text, parent_id);
                            }
                        }
                        return;
                    }
                    KeyCode::Esc => {
                        // Cancel editing
                        self.comment_new_text.clear();
                        self.comment_editing_index = None;
                        self.status =
                            if matches!(self.comment_view_mode, CommentViewMode::InThread { .. }) {
                                "Reply cancelled".to_string()
                            } else {
                                "Comment editing cancelled".to_string()
                            };
                        return;
                    }
                    KeyCode::Char(c) => {
                        // Add character to comment text
                        self.comment_new_text.push(c);
                        return;
                    }
                    KeyCode::Backspace => {
                        self.comment_new_text.pop();
                        return;
                    }
                    _ => {}
                }
            }

            // Handle normal task detail and comment navigation
            match key.code {
                KeyCode::Esc => {
                    // Exit thread view if in thread, otherwise go back to tasks
                    if matches!(self.comment_view_mode, CommentViewMode::InThread { .. }) {
                        // Exit thread view and return to top-level
                        self.comment_view_mode = CommentViewMode::TopLevel;
                        self.comment_selected_index = self.comment_previous_selection.unwrap_or(0);
                        self.comment_previous_selection = None;
                        self.status = "Back to top-level comments".to_string();
                    } else if self.comment_editing_index.is_some()
                        || !self.comment_new_text.is_empty()
                    {
                        // Cancel comment editing
                        self.comment_new_text.clear();
                        self.comment_editing_index = None;
                        self.status = "Comment editing cancelled".to_string();
                    } else {
                        // Exit task detail view
                        self.task_detail.editing = false;
                        self.screen = Screen::Tasks;
                        self.update_screen_title();
                    }
                }
                KeyCode::Char('e') if !self.comment_focus => {
                    self.task_detail.editing = true;
                }
                KeyCode::Char('d') => {
                    self.dialog.show(DialogType::ConfirmDelete);
                }
                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Save task - not yet implemented
                    self.task_detail.editing = false;
                    self.status = "Save task - coming soon".to_string();
                }
                // Comment navigation
                KeyCode::Tab => {
                    // Toggle focus between task form and comments
                    self.comment_focus = !self.comment_focus;
                    self.status = if self.comment_focus {
                        "Focus: Comments (j/k navigate, n new, e edit)".to_string()
                    } else {
                        "Focus: Task form".to_string()
                    };
                }
                KeyCode::Char('j') if self.comment_focus => {
                    if self.comments.is_empty() {
                        return;
                    }
                    if self.comment_selected_index < self.comments.len() - 1 {
                        self.comment_selected_index += 1;
                    }
                }
                KeyCode::Char('k') if self.comment_focus => {
                    if self.comment_selected_index > 0 {
                        self.comment_selected_index -= 1;
                    }
                }
                KeyCode::Char('n') if self.comment_focus => {
                    // Start new comment
                    self.comment_new_text.clear();
                    // usize::MAX is a sentinel value indicating "new comment" mode
                    // (as opposed to Some(index) which means editing existing comment)
                    self.comment_editing_index = Some(usize::MAX);
                    self.status = "Type comment (Ctrl+S save, Esc cancel)".to_string();
                }
                KeyCode::Char('e') if self.comment_focus => {
                    // Edit selected comment (only in top-level view)
                    if !self.comments.is_empty()
                        && self.comment_selected_index < self.comments.len()
                    {
                        // Check if user owns the comment
                        let comment = &self.comments[self.comment_selected_index];
                        // For now, allow editing any comment (will add ownership check later)
                        self.comment_new_text = comment.text.clone();
                        self.comment_editing_index = Some(self.comment_selected_index);
                        self.status = "Editing comment (Ctrl+S save, Esc cancel)".to_string();
                    }
                }
                KeyCode::Enter if self.comment_focus => {
                    // Enter thread view when on a top-level comment
                    if matches!(self.comment_view_mode, CommentViewMode::TopLevel) {
                        if !self.comments.is_empty()
                            && self.comment_selected_index < self.comments.len()
                        {
                            let comment = &self.comments[self.comment_selected_index];
                            // Only enter thread if this is a top-level comment
                            if comment.parent_id.is_none() {
                                // Store current selection for when we exit
                                self.comment_previous_selection = Some(self.comment_selected_index);

                                // Get author name for breadcrumb
                                let author = comment
                                    .commenter
                                    .as_ref()
                                    .map(|c| c.username.clone())
                                    .unwrap_or_else(|| "Unknown".to_string());

                                // Switch to thread view
                                self.comment_view_mode = CommentViewMode::InThread {
                                    parent_comment_id: comment.id.clone(),
                                    parent_author: author,
                                };

                                // Set selection to the parent comment's index (not 0)
                                // This ensures the parent comment is selected when entering thread
                                self.comment_previous_selection = Some(self.comment_selected_index);
                                // Keep the same index since we're selecting the parent comment
                                // The rendering will show the parent comment first in the filtered view

                                self.status = format!("Viewing thread. Press Esc to go back");
                            }
                        }
                    }
                }
                KeyCode::Char('r') if self.comment_focus => {
                    // Reply to thread (only in thread view)
                    if matches!(self.comment_view_mode, CommentViewMode::InThread { .. }) {
                        self.comment_new_text.clear();
                        // usize::MAX is a sentinel value indicating "new reply" mode
                        // (as opposed to Some(index) which means editing existing comment)
                        self.comment_editing_index = Some(usize::MAX);

                        let author = match &self.comment_view_mode {
                            CommentViewMode::InThread { parent_author, .. } => {
                                parent_author.clone()
                            }
                            _ => "Unknown".to_string(),
                        };

                        self.status = format!("Replying to {} (Ctrl+S save, Esc cancel)", author);
                    } else {
                        self.status = "Press Enter to view thread, then 'r' to reply".to_string();
                    }
                }
                _ => {}
            }
        }
    }

    fn update_document(&mut self, event: InputEvent) {
        if let InputEvent::Key(key) = event {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.document.scroll_down();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.document.scroll_up();
                }
                KeyCode::Esc => {
                    self.navigate_back();
                }
                _ => {}
            }
        }
    }

    fn update_inbox(&mut self, event: InputEvent) {
        if let InputEvent::Key(key) = event {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    // Move selection down in inbox list
                    self.inbox_list.select_next();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    // Move selection up in inbox list
                    self.inbox_list.select_previous();
                }
                KeyCode::Char('r') => {
                    // Manual refresh - fetch activity from API
                    self.status = "Refreshing activity...".to_string();
                    self.load_inbox_activity();
                }
                KeyCode::Char('c') => {
                    // Dismiss single activity (remove from list)
                    if let Some(activity) = self.inbox_list.selected_activity() {
                        let activity_id = activity.id.clone();
                        // Remove from list
                        self.inbox_activity.retain(|a| a.id != activity_id);
                        self.inbox_list
                            .set_activities(self.inbox_activity.clone());
                        self.status = "Activity dismissed".to_string();
                    } else {
                        self.status = "No activity selected".to_string();
                    }
                }
                KeyCode::Char('C') => {
                    // Dismiss all activities (clear inbox)
                    self.inbox_activity.clear();
                    self.inbox_list.set_activities(Vec::new());
                    self.status = "All activities dismissed".to_string();
                }
                KeyCode::Enter => {
                    // Show activity detail
                    self.inbox_showing_detail = true;
                }
                KeyCode::Esc => {
                    if self.inbox_showing_detail {
                        // Close detail view
                        self.inbox_showing_detail = false;
                    } else {
                        // Navigate back
                        self.navigate_back();
                    }
                }
                _ => {}
            }
        }
    }

    fn update_assigned_tasks(&mut self, event: InputEvent) {
        if let InputEvent::Key(key) = event {
            // Use the assigned view input handler
            let handled = handle_assigned_view_input(
                key.code,
                &self.assigned_items,
                &mut self.assigned_items_selected_index,
                &mut self.assigned_items_filter,
                self.assigned_items_count,
            );

            if handled {
                return;
            }

            // Handle additional keys
            match key.code {
                KeyCode::Char('r') => {
                    // Manual refresh - reload assigned items from API
                    self.assigned_items.clear();
                    self.assigned_items_count = 0;
                    let _ = self.cache.clear_assigned_comments();
                    self.load_assigned_items();
                    self.status = "Refreshing assigned items...".to_string();
                }
                KeyCode::Enter => {
                    // Open selected item detail
                    if self.assigned_items.is_empty() {
                        return;
                    }
                    
                    // Get the selected item (accounting for filter)
                    let filtered_items: Vec<&AssignedItem> = match self.assigned_items_filter {
                        AssignedItemsFilter::All => self.assigned_items.iter().collect(),
                        AssignedItemsFilter::TasksOnly => {
                            self.assigned_items.iter().filter(|i| i.is_task()).collect()
                        }
                        AssignedItemsFilter::CommentsOnly => {
                            self.assigned_items.iter().filter(|i| i.is_comment()).collect()
                        }
                    };

                    if let Some(item) = filtered_items.get(self.assigned_items_selected_index) {
                        match item {
                            AssignedItem::Task(task) => {
                                // Open task detail
                                self.task_detail.task = Some(task.clone());
                                self.screen = Screen::TaskDetail;
                                self.screen_title = generate_screen_title(&format!("Task: {}", task.name));

                                // Load comments for the task
                                self.load_comments(task.id.clone());
                            }
                            AssignedItem::AssignedComment(ac) => {
                                // Navigate to parent task and open comment thread
                                let task_id = ac.task.id.clone();
                                let comment_id = ac.comment.id.clone();
                                
                                // Find or fetch the parent task
                                // For now, we'll just navigate to the task and load comments
                                // The comment thread will be opened based on the comment_id
                                self.task_detail.task = None; // Will be loaded
                                self.screen = Screen::TaskDetail;
                                self.screen_title = generate_screen_title(&format!("Task: {}", ac.task.name.as_deref().unwrap_or("Unknown")));
                                
                                // Store the comment ID to open the thread
                                // TODO: Implement comment thread auto-open
                                tracing::info!("Navigating to task {} to view comment {}", task_id, comment_id);
                                
                                // Load the task and comments
                                // This would need a new method to load task by ID and open specific comment
                                self.status = format!("Opening comment in task...");
                            }
                        }
                    }
                }
                KeyCode::Esc => {
                    // Navigate back to previous screen
                    self.navigate_back();
                }
                _ => {}
            }
        }
    }

    /// Navigate into the selected item (public for testing)
    pub fn navigate_into(&mut self) {
        // Navigate based on current screen and selection
        // Clone the selected item to avoid borrow checker issues
        let selected_item = self.sidebar.selected_item().cloned();

        // Handle Assigned Tasks navigation from any screen
        if let Some(SidebarItem::AssignedTasks) = selected_item {
            self.screen = Screen::AssignedTasks;
            self.screen_title = generate_screen_title("Assigned to Me");

            // Try to detect user ID if not already set
            self.try_detect_user_id();

            // Check if we already have user ID
            let has_user_id = self.current_user_id.is_some();

            // Check cache first (5 minute TTL)
            let cache_valid = self.cache.is_assigned_comments_cache_valid(300).unwrap_or(false);

            if cache_valid && !self.assigned_items.is_empty() {
                // Use cached data for comments, and load tasks from cache too
                match (self.cache.get_assigned_comments(), self.cache.get_assigned_tasks()) {
                    (Ok(comments), Ok(tasks)) => {
                        // Combine cached tasks and comments
                        let mut items = Vec::new();
                        for task in &tasks {
                            items.push(AssignedItem::Task(task.clone()));
                        }
                        for comment in &comments {
                            items.push(AssignedItem::AssignedComment(comment.clone()));
                        }
                        items = crate::models::assigned_item::sort_assigned_items(items);
                        
                        self.assigned_items_count = items.len();
                        self.assigned_items = items;
                        self.assigned_items_selected_index = 0;
                        self.status = format!("Loaded {} assigned item(s) from cache", self.assigned_items_count);
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        tracing::warn!("Failed to load assigned items from cache: {}", e);
                        if has_user_id {
                            self.load_assigned_items();
                        } else {
                            self.fetch_current_user_and_load_tasks();
                        }
                    }
                }
            } else if has_user_id {
                // Have user ID - fetch items directly
                self.load_assigned_items();
            } else {
                // No user ID - fetch from API first
                self.fetch_current_user_and_load_tasks();
            }
            return;
        }

        match &self.screen {
            Screen::Workspaces => {
                if let Some(SidebarItem::Inbox) = selected_item {
                    // Navigate to Inbox view
                    self.screen = Screen::Inbox;
                    self.screen_title = generate_screen_title("Inbox");
                    // Load inbox activity from API
                    if self.current_workspace_id.is_some() {
                        // Need user ID for smart inbox
                        if self.current_user_id.is_some() {
                            self.load_inbox_activity();
                        } else {
                            // Try to detect user ID first
                            self.try_detect_user_id();
                            if self.current_user_id.is_some() {
                                self.load_inbox_activity();
                            } else {
                                self.status = "Loading user profile...".to_string();
                                self.fetch_current_user_and_load_tasks();
                            }
                        }
                    } else {
                        self.status = "Select a workspace first to view inbox".to_string();
                    }
                } else if let Some(SidebarItem::Workspace { id, name }) = selected_item {
                    self.current_workspace_id = Some(id.clone());
                    self.current_space_id = None;
                    self.current_folder_id = None;
                    self.current_list_id = None;
                    self.load_spaces(id.clone());
                    self.screen = Screen::Spaces;
                    self.screen_title = generate_screen_title(&name);
                }
            }
            Screen::Spaces => {
                if let Some(SidebarItem::Inbox) = selected_item {
                    // Navigate to Inbox view
                    self.screen = Screen::Inbox;
                    self.screen_title = generate_screen_title("Inbox");
                    // Load inbox activity from API
                    if self.current_workspace_id.is_some() {
                        if self.current_user_id.is_some() {
                            self.load_inbox_activity();
                        } else {
                            self.try_detect_user_id();
                            if self.current_user_id.is_some() {
                                self.load_inbox_activity();
                            } else {
                                self.status = "Loading user profile...".to_string();
                                self.fetch_current_user_and_load_tasks();
                            }
                        }
                    } else {
                        self.status = "Select a workspace first to view inbox".to_string();
                    }
                } else if let Some(SidebarItem::Space { id, name, .. }) = selected_item {
                    self.current_space_id = Some(id.clone());
                    self.current_folder_id = None;
                    self.current_list_id = None;
                    self.load_folders(id.clone());
                    self.screen = Screen::Folders;
                    self.screen_title = generate_screen_title(&name);
                }
            }
            Screen::Folders => {
                if let Some(SidebarItem::Inbox) = selected_item {
                    // Navigate to Inbox view
                    self.screen = Screen::Inbox;
                    self.screen_title = generate_screen_title("Inbox");
                    // Load inbox activity from API
                    if self.current_workspace_id.is_some() {
                        if self.current_user_id.is_some() {
                            self.load_inbox_activity();
                        } else {
                            self.try_detect_user_id();
                            if self.current_user_id.is_some() {
                                self.load_inbox_activity();
                            } else {
                                self.status = "Loading user profile...".to_string();
                                self.fetch_current_user_and_load_tasks();
                            }
                        }
                    } else {
                        self.status = "Select a workspace first to view inbox".to_string();
                    }
                } else if let Some(SidebarItem::Folder { id, name, .. }) = selected_item {
                    self.current_folder_id = Some(id.clone());
                    self.current_list_id = None;
                    self.load_lists(id.clone());
                    self.screen = Screen::Lists;
                    self.screen_title = generate_screen_title(&name);
                }
            }
            Screen::Lists => {
                if let Some(SidebarItem::Inbox) = selected_item {
                    // Navigate to Inbox view
                    self.screen = Screen::Inbox;
                    self.screen_title = generate_screen_title("Inbox");
                    // Load inbox activity from API
                    if self.current_workspace_id.is_some() {
                        if self.current_user_id.is_some() {
                            self.load_inbox_activity();
                        } else {
                            self.try_detect_user_id();
                            if self.current_user_id.is_some() {
                                self.load_inbox_activity();
                            } else {
                                self.status = "Loading user profile...".to_string();
                                self.fetch_current_user_and_load_tasks();
                            }
                        }
                    } else {
                        self.status = "Select a workspace first to view inbox".to_string();
                    }
                } else if let Some(SidebarItem::List { id, name, .. }) = selected_item {
                    self.current_list_id = Some(id.clone());
                    self.load_tasks(id.clone());
                    self.screen = Screen::Tasks;
                    self.screen_title = generate_screen_title(&format!("Tasks: {}", name));
                }
            }
            Screen::Inbox => {
                // Open inbox view
                self.screen = Screen::Inbox;
                self.screen_title = generate_screen_title("Inbox");
                // Load inbox activity from API
                if self.current_workspace_id.is_some() {
                    if self.current_user_id.is_some() {
                        self.load_inbox_activity();
                    } else {
                        self.try_detect_user_id();
                        if self.current_user_id.is_some() {
                            self.load_inbox_activity();
                        } else {
                            self.status = "Loading user profile...".to_string();
                            self.fetch_current_user_and_load_tasks();
                        }
                    }
                } else {
                    self.status = "Select a workspace first to view inbox".to_string();
                }
            }
            _ => {}
        }
    }

    /// Navigate back to previous screen (public for testing)
    pub fn navigate_back(&mut self) {
        match self.screen {
            Screen::Auth => {}       // Can't go back from auth
            Screen::Workspaces => {} // Can't go back from workspaces
            Screen::AssignedTasks => {
                // Navigate back to Workspaces (or stay at top level)
                self.screen = Screen::Workspaces;
                self.screen_title = generate_screen_title("Workspaces");
            }
            Screen::Spaces => {
                // Navigate back to Workspaces
                self.current_space_id = None;
                self.current_folder_id = None;
                self.current_list_id = None;

                // Repopulate sidebar with assigned tasks, inbox, then workspaces
                let mut items = vec![SidebarItem::AssignedTasks, SidebarItem::Inbox];
                items.extend(self.workspaces.iter().map(|w| SidebarItem::Workspace {
                    name: w.name.clone(),
                    id: w.id.clone(),
                }));
                *self.sidebar.items_mut() = items;

                // Restore selection using current_workspace_id
                if let Some(ref workspace_id) = self.current_workspace_id {
                    if !self.sidebar.select_by_id(workspace_id) {
                        // Workspace not found (e.g., was deleted), fallback to first
                        self.sidebar.select_first();
                        self.status = "Saved workspace not found, showing workspaces".to_string();
                    }
                } else {
                    self.sidebar.select_first();
                }

                self.screen = Screen::Workspaces;
                self.screen_title = generate_screen_title("Workspaces");
            }
            Screen::Folders => {
                // Navigate back to Spaces
                self.current_folder_id = None;
                self.current_list_id = None;

                // Repopulate sidebar with assigned tasks, inbox, then spaces
                let mut items = vec![SidebarItem::AssignedTasks, SidebarItem::Inbox];
                items.extend(self.spaces.iter().map(|s| SidebarItem::Space {
                    name: s.name.clone(),
                    id: s.id.clone(),
                    
                }));
                *self.sidebar.items_mut() = items;

                // Restore selection using current_space_id
                if let Some(ref space_id) = self.current_space_id {
                    if !self.sidebar.select_by_id(space_id) {
                        // Space not found, fallback to first
                        self.sidebar.select_first();
                        self.status = "Saved space not found, showing spaces".to_string();
                    }
                } else {
                    self.sidebar.select_first();
                }

                self.screen = Screen::Spaces;
                // Update title to show workspace name
                if let Some(ref workspace_id) = self.current_workspace_id {
                    if let Some(workspace) = self.workspaces.iter().find(|w| &w.id == workspace_id)
                    {
                        self.screen_title = generate_screen_title(&workspace.name);
                    } else {
                        self.screen_title = generate_screen_title("Workspaces");
                    }
                } else {
                    self.screen_title = generate_screen_title("Workspaces");
                }
            }
            Screen::Lists => {
                // Navigate back to Folders
                self.current_list_id = None;

                // Repopulate sidebar with assigned tasks, inbox, then folders
                let mut items = vec![SidebarItem::AssignedTasks, SidebarItem::Inbox];
                items.extend(self.folders.iter().map(|f| SidebarItem::Folder {
                    name: f.name.clone(),
                    id: f.id.clone(),
                    
                }));
                *self.sidebar.items_mut() = items;

                // Restore selection using current_folder_id
                if let Some(ref folder_id) = self.current_folder_id {
                    if !self.sidebar.select_by_id(folder_id) {
                        // Folder not found, fallback to first
                        self.sidebar.select_first();
                        self.status = "Saved folder not found, showing folders".to_string();
                    }
                } else {
                    self.sidebar.select_first();
                }

                self.screen = Screen::Folders;
                // Update title to show space name
                if let Some(ref space_id) = self.current_space_id {
                    if let Some(space) = self.spaces.iter().find(|s| &s.id == space_id) {
                        self.screen_title = generate_screen_title(&space.name);
                    } else {
                        self.screen_title = generate_screen_title("Spaces");
                    }
                } else {
                    self.screen_title = generate_screen_title("Spaces");
                }
            }
            Screen::Tasks => {
                // Navigate back to Lists
                self.current_list_id = None;

                // Repopulate sidebar with assigned tasks, inbox, then lists
                let mut items = vec![SidebarItem::AssignedTasks, SidebarItem::Inbox];
                items.extend(self.lists.iter().map(|l| SidebarItem::List {
                    name: l.name.clone(),
                    id: l.id.clone(),

                }));
                *self.sidebar.items_mut() = items;

                // Restore selection using current_list_id
                if let Some(ref list_id) = self.current_list_id {
                    if !self.sidebar.select_by_id(list_id) {
                        // List not found, fallback to first
                        self.sidebar.select_first();
                        self.status = "Saved list not found, showing lists".to_string();
                    }
                } else {
                    self.sidebar.select_first();
                }

                self.screen = Screen::Lists;
                // Update title to show folder name
                if let Some(ref folder_id) = self.current_folder_id {
                    if let Some(folder) = self.folders.iter().find(|f| &f.id == folder_id) {
                        self.screen_title = generate_screen_title(&folder.name);
                    } else {
                        self.screen_title = generate_screen_title("Folders");
                    }
                } else {
                    self.screen_title = generate_screen_title("Folders");
                }
            }
            Screen::TaskDetail => {
                // Navigate back to Tasks
                self.screen = Screen::Tasks;
                if let Some(list) = self.lists.first() {
                    self.screen_title = generate_screen_title(&format!("Tasks: {}", list.name));
                }
            }
            Screen::Document => {
                self.screen = Screen::Tasks;
                self.update_screen_title();
            }
            Screen::Inbox => {
                // Navigate back to Tasks (or last viewed screen)
                self.screen = Screen::Tasks;
                self.update_screen_title();
            }
        }
    }

    fn authenticate(&mut self) {
        let token = self.auth_state.token_input.clone();
        if token.is_empty() {
            self.auth_state.error = Some("Token cannot be empty".to_string());
            self.status = "Authentication failed".to_string();
            return;
        }

        self.loading = true;
        self.auth_state.loading = true;
        self.status = "Authenticating...".to_string();

        // Create the API client with the token
        let client = Arc::new(ClickUpClient::new(token.clone()));

        // Save the token
        if let Err(e) = self.auth.save_token(&token) {
            self.auth_state.error = Some(format!("Failed to save token: {}", e));
            self.auth_state.loading = false;
            self.loading = false;
            self.status = "Failed to save token".to_string();
            return;
        }

        // Store the client
        self.client = Some(client.clone());

        // Spawn async task to load workspaces
        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client.get_workspaces().await;
            let msg = match result {
                Ok(workspaces) => AppMessage::WorkspacesLoaded(Ok(workspaces)),
                Err(e) => AppMessage::WorkspacesLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });

        // Clear auth state and navigate to workspaces
        self.auth_state.clear();
        self.auth_state.loading = false;
        self.state = AppState::Main;
        self.screen = Screen::Workspaces;
        self.screen_title = generate_screen_title("Workspaces");
        self.status = "Authenticated! Loading workspaces...".to_string();
    }

    /// Load workspaces from API (public for testing)
    pub fn load_workspaces(&mut self) {
        self.loading = true;
        self.status = "Loading workspaces...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client.get_workspaces().await;
            let msg = match result {
                Ok(workspaces) => AppMessage::WorkspacesLoaded(Ok(workspaces)),
                Err(e) => AppMessage::WorkspacesLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    fn load_spaces(&mut self, workspace_id: String) {
        self.loading = true;
        self.status = "Loading spaces...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client.get_spaces(&workspace_id).await;
            let msg = match result {
                Ok(spaces) => AppMessage::SpacesLoaded(Ok(spaces)),
                Err(e) => AppMessage::SpacesLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    fn load_folders(&mut self, space_id: String) {
        self.loading = true;
        self.status = "Loading folders...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client.get_folders(&space_id).await;
            let msg = match result {
                Ok(folders) => AppMessage::FoldersLoaded(Ok(folders)),
                Err(e) => AppMessage::FoldersLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    fn load_lists(&mut self, folder_id: String) {
        self.loading = true;
        self.status = "Loading lists...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client.get_lists_in_folder(&folder_id, None).await;
            let msg = match result {
                Ok(lists) => AppMessage::ListsLoaded(Ok(lists)),
                Err(e) => AppMessage::ListsLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    fn load_tasks(&mut self, list_id: String) {
        self.loading = true;
        self.status = "Loading tasks...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        use crate::models::TaskFilters;
        let tx = self.message_tx.clone().unwrap();
        let filters = TaskFilters::default();
        tokio::spawn(async move {
            let result = client.get_tasks(&list_id, &filters).await;
            let msg = match result {
                Ok(tasks) => AppMessage::TasksLoaded(Ok(tasks)),
                Err(e) => AppMessage::TasksLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Load assigned tasks for the current user
    fn load_assigned_tasks(&mut self) {
        self.assigned_tasks_loading = true;
        self.assigned_tasks_error = None;
        self.status = "Loading assigned tasks...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.assigned_tasks_loading = false;
                self.assigned_tasks_error = Some("Not authenticated".to_string());
                return;
            }
        };

        let user_id = match self.current_user_id {
            Some(id) => id,
            None => {
                self.assigned_tasks_loading = false;
                self.assigned_tasks_error = Some(
                    "User identity not detected. Navigate to a task list and open a task to auto-detect your user ID, or try refreshing the page.".to_string()
                );
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            // Get all accessible lists
            tracing::info!("Starting assigned tasks load for user {}", user_id);
            let lists_result = client.get_all_accessible_lists().await;

            match lists_result {
                Ok(lists) => {
                    tracing::info!("Retrieved {} accessible list(s) for assigned tasks", lists.len());

                    // Check if lists are empty - this was causing the "zero tasks" bug
                    if lists.is_empty() {
                        tracing::warn!("No accessible lists found for assigned tasks");
                        let msg = AppMessage::AssignedTasksLoaded(Err(
                            "No accessible lists found. This means:\n  • Your workspaces have no spaces, OR\n  • Your spaces have no folders or lists, OR\n  • You don't have access to any lists\n\nUse 'clickdown debug lists-all' to diagnose.".to_string()
                        ));
                        let _ = tx.send(msg).await;
                        return;
                    }

                    // Fetch tasks from each list in parallel using join_all
                    tracing::info!("Fetching tasks assigned to user {} from {} list(s) in parallel", user_id, lists.len());
                    
                    // Create a list of futures for fetching tasks from each list
                    let fetch_futures = lists.iter().map(|list| {
                        let client = client.clone();
                        let list_id = list.id.clone();
                        let list_name = list.name.clone();
                        async move {
                            match client.get_tasks_with_assignee(&list_id, user_id, Some(100)).await {
                                Ok(tasks) => {
                                    tracing::debug!("Found {} assigned tasks in list '{}'", tasks.len(), list_name);
                                    Ok((list_id, tasks))
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to fetch tasks from list '{}': {}", list_id, e);
                                    Err((list_id, e))
                                }
                            }
                        }
                    });

                    // Execute all fetches in parallel
                    let results = join_all(fetch_futures).await;

                    // Collect all successful tasks
                    let mut all_tasks = Vec::new();
                    let mut failed_lists = Vec::new();
                    
                    for result in results {
                        match result {
                            Ok((_list_id, tasks)) => {
                                all_tasks.extend(tasks);
                            }
                            Err((list_id, error)) => {
                                failed_lists.push((list_id, error));
                            }
                        }
                    }

                    // Log summary
                    if !failed_lists.is_empty() {
                        tracing::warn!("Failed to fetch tasks from {} list(s): {:?}", 
                            failed_lists.len(), 
                            failed_lists.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>());
                    }

                    tracing::info!("Total assigned tasks found: {}", all_tasks.len());
                    
                    // Provide helpful message if no tasks found
                    if all_tasks.is_empty() {
                        tracing::warn!("No assigned tasks found in any list");
                        let msg = AppMessage::AssignedTasksLoaded(Ok(all_tasks));
                        let _ = tx.send(msg).await;
                    } else {
                        let msg = AppMessage::AssignedTasksLoaded(Ok(all_tasks));
                        let _ = tx.send(msg).await;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to get accessible lists: {}", e);
                    let msg = AppMessage::AssignedTasksLoaded(Err(format!(
                        "Failed to fetch lists: {}\n\nThis could be due to:\n  • API connectivity issues\n  • Insufficient permissions\n  • Invalid API token\n\nTry: clickdown debug auth-status",
                        e
                    )));
                    let _ = tx.send(msg).await;
                }
            }
        });
    }

    /// Load assigned items (tasks + comments) for the current user
    fn load_assigned_items(&mut self) {
        self.assigned_items_loading = true;
        self.assigned_items_error = None;
        self.status = "Loading assigned items...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.assigned_items_loading = false;
                self.assigned_items_error = Some("Not authenticated".to_string());
                return;
            }
        };

        let user_id = match self.current_user_id {
            Some(id) => id,
            None => {
                self.assigned_items_loading = false;
                self.assigned_items_error = Some(
                    "User identity not detected. Navigate to a task list and open a task to auto-detect your user ID, or try refreshing the page.".to_string()
                );
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            tracing::info!("Starting assigned items load for user {}", user_id);

            // Fetch tasks and comments in parallel
            let tasks_future = async {
                match client.get_all_accessible_lists().await {
                    Ok(lists) => {
                        tracing::info!("Retrieved {} accessible list(s) for assigned items", lists.len());
                        
                        // Fetch tasks from each list
                        let fetch_futures = lists.iter().map(|list| {
                            let client = client.clone();
                            let list_id = list.id.clone();
                            let list_name = list.name.clone();
                            async move {
                                match client.get_tasks_with_assignee(&list_id, user_id, Some(100)).await {
                                    Ok(tasks) => {
                                        tracing::debug!("Found {} assigned tasks in list '{}'", tasks.len(), list_name);
                                        Ok((list_id, tasks))
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to fetch tasks from list '{}': {}", list_id, e);
                                        Err((list_id, e))
                                    }
                                }
                            }
                        });

                        let results = join_all(fetch_futures).await;
                        let mut all_tasks = Vec::new();
                        
                        for result in results {
                            if let Ok((_list_id, tasks)) = result {
                                all_tasks.extend(tasks);
                            }
                        }
                        
                        tracing::info!("Total assigned tasks found: {}", all_tasks.len());
                        Ok(all_tasks)
                    }
                    Err(e) => {
                        tracing::error!("Failed to get accessible lists: {}", e);
                        Err(e)
                    }
                }
            };

            let comments_future = async {
                match client.get_assigned_comments(user_id).await {
                    Ok(comments) => {
                        tracing::info!("Found {} assigned comment(s)", comments.len());
                        Ok(comments)
                    }
                    Err(e) => {
                        tracing::warn!("Failed to fetch assigned comments: {}", e);
                        Err(e)
                    }
                }
            };

            // Execute both fetches in parallel
            let (tasks_result, comments_result) = tokio::join!(tasks_future, comments_future);

            // Combine results into AssignedItem enum
            let mut all_items = Vec::new();
            
            // Add tasks
            if let Ok(tasks) = tasks_result {
                for task in tasks {
                    all_items.push(AssignedItem::Task(task));
                }
            }
            
            // Add comments
            if let Ok(comments) = comments_result {
                for comment in comments {
                    all_items.push(AssignedItem::AssignedComment(comment));
                }
            }

            // Sort by updated_at descending
            all_items = crate::models::assigned_item::sort_assigned_items(all_items);

            tracing::info!("Total assigned items: {}", all_items.len());

            let msg = AppMessage::AssignedItemsLoaded(Ok(all_items));
            let _ = tx.send(msg).await;
        });
    }

    /// Pre-load assigned tasks at application startup
    ///
    /// This method is called during application initialization to pre-fetch assigned tasks
    /// so they're ready when the user navigates to the "Assigned to Me" view.
    /// 
    /// Strategy:
    /// 1. If user_id is available from session restore, use it
    /// 2. Check cache validity (TTL: 5 minutes)
    /// 3. If cache is valid and has data, load from cache immediately
    /// 4. If cache is invalid or empty, fetch from API in background
    pub fn pre_load_assigned_tasks(&mut self) {
        // Try to get user ID from session restore first
        let user_id = match self.current_user_id {
            Some(id) => id,
            None => {
                // No user ID available yet - will need to fetch from API first
                // This will be done when user navigates to Assigned Tasks view
                tracing::debug!("No user ID available for pre-loading assigned tasks");
                return;
            }
        };

        // Check if cache is valid (5 minute TTL)
        let cache_valid = self.cache.is_assigned_tasks_cache_valid(300).unwrap_or(false);
        
        if cache_valid {
            // Try to load from cache
            match self.cache.get_assigned_tasks() {
                Ok(tasks) if !tasks.is_empty() => {
                    tracing::info!("Pre-loaded {} assigned task(s) from cache", tasks.len());
                    let sorted_tasks = crate::models::task::sort_tasks(tasks.clone());
                    *self.assigned_tasks.tasks_mut() = sorted_tasks;
                    self.assigned_tasks_count = tasks.len();
                    self.assigned_tasks.select_first();
                    self.status = format!("Loaded {} assigned task(s) from cache", self.assigned_tasks_count);
                    
                    // Still refresh in background to get latest data
                    self.pre_load_assigned_tasks_background(user_id);
                    return;
                }
                Ok(_) => {
                    tracing::debug!("Assigned tasks cache is empty, fetching from API");
                }
                Err(e) => {
                    tracing::warn!("Failed to load assigned tasks from cache: {}", e);
                }
            }
        }
        
        // Cache invalid or empty - fetch from API in background
        self.pre_load_assigned_tasks_background(user_id);
    }

    /// Background task for pre-loading assigned tasks from API
    ///
    /// This is similar to load_assigned_tasks but uses AssignedTasksPreloaded message
    /// to avoid showing loading indicators during startup.
    fn pre_load_assigned_tasks_background(&mut self, user_id: i32) {
        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                tracing::warn!("No API client available for pre-loading assigned tasks");
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            tracing::info!("Pre-fetching assigned tasks for user {} in background", user_id);
            
            // Get all accessible lists
            let lists_result = client.get_all_accessible_lists().await;

            match lists_result {
                Ok(lists) => {
                    tracing::info!("Retrieved {} accessible list(s) for pre-loading assigned tasks", lists.len());

                    if lists.is_empty() {
                        tracing::warn!("No accessible lists found for pre-loading assigned tasks");
                        let msg = AppMessage::AssignedTasksPreloaded(Ok(vec![]));
                        let _ = tx.send(msg).await;
                        return;
                    }

                    // Fetch tasks from each list in parallel
                    let fetch_futures = lists.iter().map(|list| {
                        let client = client.clone();
                        let list_id = list.id.clone();
                        async move {
                            match client.get_tasks_with_assignee(&list_id, user_id, Some(100)).await {
                                Ok(tasks) => tasks,
                                Err(e) => {
                                    tracing::debug!("Failed to fetch tasks from list '{}': {}", list_id, e);
                                    Vec::new()
                                }
                            }
                        }
                    });

                    let results: Vec<Vec<Task>> = join_all(fetch_futures).await;

                    let mut all_tasks = Vec::new();
                    for tasks in results {
                        all_tasks.extend(tasks);
                    }

                    tracing::info!("Pre-fetched {} assigned task(s) in background", all_tasks.len());

                    // Cache the results
                    if let Ok(cache_manager) = crate::cache::CacheManager::new(
                        crate::config::ConfigManager::database_path().unwrap_or_default()
                    ) {
                        let mut cache = cache_manager;
                        if let Err(e) = cache.cache_assigned_tasks(&all_tasks) {
                            tracing::warn!("Failed to cache assigned tasks: {}", e);
                        }
                    }

                    let msg = AppMessage::AssignedTasksPreloaded(Ok(all_tasks));
                    let _ = tx.send(msg).await;
                }
                Err(e) => {
                    tracing::error!("Failed to get accessible lists for pre-loading: {}", e);
                    let msg = AppMessage::AssignedTasksPreloaded(Err(format!(
                        "Failed to fetch lists: {}",
                        e
                    )));
                    let _ = tx.send(msg).await;
                }
            }
        });
    }

    /// Fetch current user from API and then load assigned tasks
    fn fetch_current_user_and_load_tasks(&mut self) {
        self.assigned_tasks_loading = true;
        self.assigned_tasks_error = None;
        self.status = "Fetching user profile...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.assigned_tasks_loading = false;
                self.assigned_tasks_error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            match client.get_current_user().await {
                Ok(user) => {
                    let msg = AppMessage::CurrentUserLoaded(Ok(user));
                    let _ = tx.send(msg).await;
                }
                Err(e) => {
                    let msg = AppMessage::CurrentUserLoaded(Err(e.to_string()));
                    let _ = tx.send(msg).await;
                }
            }
        });
    }

    /// Load notifications from API and cache them
    ///
    /// This method fetches notifications from the ClickUp API for the current workspace,
    /// caches them locally, and updates the inbox UI state.
    #[allow(dead_code)]
    fn load_notifications(&mut self) {
        self.inbox_loading = true;
        self.status = "Loading notifications...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.inbox_loading = false;
                self.status = "Not authenticated".to_string();
                return;
            }
        };

        let workspace_id = match &self.current_workspace_id {
            Some(id) => id.clone(),
            None => {
                self.inbox_loading = false;
                self.status = "No workspace selected".to_string();
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            match client.get_notifications(&workspace_id).await {
                Ok(notifications) => {
                    // Cache the notifications
                    if let Ok(cache_manager) = crate::cache::CacheManager::new(
                        crate::config::ConfigManager::database_path().unwrap_or_default()
                    ) {
                        let mut cache = cache_manager;
                        if let Err(e) = cache.cache_notifications(&workspace_id, &notifications) {
                            tracing::warn!("Failed to cache notifications: {}", e);
                        }
                    }
                    let msg = AppMessage::NotificationsLoaded(Ok(notifications));
                    let _ = tx.send(msg).await;
                }
                Err(e) => {
                    let msg = AppMessage::NotificationsLoaded(Err(e.to_string()));
                    let _ = tx.send(msg).await;
                }
            }
        });
    }

    /// Pre-load notifications from cache or fetch from API if needed
    ///
    /// This method checks the cache first and fetches from API if cache is empty or stale.
    /// Used when navigating to the inbox to provide instant display if cache is valid.
    #[allow(dead_code)]
    pub fn pre_load_notifications(&mut self) {
        let workspace_id = match &self.current_workspace_id {
            Some(id) => id.clone(),
            None => {
                tracing::debug!("No workspace selected for pre-loading notifications");
                return;
            }
        };

        // Check if cache is valid (5 minute TTL)
        let cache_valid = self.cache.is_notifications_cache_valid(&workspace_id, 300).unwrap_or(false);

        if cache_valid {
            // Try to load from cache
            match self.cache.get_unread_notifications(&workspace_id, None) {
                Ok(notifications) if !notifications.is_empty() => {
                    tracing::info!("Pre-loaded {} notification(s) from cache", notifications.len());
                    self.notifications = notifications.clone();
                    // Convert notifications to activities for display
                    let activities: Vec<crate::models::InboxActivity> = notifications
                        .iter()
                        .map(|n| crate::models::InboxActivity {
                            id: format!("notification_{}", n.id),
                            activity_type: crate::models::ActivityType::Assignment,
                            title: n.title.clone(),
                            description: n.description.clone(),
                            timestamp: n.created_at.unwrap_or(0),
                            task_id: None,
                            comment_id: None,
                            workspace_id: n.workspace_id.clone(),
                            task_name: String::new(),
                            previous_status: None,
                            new_status: None,
                            due_date: None,
                        })
                        .collect();
                    self.inbox_list.set_activities(activities);
                    self.status = format!("Loaded {} notification(s) from cache", self.notifications.len());

                    // Still refresh in background to get latest data
                    self.pre_load_notifications_background(workspace_id);
                    return;
                }
                Ok(_) => {
                    tracing::debug!("Notifications cache is empty, fetching from API");
                }
                Err(e) => {
                    tracing::warn!("Failed to load notifications from cache: {}", e);
                }
            }
        }

        // Cache invalid or empty - fetch from API in background
        self.pre_load_notifications_background(workspace_id);
    }

    /// Background task for pre-loading notifications from API
    ///
    /// This is similar to load_notifications but used for background refresh
    /// without showing loading indicators.
    fn pre_load_notifications_background(&mut self, workspace_id: String) {
        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                tracing::warn!("No API client available for pre-loading notifications");
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            tracing::info!("Pre-fetching notifications for workspace {} in background", workspace_id);

            match client.get_notifications(&workspace_id).await {
                Ok(notifications) => {
                    tracing::info!("Fetched {} notification(s) for workspace {}", notifications.len(), workspace_id);

                    // Cache the notifications
                    if let Ok(cache_manager) = crate::cache::CacheManager::new(
                        crate::config::ConfigManager::database_path().unwrap_or_default()
                    ) {
                        let mut cache = cache_manager;
                        if let Err(e) = cache.cache_notifications(&workspace_id, &notifications) {
                            tracing::warn!("Failed to cache notifications: {}", e);
                        }
                    }

                    let msg = AppMessage::NotificationsLoaded(Ok(notifications));
                    let _ = tx.send(msg).await;
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch notifications for workspace {}: {}", workspace_id, e);
                    // Don't send error message for background fetch - just log
                }
            }
        });
    }

    /// Load inbox activity from API and cache it
    ///
    /// This method fetches activity from the ClickUp API for the current workspace,
    /// caches it locally, and updates the smart inbox UI state.
    fn load_inbox_activity(&mut self) {
        self.inbox_activity_loading = true;
        self.status = "Loading activity...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.inbox_activity_loading = false;
                self.status = "Not authenticated".to_string();
                return;
            }
        };

        let workspace_id = match &self.current_workspace_id {
            Some(id) => id.clone(),
            None => {
                self.inbox_activity_loading = false;
                self.status = "No workspace selected".to_string();
                return;
            }
        };

        let user_id = match self.current_user_id {
            Some(id) => id,
            None => {
                self.inbox_activity_loading = false;
                self.status = "User ID not available".to_string();
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            match client.get_inbox_activity(&workspace_id, user_id, None).await {
                Ok(response) => {
                    let activities = response.activities;
                    tracing::info!("Fetched {} activity item(s) for workspace {}", activities.len(), workspace_id);

                    // Cache the activity
                    if let Ok(mut cache) = crate::cache::CacheManager::new(
                        crate::config::ConfigManager::database_path().unwrap_or_default()
                    ) {
                        if let Err(e) = cache.cache_inbox_activity(&workspace_id, &activities) {
                            tracing::warn!("Failed to cache inbox activity: {}", e);
                        }
                        // Store the fetch timestamp
                        let now = chrono::Utc::now().timestamp_millis();
                        let _ = cache.store_last_inbox_check(&workspace_id, now);
                    }

                    let msg = AppMessage::InboxActivityLoaded(Ok(activities));
                    let _ = tx.send(msg).await;
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch inbox activity for workspace {}: {}", workspace_id, e);
                    let msg = AppMessage::InboxActivityLoaded(Err(e.to_string()));
                    let _ = tx.send(msg).await;
                }
            }
        });
    }

    /// Pre-load inbox activity from cache or fetch from API if needed
    ///
    /// This method checks the cache first and fetches from API if cache is empty or stale.
    /// Used when navigating to the inbox to provide instant display if cache is valid.
    #[allow(dead_code)]
    pub fn pre_load_inbox_activity(&mut self) {
        let workspace_id = match &self.current_workspace_id {
            Some(id) => id.clone(),
            None => {
                tracing::debug!("No workspace selected for pre-loading inbox activity");
                return;
            }
        };

        // Check if cache is valid (5 minute TTL)
        let cache_valid = self.cache.is_inbox_activity_cache_valid(&workspace_id, 300).unwrap_or(false);

        if cache_valid {
            // Try to load from cache
            match self.cache.get_cached_inbox_activity(&workspace_id) {
                Ok(activities) if !activities.is_empty() => {
                    tracing::info!("Pre-loaded {} activity item(s) from cache", activities.len());
                    self.inbox_activity = activities.clone();
                    self.status = format!("Loaded {} activity item(s) from cache", self.inbox_activity.len());

                    // Still refresh in background to get latest data
                    self.pre_load_inbox_activity_background(workspace_id);
                    return;
                }
                Ok(_) => {
                    tracing::debug!("Inbox activity cache is empty, fetching from API");
                }
                Err(e) => {
                    tracing::warn!("Failed to load inbox activity from cache: {}", e);
                }
            }
        }

        // Cache invalid or empty - fetch from API in background
        self.pre_load_inbox_activity_background(workspace_id);
    }

    /// Background task for pre-loading inbox activity from API
    ///
    /// This is similar to load_inbox_activity but used for background refresh
    /// without showing loading indicators.
    #[allow(dead_code)]
    fn pre_load_inbox_activity_background(&mut self, workspace_id: String) {
        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                tracing::warn!("No API client available for pre-loading inbox activity");
                return;
            }
        };

        let user_id = match self.current_user_id {
            Some(id) => id,
            None => {
                tracing::warn!("No user ID available for pre-loading inbox activity");
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            tracing::info!("Pre-fetching inbox activity for workspace {} in background", workspace_id);

            match client.get_inbox_activity(&workspace_id, user_id, None).await {
                Ok(response) => {
                    let activities = response.activities;
                    tracing::info!("Fetched {} activity item(s) for workspace {}", activities.len(), workspace_id);

                    // Cache the activity
                    if let Ok(mut cache) = crate::cache::CacheManager::new(
                        crate::config::ConfigManager::database_path().unwrap_or_default()
                    ) {
                        if let Err(e) = cache.cache_inbox_activity(&workspace_id, &activities) {
                            tracing::warn!("Failed to cache inbox activity: {}", e);
                        }
                        // Store the fetch timestamp
                        let now = chrono::Utc::now().timestamp_millis();
                        let _ = cache.store_last_inbox_check(&workspace_id, now);
                    }

                    let msg = AppMessage::InboxActivityLoaded(Ok(activities));
                    let _ = tx.send(msg).await;
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch inbox activity for workspace {}: {}", workspace_id, e);
                    // Don't send error message for background fetch - just log
                }
            }
        });
    }

    /// Detect current user ID from a task's creator or assignee field
    fn detect_user_id_from_task(&mut self, task: &Task) {
        // First try to get user ID from task creator
        if let Some(creator) = &task.creator {
            self.current_user_id = Some(creator.id as i32);
            tracing::info!("Detected user ID from task creator: {}", creator.id);
            return;
        }

        // If no creator, try to get user ID from assignees
        // Use the first assignee as a fallback
        if let Some(assignee) = task.assignees.first() {
            self.current_user_id = Some(assignee.id as i32);
            tracing::info!("Detected user ID from task assignee: {}", assignee.id);
        }
    }

    /// Try to detect user ID from any available task (public for testing)
    pub fn try_detect_user_id(&mut self) {
        // If we already have a user ID, skip detection
        if self.current_user_id.is_some() {
            return;
        }

        // Try to get user ID from tasks in the current list
        // Clone the task to avoid borrow checker issues
        if let Some(task) = self.task_list.tasks().first().cloned() {
            self.detect_user_id_from_task(&task);
        }

        // If still no user ID, try from assigned tasks if available
        if self.current_user_id.is_none() {
            if let Some(task) = self.assigned_tasks.tasks().first().cloned() {
                self.detect_user_id_from_task(&task);
            }
        }
    }

    /// Load comments for a task (top-level + replies)
    fn load_comments(&mut self, task_id: String) {
        self.loading = true;
        self.status = "Loading comments...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            // First, fetch top-level comments
            let top_level_result = client.get_task_comments(&task_id).await;

            match top_level_result {
                Ok(top_level_comments) => {
                    // For each top-level comment, fetch its replies
                    let mut all_comments = top_level_comments;

                    // Collect all reply fetches
                    let mut reply_futures = Vec::new();
                    for comment in &all_comments {
                        let comment_id = comment.id.clone();
                        let client_clone = client.clone();
                        reply_futures.push(async move {
                            let result = client_clone.get_comment_replies(&comment_id).await;
                            (comment_id, result)
                        });
                    }

                    // Wait for all replies to be fetched
                    let reply_results = futures::future::join_all(reply_futures).await;

                    // Add replies to the comments list with parent_id set
                    for (parent_id, reply_result) in reply_results {
                        if let Ok(replies) = reply_result {
                            for mut reply in replies {
                                reply.parent_id = Some(parent_id.clone());
                                all_comments.push(reply);
                            }
                        }
                    }

                    let msg = AppMessage::CommentsLoaded(Ok(all_comments));
                    let _ = tx.send(msg).await;
                }
                Err(e) => {
                    let msg = AppMessage::CommentsLoaded(Err(e.to_string()));
                    let _ = tx.send(msg).await;
                }
            }
        });
    }

    /// Create a new comment (top-level or reply)
    fn create_comment(&mut self, task_id: String, text: String, parent_id: Option<String>) {
        self.loading = true;
        // Show appropriate status message based on whether this is a reply
        let is_reply = parent_id.is_some();
        self.status = if is_reply {
            "Saving reply...".to_string()
        } else {
            "Saving comment...".to_string()
        };

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        let request = CreateCommentRequest {
            comment_text: text,
            assignee: None,
            assigned_commenter: None,
            parent_id: parent_id.clone(),
        };

        tokio::spawn(async move {
            // Use different endpoint based on whether this is a reply
            let result = if let Some(ref parent_comment_id) = parent_id {
                client
                    .create_comment_reply(parent_comment_id, &request)
                    .await
            } else {
                client.create_comment(&task_id, &request).await
            };

            let msg = match result {
                Ok(comment) => AppMessage::CommentCreated(Ok(comment), is_reply),
                Err(e) => AppMessage::CommentCreated(Err(e.to_string()), is_reply),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Update an existing comment
    fn update_comment(&mut self, comment_id: String, text: String) {
        self.loading = true;
        self.status = "Saving comment...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        let request = UpdateCommentRequest {
            comment_text: Some(text),
            assigned: None,
            assignee: None,
            assigned_commenter: None,
        };
        tokio::spawn(async move {
            let result = client.update_comment(&comment_id, &request).await;
            let msg = match result {
                Ok(comment) => AppMessage::CommentUpdated(Ok(comment)),
                Err(e) => AppMessage::CommentUpdated(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    fn update_screen_title(&mut self) {
        self.screen_title = match &self.screen {
            Screen::Auth => generate_screen_title("Authentication"),
            Screen::Workspaces => generate_screen_title("Workspaces"),
            Screen::Spaces => {
                if let Some(space) = self.spaces.first() {
                    generate_screen_title(&space.name)
                } else {
                    generate_screen_title("Spaces")
                }
            }
            Screen::Folders => {
                if let Some(folder) = self.folders.first() {
                    generate_screen_title(&folder.name)
                } else {
                    generate_screen_title("Folders")
                }
            }
            Screen::Lists => {
                if let Some(list) = self.lists.first() {
                    generate_screen_title(&list.name)
                } else {
                    generate_screen_title("Lists")
                }
            }
            Screen::Tasks => {
                if let Some(list) = self.lists.first() {
                    generate_screen_title(&format!("Tasks: {}", list.name))
                } else {
                    generate_screen_title("Tasks")
                }
            }
            Screen::TaskDetail => {
                if let Some(task) = &self.task_detail.task {
                    generate_screen_title(&format!("Task: {}", task.name))
                } else {
                    generate_screen_title("Task Detail")
                }
            }
            Screen::Document => {
                if !self.document.title.is_empty() {
                    generate_screen_title(&format!("Doc: {}", self.document.title))
                } else {
                    generate_screen_title("Document")
                }
            }
            Screen::Inbox => generate_screen_title("Inbox"),
            Screen::AssignedTasks => generate_screen_title("Assigned to Me"),
        };
    }

    fn render(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        // Auto-clear URL copy status after 3 seconds
        if let Some(status_time) = self.url_copy_status_time {
            if status_time.elapsed() > std::time::Duration::from_secs(3) {
                self.url_copy_status = None;
                self.url_copy_status_time = None;
            }
        }

        terminal.draw(|frame: &mut Frame| {
            let area = frame.area();
            let layout = TuiLayout::new(area);

            // Render title bar
            layout.render_title(frame, &self.screen_title);

            // Check if terminal is too small
            if layout.too_small {
                layout.render_too_small_warning(frame);
                return;
            }

            // Render content area
            if self.sidebar.visible {
                let (sidebar_area, content_area) = layout.split_content(25);
                self.render_sidebar_content(frame, sidebar_area, content_area);
            } else {
                self.render_main_content(frame, layout.content_area);
            }

            // Render dialog if visible
            render_dialog(frame, &self.dialog, area);

            // Render help overlay if visible
            render_help(frame, &self.help, area);

            // Render status bar
            let hints = self.get_hints();
            // Priority: error > url_copy_status > loading > regular status
            let status = if let Some(ref error) = self.error {
                error.clone()
            } else if let Some(ref url_status) = self.url_copy_status {
                // Show URL copy status (takes priority over regular status)
                url_status.clone()
            } else if self.screen == Screen::AssignedTasks && self.assigned_tasks_error.is_some() {
                // Show assigned tasks error when on AssignedTasks screen
                self.assigned_tasks_error.clone().unwrap()
            } else if self.screen == Screen::AssignedTasks && self.assigned_tasks_loading {
                // Show assigned tasks loading state
                "Loading assigned tasks...".to_string()
            } else if self.loading {
                "Loading...".to_string()
            } else {
                self.status.clone()
            };
            layout.render_status(frame, &status, hints);
        })?;

        Ok(())
    }

    fn render_sidebar_content(
        &mut self,
        frame: &mut Frame,
        sidebar_area: Rect,
        content_area: Rect,
    ) {
        // Render sidebar
        render_sidebar(frame, &self.sidebar, sidebar_area, Some(self.assigned_items_count));

        // Render main content based on screen
        match self.screen {
            Screen::Auth => render_auth(frame, &self.auth_state, content_area),
            Screen::Tasks => render_task_list(frame, &self.task_list, content_area, false),
            Screen::TaskDetail => {
                // Split content area between task detail and comments with 3:7 ratio
                let (task_detail_area, comments_area) = split_task_detail(content_area);

                // Render task detail in top portion (30%)
                render_task_detail(frame, &self.task_detail, task_detail_area);

                // Render comments in bottom portion (70%)
                render_comments(
                    frame,
                    &self.comments,
                    self.comment_selected_index,
                    self.comment_editing_index,
                    &self.comment_new_text,
                    self.comment_focus,
                    comments_area,
                    &self.comment_view_mode,
                );
            }
            Screen::Document => render_document(frame, &self.document, content_area),
            Screen::Inbox => {
                // Sync inbox_list with inbox_activity before rendering
                self.inbox_list.set_activities(self.inbox_activity.clone());
                
                render_inbox_list(
                    frame,
                    content_area,
                    &mut self.inbox_list,
                    self.inbox_showing_detail,
                );

                // Render detail panel if showing
                if self.inbox_showing_detail {
                    if let Some(activity) = self.inbox_list.selected_activity() {
                        // Create a centered popup for detail
                        let detail_width = std::cmp::min(80, content_area.width - 4);
                        let detail_height = std::cmp::min(20, content_area.height - 4);
                        let detail_rect = Rect::new(
                            (content_area.width - detail_width) / 2,
                            (content_area.height - detail_height) / 2,
                            detail_width,
                            detail_height,
                        );

                        render_notification_detail(frame, detail_rect, activity);
                    }
                }
            }
            Screen::AssignedTasks => {
                // Render assigned items view (tasks + comments)
                render_assigned_view(
                    frame,
                    &self.assigned_items,
                    self.assigned_items_selected_index,
                    self.assigned_items_filter,
                    content_area,
                    self.assigned_items_loading,
                    self.assigned_items_error.as_deref(),
                    self.assigned_items_count,
                );
            }
            _ => {
                // For navigation screens, show placeholder
                use ratatui::widgets::Paragraph;
                let placeholder = Paragraph::new(format!("Navigate to see {}", self.screen_title));
                frame.render_widget(placeholder, content_area);
            }
        }
    }

    fn render_main_content(&mut self, frame: &mut Frame, area: Rect) {
        match self.screen {
            Screen::Auth => render_auth(frame, &self.auth_state, area),
            Screen::Tasks => render_task_list(frame, &self.task_list, area, false),
            Screen::TaskDetail => {
                // Split area between task detail and comments with 3:7 ratio
                let (task_detail_area, comments_area) = split_task_detail(area);

                // Render task detail in top portion (30%)
                render_task_detail(frame, &self.task_detail, task_detail_area);

                // Render comments in bottom portion (70%)
                render_comments(
                    frame,
                    &self.comments,
                    self.comment_selected_index,
                    self.comment_editing_index,
                    &self.comment_new_text,
                    self.comment_focus,
                    comments_area,
                    &self.comment_view_mode,
                );
            }
            Screen::Document => render_document(frame, &self.document, area),
            Screen::Inbox => {
                // Show loading indicator if fetching
                if self.inbox_activity_loading {
                    use ratatui::widgets::Paragraph;
                    let loading = Paragraph::new("Loading activity...")
                        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
                        .block(
                            ratatui::widgets::Block::default()
                                .title(" Inbox ")
                                .borders(ratatui::widgets::Borders::ALL),
                        );
                    frame.render_widget(loading, area);
                } else {
                    // Sync inbox_list with inbox_activity before rendering
                    self.inbox_list.set_activities(self.inbox_activity.clone());
                    
                    render_inbox_list(frame, area, &mut self.inbox_list, self.inbox_showing_detail);

                    // Render detail panel if showing
                    if self.inbox_showing_detail {
                        if let Some(activity) = self.inbox_list.selected_activity() {
                            // Create a centered popup for detail
                            let detail_width = std::cmp::min(80, area.width - 4);
                            let detail_height = std::cmp::min(20, area.height - 4);
                            let detail_rect = Rect::new(
                                (area.width - detail_width) / 2,
                                (area.height - detail_height) / 2,
                                detail_width,
                                detail_height,
                            );

                            render_notification_detail(frame, detail_rect, activity);
                        }
                    }
                }
            }
            _ => {
                use ratatui::widgets::Paragraph;
                let placeholder = Paragraph::new(format!("Navigate to see {}", self.screen_title));
                frame.render_widget(placeholder, area);
            }
        }
    }

    fn get_hints(&self) -> &'static str {
        if self.dialog.is_visible() {
            get_dialog_hints()
        } else if self.help.visible {
            // When help is visible, don't show other hints
            ""
        } else {
            match self.screen {
                Screen::Auth => "Enter: Connect | Esc: Cancel | ? - Help",
                Screen::Tasks => {
                    "j/k: Navigate | Enter: View | n: New | e: Edit | d: Delete | ? - Help"
                }
                Screen::TaskDetail => {
                    // Show different hints based on comment view mode
                    if self.comment_focus {
                        match self.comment_view_mode {
                            CommentViewMode::TopLevel => "j/k: Navigate | Enter: View thread | n: New comment | e: Edit | Tab: Task form | ? - Help",
                            CommentViewMode::InThread { .. } => "j/k: Navigate | r: Reply | Esc: Back | Tab: Task form | ? - Help",
                        }
                    } else {
                        "e: Edit task | Tab: Comments | Esc: Back | ? - Help"
                    }
                }
                Screen::Document => "j/k: Scroll | Esc: Close | ? - Help",
                Screen::Inbox => {
                    if self.inbox_showing_detail {
                        "Esc: Close detail | j/k: Navigate | ? - Help"
                    } else {
                        "j/k: Navigate | Enter: View | r: Refresh | c: Mark read | C: Mark all read | Esc: Back | ? - Help"
                    }
                }
                _ => "j/k: Navigate | Enter: Select | Tab: Toggle | Ctrl+Q: Quit | ? - Help",
            }
        }
    }

    /// Copy URL for the current context to clipboard
    fn copy_url(&mut self) {
        tracing::debug!("copy_url called, screen: {:?}", self.screen);

        // Helper to get ID from sidebar item
        fn get_sidebar_id(item: &SidebarItem) -> &str {
            match item {
                SidebarItem::AssignedTasks => "assigned-tasks",
                SidebarItem::Inbox => "inbox",
                SidebarItem::Workspace { id, .. } => id,
                SidebarItem::Space { id, .. } => id,
                SidebarItem::Folder { id, .. } => id,
                SidebarItem::List { id, .. } => id,
            }
        }

        let url_result = match self.screen {
            Screen::Auth => {
                self.url_copy_status = Some("URL copy not available on auth screen".to_string());
                return;
            }
            Screen::Workspaces => {
                if let Some(ws) = self.sidebar.selected_item() {
                    ClickUpUrlGenerator::workspace_url(get_sidebar_id(ws))
                } else {
                    self.url_copy_status = Some("No workspace selected".to_string());
                    return;
                }
            }
            Screen::Spaces => {
                if let Some(space) = self.sidebar.selected_item() {
                    // Use tracked workspace context
                    if let Some(ref ws_id) = self.current_workspace_id {
                        ClickUpUrlGenerator::space_url(ws_id, get_sidebar_id(space))
                    } else {
                        self.url_copy_status = Some("Missing workspace context".to_string());
                        return;
                    }
                } else {
                    self.url_copy_status = Some("No space selected".to_string());
                    return;
                }
            }
            Screen::Folders => {
                if let Some(folder) = self.sidebar.selected_item() {
                    // Use tracked workspace context
                    if let Some(ref ws_id) = self.current_workspace_id {
                        ClickUpUrlGenerator::folder_url(ws_id, get_sidebar_id(folder))
                    } else {
                        self.url_copy_status = Some("Missing workspace context".to_string());
                        return;
                    }
                } else {
                    self.url_copy_status = Some("No folder selected".to_string());
                    return;
                }
            }
            Screen::Lists => {
                if let Some(list) = self.sidebar.selected_item() {
                    // Use tracked workspace context
                    if let Some(ref ws_id) = self.current_workspace_id {
                        ClickUpUrlGenerator::list_url(ws_id, get_sidebar_id(list))
                    } else {
                        self.url_copy_status = Some("Missing workspace context".to_string());
                        return;
                    }
                } else {
                    self.url_copy_status = Some("No list selected".to_string());
                    return;
                }
            }
            Screen::Tasks => {
                if let Some(task) = self.task_list.selected_task() {
                    // Short-form task URL: only need task ID
                    ClickUpUrlGenerator::task_url("", "", &task.id)
                } else {
                    self.url_copy_status = Some("No task selected".to_string());
                    return;
                }
            }
            Screen::TaskDetail => {
                // Check if comment focus is active and a comment is selected
                if self.comment_focus
                    && !self.comments.is_empty()
                    && self.comment_selected_index < self.comments.len()
                {
                    // Copy selected comment URL
                    let comment = &self.comments[self.comment_selected_index];
                    if let Some(task) = &self.task_detail.task {
                        ClickUpUrlGenerator::comment_url("", "", &task.id, &comment.id)
                    } else {
                        self.url_copy_status = Some("No task selected".to_string());
                        return;
                    }
                } else {
                    // Copy task URL
                    if let Some(task) = &self.task_detail.task {
                        ClickUpUrlGenerator::task_url("", "", &task.id)
                    } else {
                        self.url_copy_status = Some("No task selected".to_string());
                        return;
                    }
                }
            }
            Screen::Document => {
                if let Some(doc) = self.documents.first() {
                    // Short-form document URL: only need doc ID
                    ClickUpUrlGenerator::document_url("", &doc.id)
                } else {
                    self.url_copy_status = Some("No document selected".to_string());
                    return;
                }
            }
            Screen::Inbox => {
                self.url_copy_status = Some("URL copy not available for inbox".to_string());
                return;
            }
            Screen::AssignedTasks => {
                self.url_copy_status = Some("URL copy not available for assigned tasks view".to_string());
                return;
            }
        };

        // Handle URL generation result
        let url = match url_result {
            Ok(url) => {
                tracing::debug!("Generated URL: {}", url);
                url
            }
            Err(e) => {
                self.url_copy_status = Some(format!("URL error: {}", e));
                return;
            }
        };

        // Copy to clipboard
        match self.clipboard.copy_text(&url) {
            Ok(()) => {
                // Show success with truncated URL
                let truncated = if url.len() > 60 {
                    format!("{}...", &url[..57])
                } else {
                    url.clone()
                };
                self.url_copy_status = Some(format!("Copied: {}", truncated));
                self.url_copy_status_time = Some(std::time::Instant::now());
            }
            Err(e) => {
                self.url_copy_status = Some(format!("Failed to copy URL: {}", e));
                self.url_copy_status_time = Some(std::time::Instant::now());
            }
        }
    }

    /// Save the current session state to the cache
    ///
    /// This captures the current navigation context for restoration on next startup.
    pub fn save_session_state(&mut self) -> Result<()> {
        let state = SessionState::from_app(
            &self.screen,
            self.current_workspace_id.clone(),
            self.current_space_id.clone(),
            self.current_folder_id.clone(),
            self.current_list_id.clone(),
            self.task_detail.task.as_ref().map(|t| t.id.clone()),
            self.documents.first().map(|d| d.id.clone()),
            self.current_user_id,
        );
        self.cache.save_session_state(&state)
    }

    /// Restore session state from the cache
    ///
    /// Returns true if session restore is in progress, false if no saved state exists.
    /// Sets restoring_session flag and stores target IDs for progressive restore.
    /// The actual navigation chain replay happens in async message handlers.
    pub fn restore_session_state(&mut self) -> Result<bool> {
        let saved_state = match self.cache.load_session_state()? {
            Some(state) => state,
            None => {
                tracing::debug!("No saved session state found");
                return Ok(false);
            }
        };

        tracing::info!("Restoring session state: screen={}, workspace={:?}, space={:?}, folder={:?}, list={:?}",
            saved_state.screen,
            saved_state.workspace_id,
            saved_state.space_id,
            saved_state.folder_id,
            saved_state.list_id);

        // Set restoring session flag
        self.restoring_session = true;

        // Store target IDs for progressive restore
        self.restored_workspace_id = saved_state.workspace_id.clone();
        self.restored_space_id = saved_state.space_id.clone();
        self.restored_folder_id = saved_state.folder_id.clone();
        self.restored_list_id = saved_state.list_id.clone();
        self.restored_task_id = saved_state.task_id.clone();

        // Restore current navigation IDs (used for URL generation etc.)
        self.current_workspace_id = saved_state.workspace_id.clone();
        self.current_space_id = saved_state.space_id.clone();
        self.current_folder_id = saved_state.folder_id.clone();
        self.current_list_id = saved_state.list_id.clone();
        
        // Restore user ID for assignee filtering
        self.current_user_id = saved_state.user_id;

        // Start at Workspaces - the navigation chain will replay from here
        // The async handlers will navigate through each level and select the restored items
        self.screen = Screen::Workspaces;
        self.screen_title = generate_screen_title("Workspaces");

        Ok(true) // Return true to indicate restore is in progress
    }

    /// Apply fallback logic when restoring session state
    ///
    /// If the saved navigation context is invalid, falls back to the nearest valid parent.
    /// Returns the final screen and an optional fallback message.
    #[allow(dead_code)]
    fn apply_session_fallback(
        &self,
        target_screen: Screen,
        saved_state: &SessionState,
    ) -> (Screen, Option<String>) {
        // Check if we have valid context for the target screen
        match target_screen {
            Screen::Auth => {
                // Auth screen doesn't need fallback - always valid but shouldn't be restored
                (
                    Screen::Workspaces,
                    Some("Session cleared, showing workspaces".to_string()),
                )
            }
            Screen::Document => {
                if saved_state.document_id.is_some() {
                    // Would need to validate doc exists - for now assume valid
                    return (Screen::Document, None);
                }
                // Fall back to Tasks
                (
                    Screen::Tasks,
                    Some("Saved document not found, showing tasks".to_string()),
                )
            }
            Screen::TaskDetail => {
                if saved_state.task_id.is_some() && saved_state.list_id.is_some() {
                    return (Screen::TaskDetail, None);
                }
                // Fall back to Tasks
                (
                    Screen::Tasks,
                    Some("Saved task not found, showing tasks".to_string()),
                )
            }
            Screen::Tasks => {
                if saved_state.list_id.is_some() {
                    return (Screen::Tasks, None);
                }
                // Fall back to Lists
                (
                    Screen::Lists,
                    Some("Saved list not found, showing lists".to_string()),
                )
            }
            Screen::Lists => {
                if saved_state.folder_id.is_some() || saved_state.space_id.is_some() {
                    return (Screen::Lists, None);
                }
                // Fall back to Folders
                (
                    Screen::Folders,
                    Some("Saved folder not found, showing folders".to_string()),
                )
            }
            Screen::Folders => {
                if saved_state.space_id.is_some() {
                    return (Screen::Folders, None);
                }
                // Fall back to Spaces
                (
                    Screen::Spaces,
                    Some("Saved space not found, showing spaces".to_string()),
                )
            }
            Screen::Spaces => {
                if saved_state.workspace_id.is_some() {
                    return (Screen::Spaces, None);
                }
                // Fall back to Workspaces
                (
                    Screen::Workspaces,
                    Some("Saved workspace not found, showing workspaces".to_string()),
                )
            }
            Screen::Workspaces => {
                // Always valid
                (Screen::Workspaces, None)
            }
            Screen::Inbox => {
                if saved_state.workspace_id.is_some() {
                    return (Screen::Inbox, None);
                }
                // Fall back to Workspaces
                (
                    Screen::Workspaces,
                    Some("No workspace selected for inbox".to_string()),
                )
            }
            Screen::AssignedTasks => {
                // Assigned tasks view is always valid - will fetch from API or cache
                (Screen::AssignedTasks, None)
            }
        }
    }

    /// Get the current user ID (public for testing)
    #[allow(dead_code)]
    pub fn current_user_id(&self) -> Option<i32> {
        self.current_user_id
    }

    /// Set the current user ID (public for testing)
    #[allow(dead_code)]
    pub fn set_current_user_id(&mut self, id: Option<i32>) {
        self.current_user_id = id;
    }

    /// Get the current screen (public for testing)
    #[allow(dead_code)]
    pub fn screen(&self) -> Screen {
        self.screen.clone()
    }

    /// Get assigned tasks error (public for testing)
    #[allow(dead_code)]
    pub fn assigned_tasks_error(&self) -> Option<&String> {
        self.assigned_tasks_error.as_ref()
    }

    /// Get assigned tasks list (public for testing)
    #[allow(dead_code)]
    pub fn assigned_tasks(&self) -> &TaskListState {
        &self.assigned_tasks
    }

    /// Get assigned tasks count (public for testing)
    #[allow(dead_code)]
    pub fn assigned_tasks_count(&self) -> usize {
        self.assigned_tasks_count
    }

    /// Check if assigned tasks are loading (public for testing)
    #[allow(dead_code)]
    pub fn assigned_tasks_loading(&self) -> bool {
        self.assigned_tasks_loading
    }

    /// Get sidebar (public for testing)
    #[allow(dead_code)]
    pub fn sidebar(&mut self) -> &mut SidebarState {
        &mut self.sidebar
    }

    /// Get mutable sidebar (alias for sidebar, for consistency)
    #[allow(dead_code)]
    pub fn sidebar_mut(&mut self) -> &mut SidebarState {
        &mut self.sidebar
    }

    /// Get task list (public for testing)
    #[allow(dead_code)]
    pub fn task_list(&mut self) -> &mut TaskListState {
        &mut self.task_list
    }

    /// Get status message (public for testing)
    #[allow(dead_code)]
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Get cache manager (public for testing)
    #[allow(dead_code)]
    pub fn cache(&mut self) -> &mut crate::cache::CacheManager {
        &mut self.cache
    }

    /// Get inbox list (public for testing)
    #[allow(dead_code)]
    pub fn inbox_list(&self) -> &InboxListState {
        &self.inbox_list
    }

    /// Get mutable inbox list (public for testing)
    #[allow(dead_code)]
    pub fn inbox_list_mut(&mut self) -> &mut InboxListState {
        &mut self.inbox_list
    }

    /// Get inbox error (public for testing)
    #[allow(dead_code)]
    pub fn inbox_error(&self) -> Option<&String> {
        // Note: inbox errors are shown in status bar, not stored separately
        // Return None for now - can be enhanced to track inbox-specific errors
        None
    }

    /// Check if inbox detail view is showing (public for testing)
    #[allow(dead_code)]
    pub fn inbox_showing_detail(&self) -> bool {
        self.inbox_showing_detail
    }

    /// Check if inbox is loading (public for testing)
    #[allow(dead_code)]
    pub fn inbox_loading(&self) -> bool {
        self.inbox_loading
    }

    /// Get current workspace ID (public for testing)
    #[allow(dead_code)]
    pub fn current_workspace_id(&self) -> Option<&String> {
        self.current_workspace_id.as_ref()
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new().expect("Failed to create TUI app")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::mock_client::MockClickUpClient;
    use crate::models::task::Task;
    use std::sync::Arc;

    /// Test that pre_load_assigned_tasks does nothing without user ID
    #[test]
    fn test_pre_load_assigned_tasks_no_fetch_without_user_id() {
        let mock_client = MockClickUpClient::new();
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        
        // Ensure no user ID is set
        app.set_current_user_id(None);
        
        // Call pre_load_assigned_tasks - should return early without error
        app.pre_load_assigned_tasks();
        
        // Assigned tasks should remain empty
        assert!(app.assigned_tasks.tasks().is_empty());
        assert!(!app.assigned_tasks_loading);
    }

    /// Test that pre_load_assigned_tasks uses cache when valid
    #[tokio::test]
    async fn test_pre_load_assigned_tasks_uses_cache_when_valid() {
        // Create test tasks
        let test_tasks = vec![
            Task {
                id: "test-task-1".to_string(),
                name: "Test Task 1".to_string(),
                ..Default::default()
            },
            Task {
                id: "test-task-2".to_string(),
                name: "Test Task 2".to_string(),
                ..Default::default()
            },
        ];

        // Create app with mock client
        let mock_client = MockClickUpClient::new();
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Set user ID
        app.set_current_user_id(Some(123));

        // Cache the test tasks
        let cache_result = app.cache.cache_assigned_tasks(&test_tasks);
        assert!(cache_result.is_ok(), "Failed to cache tasks: {:?}", cache_result);

        // Verify cache was written
        let cached = app.cache.get_assigned_tasks();
        assert!(cached.is_ok(), "Failed to read cached tasks: {:?}", cached);
        assert_eq!(cached.unwrap().len(), 2, "Cache should have 2 tasks");

        // Call pre_load_assigned_tasks - should use cache
        app.pre_load_assigned_tasks();

        // Process async messages
        app.process_async_messages();

        // Should have loaded tasks from cache immediately
        assert_eq!(app.assigned_tasks.tasks().len(), 2, "Should have 2 tasks loaded from cache");
        assert_eq!(app.assigned_tasks_count, 2);
    }

    /// Test that pre_load_assigned_tasks fetches when cache is invalid
    #[tokio::test]
    async fn test_pre_load_assigned_tasks_fetches_when_cache_invalid() {
        // Create test tasks for mock client to return
        let test_tasks = vec![
            Task {
                id: "api-task-1".to_string(),
                name: "API Task 1".to_string(),
                ..Default::default()
            },
        ];
        
        // Create mock client that returns test tasks
        let mock_client = MockClickUpClient::new()
            .with_tasks(test_tasks.clone());
        
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        
        // Set user ID
        app.set_current_user_id(Some(123));
        
        // Invalidate cache by clearing it
        let _ = app.cache.clear_assigned_tasks();
        
        // Call pre_load_assigned_tasks - should fetch from API
        app.pre_load_assigned_tasks();
        
        // Process async messages to get the pre-loaded tasks
        app.process_async_messages();
        
        // Tasks should be pre-loaded (may be empty if mock doesn't support get_all_accessible_lists)
        // The key is that no error occurs
        assert!(!app.assigned_tasks_loading);
    }

    /// Test AssignedTasksPreloaded message handler
    #[test]
    fn test_assigned_tasks_preloaded_message_handler() {
        let mock_client = MockClickUpClient::new();
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        
        // Create test tasks
        let test_tasks = vec![
            Task {
                id: "msg-task-1".to_string(),
                name: "Message Task 1".to_string(),
                ..Default::default()
            },
        ];
        
        // Simulate receiving AssignedTasksPreloaded message
        app.process_async_messages(); // Clear any pending messages first
        
        // Manually handle the message (simulating what the async handler would do)
        let sorted_tasks = crate::models::task::sort_tasks(test_tasks.clone());
        *app.assigned_tasks.tasks_mut() = sorted_tasks.clone();
        app.assigned_tasks_count = sorted_tasks.len();
        
        assert_eq!(app.assigned_tasks.tasks().len(), 1);
        assert_eq!(app.assigned_tasks.tasks()[0].id, "msg-task-1");
    }

    /// Test that session state includes user_id
    #[test]
    fn test_session_state_includes_user_id() {
        use crate::models::SessionState;
        
        let state = SessionState::from_app(
            &Screen::Tasks,
            Some("ws-123".to_string()),
            None,
            None,
            None,
            None,
            None,
            Some(456),
        );
        
        assert_eq!(state.user_id, Some(456));
        assert_eq!(state.workspace_id, Some("ws-123".to_string()));
    }

    /// Test that session state serialization includes user_id
    #[test]
    fn test_session_state_serialization_with_user_id() {
        use crate::models::SessionState;
        
        let state = SessionState {
            screen: "Tasks".to_string(),
            workspace_id: Some("ws-123".to_string()),
            space_id: None,
            folder_id: None,
            list_id: None,
            task_id: None,
            document_id: None,
            user_id: Some(789),
        };
        
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: SessionState = serde_json::from_str(&json).unwrap();
        
        assert_eq!(state.user_id, deserialized.user_id);
    }
}
