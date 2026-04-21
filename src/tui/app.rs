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
    ClickUpSpace, Comment, CreateCommentRequest, CreateTaskRequest, Document, Folder, List, SessionState, Task,
    UpdateCommentRequest, User, Workspace,
};
use crate::tui::widgets::SidebarItem;
use crate::utils::{ClickUpUrlGenerator, ClipboardService, UrlGenerator};

use super::input::{is_quit, InputEvent};
use super::layout::{generate_screen_title, split_task_detail, TuiLayout};
use super::terminal;
use super::widgets::{
    get_dialog_hints, get_help_hints, render_assignee_picker, render_auth, render_comments,
    render_dialog, render_document, render_help, render_sidebar, render_status_picker,
    render_task_detail, render_task_list, AuthState, DialogState, DialogType, DocumentState,
    GroupedTaskList, HelpContext, HelpState, ListRow, SidebarState, TaskDetailState,
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

/// Focus field for task creation form
#[derive(Debug, Clone, PartialEq)]
pub enum TaskCreationField {
    Name,
    Description,
}

#[derive(Debug, Clone)]
pub struct CommentsLoadedResponse {
    all_comments: Vec<Comment>,
    top_level_comments: usize
}

/// Async messages for API results
#[derive(Debug, Clone)]
pub enum AppMessage {
    WorkspacesLoaded(Result<Vec<Workspace>, String>),
    SpacesLoaded(Result<Vec<ClickUpSpace>, String>),
    FoldersLoaded(Result<Vec<Folder>, String>),
    ListsLoaded(Result<Vec<List>, String>),
    TasksLoaded(Result<Vec<Task>, String>),
    CommentsLoaded(Result<CommentsLoadedResponse, String>),
    CommentCreated(Result<Comment, String>, bool), // bool = is_reply
    CommentUpdated(Result<Comment, String>),
    CurrentUserLoaded(Result<User, String>),
    MembersLoaded(Result<Vec<User>, String>),
    AssigneesUpdated(Result<Task, String>),
    TaskStatusUpdated(Result<Task, String>),
    // URL navigation async messages
    TaskFetchedForNavigation(Result<Task, String>, Screen),
    CommentFetchedForNavigation(Result<Task, String>, String, Screen), // task result, comment_id, prev_screen
    DocumentFetchedForNavigation(Result<Document, String>, Screen),
    CommentsLoadedForCommentNavigation(Result<Vec<Comment>, String>, String), // comments, comment_id
    // Task creation
    TaskCreated(Result<Task, String>),
    // Task deletion
    TaskDeleted(Result<String, String>), // Ok(task_id) or Err(message)
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

    /// Task list state (grouped by status)
    task_list: GroupedTaskList,

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
    comment_top_level_count: usize, // stores top level comment length
    comment_editing_index: Option<usize>,
    comment_new_text: String,
    comment_focus: bool, // true = focus on comments, false = focus on task form

    /// Comment thread navigation state
    comment_view_mode: CommentViewMode,
    comment_previous_selection: Option<usize>, // Store selection when entering thread

    /// Task creation form state
    task_name_input: String,
    task_description_input: String,
    task_creating: bool,
    task_creation_focus: TaskCreationField,

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

    /// Status picker UI state
    status_picker_open: bool,
    status_picker_statuses: Vec<crate::models::TaskStatus>,
    status_picker_cursor: usize,
    status_picker_original_status: Option<String>,
    status_picker_task_id: Option<String>,

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

    /// Keyboard chord leader key state (for `g` → `u` style shortcuts)
    chord_leader_pending: Option<KeyCode>,

    /// URL input dialog state
    url_input_open: bool,
    url_input_text: String,
    url_input_error: Option<String>,
    url_input_cursor: usize,

    /// Navigation loading state for URL-based navigation
    navigating: bool,
    navigating_level: String,
}

/// Application state
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Initializing,
    Unauthenticated,
    Main,
    Quitting,
}

/// Test-only methods
impl TuiApp {
    /// Check if any text input is currently active (comments, task creation, URL input, etc.)
    /// Used to guard global shortcuts like 'u' for URL copy
    #[allow(dead_code)]
    pub fn is_text_input_active(&self) -> bool {
        self.url_input_open
            || self.status_picker_open
            || self.assignee_picker_open
            || self.task_creating
            || self.comment_editing_index.is_some()
            || !self.comment_new_text.is_empty()
    }

    /// Handle text input when any text input field is active
    /// Delegates to the appropriate handler based on which input is active
    pub fn handle_text_input(&mut self, key: crossterm::event::KeyEvent) {
        if self.url_input_open {
            self.handle_url_input(key);
        } else if self.status_picker_open {
            self.handle_status_picker_input(key);
        } else if self.assignee_picker_open {
            self.handle_assignee_picker_input(key);
        } else if self.task_creating {
            self.handle_task_creation_input(key);
        } else if self.comment_editing_index.is_some() || !self.comment_new_text.is_empty() {
            self.handle_comment_input(key);
        }
    }

