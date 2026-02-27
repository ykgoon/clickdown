//! Main TUI application

use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use arboard::Clipboard;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{Frame, prelude::Rect, layout::{Layout, Direction, Constraint}};
use tokio::sync::mpsc;

use crate::api::{ClickUpApi, AuthManager, ClickUpClient};
use crate::config::ConfigManager;
use crate::models::{Workspace, ClickUpSpace, Folder, List, Task, Document, Comment, CreateCommentRequest, UpdateCommentRequest};
use crate::utils::{ClickUpUrlGenerator, ClipboardService, UrlError, UrlGenerator};
use crate::tui::widgets::SidebarItem;

use super::terminal;
use super::layout::{TuiLayout, generate_screen_title, split_task_detail};
use super::input::{InputEvent, is_quit, is_enter, is_escape};
use super::widgets::{
    SidebarState, render_sidebar, get_sidebar_hints,
    TaskListState, render_task_list, get_task_list_hints,
    TaskDetailState, render_task_detail, get_task_detail_hints,
    AuthState, render_auth, get_auth_hints,
    DocumentState, render_document, get_document_hints,
    DialogState, DialogType, render_dialog, get_dialog_hints,
    HelpState, render_help,
    render_comments,
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
}

/// Main TUI application state
pub struct TuiApp {
    /// Current screen
    screen: Screen,

    /// Application state
    state: AppState,

    /// API client
    client: Option<Arc<dyn ClickUpApi>>,

    /// Config manager
    config: ConfigManager,

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
}

/// Application state
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Initializing,
    Unauthenticated,
    LoadingWorkspaces,
    Main,
}

impl TuiApp {
    pub fn new() -> Result<Self> {
        let config = ConfigManager::new().unwrap_or_default();
        let auth = AuthManager::new().unwrap_or_default();

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
            config,
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
            message_rx: Some(message_rx),
            message_tx: Some(message_tx.clone()),
            clipboard: ClipboardService::new(),
            url_copy_status: None,
            url_copy_status_time: None,
            current_workspace_id: None,
            current_space_id: None,
            current_folder_id: None,
            current_list_id: None,
        };

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
                    InputEvent::Quit => break,
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

