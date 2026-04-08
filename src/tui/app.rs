//! Main TUI application

use anyhow::Result;
use arboard::Clipboard;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::Rect;
use ratatui::Frame;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::api::{AuthManager, ClickUpApi, ClickUpClient};
use crate::cache::CacheManager;
use crate::config::ConfigManager;
use crate::models::{
    ClickUpSpace, Comment, CreateCommentRequest, Document, Folder, List, SessionState, Task,
    UpdateCommentRequest, User, Workspace,
};
use crate::tui::widgets::SidebarItem;
use crate::utils::{ClickUpUrlGenerator, ClipboardService, UrlGenerator};

use super::input::{is_quit, InputEvent};
use super::layout::{generate_screen_title, split_task_detail, TuiLayout};
use super::terminal;
use super::widgets::{
    get_dialog_hints, render_assignee_picker, render_auth, render_comments, render_dialog,
    render_document, render_help, render_sidebar, render_task_detail, render_task_list, AuthState,
    DialogState, DialogType, DocumentState, HelpState, SidebarState, TaskDetailState,
    TaskListState,
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
    CurrentUserLoaded(Result<User, String>),
    MembersLoaded(Result<Vec<User>, String>),
    AssigneesUpdated(Result<Task, String>),
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

    /// Comment UI state
    comment_selected_index: usize,
    comment_editing_index: Option<usize>,
    comment_new_text: String,
    comment_focus: bool, // true = focus on comments, false = focus on task form

    /// Comment thread navigation state
    comment_view_mode: CommentViewMode,
    comment_previous_selection: Option<usize>, // Store selection when entering thread

    /// Per-list assigned filter state
    assigned_filter_active: bool,

    /// User identity for assignee filtering
    current_user_id: Option<i32>,

    /// In-memory cache for list members (keyed by list ID)
    cached_list_members: std::collections::HashMap<String, Vec<User>>,

    /// Assignee picker UI state
    assignee_picker_open: bool,
    assignee_picker_members: Vec<User>,
    assignee_picker_selected: std::collections::HashSet<i64>,
    assignee_picker_original: std::collections::HashSet<i64>,
    assignee_picker_cursor: usize,

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

/// Test-only methods
impl TuiApp {
    /// Get the current user ID (for testing)
    #[allow(dead_code)]
    pub fn current_user_id(&self) -> Option<i32> {
        self.current_user_id
    }

    /// Get the message sender for testing (sending async messages)
    #[allow(dead_code)]
    pub fn message_tx_for_testing(&self) -> mpsc::Sender<AppMessage> {
        self.message_tx.clone().expect("message_tx should be set")
    }

    /// Set the assigned filter active state (for testing)
    #[allow(dead_code)]
    pub fn set_assigned_filter_active(&mut self, active: bool) {
        self.assigned_filter_active = active;
    }

    /// Set the current list ID (for testing)
    #[allow(dead_code)]
    pub fn set_current_list_id(&mut self, list_id: Option<String>) {
        self.current_list_id = list_id;
    }

    /// Set the current user ID (for testing)
    #[allow(dead_code)]
    pub fn set_current_user_id(&mut self, user_id: Option<i32>) {
        self.current_user_id = user_id;
    }

    /// Check if loading is in progress (for testing)
    #[allow(dead_code)]
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Get the current status message (for testing)
    #[allow(dead_code)]
    pub fn status_message(&self) -> &str {
        &self.status
    }

    /// Check if the assignee picker is open (for testing)
    #[allow(dead_code)]
    pub fn is_assignee_picker_open(&self) -> bool {
        self.assignee_picker_open
    }

    /// Set the cached list members (for testing)
    #[allow(dead_code)]
    pub fn set_cached_list_members(&mut self, list_id: &str, members: Vec<User>) {
        self.cached_list_members
            .insert(list_id.to_string(), members);
    }
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
            comment_selected_index: 0,
            comment_editing_index: None,
            comment_new_text: String::new(),
            comment_focus: false,
            comment_view_mode: CommentViewMode::TopLevel,
            comment_previous_selection: None,
            assigned_filter_active: false,
            current_user_id: None,
            cached_list_members: std::collections::HashMap::new(),
            assignee_picker_open: false,
            assignee_picker_members: Vec::new(),
            assignee_picker_selected: std::collections::HashSet::new(),
            assignee_picker_original: std::collections::HashSet::new(),
            assignee_picker_cursor: 0,
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

                // Fetch current user profile in background for assignee filtering
                if let Some(client) = &app.client {
                    let client = client.clone();
                    let tx = app.message_tx.clone().unwrap();
                    tokio::spawn(async move {
                        match client.get_current_user().await {
                            Ok(user) => {
                                let _ = tx.send(AppMessage::CurrentUserLoaded(Ok(user))).await;
                            }
                            Err(e) => {
                                let _ = tx
                                    .send(AppMessage::CurrentUserLoaded(Err(e.to_string())))
                                    .await;
                            }
                        }
                    });
                }
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
            comment_selected_index: 0,
            comment_editing_index: None,
            comment_new_text: String::new(),
            comment_focus: false,
            comment_view_mode: CommentViewMode::TopLevel,
            comment_previous_selection: None,
            assigned_filter_active: false,
            current_user_id: None,
            cached_list_members: std::collections::HashMap::new(),
            assignee_picker_open: false,
            assignee_picker_members: Vec::new(),
            assignee_picker_selected: std::collections::HashSet::new(),
            assignee_picker_original: std::collections::HashSet::new(),
            assignee_picker_cursor: 0,
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

    /// Create a new TUI app with a custom client and in-memory cache (for testing)
    #[allow(dead_code)]
    pub fn with_client_and_test_cache(client: Arc<dyn ClickUpApi>) -> Result<Self> {
        use std::env;

        // Use a unique temporary database file for tests to avoid FK conflicts
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join(format!("clickdown_test_{}.db", std::process::id()));

        // Remove existing file if present
        let _ = std::fs::remove_file(&db_path);

        let cache = CacheManager::new(db_path)?;
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
            comment_selected_index: 0,
            comment_editing_index: None,
            comment_new_text: String::new(),
            comment_focus: false,
            comment_view_mode: CommentViewMode::TopLevel,
            comment_previous_selection: None,
            assigned_filter_active: false,
            current_user_id: None,
            cached_list_members: std::collections::HashMap::new(),
            assignee_picker_open: false,
            assignee_picker_members: Vec::new(),
            assignee_picker_selected: std::collections::HashSet::new(),
            assignee_picker_original: std::collections::HashSet::new(),
            assignee_picker_cursor: 0,
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
                                // Populate sidebar with workspaces
                                let mut items = Vec::new();
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
                                // Populate sidebar with spaces
                                let mut items = Vec::new();
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
                                // Populate sidebar with folders
                                let mut items = Vec::new();
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
                                // Populate sidebar with lists
                                let mut items = Vec::new();
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
                    AppMessage::CurrentUserLoaded(result) => {
                        match result {
                            Ok(user) => {
                                self.current_user_id = Some(user.id as i32);
                                tracing::info!(
                                    "Detected current user ID from API: {} ({})",
                                    user.id,
                                    user.username
                                );

                                // If assigned filter is active, re-fetch with the fresh user ID
                                // to replace any stale results from a cached ID
                                if self.assigned_filter_active {
                                    if let Some(list_id) = &self.current_list_id {
                                        self.load_tasks_with_assigned_filter(list_id.clone());
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Background user profile fetch failed: {}", e);
                            }
                        }
                    }
                    AppMessage::MembersLoaded(result) => {
                        match result {
                            Ok(members) => {
                                // Cache the members
                                if let Some(list_id) = &self.current_list_id {
                                    self.cached_list_members
                                        .insert(list_id.clone(), members.clone());
                                }
                                // Open the picker
                                self.open_assignee_picker(members);
                            }
                            Err(e) => {
                                self.status = format!("Failed to load members: {}", e);
                            }
                        }
                    }
                    AppMessage::AssigneesUpdated(result) => {
                        match result {
                            Ok(updated_task) => {
                                // Update the task in the tasks list (app cache)
                                for task in &mut self.tasks {
                                    if task.id == updated_task.id {
                                        *task = updated_task.clone();
                                        break;
                                    }
                                }
                                // Update the task in the task list widget
                                for task in self.task_list.tasks_mut() {
                                    if task.id == updated_task.id {
                                        *task = updated_task.clone();
                                        break;
                                    }
                                }
                                // Update task detail view
                                self.task_detail.task = Some(updated_task.clone());
                                self.assignee_picker_open = false;
                                self.status = "Assignees updated".to_string();
                            }
                            Err(e) => {
                                self.status = format!("Failed to update assignees: {}", e);
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
                KeyCode::Char('a') => {
                    // Toggle "Assigned to Me" filter
                    self.assigned_filter_active = !self.assigned_filter_active;
                    if let Some(list_id) = &self.current_list_id {
                        if self.assigned_filter_active {
                            self.status = "Filtering: Assigned to Me".to_string();
                            self.load_tasks_with_assigned_filter(list_id.clone());
                        } else {
                            self.status = "Showing all tasks".to_string();
                            self.load_tasks(list_id.clone());
                        }
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
        // Handle assignee picker input first (takes priority)
        if self.assignee_picker_open {
            if let InputEvent::Key(key) = event {
                match key.code {
                    KeyCode::Char('j') => {
                        if self.assignee_picker_cursor
                            < self.assignee_picker_members.len().saturating_sub(1)
                        {
                            self.assignee_picker_cursor += 1;
                        }
                    }
                    KeyCode::Char('k') => {
                        if self.assignee_picker_cursor > 0 {
                            self.assignee_picker_cursor -= 1;
                        }
                    }
                    KeyCode::Char(' ') => {
                        // Toggle selection of current member
                        if let Some(member) = self
                            .assignee_picker_members
                            .get(self.assignee_picker_cursor)
                        {
                            if self.assignee_picker_selected.contains(&member.id) {
                                self.assignee_picker_selected.remove(&member.id);
                            } else {
                                self.assignee_picker_selected.insert(member.id);
                            }
                        }
                    }
                    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Save assignees
                        self.save_assignees();
                    }
                    KeyCode::Esc => {
                        // Cancel
                        self.assignee_picker_open = false;
                        self.status = "Assignment cancelled".to_string();
                    }
                    _ => {}
                }
            }
            return;
        }

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
                KeyCode::Char('A') if !self.comment_focus => {
                    // Open assignee picker
                    self.open_assignee_picker_flow();
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

    /// Navigate into the selected item (public for testing)
    pub fn navigate_into(&mut self) {
        // Navigate based on current screen and selection
        // Clone the selected item to avoid borrow checker issues
        let selected_item = self.sidebar.selected_item().cloned();

        // Handle navigation based on current screen
        match &self.screen {
            Screen::Workspaces => {
                if let Some(SidebarItem::Workspace { id, name }) = selected_item {
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
                if let Some(SidebarItem::Space { id, name, .. }) = selected_item {
                    self.current_space_id = Some(id.clone());
                    self.current_folder_id = None;
                    self.current_list_id = None;
                    self.load_folders(id.clone());
                    self.screen = Screen::Folders;
                    self.screen_title = generate_screen_title(&name);
                }
            }
            Screen::Folders => {
                if let Some(SidebarItem::Folder { id, name, .. }) = selected_item {
                    self.current_folder_id = Some(id.clone());
                    self.current_list_id = None;
                    self.load_lists(id.clone());
                    self.screen = Screen::Lists;
                    self.screen_title = generate_screen_title(&name);
                }
            }
            Screen::Lists => {
                if let Some(SidebarItem::List { id, name, .. }) = selected_item {
                    self.current_list_id = Some(id.clone());
                    self.load_tasks(id.clone());
                    self.screen = Screen::Tasks;
                    self.screen_title = generate_screen_title(&format!("Tasks: {}", name));
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
            Screen::Spaces => {
                // Navigate back to Workspaces
                self.current_space_id = None;
                self.current_folder_id = None;
                self.current_list_id = None;

                // Repopulate sidebar with workspaces
                let mut items = Vec::new();
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

                // Repopulate sidebar with spaces
                let mut items = Vec::new();
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

                // Repopulate sidebar with folders
                let mut items = Vec::new();
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

                // Repopulate sidebar with lists
                let mut items = Vec::new();
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

    /// Load tasks filtered by the current user as assignee
    fn load_tasks_with_assigned_filter(&mut self, list_id: String) {
        self.loading = true;
        self.status = "Loading assigned tasks...".to_string();

        let user_id = match self.current_user_id {
            Some(id) => id,
            None => {
                self.loading = false;
                self.status = "User ID not available for filtering".to_string();
                return;
            }
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
        tokio::spawn(async move {
            let result = client
                .get_tasks_with_assignee(&list_id, user_id, Some(100))
                .await;
            let msg = match result {
                Ok(tasks) => AppMessage::TasksLoaded(Ok(tasks)),
                Err(e) => AppMessage::TasksLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    fn load_tasks(&mut self, list_id: String) {
        // If the assigned filter is active, use the filtered version
        if self.assigned_filter_active {
            self.load_tasks_with_assigned_filter(list_id);
            return;
        }

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

    /// Open the assignee picker with the given members list
    fn open_assignee_picker(&mut self, members: Vec<User>) {
        // Build the set of currently assigned user IDs from the selected task
        let current_assignee_ids: std::collections::HashSet<i64> = self
            .task_detail
            .task
            .as_ref()
            .map(|t| t.assignees.iter().map(|u| u.id).collect())
            .unwrap_or_default();

        self.assignee_picker_members = members;
        self.assignee_picker_selected = current_assignee_ids.clone();
        self.assignee_picker_original = current_assignee_ids;
        self.assignee_picker_cursor = 0;
        self.assignee_picker_open = true;
    }

    /// Open the assignee picker, fetching members if not cached
    fn open_assignee_picker_flow(&mut self) {
        // Guard: need a task
        if self.task_detail.task.is_none() {
            self.status = "No task selected".to_string();
            return;
        }

        // Guard: need list context
        let list_id = match &self.current_list_id {
            Some(id) => id.clone(),
            None => {
                self.status = "Cannot assign: list context not available".to_string();
                return;
            }
        };

        // Check cache first
        if let Some(cached) = self.cached_list_members.get(&list_id) {
            self.open_assignee_picker(cached.clone());
            return;
        }

        // Cache miss: fetch from API
        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.status = "Not authenticated".to_string();
                return;
            }
        };

        self.loading = true;
        self.status = "Loading members...".to_string();

        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client.get_list_members(&list_id).await;
            let msg = match result {
                Ok(members) => AppMessage::MembersLoaded(Ok(members)),
                Err(e) => AppMessage::MembersLoaded(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Save current assignee selection to the task
    fn save_assignees(&mut self) {
        let task = match &self.task_detail.task {
            Some(t) => t.clone(),
            None => {
                self.status = "No task selected".to_string();
                return;
            }
        };

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.status = "Not authenticated".to_string();
                return;
            }
        };

        let task_id = task.id.clone();
        let tx = self.message_tx.clone().unwrap();

        use crate::models::{AssigneesUpdate, UpdateTaskRequest};
        let update = UpdateTaskRequest {
            name: None,
            description: None,
            status: None,
            priority: None,
            assignees: Some(AssigneesUpdate::replace_all(
                self.assignee_picker_original.clone(),
                self.assignee_picker_selected.clone(),
            )),
            due_date: None,
        };

        tokio::spawn(async move {
            let result = client.update_task(&task_id, &update).await;
            let msg = match result {
                Ok(task) => AppMessage::AssigneesUpdated(Ok(task)),
                Err(e) => AppMessage::AssigneesUpdated(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
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
                    let base = format!("Tasks: {}", list.name);
                    if self.assigned_filter_active {
                        generate_screen_title(&format!("{} (Assigned to Me)", base))
                    } else {
                        generate_screen_title(&base)
                    }
                } else {
                    if self.assigned_filter_active {
                        generate_screen_title("Tasks (Assigned to Me)")
                    } else {
                        generate_screen_title("Tasks")
                    }
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

            // Render assignee picker overlay if open
            if self.assignee_picker_open {
                render_assignee_picker(
                    frame,
                    area,
                    &self.assignee_picker_members,
                    &self.assignee_picker_selected,
                    self.assignee_picker_cursor,
                );
            }

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
        render_sidebar(frame, &self.sidebar, sidebar_area);
        self.render_main_content(frame, content_area);
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

        // Restore user ID for assignee filtering (only if valid - not 0)
        if let Some(uid) = saved_state.user_id {
            if uid != 0 {
                self.current_user_id = Some(uid);
                tracing::debug!("Restored user_id from session cache: {}", uid);
            } else {
                tracing::debug!("Ignoring invalid cached user_id: 0, will fetch fresh");
            }
        }

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
        }
    }

    /// Get the current screen (public for testing)
    #[allow(dead_code)]
    pub fn screen(&self) -> Screen {
        self.screen.clone()
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

    /// Test that task list widget is updated when assignees are saved
    /// This is a regression test for the bug where task_list.tasks_mut() was not updated
    #[tokio::test]
    async fn test_assignee_update_reflects_in_task_list_widget() {
        use crate::models::User;

        // Create a task with no assignees
        let task = Task {
            id: "test-task-1".to_string(),
            name: "Test Task".to_string(),
            assignees: vec![],
            description: None,
            status: None,
            priority: None,
            due_date: None,
            ..Default::default()
        };

        // Create updated task with assignees (what API returns after update)
        let updated_task = Task {
            id: task.id.clone(),
            name: task.name.clone(),
            assignees: vec![
                User {
                    id: 1,
                    username: "Alice".to_string(),
                    color: None,
                    email: None,
                    profile_picture: None,
                    initials: None,
                },
                User {
                    id: 2,
                    username: "Bob".to_string(),
                    color: None,
                    email: None,
                    profile_picture: None,
                    initials: None,
                },
            ],
            description: task.description.clone(),
            status: task.status.clone(),
            priority: task.priority.clone(),
            due_date: task.due_date,
            ..task.clone()
        };

        let mock_client = MockClickUpClient::new()
            .with_tasks(vec![task.clone()])
            .with_update_task_response(updated_task.clone());

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Simulate loading tasks into both the app cache and the widget
        app.tasks = vec![task.clone()];
        *app.task_list.tasks_mut() = vec![task.clone()];

        // Select the task in the task list
        app.task_list.select(Some(0));

        // Open task detail view (simulating pressing Enter on task)
        app.task_detail.task = Some(task.clone());

        // Send the AssigneesUpdated message through the channel
        let tx = app.message_tx.clone().unwrap();
        let _ = tx.try_send(AppMessage::AssigneesUpdated(Ok(updated_task)));

        // Process async messages (this will trigger the actual handler code)
        app.process_async_messages();

        // BUG REPRODUCTION: Before the fix, task_list widget still had the old task data
        // This test will FAIL before the fix and PASS after the fix
        let widget_task = app
            .task_list
            .selected_task()
            .expect("Task should be selected");

        assert_eq!(
            widget_task.assignees.len(),
            2,
            "Task list widget should have updated assignees"
        );
        assert_eq!(widget_task.assignees[0].username, "Alice");
        assert_eq!(widget_task.assignees[1].username, "Bob");
    }
}