    /// Handle keyboard input within the assignee picker
    fn handle_assignee_picker_input(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
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
                self.save_assignees();
            }
            KeyCode::Esc => {
                self.assignee_picker_open = false;
                self.status = "Assignment cancelled".to_string();
            }
            _ => {}
        }
    }

    /// Handle keyboard input within task creation form
    fn handle_task_creation_input(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        match key.code {
            KeyCode::Tab => {
                self.task_creation_focus = match self.task_creation_focus {
                    TaskCreationField::Name => TaskCreationField::Description,
                    TaskCreationField::Description => TaskCreationField::Name,
                };
                self.status = format!(
                    "Focus: {}",
                    match self.task_creation_focus {
                        TaskCreationField::Name => "Task Name",
                        TaskCreationField::Description => "Description",
                    }
                );
                return;
            }
            KeyCode::Esc => {
                self.task_name_input.clear();
                self.task_description_input.clear();
                self.task_creating = false;
                self.task_detail.creating = false;
                self.screen = Screen::Tasks;
                self.update_screen_title();
                self.status = "Task creation cancelled".to_string();
                return;
            }
            KeyCode::Enter => {
                if matches!(self.task_creation_focus, TaskCreationField::Name) {
                    self.task_creation_focus = TaskCreationField::Description;
                    self.status = "Focus: Description".to_string();
                }
                return;
            }
            _ => {}
        }
        if let KeyCode::Char('s') = key.code {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                if self.task_name_input.trim().is_empty() {
                    self.status = "Task name is required".to_string();
                    return;
                }
                if let Some(list_id) = &self.current_list_id {
                    self.create_task(list_id.clone());
                }
                return;
            }
        }
        if let KeyCode::Char(c) = key.code {
            // Add character to focused field
            match self.task_creation_focus {
                TaskCreationField::Name => self.task_name_input.push(c),
                TaskCreationField::Description => self.task_description_input.push(c),
            }
            return;
        }
        if let KeyCode::Backspace = key.code {
            // Remove character from focused field
            match self.task_creation_focus {
                TaskCreationField::Name => {
                    self.task_name_input.pop();
                }
                TaskCreationField::Description => {
                    self.task_description_input.pop();
                }
            }
            return;
        }
    }

    /// Handle keyboard input within comment editing
    fn handle_comment_input(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        match key.code {
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(edit_idx) = self.comment_editing_index {
                    if edit_idx == usize::MAX {
                        if let Some(_task) = &self.task_detail.task {
                            let text = self.comment_new_text.clone();
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
                            let parent_id = match &self.comment_view_mode {
                                CommentViewMode::InThread {
                                    parent_comment_id, ..
                                } => Some(parent_comment_id.clone()),
                                CommentViewMode::TopLevel => None,
                            };
                            self.create_comment(task_id, text, parent_id);
                        }
                    } else {
                        if let Some(_task) = &self.task_detail.task {
                            let text = self.comment_new_text.clone();
                            if text.trim().is_empty() {
                                self.status = "Comment cannot be empty".to_string();
                                return;
                            }
                            let comment_id = self.comments[edit_idx].id.clone();
                            self.update_comment(comment_id, text);
                        }
                    }
                } else if !self.comment_new_text.is_empty() {
                    if let Some(_task) = &self.task_detail.task {
                        let text = self.comment_new_text.clone();
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
                        let parent_id = match &self.comment_view_mode {
                            CommentViewMode::InThread {
                                parent_comment_id, ..
                            } => Some(parent_comment_id.clone()),
                            CommentViewMode::TopLevel => None,
                        };
                        self.create_comment(task_id, text, parent_id);
                    }
                }
            }
            KeyCode::Esc => {
                self.comment_new_text.clear();
                self.comment_editing_index = None;
                self.status = if matches!(self.comment_view_mode, CommentViewMode::InThread { .. })
                {
                    "Reply cancelled".to_string()
                } else {
                    "Comment editing cancelled".to_string()
                };
            }
            KeyCode::Char(c) => {
                self.comment_new_text.push(c);
            }
            KeyCode::Backspace => {
                self.comment_new_text.pop();
            }
            _ => {}
        }
    }

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

    /// Check if URL input dialog is open (for testing)
    #[allow(dead_code)]
    pub fn is_url_input_open(&self) -> bool {
        self.url_input_open
    }

    /// Get the URL input text (for testing)
    #[allow(dead_code)]
    pub fn url_input_text(&self) -> &str {
        &self.url_input_text
    }

    /// Check if help dialog is visible (for testing)
    #[allow(dead_code)]
    pub fn is_help_visible(&self) -> bool {
        self.help.visible
    }

    /// Check if comment editing is active (for testing)
    #[allow(dead_code)]
    pub fn is_comment_editing_active(&self) -> bool {
        self.comment_editing_index.is_some()
    }

    /// Check if task creation is active (for testing)
    #[allow(dead_code)]
    pub fn is_task_creating(&self) -> bool {
        self.task_creating
    }

    /// Get the comment new text (for testing)
    #[allow(dead_code)]
    pub fn comment_new_text(&self) -> &str {
        &self.comment_new_text
    }

    /// Get the task name input (for testing)
    #[allow(dead_code)]
    pub fn task_name_input(&self) -> &str {
        &self.task_name_input
    }

    /// Get the task description input (for testing)
    #[allow(dead_code)]
    pub fn task_description_input(&self) -> &str {
        &self.task_description_input
    }

    /// Get the task creation focus field (for testing)
    #[allow(dead_code)]
    pub fn task_creation_focus(&self) -> &TaskCreationField {
        &self.task_creation_focus
    }

    /// Set task creating state (for testing)
    #[allow(dead_code)]
    pub fn set_task_creating(&mut self, creating: bool) {
        self.task_creating = creating;
        self.task_detail.creating = creating;
    }

    /// Set task creation focus (for testing)
    #[allow(dead_code)]
    pub fn set_task_creation_focus(&mut self, focus: TaskCreationField) {
        self.task_creation_focus = focus;
    }

    /// Set task detail task (for testing)
    #[allow(dead_code)]
    pub fn set_task_detail_task(&mut self, task: Task) {
        self.task_detail.task = Some(task);
    }

    /// Set comments (for testing)
    #[allow(dead_code)]
    pub fn set_comments(&mut self, comments: Vec<crate::models::Comment>) {
        self.comments = comments;
    }

    /// Set comment focus (for testing)
    #[allow(dead_code)]
    pub fn set_comment_focus(&mut self, focus: bool) {
        self.comment_focus = focus;
    }

    /// Get help dialog current page (for testing)
    #[allow(dead_code)]
    pub fn help_page(&self) -> u8 {
        self.help.page
    }

    /// Get the URL input error message (for testing)
    #[allow(dead_code)]
    pub fn url_input_error(&self) -> Option<&str> {
        self.url_input_error.as_deref()
    }

    /// Get the task list for testing
    #[allow(dead_code)]
    pub fn task_list_for_test(&self) -> &crate::tui::widgets::GroupedTaskList {
        &self.task_list
    }

    /// Get mutable access to tasks for testing
    #[allow(dead_code)]
    pub fn tasks_mut_for_test(&mut self) -> &mut Vec<Task> {
        &mut self.tasks
    }

    /// Get mutable access to task list for testing
    #[allow(dead_code)]
    pub fn task_list_mut_for_test(&mut self) -> &mut crate::tui::widgets::GroupedTaskList {
        &mut self.task_list
    }

    /// Get mutable access to dialog for testing
    #[allow(dead_code)]
    pub fn dialog_mut_for_test(&mut self) -> &mut crate::tui::widgets::DialogState {
        &mut self.dialog
    }

    /// Set screen for testing
    #[allow(dead_code)]
    pub fn set_screen_for_test(&mut self, screen: Screen) {
        self.screen = screen;
    }

    /// Check if dialog is visible (for testing)
    #[allow(dead_code)]
    pub fn is_dialog_visible(&self) -> bool {
        self.dialog.is_visible()
    }

    /// Get dialog type (for testing)
    #[allow(dead_code)]
    pub fn dialog_type_for_test(&self) -> Option<&crate::tui::widgets::DialogType> {
        self.dialog.dialog_type.as_ref()
    }

    /// Check if dialog is confirmed (for testing)
    #[allow(dead_code)]
    pub fn is_dialog_confirmed(&self) -> bool {
        self.dialog.confirmed()
    }

    /// Get task count (for testing)
    #[allow(dead_code)]
    pub fn task_count(&self) -> usize {
        self.tasks.len()
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
            task_list: GroupedTaskList::new(),
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
            comment_top_level_count: 0,
            comment_view_mode: CommentViewMode::TopLevel,
            comment_previous_selection: None,
            task_name_input: String::new(),
            task_description_input: String::new(),
            task_creating: false,
            task_creation_focus: TaskCreationField::Name,
            assigned_filter_active: false,
            current_user_id: None,
            cached_list_members: std::collections::HashMap::new(),
            assignee_picker_open: false,
            assignee_picker_members: Vec::new(),
            assignee_picker_selected: std::collections::HashSet::new(),
            assignee_picker_original: std::collections::HashSet::new(),
            assignee_picker_cursor: 0,
            status_picker_open: false,
            status_picker_statuses: Vec::new(),
            status_picker_cursor: 0,
            status_picker_original_status: None,
            status_picker_task_id: None,
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
            chord_leader_pending: None,
            url_input_open: false,
            url_input_text: String::new(),
            url_input_error: None,
            url_input_cursor: 0,
            navigating: false,
            navigating_level: String::new(),
        };
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
            task_list: GroupedTaskList::new(),
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
            comment_top_level_count: 0,
            task_name_input: String::new(),
            task_description_input: String::new(),
            task_creating: false,
            task_creation_focus: TaskCreationField::Name,
            assigned_filter_active: false,
            current_user_id: None,
            cached_list_members: std::collections::HashMap::new(),
            assignee_picker_open: false,
            assignee_picker_members: Vec::new(),
            assignee_picker_selected: std::collections::HashSet::new(),
            assignee_picker_original: std::collections::HashSet::new(),
            assignee_picker_cursor: 0,
            status_picker_open: false,
            status_picker_statuses: Vec::new(),
            status_picker_cursor: 0,
            status_picker_original_status: None,
            status_picker_task_id: None,
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
            chord_leader_pending: None,
            url_input_open: false,
            url_input_text: String::new(),
            url_input_error: None,
            url_input_cursor: 0,
            navigating: false,
            navigating_level: String::new(),
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
            task_list: GroupedTaskList::new(),
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
            comment_top_level_count: 0,
            comment_new_text: String::new(),
            comment_focus: false,
            comment_view_mode: CommentViewMode::TopLevel,
            comment_previous_selection: None,
            task_name_input: String::new(),
            task_description_input: String::new(),
            task_creating: false,
            task_creation_focus: TaskCreationField::Name,
            assigned_filter_active: false,
            current_user_id: None,
            cached_list_members: std::collections::HashMap::new(),
            assignee_picker_open: false,
            assignee_picker_members: Vec::new(),
            assignee_picker_selected: std::collections::HashSet::new(),
            assignee_picker_original: std::collections::HashSet::new(),
            assignee_picker_cursor: 0,
            status_picker_open: false,
            status_picker_statuses: Vec::new(),
            status_picker_cursor: 0,
            status_picker_original_status: None,
            status_picker_task_id: None,
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
            chord_leader_pending: None,
            url_input_open: false,
            url_input_text: String::new(),
            url_input_error: None,
            url_input_cursor: 0,
            navigating: false,
            navigating_level: String::new(),
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
                    InputEvent::Key(_) | InputEvent::Resize => self.update(event),
                    InputEvent::None => {}
                }
            }

            // Check if update() signaled to quit (dialog confirmation moved to update())
            if self.state == AppState::Quitting {
                if let Err(e) = self.save_session_state() {
                    tracing::error!("Failed to save session state: {}", e);
                }
                break;
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
                                // Store tasks as source of truth
                                self.tasks = tasks;
                                // Build grouped task list
                                self.task_list = GroupedTaskList::from_tasks(self.tasks.clone());

                                // Check if we're restoring a session
                                if self.restoring_session {
                                    // Try to select the restored task
                                    if let Some(ref restored_id) = self.restored_task_id {
                                        // Find and select the restored task by ID
                                        let found = self
                                            .task_list
                                            .rows()
                                            .iter()
                                            .enumerate()
                                            .find(|(_, r)| matches!(r, ListRow::Task(t) if &t.id == restored_id))
                                            .map(|(idx, _)| idx);

                                        if let Some(task_idx) = found {
                                            self.task_list.select(Some(task_idx));
                                            self.restoring_session = false;
                                            self.status = format!(
                                                "Restored to Tasks view - {} task(s) loaded",
                                                self.task_list.rows().iter().filter(|r| matches!(r, ListRow::Task(_))).count()
                                            );
                                            tracing::info!("Session restore complete: tasks loaded, task {} selected", restored_id);
                                        } else {
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
                                        self.restoring_session = false;
                                        self.task_list.select_first();
                                        self.status = format!(
                                            "Loaded {} task(s)",
                                            self.task_list.rows().iter().filter(|r| matches!(r, ListRow::Task(_))).count()
                                        );
                                    }
                                } else {
                                    self.task_list.select_first();
                                    self.status = format!(
                                        "Loaded {} task(s)",
                                        self.task_list.rows().iter().filter(|r| matches!(r, ListRow::Task(_))).count()
                                    );
                                }

                                // Clear any previous error state
                                self.error = None;
                            }
                            Err(e) => {
                                self.loading = false;
                                self.error = Some(format!("Failed to load tasks: {}", e));
                                self.status = "Failed to load tasks".to_string();
                                // Clear tasks on error to prevent stale data
                                self.tasks.clear();
                                self.task_list = GroupedTaskList::new();
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
                                tracing::debug!("Loaded {} comments", comments.all_comments.len());
                                for (i, comment) in comments.all_comments.iter().enumerate() {
                                    tracing::debug!(
                                        "Comment {}: id={}, parent_id={:?}, author={:?}",
                                        i,
                                        comment.id,
                                        comment.parent_id,
                                        comment.commenter.as_ref().map(|c| &c.username)
                                    );
                                }
                                self.comment_top_level_count = comments.top_level_comments;
                                self.comments = comments.all_comments;
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

                                if !is_reply {
                                    self.comment_top_level_count += 1;
                                }
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
                                // Rebuild grouped task list (status may have changed group)
                                self.rebuild_task_list();
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
                    AppMessage::TaskStatusUpdated(result) => {
                        match result {
                            Ok(updated_task) => {
                                // Update the task in the tasks list (app cache)
                                for task in &mut self.tasks {
                                    if task.id == updated_task.id {
                                        *task = updated_task.clone();
                                        break;
                                    }
                                }
                                // Rebuild grouped task list (status may have changed group)
                                self.rebuild_task_list();
                                // Update task detail view
                                self.task_detail.task = Some(updated_task.clone());
                                self.status_picker_open = false;
                                self.status = "Status updated".to_string();
                            }
                            Err(e) => {
                                // Rollback: restore original status in tasks
                                if let Some(ref original_status) = self.status_picker_original_status {
                                    if let Some(ref task_id) = self.status_picker_task_id {
                                        for task in &mut self.tasks {
                                            if task.id == *task_id {
                                                task.status = Some(crate::models::TaskStatus {
                                                    id: None,
                                                    status: original_status.clone(),
                                                    color: None,
                                                    type_field: None,
                                                    orderindex: None,
                                                    status_group: None,
                                                });
                                                break;
                                            }
                                        }
                                    }
                                }
                                // Rebuild grouped task list after rollback
                                self.rebuild_task_list();
                                self.status_picker_open = false;
                                self.status = format!("Failed to update status: {}", e);
                            }
                        }
                    }
                    AppMessage::TaskFetchedForNavigation(result, prev_screen) => {
                        match result {
                            Ok(task) => {
                                self.navigating = false;
                                self.navigating_level.clear();
                                // Open the task in detail view
                                self.task_detail.task = Some(task.clone());
                                self.screen = Screen::TaskDetail;
                                self.comment_view_mode = CommentViewMode::TopLevel;
                                self.comments.clear();
                                self.comment_selected_index = 0;
                                self.status = format!("Navigated to task: {}", task.name);
                                // Load comments for the task
                                self.load_comments(task.id);
                            }
                            Err(e) => {
                                self.navigating = false;
                                self.navigating_level.clear();
                                self.status = format!("Resource not found: {}", e);
                                // Restore previous screen
                                self.screen = prev_screen;
                            }
                        }
                    }
                    AppMessage::CommentFetchedForNavigation(result, comment_id, prev_screen) => {
                        match result {
                            Ok(task) => {
                                // Navigate to task detail, then find the comment
                                self.task_detail.task = Some(task.clone());
                                self.screen = Screen::TaskDetail;
                                self.comment_view_mode = CommentViewMode::TopLevel;
                                self.comments.clear();
                                self.comment_selected_index = 0;

                                // Load comments and then find the target one
                                let tx = self.message_tx.clone().unwrap();
                                let task_id = task.id.clone();
                                let client = match &self.client {
                                    Some(c) => c.clone(),
                                    None => {
                                        self.navigating = false;
                                        self.navigating_level.clear();
                                        self.status = "Not authenticated".to_string();
                                        self.screen = prev_screen;
                                        return;
                                    }
                                };
                                tokio::spawn(async move {
                                    let result = client.get_task_comments(&task_id).await;
                                    let msg = match result {
                                        Ok(comments) => {
                                            AppMessage::CommentsLoadedForCommentNavigation(
                                                Ok(comments),
                                                comment_id,
                                            )
                                        }
                                        Err(e) => {
                                            AppMessage::CommentsLoadedForCommentNavigation(
                                                Err(e.to_string()),
                                                comment_id,
                                            )
                                        }
                                    };
                                    let _ = tx.send(msg).await;
                                });
                            }
                            Err(e) => {
                                self.navigating = false;
                                self.navigating_level.clear();
                                self.status = format!("Resource not found: {}", e);
                                self.screen = prev_screen;
                            }
                        }
                    }
                    AppMessage::DocumentFetchedForNavigation(result, prev_screen) => {
                        match result {
                            Ok(doc) => {
                                self.navigating = false;
                                self.navigating_level.clear();
                                self.documents = vec![doc.clone()];
                                self.screen = Screen::Document;
                                self.screen_title = generate_screen_title(&doc.name);
                                self.status = format!("Navigated to document: {}", doc.name);
                            }
                            Err(e) => {
                                self.navigating = false;
                                self.navigating_level.clear();
                                self.status = format!("Document not found: {}", e);
                                self.screen = prev_screen;
                            }
                        }
                    }
                    AppMessage::CommentsLoadedForCommentNavigation(result, comment_id) => {
                        match result {
                            Ok(comments) => {
                                self.comments = comments;
                                self.comment_view_mode = CommentViewMode::TopLevel;
                                // Find and select the target comment
                                let found = self
                                    .comments
                                    .iter()
                                    .position(|c| c.id == comment_id);
                                if let Some(idx) = found {
                                    self.comment_selected_index = idx;
                                    self.navigating = false;
                                    self.navigating_level.clear();
                                    self.status = format!(
                                        "Navigated to comment: {}",
                                        &self.comments[idx].text[..self.comments[idx].text.len().min(40)]
                                    );
                                } else {
                                    self.navigating = false;
                                    self.navigating_level.clear();
                                    self.status = "Comment not found in task".to_string();
                                }
                            }
                            Err(e) => {
                                self.navigating = false;
                                self.navigating_level.clear();
                                self.status = format!("Failed to load comments: {}", e);
                            }
                        }
                    }
                    AppMessage::TaskCreated(result) => {
                        match result {
                            Ok(task) => {
                                self.loading = false;
                                self.task_name_input.clear();
                                self.task_description_input.clear();
                                self.task_creating = false;
                                self.task_detail.creating = false;
                                self.status = format!("Task created: {}", task.name);
                                // Reload the task list and return to tasks view
                                if let Some(list_id) = &self.current_list_id {
                                    let list_id_clone = list_id.clone();
                                    let filter_active = self.assigned_filter_active;
                                    if filter_active {
                                        self.load_tasks_with_assigned_filter(list_id_clone);
                                    } else {
                                        self.load_tasks(list_id_clone);
                                    }
                                }
                                self.screen = Screen::Tasks;
                                self.update_screen_title();
                            }
                            Err(e) => {
                                self.loading = false;
                                self.error = Some(format!("Failed to create task: {}", e));
                                self.status = "Task creation failed".to_string();
                                // Keep task_creating = true so the form stays open
                            }
                        }
                    }
                    AppMessage::TaskDeleted(result) => {
                        match result {
                            Ok(task_id) => {
                                // Remove the task from the local list
                                self.tasks.retain(|t| t.id != task_id);
                                // Clear selection
                                self.task_list.select(None);
                                self.status = "Task deleted".to_string();
                            }
                            Err(e) => {
                                self.error = Some(format!("Failed to delete task: {}", e));
                                self.status = "Failed to delete task".to_string();
                                // Task remains in list — user can retry
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

            // Handle dialog input — return the key event so update() can process it
            // This keeps dialog confirmation testable through the public update() method
            if self.dialog.is_visible() {
                if let event::Event::Key(key) = evt {
                    match key.code {
                        KeyCode::Left | KeyCode::Right => {
                            self.dialog.toggle();
                            return Ok(Some(InputEvent::None));
                        }
                        KeyCode::Enter | KeyCode::Esc => {
                            // Pass through to update() for handling
                            return Ok(Some(InputEvent::Key(key)));
                        }
                        _ => return Ok(Some(InputEvent::None)),
                    }
                }
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
        // When help is visible, handle pagination and close
        if self.help.visible {
            if let InputEvent::Key(key) = event {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down | KeyCode::Right => {
                        self.help.next_page();
                        return;
                    }
                    KeyCode::Char('k') | KeyCode::Up | KeyCode::Left => {
                        self.help.prev_page();
                        return;
                    }
                    KeyCode::Esc => {
                        self.help.hide();
                        return;
                    }
                    KeyCode::Char('?') => {
                        self.help.toggle();
                        return;
                    }
                    // Other keys are ignored while help is visible
                    _ => return,
                }
            }
        }

        // Handle help toggle with ?
        if let InputEvent::Key(key) = event {
            if key.code == KeyCode::Char('?') {
                self.help.toggle();
                return;
            }

            // Handle chord completion: if leader is pending, check for matching second key
            if let Some(leader) = self.chord_leader_pending.take() {
                if leader == KeyCode::Char('g') && key.code == KeyCode::Char('u') {
                    self.open_url_input_dialog();
                    return;
                }
                // Non-matching second key: pass through to normal handling below
                // (the key variable still holds the original KeyEvent)
            } else if key.code == KeyCode::Char('g') && !self.is_text_input_active() {
                // Set leader pending and wait for second key (only when not in text input)
                self.chord_leader_pending = Some(KeyCode::Char('g'));
                return;
            }

            // Handle Esc to clear chord pending state
            if key.code == KeyCode::Esc {
                self.chord_leader_pending = None;
                // Fall through to normal Esc handling
            }

            // Handle URL input dialog (modal overlay)
            // MUST check this BEFORE the 'u' key handler, otherwise pasted 'u' characters
            // from URLs will trigger copy_url() instead of being inserted into the input
            if self.url_input_open {
                self.handle_url_input(key);
                return;
            }

            // Handle status picker input (modal overlay)
            if self.status_picker_open {
                self.handle_status_picker_input(key);
                return;
            }

            // Handle dialog confirmation (Enter/Esc) — must be first, before ANY other handler
            // so dialog takes priority over text input, screen handlers, etc.
            if self.dialog.is_visible() {
                match key.code {
                    KeyCode::Enter => {
                        if self.dialog.confirmed() {
                            match &self.dialog.dialog_type {
                                Some(DialogType::ConfirmQuit) => {
                                    // Save session state before quitting
                                    if let Err(e) = self.save_session_state() {
                                        tracing::error!("Failed to save session state: {}", e);
                                    }
                                    // Signal the run_loop to break
                                    self.dialog.hide();
                                    self.state = AppState::Quitting;
                                    return;
                                }
                                Some(DialogType::ConfirmDelete) => {
                                    // Delete the selected task
                                    self.delete_selected_task();
                                }
                                _ => {}
                            }
                        }
                        self.dialog.hide();
                        return;
                    }
                    KeyCode::Esc => {
                        self.dialog.hide();
                        return;
                    }
                    _ => {}
                }
            }

            // Handle text input — delegate to appropriate handler when any text input is active
            // This guards global shortcuts like 'u' from interfering with typing
            if self.is_text_input_active() {
                self.handle_text_input(key);
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
                    // Create new task - open creation form
                    self.task_name_input.clear();
                    self.task_description_input.clear();
                    self.task_creating = true;
                    self.task_creation_focus = TaskCreationField::Name;
                    self.task_detail.task = None;
                    self.task_detail.creating = true;
                    self.task_detail.editing = false;
                    self.screen = Screen::TaskDetail;
                    self.screen_title = generate_screen_title("New Task");
                    self.status = "Enter task name (Ctrl+S to create, Esc to cancel)".to_string();
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
                KeyCode::Char('s') => {
                    // Open status picker if a task is selected
                    if let Some(task) = self.task_list.selected_task().cloned() {
                        self.open_status_picker(task);
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

        // Handle task creation form input (takes priority over comment editing)
        if self.task_creating {
            if let InputEvent::Key(key) = event {
                match key.code {
                    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Validate and create task
                        if self.task_name_input.trim().is_empty() {
                            self.status = "Task name is required".to_string();
                            return;
                        }
                        if let Some(list_id) = &self.current_list_id {
                            self.create_task(list_id.clone());
                        } else {
                            self.status = "No list selected".to_string();
                        }
                        return;
                    }
                    KeyCode::Esc => {
                        // Cancel task creation
                        self.task_name_input.clear();
                        self.task_description_input.clear();
                        self.task_creating = false;
                        self.task_detail.creating = false;
                        self.screen = Screen::Tasks;
                        self.update_screen_title();
                        self.status = "Task creation cancelled".to_string();
                        return;
                    }
                    KeyCode::Tab => {
                        // Toggle focus between name and description
                        self.task_creation_focus = match self.task_creation_focus {
                            TaskCreationField::Name => TaskCreationField::Description,
                            TaskCreationField::Description => TaskCreationField::Name,
                        };
                        return;
                    }
                    KeyCode::Char(c) => {
                        // Add character to focused field
                        match self.task_creation_focus {
                            TaskCreationField::Name => self.task_name_input.push(c),
                            TaskCreationField::Description => self.task_description_input.push(c),
                        }
                        return;
                    }
                    KeyCode::Backspace => {
                        // Remove character from focused field
                        match self.task_creation_focus {
                            TaskCreationField::Name => { self.task_name_input.pop(); }
                            TaskCreationField::Description => { self.task_description_input.pop(); }
                        }
                        return;
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
                KeyCode::Char('s') => {
                    // Open status picker for the current task
                    if let Some(task) = &self.task_detail.task {
                        self.open_status_picker(task.clone());
                    } else {
                        self.status = "No task selected".to_string();
                    }
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

                    // if top level comment, prevent user from going beyond top level limit
                    if self.comment_view_mode == CommentViewMode::TopLevel {
                        if self.comment_selected_index < self.comment_top_level_count - 1 {
                            self.comment_selected_index += 1;
                            return;
                        }

                        // loop to first item
                        self.comment_selected_index = 0;
                        return;
                    }

                    self.comment_selected_index += 1;
                }
                KeyCode::Char('k') if self.comment_focus => {
                    if self.comment_selected_index > 0 {
                        self.comment_selected_index -= 1;
                        return;
                    }

                    // loop to last item if top level comment
                    if self.comment_view_mode == CommentViewMode::TopLevel && self.comment_top_level_count > 0 {
                        self.comment_selected_index = self.comment_top_level_count - 1;
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

    /// Open the status picker for a task
    fn open_status_picker(&mut self, task: crate::models::Task) {
        // Store the task ID we're changing status for
        self.status_picker_task_id = Some(task.id.clone());
        
        // Store original status for potential rollback
        self.status_picker_original_status = task.status.as_ref().map(|s| s.status.clone());
        
        // Build list of available statuses (from task's space, or default)
        let statuses = vec![
            crate::models::TaskStatus {
                id: None,
                status: "To Do".to_string(),
                color: Some("#8794a6".to_string()),
                type_field: None,
                orderindex: Some(0),
                status_group: Some("todo".to_string()),
            },
            crate::models::TaskStatus {
                id: None,
                status: "In Progress".to_string(),
                color: Some("#4f46de".to_string()),
                type_field: None,
                orderindex: Some(1),
                status_group: Some("in_progress".to_string()),
            },
            crate::models::TaskStatus {
                id: None,
                status: "Done".to_string(),
                color: Some("#0f4a58".to_string()),
                type_field: None,
                orderindex: Some(2),
                status_group: Some("done".to_string()),
            },
        ];
        
        // If task has custom statuses, add them
        if let Some(_space) = &task.space {
            // TODO: Fetch actual space statuses from API
            // For now, use the space's status list if available
        }
        
        self.status_picker_statuses = statuses;
        self.status_picker_cursor = 0;
        self.status_picker_open = true;
        self.status = "Select new status (j/k navigate, Enter select, Esc cancel)".to_string();
    }

    /// Handle keyboard input for status picker
    fn handle_status_picker_input(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.status_picker_cursor < self.status_picker_statuses.len().saturating_sub(1) {
                    self.status_picker_cursor += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.status_picker_cursor > 0 {
                    self.status_picker_cursor -= 1;
                }
            }
            KeyCode::Enter => {
                self.save_status();
            }
            KeyCode::Esc => {
                self.status_picker_open = false;
                self.status = "Status change cancelled".to_string();
            }
            _ => {}
        }
    }

    /// Save the selected status to the task
    fn save_status(&mut self) {
        let task_id = match &self.status_picker_task_id {
            Some(id) => id.clone(),
            None => {
                self.status_picker_open = false;
                self.status = "No task selected".to_string();
                return;
            }
        };

        let new_status = match self.status_picker_statuses.get(self.status_picker_cursor) {
            Some(status) => status.status.clone(),
            None => {
                self.status_picker_open = false;
                self.status = "No status selected".to_string();
                return;
            }
        };

        // Optimistic UI update: update the task in self.tasks and rebuild grouped list
        for task in &mut self.tasks {
            if task.id == task_id {
                task.status = Some(crate::models::TaskStatus {
                    id: None,
                    status: new_status.clone(),
                    color: None,
                    type_field: None,
                    orderindex: None,
                    status_group: None,
                });
                break;
            }
        }
        self.rebuild_task_list();

        // Make API call
        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.status_picker_open = false;
                self.status = "Not authenticated".to_string();
                return;
            }
        };

        self.loading = true;
        self.status = "Updating status...".to_string();

        let task_id_clone = task_id.clone();
        let new_status_clone = new_status.clone();
        let tx = self.message_tx.clone().unwrap();

        use crate::models::UpdateTaskRequest;
        let update = UpdateTaskRequest {
            name: None,
            description: None,
            status: Some(new_status_clone),
            priority: None,
            assignees: None,
            due_date: None,
        };

        tokio::spawn(async move {
            let result = client.update_task(&task_id_clone, &update).await;
            let msg = match result {
                Ok(task) => AppMessage::TaskStatusUpdated(Ok(task)),
                Err(e) => AppMessage::TaskStatusUpdated(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });

        self.status_picker_open = false;
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
                    let total_top_level_comments = top_level_comments.len();
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

                    let msg = AppMessage::CommentsLoaded(Ok(CommentsLoadedResponse { all_comments, top_level_comments: total_top_level_comments }));
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

    /// Create a new task
    fn create_task(&mut self, list_id: String) {
        self.loading = true;
        self.status = "Creating task...".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.loading = false;
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        let name = self.task_name_input.clone();
        let description = if self.task_description_input.is_empty() {
            None
        } else {
            Some(self.task_description_input.clone())
        };

        let request = CreateTaskRequest {
            name,
            description,
            status: None,
            priority: None,
            assignees: None,
            due_date: None,
        };

        tokio::spawn(async move {
            let result = client.create_task(&list_id, &request).await;
            let msg = match result {
                Ok(task) => AppMessage::TaskCreated(Ok(task)),
                Err(e) => AppMessage::TaskCreated(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Delete the currently selected task
    fn delete_selected_task(&mut self) {
        let task_id = match self.task_list.selected_task() {
            Some(task) => task.id.clone(),
            None => {
                self.status = "No task selected".to_string();
                return;
            }
        };

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.error = Some("Not authenticated".to_string());
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        let task_id_clone = task_id.clone();
        self.status = format!("Deleting task {}...", task_id);
        tokio::spawn(async move {
            let result = client.delete_task(&task_id_clone).await;
            let msg = match result {
                Ok(()) => AppMessage::TaskDeleted(Ok(task_id_clone)),
                Err(e) => AppMessage::TaskDeleted(Err(e.to_string())),
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

            // Render status picker overlay if open
            if self.status_picker_open {
                let current_status = self
                    .status_picker_task_id
                    .as_ref()
                    .and_then(|task_id| {
                        self.tasks
                            .iter()
                            .find(|t| &t.id == task_id)
                            .and_then(|t| t.status.as_ref().map(|s| s.status.as_str()))
                    });
                render_status_picker(
                    frame,
                    area,
                    &self.status_picker_statuses,
                    self.status_picker_cursor,
                    current_status,
                );
            }

            // Render URL input dialog if open
            if self.url_input_open {
                self.render_url_input_dialog(frame, area);
            }

            // Render help overlay if visible
            let help_context = self.get_help_context();
            render_help(frame, &self.help, &help_context, area);

            // Render status bar
            let hints = self.get_hints();
            // Priority: error > navigating > url_copy_status > loading > regular status
            let status = if let Some(ref error) = self.error {
                error.clone()
            } else if self.navigating {
                format!("Loading... {}", self.navigating_level)
            } else if let Some(ref url_status) = self.url_copy_status {
                // Show URL copy status (takes priority over regular status)
                url_status.clone()
            } else if self.loading {
                "Loading...".to_string()
            } else {
                self.status.clone()
            };
            layout.render_status(frame, &status, &hints);
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
                render_task_detail(
                    frame,
                    &self.task_detail,
                    task_detail_area,
                    &self.task_name_input,
                    &self.task_description_input,
                    &self.task_creation_focus,
                );

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

    /// Render the URL input dialog as a centered modal overlay
    fn render_url_input_dialog(&self, frame: &mut Frame, area: Rect) {
        use ratatui::{
            style::{Modifier, Style},
            text::{Line, Span},
            widgets::{Block, Borders, Clear, Paragraph},
        };

        // Center the dialog: 60% width, ~12 rows tall
        let popup_layout = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Percentage(35),
                ratatui::layout::Constraint::Length(12),
                ratatui::layout::Constraint::Percentage(35),
            ])
            .split(area);

        let dialog_area = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Percentage(20),
                ratatui::layout::Constraint::Percentage(60),
                ratatui::layout::Constraint::Percentage(20),
            ])
            .split(popup_layout[1])[1];

        // Clear underlying content
        frame.render_widget(Clear, dialog_area);

        // Dialog border and background
        let block = Block::default()
            .title(" Navigate to URL ")
            .borders(Borders::ALL)
            .style(
                Style::default()
                    .bg(ratatui::style::Color::Rgb(30, 30, 46))
                    .fg(ratatui::style::Color::Rgb(137, 180, 250)),
            );
        frame.render_widget(block, dialog_area);

        let inner = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(2)
            .constraints([
                ratatui::layout::Constraint::Length(1),  // prompt
                ratatui::layout::Constraint::Length(1),  // spacer
                ratatui::layout::Constraint::Length(3),  // input field
                ratatui::layout::Constraint::Length(2),  // error message
                ratatui::layout::Constraint::Length(1),  // spacer
                ratatui::layout::Constraint::Length(1),  // hints
            ])
            .split(dialog_area);

        // Prompt
        let prompt = Paragraph::new("Enter a ClickUp URL:").style(
            Style::default()
                .fg(ratatui::style::Color::Rgb(205, 214, 244))
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(prompt, inner[0]);

        // Input field with cursor
        let cursor_pos = self.url_input_cursor.min(self.url_input_text.len());
        let input_line: Line = if self.url_input_text.is_empty() {
            Line::from(Span::styled(
                "https://app.clickup.com/...",
                Style::default().fg(ratatui::style::Color::DarkGray),
            ))
        } else {
            // Build the line with cursor position tracking
            let before_cursor: String = self.url_input_text.chars().take(cursor_pos).collect();
            let after_cursor: String = self.url_input_text.chars().skip(cursor_pos).collect();
            let mut spans = vec![
                Span::styled(
                    before_cursor,
                    Style::default().fg(ratatui::style::Color::Rgb(205, 214, 244)),
                ),
            ];
            if after_cursor.is_empty() {
                // Show cursor block at end
                spans.push(Span::styled(
                    " ",
                    Style::default()
                        .fg(ratatui::style::Color::Rgb(205, 214, 244))
                        .bg(ratatui::style::Color::Rgb(88, 91, 112)),
                ));
            } else {
                spans.push(Span::styled(
                    after_cursor,
                    Style::default()
                        .fg(ratatui::style::Color::Rgb(205, 214, 244))
                        .bg(ratatui::style::Color::Rgb(88, 91, 112)),
                ));
            }
            Line::from(spans)
        };
        frame.render_widget(Paragraph::new(input_line), inner[2]);

        // Error message (if any)
        if let Some(ref error) = self.url_input_error {
            let error_para = Paragraph::new(Span::styled(
                format!("⚠ {}", error),
                Style::default().fg(ratatui::style::Color::Rgb(243, 139, 168)),
            ));
            frame.render_widget(error_para, inner[3]);
        }

        // Hints
        let hints = Paragraph::new("Enter: Navigate | Esc: Cancel | Ctrl+V: Paste").style(
            Style::default()
                .fg(ratatui::style::Color::DarkGray),
        );
        frame.render_widget(hints, inner[5]);
    }

    /// Determine the current help context based on screen and focus state
    fn get_help_context(&self) -> HelpContext {
        match self.screen {
            Screen::Auth => HelpContext::Auth,
            Screen::Workspaces | Screen::Spaces | Screen::Folders | Screen::Lists => {
                HelpContext::Navigation
            }
            Screen::Tasks => HelpContext::TaskList,
            Screen::TaskDetail => {
                if self.comment_focus {
                    HelpContext::Comments
                } else {
                    HelpContext::TaskDetail
                }
            }
            Screen::Document => HelpContext::Document,
        }
    }

    fn get_hints(&self) -> String {
        if self.dialog.is_visible() {
            get_dialog_hints().to_string()
        } else if self.status_picker_open {
            "j/k: Navigate | Enter: Select | Esc: Cancel".to_string()
        } else if self.help.visible {
            get_help_hints(&self.help)
        } else {
            match self.screen {
                Screen::Auth => "Enter: Connect | Esc: Cancel | ? - Help".to_string(),
                Screen::Tasks => {
                    "j/k: Navigate | Enter: View | n: New | e: Edit | d: Delete | a: Filter | s: Status | ? - Help".to_string()
                }
                Screen::TaskDetail => {
                    // Show different hints based on comment view mode
                    if self.comment_focus {
                        match self.comment_view_mode {
                            CommentViewMode::TopLevel => "j/k: Navigate | Enter: View thread | n: New comment | e: Edit | Tab: Task form | ? - Help".to_string(),
                            CommentViewMode::InThread { .. } => "j/k: Navigate | r: Reply | Esc: Back | Tab: Task form | ? - Help".to_string(),
                        }
                    } else {
                        "e: Edit task | Tab: Comments | Esc: Back | ? - Help".to_string()
                    }
                }
                Screen::Document => "j/k: Scroll | Esc: Close | ? - Help".to_string(),
                _ => "j/k: Navigate | Enter: Select | Tab: Toggle | Ctrl+Q: Quit | ? - Help".to_string(),
            }
        }
    }

    /// Open the URL input dialog
    fn open_url_input_dialog(&mut self) {
        // Guard: must be authenticated
        if self.state == AppState::Unauthenticated {
            self.status = "Please authenticate first".to_string();
            return;
        }
        self.url_input_open = true;
        self.url_input_text = String::new();
        self.url_input_error = None;
        self.url_input_cursor = 0;
    }

    /// Handle URL input dialog submission
    fn submit_url_input(&mut self) {
        if self.url_input_text.trim().is_empty() {
            self.url_input_error = Some("Please enter a URL".to_string());
            return;
        }

        let url = self.url_input_text.clone();
        self.close_url_input_dialog();

        // Parse the URL
        use crate::utils::UrlParser;
        match UrlParser::parse(&url) {
            Ok(parsed) => {
                self.navigate_from_parsed_url(parsed);
            }
            Err(e) => {
                // Reopen dialog with error and restore the URL text
                self.url_input_open = true;
                self.url_input_text = url.clone();
                let error_msg = format!("Unrecognized ClickUp URL format: {}", e);
                self.url_input_error = Some(error_msg);
                self.url_input_cursor = url.len();
            }
        }
    }

    /// Close the URL input dialog
    fn close_url_input_dialog(&mut self) {
        self.url_input_open = false;
        self.url_input_text.clear();
        self.url_input_error = None;
        self.url_input_cursor = 0;
    }

    /// Handle keyboard input within the URL input dialog
    pub fn handle_url_input(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        match key.code {
            KeyCode::Esc => {
                self.close_url_input_dialog();
            }
            KeyCode::Enter => {
                self.submit_url_input();
            }
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Paste from clipboard
                use arboard::Clipboard;
                if let Ok(mut clipboard) = Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        for c in text.chars() {
                            self.url_input_text.insert(self.url_input_cursor, c);
                            self.url_input_cursor += 1;
                        }
                        self.url_input_error = None;
                    }
                }
            }
            KeyCode::Char(c) => {
                self.url_input_text.insert(self.url_input_cursor, c);
                self.url_input_cursor += 1;
                self.url_input_error = None;
            }
            KeyCode::Backspace => {
                if self.url_input_cursor > 0 {
                    self.url_input_text.remove(self.url_input_cursor - 1);
                    self.url_input_cursor -= 1;
                    self.url_input_error = None;
                }
            }
            KeyCode::Left => {
                if self.url_input_cursor > 0 {
                    self.url_input_cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.url_input_cursor < self.url_input_text.len() {
                    self.url_input_cursor += 1;
                }
            }
            _ => {}
        }
    }

    /// Navigate to a resource based on a parsed URL
    fn navigate_from_parsed_url(&mut self, parsed: crate::utils::ParsedUrl) {
        use crate::utils::ParsedUrl;
        match parsed {
            ParsedUrl::Workspace { workspace_id } => {
                self.navigate_to_workspace(workspace_id);
            }
            ParsedUrl::Space { workspace_id, space_id } => {
                self.navigate_to_space(workspace_id, space_id);
            }
            ParsedUrl::Folder { workspace_id, folder_id } => {
                self.navigate_to_folder(workspace_id, folder_id);
            }
            ParsedUrl::List { workspace_id, list_id } => {
                self.navigate_to_list(workspace_id, list_id);
            }
            ParsedUrl::Task { task_id } => {
                self.navigate_to_task(task_id);
            }
            ParsedUrl::Comment { task_id, comment_id } => {
                self.navigate_to_comment(task_id, comment_id);
            }
            ParsedUrl::Document { doc_id } => {
                self.navigate_to_document(doc_id);
            }
        }
    }

    // --- URL-based navigation implementations ---

    /// Navigate to a workspace by ID
    fn navigate_to_workspace(&mut self, workspace_id: String) {
        self.navigating = true;
        self.navigating_level = "workspace".to_string();

        // Check if already at target
        if self.screen == Screen::Workspaces {
            if let Some(selected) = self.sidebar.selected_item() {
                use super::widgets::SidebarItem;
                let id = match selected {
                    SidebarItem::Workspace { id, .. } => id.clone(),
                    _ => String::new(),
                };
                if id == workspace_id {
                    self.navigating = false;
                    self.navigating_level.clear();
                    self.status = "Already viewing this workspace".to_string();
                    return;
                }
            }
        }

        // Navigate to workspaces screen and select the workspace
        self.screen = Screen::Workspaces;
        self.current_workspace_id = None;
        self.current_space_id = None;
        self.current_folder_id = None;
        self.current_list_id = None;
        self.sidebar.select_first();

        // Find and select the target workspace in the list
        for (i, ws) in self.workspaces.iter().enumerate() {
            if ws.id == workspace_id {
                self.sidebar.select(Some(i));
                self.current_workspace_id = Some(workspace_id);
                self.navigating = false;
                self.navigating_level.clear();
                self.status = format!("Navigated to workspace: {}", ws.name);
                return;
            }
        }

        // Workspace not found in current list
        self.navigating = false;
        self.navigating_level.clear();
        self.status = "Workspace not found in current workspace list".to_string();
    }

    /// Navigate to a space by workspace + space ID
    fn navigate_to_space(&mut self, workspace_id: String, space_id: String) {
        self.navigating = true;
        self.navigating_level = "space".to_string();

        // First ensure we're in the right workspace
        self.current_workspace_id = Some(workspace_id.clone());
        self.screen = Screen::Spaces;
        self.current_space_id = None;
        self.current_folder_id = None;
        self.current_list_id = None;
        self.sidebar.select_first();

        // Find target space
        for (i, space) in self.spaces.iter().enumerate() {
            if space.id == space_id {
                self.sidebar.select(Some(i));
                self.current_space_id = Some(space_id);
                self.navigating = false;
                self.navigating_level.clear();
                self.status = format!("Navigated to space: {}", space.name);
                return;
            }
        }

        // Space not found — need to load spaces first
        self.load_spaces(workspace_id);
        self.status = "Loading spaces...".to_string();
    }

    /// Navigate to a folder by workspace + folder ID
    fn navigate_to_folder(&mut self, workspace_id: String, folder_id: String) {
        self.navigating = true;
        self.navigating_level = "folder".to_string();

        self.current_workspace_id = Some(workspace_id.clone());
        self.current_space_id = None;
        self.current_folder_id = None;
        self.current_list_id = None;
        self.screen = Screen::Folders;
        self.sidebar.select_first();

        for (i, folder) in self.folders.iter().enumerate() {
            if folder.id == folder_id {
                self.sidebar.select(Some(i));
                self.current_folder_id = Some(folder_id);
                self.navigating = false;
                self.navigating_level.clear();
                self.status = format!("Navigated to folder: {}", folder.name);
                return;
            }
        }

        self.navigating = false;
        self.navigating_level.clear();
        self.status = "Folder not found in current folder list".to_string();
    }

    /// Navigate to a list by workspace + list ID
    fn navigate_to_list(&mut self, workspace_id: String, list_id: String) {
        self.navigating = true;
        self.navigating_level = "list".to_string();

        self.current_workspace_id = Some(workspace_id);
        self.current_space_id = None;
        self.current_folder_id = None;
        self.current_list_id = None;
        self.screen = Screen::Lists;
        self.sidebar.select_first();

        for (i, list) in self.lists.iter().enumerate() {
            if list.id == list_id {
                self.sidebar.select(Some(i));
                self.current_list_id = Some(list_id);
                self.navigating = false;
                self.navigating_level.clear();
                self.status = format!("Navigated to list: {}", list.name);
                return;
            }
        }

        self.navigating = false;
        self.navigating_level.clear();
        self.status = "List not found in current list".to_string();
    }

    /// Navigate to a task by task ID (short-form URL)
    fn navigate_to_task(&mut self, task_id: String) {
        self.navigating = true;
        self.navigating_level = "task".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.navigating = false;
                self.navigating_level.clear();
                self.status = "Not authenticated".to_string();
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        let target_task_id = task_id.clone();
        let prev_screen = self.screen.clone();
        tokio::spawn(async move {
            // Fetch task from API to get its context
            let result = client.get_task(&target_task_id).await;
            let msg = match result {
                Ok(task) => AppMessage::TaskFetchedForNavigation(Ok(task), prev_screen),
                Err(e) => AppMessage::TaskFetchedForNavigation(Err(e.to_string()), prev_screen),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Navigate to a comment by task ID + comment ID
    fn navigate_to_comment(&mut self, task_id: String, comment_id: String) {
        // First navigate to the task, then find the comment
        self.navigating = true;
        self.navigating_level = "comment".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.navigating = false;
                self.navigating_level.clear();
                self.status = "Not authenticated".to_string();
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        let target_task_id = task_id.clone();
        let target_comment_id = comment_id.clone();
        let prev_screen = self.screen.clone();
        tokio::spawn(async move {
            let result = client.get_task(&target_task_id).await;
            let msg = match result {
                Ok(task) => AppMessage::CommentFetchedForNavigation(
                    Ok(task),
                    target_comment_id,
                    prev_screen,
                ),
                Err(e) => AppMessage::CommentFetchedForNavigation(
                    Err(e.to_string()),
                    target_comment_id,
                    prev_screen,
                ),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Navigate to a document by doc ID
    fn navigate_to_document(&mut self, doc_id: String) {
        self.navigating = true;
        self.navigating_level = "document".to_string();

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.navigating = false;
                self.navigating_level.clear();
                self.status = "Not authenticated".to_string();
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();
        let target_doc_id = doc_id.clone();
        let prev_screen = self.screen.clone();
        tokio::spawn(async move {
            // Search for the document by ID using filters
            use crate::models::DocumentFilters;
            let filters = DocumentFilters::default();
            let result = client.search_docs(&filters).await;
            match result {
                Ok(docs) => {
                    // Find the matching document
                    let doc = docs.iter().find(|d| d.id == target_doc_id);
                    if let Some(doc) = doc.cloned() {
                        let _ = tx
                            .send(AppMessage::DocumentFetchedForNavigation(Ok(doc), prev_screen))
                            .await;
                    } else {
                        let _ = tx
                            .send(AppMessage::DocumentFetchedForNavigation(
                                Err("Document not found in search results".to_string()),
                                prev_screen,
                            ))
                            .await;
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(AppMessage::DocumentFetchedForNavigation(
                            Err(e.to_string()),
                            prev_screen,
                        ))
                        .await;
                }
            }
        });
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
    pub fn task_list(&mut self) -> &mut GroupedTaskList {
        &mut self.task_list
    }

    /// Rebuild the grouped task list from `self.tasks`.
    /// Preserves the currently selected task by ID if it still exists.
    fn rebuild_task_list(&mut self) {
        let selected_id = self.task_list.selected_task().map(|t| t.id.clone());
        self.task_list = GroupedTaskList::from_tasks(self.tasks.clone());
        if let Some(ref id) = selected_id {
            if !self
                .task_list
                .rows()
                .iter()
                .any(|r| matches!(r, crate::tui::widgets::ListRow::Task(t) if &t.id == id))
            {
                // Selected task no longer exists, select first
                self.task_list.select_first();
            }
        }
    }

    /// Public wrapper for testing
    #[allow(dead_code)]
    pub fn rebuild_task_list_for_test(&mut self) {
        self.rebuild_task_list();
    }

    /// Get task detail (public for testing)
    #[allow(dead_code)]
    pub fn task_detail(&mut self) -> &mut TaskDetailState {
        &mut self.task_detail
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

    /// Get status picker state (public for testing)
    #[allow(dead_code)]
    pub fn is_status_picker_open(&self) -> bool {
        self.status_picker_open
    }

    /// Set screen directly (public for testing)
    #[allow(dead_code)]
    pub fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
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

    /// Test is_text_input_active() returns false when no input is active
    #[test]
    fn test_is_text_input_active_inactive() {
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        assert!(!app.is_text_input_active());
    }

    /// Test is_text_input_active() returns true when URL input is open
    #[test]
    fn test_is_text_input_active_url_input() {
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.url_input_open = true;

        assert!(app.is_text_input_active());
    }

    /// Test is_text_input_active() returns true when status picker is open
    #[test]
    fn test_is_text_input_active_status_picker() {
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.status_picker_open = true;

        assert!(app.is_text_input_active());
    }

    /// Test is_text_input_active() returns true when assignee picker is open
    #[test]
    fn test_is_text_input_active_assignee_picker() {
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.assignee_picker_open = true;

        assert!(app.is_text_input_active());
    }

    /// Test is_text_input_active() returns true when task creating
    #[test]
    fn test_is_text_input_active_task_creating() {
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.task_creating = true;

        assert!(app.is_text_input_active());
    }

    /// Test is_text_input_active() returns true when comment editing index is set
    #[test]
    fn test_is_text_input_active_comment_editing() {
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.comment_editing_index = Some(0);

        assert!(app.is_text_input_active());
    }

    /// Test is_text_input_active() returns true when comment new text is not empty
    #[test]
    fn test_is_text_input_active_comment_new_text() {
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.comment_new_text = "typing...".to_string();

        assert!(app.is_text_input_active());
    }

    /// Test session state includes user_id
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
    /// This is a regression test for the bug where the grouped task list was not rebuilt
    /// after task updates.
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

        // Simulate loading tasks into the app cache and rebuild grouped list
        app.tasks = vec![task.clone()];
        app.task_list = GroupedTaskList::from_tasks(app.tasks.clone());

        // Select the task in the task list
        app.task_list.select_first();

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

    /// Test that pressing 'n' in task list view opens the task creation form
    #[test]
    fn test_n_key_opens_task_creation_form() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mock_client = MockClickUpClient::new()
            .with_tasks(vec![]);

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Set up the app in Tasks screen with a list context
        app.set_screen(Screen::Tasks);
        app.current_list_id = Some("test-list-1".to_string());

        // Press 'n' key
        let n_key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
        app.update(InputEvent::Key(n_key));

        // Should navigate to TaskDetail screen
        assert_eq!(app.screen, Screen::TaskDetail, "Should navigate to TaskDetail");

        // Should be in creation mode
        assert!(app.task_creating, "task_creating should be true");
        assert!(app.task_detail.creating, "task_detail.creating should be true");

        // Input fields should be empty
        assert_eq!(app.task_name_input, "", "Name input should be empty");
        assert_eq!(app.task_description_input, "", "Description input should be empty");

        // Focus should be on name field
        assert_eq!(app.task_creation_focus, TaskCreationField::Name, "Focus should be on Name");

        // Status message should indicate how to create
        assert!(
            app.status().contains("Ctrl+S"),
            "Status should mention Ctrl+S: {}",
            app.status()
        );

        // Screen title should indicate new task
        assert!(
            app.screen_title.contains("New Task"),
            "Title should say 'New Task': {}",
            app.screen_title
        );
    }
}