    /// Process async messages from API calls
    fn process_async_messages(&mut self) {
        if let Some(ref mut rx) = self.message_rx {
            // Try to receive messages without blocking
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    AppMessage::WorkspacesLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(workspaces) => {
                                self.workspaces = workspaces.clone();
                                // Populate sidebar with workspaces
                                self.sidebar.items = self.workspaces.iter()
                                    .map(|w| SidebarItem::Workspace {
                                        name: w.name.clone(),
                                        id: w.id.clone()
                                    })
                                    .collect();
                                self.sidebar.select_first();
                                self.state = AppState::Main;
                                // Clear any previous error state
                                self.error = None;
                                self.status = format!("Loaded {} workspace(s)", self.workspaces.len());
                            }
                            Err(e) => {
                                self.error = Some(format!("Failed to load workspaces: {}", e));
                                self.status = "Failed to load workspaces".to_string();
                            }
                        }
                    }
                    AppMessage::SpacesLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(spaces) => {
                                self.spaces = spaces.clone();
                                // Populate sidebar with spaces
                                self.sidebar.items = self.spaces.iter()
                                    .map(|s| SidebarItem::Space {
                                        name: s.name.clone(),
                                        id: s.id.clone(),
                                        indent: 1,
                                    })
                                    .collect();
                                self.sidebar.select_first();
                                // Clear any previous error state
                                self.error = None;
                                self.status = format!("Loaded {} space(s)", self.spaces.len());
                            }
                            Err(e) => {
                                self.error = Some(format!("Failed to load spaces: {}", e));
                                self.status = "Failed to load spaces".to_string();
                            }
                        }
                    }
                    AppMessage::FoldersLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(folders) => {
                                self.folders = folders.clone();
                                // Populate sidebar with folders
                                self.sidebar.items = self.folders.iter()
                                    .map(|f| SidebarItem::Folder {
                                        name: f.name.clone(),
                                        id: f.id.clone(),
                                        indent: 2,
                                    })
                                    .collect();
                                self.sidebar.select_first();
                                // Clear any previous error state
                                self.error = None;
                                self.status = format!("Loaded {} folder(s)", self.folders.len());
                            }
                            Err(e) => {
                                self.error = Some(format!("Failed to load folders: {}", e));
                                self.status = "Failed to load folders".to_string();
                            }
                        }
                    }
                    AppMessage::ListsLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(lists) => {
                                self.lists = lists.clone();
                                // Populate sidebar with lists
                                self.sidebar.items = self.lists.iter()
                                    .map(|l| SidebarItem::List {
                                        name: l.name.clone(),
                                        id: l.id.clone(),
                                        indent: 3,
                                    })
                                    .collect();
                                self.sidebar.select_first();
                                // Clear any previous error state
                                self.error = None;
                                self.status = format!("Loaded {} list(s)", self.lists.len());
                            }
                            Err(e) => {
                                self.error = Some(format!("Failed to load lists: {}", e));
                                self.status = "Failed to load lists".to_string();
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
                                self.task_list.tasks = sorted_tasks.clone();
                                self.tasks = sorted_tasks;
                                self.task_list.select_first();
                                // Clear any previous error state
                                self.error = None;
                                self.status = format!("Loaded {} task(s)", self.task_list.tasks.len());
                            }
                            Err(e) => {
                                self.error = Some(format!("Failed to load tasks: {}", e));
                                self.status = "Failed to load tasks".to_string();
                                // Clear tasks on error to prevent stale data
                                self.task_list.tasks.clear();
                                self.tasks.clear();
                            }
                        }
                    }
                    AppMessage::CommentsLoaded(result) => {
                        self.loading = false;
                        match result {
                            Ok(comments) => {
                                tracing::debug!("Loaded {} comments", comments.len());
                                for (i, comment) in comments.iter().enumerate() {
                                    tracing::debug!("Comment {}: id={}, parent_id={:?}, author={:?}",
                                        i,
                                        comment.id,
                                        comment.parent_id,
                                        comment.commenter.as_ref().map(|c| &c.username));
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
                                self.error = Some(format!("Failed to create {}: {}", 
                                    if is_reply { "reply" } else { "comment" }, e));
                                self.status = format!("Failed to create {}", 
                                    if is_reply { "reply" } else { "comment" });
                            }
                        }
                    }
                    AppMessage::CommentUpdated(result) => {
                        self.loading = false;
                        match result {
                            Ok(comment) => {
                                if let Some(idx) = self.comments.iter().position(|c| c.id == comment.id) {
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
                                        // TODO: Delete task
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
                event::Event::Resize(w, h) => Ok(Some(InputEvent::Resize(w, h))),
                _ => Ok(Some(InputEvent::None)),
            }
        } else {
            Ok(None)
        }
    }
    
    fn update(&mut self, event: InputEvent) {
        // Handle help toggle
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
                                    self.status = "Paste failed: could not read clipboard".to_string();
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
                    // TODO: Create new task
                    self.status = "Create task - not yet implemented".to_string();
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
                                        self.status = if matches!(self.comment_view_mode, CommentViewMode::InThread { .. }) {
                                            "Reply cannot be empty".to_string()
                                        } else {
                                            "Comment cannot be empty".to_string()
                                        };
                                        return;
                                    }
                                    let task_id = _task.id.clone();

                                    // Determine parent_id based on view mode
                                    let parent_id = match &self.comment_view_mode {
                                        CommentViewMode::InThread { parent_comment_id, .. } => {
                                            Some(parent_comment_id.clone())
                                        }
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
                                    self.status = if matches!(self.comment_view_mode, CommentViewMode::InThread { .. }) {
                                        "Reply cannot be empty".to_string()
                                    } else {
                                        "Comment cannot be empty".to_string()
                                    };
                                    return;
                                }
                                let task_id = _task.id.clone();

                                // Determine parent_id based on view mode
                                let parent_id = match &self.comment_view_mode {
                                    CommentViewMode::InThread { parent_comment_id, .. } => {
                                        Some(parent_comment_id.clone())
                                    }
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
                        self.status = if matches!(self.comment_view_mode, CommentViewMode::InThread { .. }) {
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
                    } else if self.comment_editing_index.is_some() || !self.comment_new_text.is_empty() {
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
                    // TODO: Save task
                    self.task_detail.editing = false;
                    self.status = "Task saved".to_string();
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
                    if !self.comments.is_empty() && self.comment_selected_index < self.comments.len() {
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
                        if !self.comments.is_empty() && self.comment_selected_index < self.comments.len() {
                            let comment = &self.comments[self.comment_selected_index];
                            // Only enter thread if this is a top-level comment
                            if comment.parent_id.is_none() {
                                // Store current selection for when we exit
                                self.comment_previous_selection = Some(self.comment_selected_index);
                                
                                // Get author name for breadcrumb
                                let author = comment.commenter.as_ref()
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
                            CommentViewMode::InThread { parent_author, .. } => parent_author.clone(),
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
                    // TODO: Scroll down
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    // TODO: Scroll up
                }
                KeyCode::Esc => {
                    self.navigate_back();
                }
                _ => {}
            }
        }
    }
    
    fn navigate_into(&mut self) {
        // Navigate based on current screen and selection
        // Clone the selected item to avoid borrow checker issues
        let selected_item = self.sidebar.selected_item().cloned();

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
    
    fn navigate_back(&mut self) {
        match self.screen {
            Screen::Auth => {} // Can't go back from auth
            Screen::Workspaces => {} // Can't go back from workspaces
            Screen::Spaces => {
                self.current_space_id = None;
                self.current_folder_id = None;
                self.current_list_id = None;
                self.screen = Screen::Workspaces;
                self.screen_title = generate_screen_title("Workspaces");
            }
            Screen::Folders => {
                self.current_folder_id = None;
                self.current_list_id = None;
                self.screen = Screen::Spaces;
                if let Some(space) = self.spaces.first() {
                    self.screen_title = generate_screen_title(&space.name);
                }
            }
            Screen::Lists => {
                self.current_list_id = None;
                self.screen = Screen::Folders;
                if let Some(folder) = self.folders.first() {
                    self.screen_title = generate_screen_title(&folder.name);
                }
            }
            Screen::Tasks => {
                self.current_list_id = None;
                self.screen = Screen::Lists;
                if let Some(list) = self.lists.first() {
                    self.screen_title = generate_screen_title(&list.name);
                }
            }
            Screen::TaskDetail => {
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
    
    fn load_workspaces(&mut self) {
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
                client.create_comment_reply(parent_comment_id, &request).await
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
        };
    }
    
    fn render(&mut self, terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>) -> Result<()> {
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
            } else if self.loading {
                "Loading...".to_string()
            } else {
                self.status.clone()
            };
            layout.render_status(frame, &status, hints);
        })?;
        
        Ok(())
    }
    
    fn render_sidebar_content(&mut self, frame: &mut Frame, sidebar_area: Rect, content_area: Rect) {
        // Render sidebar
        render_sidebar(frame, &self.sidebar, sidebar_area);

        // Render main content based on screen
        match self.screen {
            Screen::Auth => render_auth(frame, &self.auth_state, content_area),
            Screen::Tasks => render_task_list(frame, &self.task_list, content_area),
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
            Screen::Tasks => render_task_list(frame, &self.task_list, area),
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
        } else {
            match self.screen {
                Screen::Auth => get_auth_hints(),
                Screen::Tasks => get_task_list_hints(),
                Screen::TaskDetail => {
                    // Show different hints based on comment view mode
                    if self.comment_focus {
                        match self.comment_view_mode {
                            CommentViewMode::TopLevel => "j/k: Navigate | Enter: View thread | n: New comment | e: Edit | Tab: Task form",
                            CommentViewMode::InThread { .. } => "j/k: Navigate | r: Reply | Esc: Back | Tab: Task form",
                        }
                    } else {
                        "e: Edit task | Tab: Comments | Esc: Back"
                    }
                }
                Screen::Document => get_document_hints(),
                _ => get_sidebar_hints(),
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
                if self.comment_focus && !self.comments.is_empty() && self.comment_selected_index < self.comments.len() {
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
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new().expect("Failed to create TUI app")
    }
}
