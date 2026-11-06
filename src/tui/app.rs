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
    ClickUpSpace, Comment, CreateCommentRequest, CreateTaskRequest, Document, Folder, List,
    SessionState, Task, UpdateCommentRequest, User, Workspace,
};
use crate::tui::widgets::SidebarItem;
use crate::utils::{ClickUpUrlGenerator, ClipboardService, UrlGenerator};

use super::input::InputEvent;
use super::layout::{generate_screen_title, split_task_detail, TuiLayout};
use super::terminal;
use super::widgets::{
    render_assignee_picker, render_auth, render_command_menu,
    render_comments, render_mention_picker,
    render_dialog, render_document, render_help, render_sidebar, render_status_picker,
    render_task_detail_expanded, render_task_list, AuthState, DialogState, DialogType, DocumentState,
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

/// Which comment-compose action a `SPC c` command requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposeKind {
    /// Start a new top-level comment (or reply context as-is)
    New,
    /// Edit the currently selected comment
    Edit,
    /// Reply inside the currently open thread
    Reply,
}

/// Which text buffer insert mode is editing (vim-style modal input).
/// While `insert_mode_target` is `Some`, all printable keys go into the
/// buffer and normal-mode commands are not interpreted.
#[derive(Debug, Clone, PartialEq)]
pub enum InsertTarget {
    /// Task description editor (view/edit task description)
    Description,
    /// New comment, reply, or edit of an existing comment
    /// (the exact mode is tracked by `comment_editing_index`)
    Comment,
    /// Mention picker opened from an insert-mode buffer (issue 6)
    Mention,
}

/// Command menu (SPACEmenu) state: which command group is shown.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandMenuLevel {
    /// Root menu: pick a command group
    Root,
    /// Group selected, waiting for the action sub-key
    Group(char),
}

/// Which task-detail panel is expanded full-screen (issue 5)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpandedPanel {
    Description,
    Comments,
}

#[derive(Debug, Clone)]
pub struct CommentCreatedMessageMeta {
    is_reply: bool,
    task_id: String,
}

/// Async messages for API results
#[derive(Debug, Clone)]
pub enum AppMessage {
    WorkspacesLoaded(Result<Vec<Workspace>, String>),
    /// Token verification during authentication: workspaces fetch outcome
    /// plus the candidate token, persisted only on success.
    WorkspacesVerified(Result<Vec<Workspace>, String>, String),
    SpacesLoaded(Result<Vec<ClickUpSpace>, String>),
    FoldersLoaded(Result<Vec<Folder>, String>),
    ListsLoaded(Result<Vec<List>, String>),
    TasksLoaded(Result<Vec<Task>, String>),
    CommentsLoaded(Result<Vec<Comment>, String>),
    CommentCreated(Result<Comment, String>, CommentCreatedMessageMeta),
    CommentUpdated(Result<Comment, String>),
    CurrentUserLoaded(Result<User, String>),
    MembersLoaded(Result<Vec<User>, String>),
    /// Full status configuration for a list (fetched on demand)
    ListStatusesLoaded(String, Result<Vec<crate::models::TaskStatus>, String>),
    AssigneesUpdated(Result<Task, String>),
    TaskStatusUpdated(Result<Task, String>),
    TaskDescriptionUpdated(Result<Task, String>),
    MentionMembersLoaded(Result<Vec<User>, String>),
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

    /// Set when an API call reports the stored token is invalid or expired,
    /// so the UI can direct the user to re-authenticate.
    auth_invalid: bool,

    /// True only for the production constructor: allows writing/clearing the
    /// on-disk token file. Test constructors disable this so tests never
    /// touch the real credentials.
    manage_stored_token: bool,

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

    /// Vim-style modal input state (issue 7):
    /// While Some(target), all printable keys are inserted into the target
    /// buffer instead of being interpreted as commands.
    insert_mode_target: Option<InsertTarget>,
    /// Command menu (SPACEmenu) state: None = closed
    command_menu: Option<CommandMenuLevel>,

    /// Full-screen expansion state (issue 5): which panel is expanded
    expanded_panel: Option<ExpandedPanel>,
    /// Edit buffer for the task description (insert mode, issue 7)
    description_edit_buffer: String,
    /// Manual scroll offset for the expanded description panel (issue 5)
    description_scroll_offset: usize,
    /// Members for the @mention picker (issue 6), fetched on demand
    mention_members: Vec<User>,
    mention_cursor: usize,
    mention_picker_open: bool,
    /// Insert target to return to when the mention picker closes
    mention_prev_target: Option<InsertTarget>,
    /// Current mention filter query (text typed after @)
    mention_query: String,

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
    /// Full status configuration per list ID (fetched from the ClickUp API)
    cached_list_statuses: std::collections::HashMap<String, Vec<crate::models::TaskStatus>>,

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
    status_picker_original_status: Option<crate::models::TaskStatus>,
    status_picker_task_id: Option<String>,

    /// Task captured when the delete confirmation dialog is opened
    pending_delete_task_id: Option<String>,

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

/// Convert a character index into a byte index, clamping to the string length.
fn char_index_to_byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(byte_index, _)| byte_index)
        .unwrap_or(text.len())
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
            || self.insert_mode_target.is_some()
            || self.mention_picker_open
            || self.command_menu.is_some()
    }

    /// Handle text input when any text input field is active
    /// Delegates to the appropriate handler based on which input is active
    pub fn handle_text_input(&mut self, key: crossterm::event::KeyEvent) {
        if self.url_input_open {
            self.handle_url_input(key);
        } else if self.mention_picker_open {
            self.handle_mention_picker_input(key);
        } else if self.command_menu.is_some() {
            self.handle_command_menu_input(key);
        } else if self.insert_mode_target.is_some() {
            self.handle_insert_mode(key);
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

    // ==================== Vim-style modal input engine (issue 7) ====================

    /// Open the SPACEmenu command overlay (normal mode only)
    fn open_command_menu(&mut self) {
        self.command_menu = Some(CommandMenuLevel::Root);
    }

    /// Current insert-mode buffer contents for the active target
    fn insert_buffer(&self) -> String {
        match self.insert_mode_target.as_ref() {
            Some(InsertTarget::Description) => self.description_edit_buffer.clone(),
            Some(InsertTarget::Comment) => self.comment_new_text.clone(),
            Some(InsertTarget::Mention) => String::new(),
            None => String::new(),
        }
    }

    fn insert_set_buffer(&mut self, value: String) {
        match self.insert_mode_target.as_ref() {
            Some(InsertTarget::Description) => self.description_edit_buffer = value,
            Some(InsertTarget::Comment) => self.comment_new_text = value,
            Some(InsertTarget::Mention) => {}
            None => {}
        }
    }

    /// Handle keys while insert mode is active.
    /// Ctrl+S saves and exits; Esc exits to normal mode without saving;
    /// @ opens the mention picker (issue 6); everything else is text.
    fn handle_insert_mode(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyModifiers;
        match key.code {
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.commit_insert_mode();
            }
            KeyCode::Esc => {
                // Leave insert mode without saving (vim-style)
                let target = self.insert_mode_target.take();
                match target {
                    Some(InsertTarget::Description) => {
                        self.description_edit_buffer.clear();
                        self.status = "Description edit cancelled".to_string();
                    }
                    Some(InsertTarget::Comment) => {
                        self.cancel_comment_editing();
                    }
                    _ => {}
                }
            }
            KeyCode::Enter => {
                let mut buf = self.insert_buffer();
                buf.push('\n');
                self.insert_set_buffer(buf);
            }
            KeyCode::Backspace => {
                let buf = self.insert_buffer();
                let mut trimmed = buf.clone();
                trimmed.pop();
                self.insert_set_buffer(trimmed);
            }
            KeyCode::Tab => {
                // Only meaningful in task creation: switch field
                if self.task_creating {
                    self.handle_task_creation_input(key);
                }
            }
            KeyCode::Char(c) => {
                let mut buf = self.insert_buffer();
                buf.push(c);
                self.insert_set_buffer(buf);
                // '@' opens the mention picker (issue 6)
                if c == '@' {
                    self.open_mention_picker();
                }
            }
            _ => {}
        }
    }

    /// Cancel comment editing (normal-mode Esc path)
    fn cancel_comment_editing(&mut self) {
        self.comment_new_text.clear();
        self.comment_editing_index = None;
        self.status = if matches!(self.comment_view_mode, CommentViewMode::InThread { .. }) {
            "Reply cancelled".to_string()
        } else {
            "Comment editing cancelled".to_string()
        };
    }

    /// Commit the active insert-mode buffer (Ctrl+S semantics)
    fn commit_insert_mode(&mut self) {
        match self.insert_mode_target.take() {
            Some(InsertTarget::Description) => {
                self.save_description_edit();
            }
            Some(InsertTarget::Comment) => {
                // Reuse the existing comment save path
                self.save_comment_from_buffer();
            }
            Some(InsertTarget::Mention) => {}
            None => {}
        }
    }

    /// Save the description edit buffer to the task
    fn save_description_edit(&mut self) {
        let task = match &self.task_detail.task {
            Some(t) => t.clone(),
            None => {
                self.description_edit_buffer.clear();
                self.status = "No task selected".to_string();
                return;
            }
        };
        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.description_edit_buffer.clear();
                self.status = "Not authenticated".to_string();
                return;
            }
        };

        let new_text = self.description_edit_buffer.clone();
        if new_text.trim().is_empty() {
            self.description_edit_buffer.clear();
            self.status = "Description cannot be empty".to_string();
            return;
        }

        let task_id = task.id.clone();
        let tx = self.message_tx.clone().unwrap();
        use crate::models::UpdateTaskRequest;
        let update = UpdateTaskRequest {
            name: None,
            description: Some(new_text),
            status: None,
            priority: None,
            assignees: None,
            due_date: None,
        };

        self.loading = true;
        self.status = "Saving description...".to_string();
        tokio::spawn(async move {
            let result = client.update_task(&task_id, &update).await;
            let msg = match result {
                Ok(task) => AppMessage::TaskDescriptionUpdated(Ok(task)),
                Err(e) => AppMessage::TaskDescriptionUpdated(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Save the comment buffer (create, reply, or edit) — extracted from the
    /// pre-existing Ctrl+S handler so insert mode can reuse it.
    fn save_comment_from_buffer(&mut self) {
        let text = self.comment_new_text.clone();
        if text.trim().is_empty() {
            self.status = if matches!(self.comment_view_mode, CommentViewMode::InThread { .. }) {
                "Reply cannot be empty".to_string()
            } else {
                "Comment cannot be empty".to_string()
            };
            return;
        }
        if let Some(_task) = &self.task_detail.task {
            let task_id = _task.id.clone();
            let parent_id = match &self.comment_view_mode {
                CommentViewMode::InThread {
                    parent_comment_id, ..
                } => Some(parent_comment_id.clone()),
                CommentViewMode::TopLevel => None,
            };
            if let Some(edit_idx) = self.comment_editing_index {
                if edit_idx == usize::MAX {
                    self.create_comment(task_id, text, parent_id);
                } else if let Some(comment_id) =
                    self.comments.get(edit_idx).map(|c| c.id.clone())
                {
                    self.update_comment(comment_id, text);
                } else {
                    self.comment_new_text.clear();
                    self.comment_editing_index = None;
                    self.status = "Comment no longer exists".to_string();
                }
            } else {
                self.create_comment(task_id, text, parent_id);
            }
        }
    }

    // ==================== SPACEmenu command overlay (issue 7) ====================

    /// Toggle an expanded panel (SPACEmenu `t v` / `c v`).
/// If the same panel is already expanded it is restored; otherwise it replaces any current expansion.
pub fn toggle_expanded_panel(&mut self, panel: ExpandedPanel) {
    if self.expanded_panel == Some(panel) {
        self.expanded_panel = None;
        self.status = "Panel restored".to_string();
    } else {
        self.expanded_panel = Some(panel);
        self.status = if panel == ExpandedPanel::Description {
            "Description expanded (v restore, j/k scroll)".to_string()
        } else {
            "Comments expanded (v restore, j/k scroll)".to_string()
        };
    }
}

/// Start composing a comment/reply in INSERT mode (SPACEmenu `c n/e/r`).
fn start_comment_compose(&mut self, kind: ComposeKind) {
    if self.screen != Screen::TaskDetail {
        self.status = "Comments live in task detail".to_string();
        return;
    }
    match kind {
        ComposeKind::New => {
            if !self.comment_focus {
                self.comment_focus = true;
            }
            self.comment_new_text.clear();
            // usize::MAX is a sentinel value indicating "new comment" mode
            self.comment_editing_index = Some(usize::MAX);
            self.insert_mode_target = Some(InsertTarget::Comment);
            self.status =
                "INSERT mode: type comment, Ctrl+S save, Esc to normal mode".to_string();
        }
        ComposeKind::Edit => {
            if !self.comment_focus {
                self.status = "Tab to comments, then SPC c e".to_string();
                return;
            }
            if self.comments.is_empty()
                || self.comment_selected_index >= self.comments.len()
            {
                self.status = "No comment selected to edit".to_string();
                return;
            }
            let comment = &self.comments[self.comment_selected_index];
            self.comment_new_text = comment.text.clone();
            self.comment_editing_index = Some(self.comment_selected_index);
            self.insert_mode_target = Some(InsertTarget::Comment);
            self.status =
                "INSERT mode: edit comment, Ctrl+S save, Esc normal mode".to_string();
        }
        ComposeKind::Reply => {
            if !matches!(self.comment_view_mode, CommentViewMode::InThread { .. }) {
                self.status = "Enter a thread first (Enter on a comment), then SPC c r".to_string();
                return;
            }
            self.comment_new_text.clear();
            self.comment_editing_index = Some(usize::MAX);
            self.insert_mode_target = Some(InsertTarget::Comment);
            let author = match &self.comment_view_mode {
                CommentViewMode::InThread { parent_author, .. } => parent_author.clone(),
                _ => "Unknown".to_string(),
            };
            self.status = format!("INSERT reply to {}: Ctrl+S save, Esc normal mode", author);
        }
    }
}

fn handle_command_menu_input(&mut self, key: crossterm::event::KeyEvent) {
        let level = match &self.command_menu {
            Some(l) => l.clone(),
            None => return,
        };
        match level {
            CommandMenuLevel::Root => match key.code {
                KeyCode::Esc => {
                    self.command_menu = None;
                    self.status = "Command menu closed".to_string();
                }
                KeyCode::Char(c) => {
                    match c {
                        'g' => self.command_menu = Some(CommandMenuLevel::Group('g')),
                        't' => self.command_menu = Some(CommandMenuLevel::Group('t')),
                        'c' => self.command_menu = Some(CommandMenuLevel::Group('c')),
                        'y' => self.command_menu = Some(CommandMenuLevel::Group('y')),
                        'w' => self.command_menu = Some(CommandMenuLevel::Group('w')),
                        'q' => {
                            // `SPC q` -- quit via confirmation dialog (saves session)
                            self.command_menu = None;
                            self.dialog.show(DialogType::ConfirmQuit);
                        }
                        _ => {
                            self.command_menu = None;
                            self.status = format!("No command group '{}'", c);
                        }
                    }
                }
                _ => {}
            },
            CommandMenuLevel::Group(group) => {
                let g = group;
                // Consume the key and close the menu regardless
                self.command_menu = None;
                match (g, key.code) {
                    ('g', KeyCode::Char('u')) => self.open_url_input_dialog(),
                    ('t', KeyCode::Char('n')) => {
                        // New task from tasks screen
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
                    ('t', KeyCode::Char('d')) => {
                        // Delete current task
                        if self.screen == Screen::Tasks {
                            let task_id = self.task_list.selected_task().map(|t| t.id.clone());
                            self.open_delete_confirmation(task_id);
                        } else if self.screen == Screen::TaskDetail {
                            let task_id =
                                self.task_detail.task.as_ref().map(|t| t.id.clone());
                            self.open_delete_confirmation(task_id);
                        }
                    }
                    ('t', KeyCode::Char('s')) => {
                        // Status picker for current task
                        if self.screen == Screen::Tasks {
                            if let Some(task) = self.task_list.selected_task().cloned() {
                                self.open_status_picker(task);
                            }
                        } else if self.screen == Screen::TaskDetail {
                            if let Some(task) = &self.task_detail.task {
                                self.open_status_picker(task.clone());
                            }
                        }
                    }
                    ('t', KeyCode::Char('e')) => {
                        // Edit: tasks screen opens the task editor; task detail
                        // enters INSERT mode on the description
                        if self.screen == Screen::TaskDetail {
                            self.description_edit_buffer = self
                                .task_detail
                                .task
                                .as_ref()
                                .and_then(|t| t.description.as_ref().map(|d| d.as_text()))
                                .unwrap_or_default();
                            self.insert_mode_target = Some(InsertTarget::Description);
                            self.status = "INSERT mode: edit description, Ctrl+S save, Esc normal mode".to_string();
                            return;
                        }
                        if self.screen == Screen::Tasks {
                            if let Some(task) = self.task_list.selected_task().cloned() {
                                self.task_detail.task = Some(task.clone());
                                self.task_detail.editing = true;
                                self.task_detail.creating = false;
                                self.comments.clear();
                                self.comment_selected_index = 0;
                                self.comment_editing_index = None;
                                self.comment_new_text.clear();
                                self.comment_focus = false;
                                self.comment_view_mode = CommentViewMode::TopLevel;
                                self.screen = Screen::TaskDetail;
                                self.update_screen_title();
                                self.load_comments(task.id);
                            }
                        }
                    }
                    ('t', KeyCode::Char('a')) => {
                        // Assign people to the current task
                        if self.screen == Screen::TaskDetail {
                            self.open_assignee_picker_flow();
                        } else {
                            self.status = "Assign works from task detail".to_string();
                        }
                    }
                    ('t', KeyCode::Char('f')) => {
                        // Toggle "Assigned to Me" filter (tasks screen)
                        if self.screen == Screen::Tasks {
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
                    }
                    ('t', KeyCode::Char('v')) => {
                        // Expand/collapse the description panel
                        if self.screen == Screen::TaskDetail {
                            self.toggle_expanded_panel(ExpandedPanel::Description);
                        }
                    }
                    ('c', KeyCode::Char('n')) => self.start_comment_compose(ComposeKind::New),
                    ('c', KeyCode::Char('e')) => self.start_comment_compose(ComposeKind::Edit),
                    ('c', KeyCode::Char('r')) => self.start_comment_compose(ComposeKind::Reply),
                    ('c', KeyCode::Char('v')) => {
                        // Expand/collapse the comments panel
                        if self.screen == Screen::TaskDetail {
                            self.toggle_expanded_panel(ExpandedPanel::Comments);
                        }
                    }
                    ('y', KeyCode::Char('u')) => {
                        tracing::debug!("Copy URL via SPACEmenu (y u)");
                        self.copy_url();
                    }
                    ('w', KeyCode::Char('q')) => {
                        self.dialog.show(DialogType::ConfirmQuit);
                    }
                    _ => {
                        self.status = format!("No command '{}{}'", g,
                            match key.code {
                                KeyCode::Char(c) => c.to_string(),
                                _ => format!("{:?}", key.code),
                            });
                    }
                }
            }
        }
    }

    // ==================== @mention picker (issue 6) ====================

    /// Open the mention picker; fetch members if not cached
    fn open_mention_picker(&mut self) {
        // Remember where to return
        self.mention_prev_target = self.insert_mode_target.take();
        self.mention_cursor = 0;
        self.mention_query.clear();
        self.mention_picker_open = true;

        if self.mention_members.is_empty() {
            // Fetch members for the current list context
            let list_id = match &self.current_list_id {
                Some(id) => id.clone(),
                None => return, // No context; picker shows empty
            };
            let client = match &self.client {
                Some(c) => c.clone(),
                None => return,
            };
            let tx = self.message_tx.clone().unwrap();
            tokio::spawn(async move {
                let result = client.get_list_members(&list_id).await;
                let msg = match result {
                    Ok(members) => AppMessage::MentionMembersLoaded(Ok(members)),
                    Err(e) => AppMessage::MentionMembersLoaded(Err(e.to_string())),
                };
                let _ = tx.send(msg).await;
            });
        }
    }

    /// Handle keys while the mention picker overlay is open
    fn handle_mention_picker_input(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyModifiers;
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.mention_cursor + 1 < self.mention_members.len() {
                    self.mention_cursor += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.mention_cursor = self.mention_cursor.saturating_sub(1);
            }
            KeyCode::Esc => {
                // Close without inserting; return to insert mode
                self.mention_picker_open = false;
                self.insert_mode_target = self.mention_prev_target.take();
                self.status = "Mention cancelled".to_string();
            }
            KeyCode::Enter => {
                self.insert_selected_mention();
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+S saves the buffer and closes everything
                self.mention_picker_open = false;
                self.insert_mode_target = self.mention_prev_target.take();
                self.commit_insert_mode();
            }
            KeyCode::Char(c) => {
                // Type-to-filter within the picker
                self.mention_query.push(c);
                self.mention_cursor = 0;
            }
            KeyCode::Backspace => {
                self.mention_query.pop();
            }
            _ => {}
        }
    }

    /// Insert the selected mention into the underlying buffer
    fn insert_selected_mention(&mut self) {
        // Filtered member list, matching the rendered order
        let query = self.mention_query.to_lowercase();
        let candidates: Vec<&User> = self
            .mention_members
            .iter()
            .filter(|m| m.username.to_lowercase().contains(&query))
            .collect();

        if let Some(member) = candidates.get(self.mention_cursor) {
            let username = member.username.clone();
            // Replace the trailing "@<query>" typed so far with "@username "
            // NOTE: read/write via mention_prev_target — insert_mode_target was
            // moved there when the picker opened.
            let buf = match self.mention_prev_target.as_ref() {
                Some(InsertTarget::Description) => self.description_edit_buffer.clone(),
                Some(InsertTarget::Comment) => self.comment_new_text.clone(),
                _ => String::new(),
            };
            let query_suffix = self.mention_query.clone();
            let at_pos = buf.rfind('@');
            let mut new_buf = buf.clone();
            if let Some(pos) = at_pos {
                // Only strip the query if it directly follows the '@'
                let after_at = &buf[pos + 1..];
                if after_at.starts_with(&query_suffix) {
                    new_buf = format!("{}@{} {}", &buf[..pos], username, &buf[pos + 1 + query_suffix.len()..]);
                } else {
                    new_buf = format!("{} @{} ", buf, username);
                }
            } else {
                new_buf = format!("{} @{} ", buf, username);
            }

            match self.mention_prev_target.as_ref() {
                Some(InsertTarget::Description) => self.description_edit_buffer = new_buf,
                Some(InsertTarget::Comment) => self.comment_new_text = new_buf,
                _ => {}
            }
            self.status = format!("Mentioned @{}", username);
        } else {
            self.status = "No matching member".to_string();
        }

        // Return to insert mode
        self.mention_picker_open = false;
        self.insert_mode_target = self.mention_prev_target.take();
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
                            match self
                                .comments
                                .get(edit_idx)
                                .map(|comment| comment.id.clone())
                            {
                                Some(comment_id) => self.update_comment(comment_id, text),
                                None => {
                                    self.comment_new_text.clear();
                                    self.comment_editing_index = None;
                                    self.status = "Comment no longer exists".to_string();
                                }
                            }
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
            KeyCode::Enter => {
                // Newline inside the comment text
                self.comment_new_text.push('\n');
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

    /// Check whether the loading flag is set (for testing)
    #[allow(dead_code)]
    pub fn loading_true_for_test(&self) -> bool {
        self.loading
    }

    /// Insert mode active? (for testing)
    #[allow(dead_code)]
    pub fn insert_mode_active_for_test(&self) -> bool {
        self.insert_mode_target.is_some()
    }

    /// Description insert buffer (for testing)
    #[allow(dead_code)]
    pub fn description_buffer_for_test(&self) -> String {
        self.description_edit_buffer.clone()
    }

    /// Command menu open? (for testing)
    #[allow(dead_code)]
    pub fn command_menu_open_for_test(&self) -> bool {
        self.command_menu.is_some()
    }

    /// URL input dialog open? (for testing)
    #[allow(dead_code)]
    pub fn url_input_open_for_test(&self) -> bool {
        self.url_input_open
    }

    /// Test-only: set the auth token input as if typed on the auth screen.
    #[doc(hidden)]
    pub fn set_auth_token_for_test(&mut self, token: &str) {
        self.auth_state.token_input = token.to_string();
        self.auth_state.cursor_pos = token.chars().count();
    }

    /// Test-only: whether the last API failure flagged the token as invalid.
    #[doc(hidden)]
    pub fn auth_invalid_for_test(&self) -> bool {
        self.auth_invalid
    }

    /// Test-only: current auth-screen error text, if any.
    #[doc(hidden)]
    pub fn auth_error_for_test(&self) -> Option<&str> {
        self.auth_state.error.as_deref()
    }

    /// Expanded panel (for testing)
    #[allow(dead_code)]
    pub fn expanded_panel_for_test(&self) -> Option<ExpandedPanel> {
        self.expanded_panel
    }

    pub fn mention_picker_open_for_test(&self) -> bool {
        self.mention_picker_open
    }

    pub fn set_mention_members_for_test(&mut self, members: Vec<User>) {
        self.mention_members = members;
    }

    pub fn status_picker_statuses_for_test(&self) -> &[crate::models::TaskStatus] {
        &self.status_picker_statuses
    }

    pub fn set_cached_list_statuses_for_test(&mut self, list_id: &str, statuses: Vec<crate::models::TaskStatus>) {
        self.cached_list_statuses
            .insert(list_id.to_string(), statuses);
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

    /// Get the loaded comments (for testing)
    #[allow(dead_code)]
    pub fn comments_for_test(&self) -> &[crate::models::Comment] {
        &self.comments
    }

    /// Get the selected comment index (for testing)
    #[allow(dead_code)]
    pub fn comment_selected_index_for_test(&self) -> usize {
        self.comment_selected_index
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
            auth_invalid: false,
            manage_stored_token: true,
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
            task_name_input: String::new(),
            task_description_input: String::new(),
            task_creating: false,
            insert_mode_target: None,
            command_menu: None,
            expanded_panel: None,
            description_edit_buffer: String::new(),
            description_scroll_offset: 0,
            mention_members: Vec::new(),
            mention_cursor: 0,
            mention_picker_open: false,
            mention_prev_target: None,
            mention_query: String::new(),
            task_creation_focus: TaskCreationField::Name,
            assigned_filter_active: false,
            current_user_id: None,
            cached_list_members: std::collections::HashMap::new(),
            cached_list_statuses: std::collections::HashMap::new(),
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
            pending_delete_task_id: None,
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
            auth_invalid: false,
            manage_stored_token: false,
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
            task_name_input: String::new(),
            task_description_input: String::new(),
            task_creating: false,
            insert_mode_target: None,
            command_menu: None,
            expanded_panel: None,
            description_edit_buffer: String::new(),
            description_scroll_offset: 0,
            mention_members: Vec::new(),
            mention_cursor: 0,
            mention_picker_open: false,
            mention_prev_target: None,
            mention_query: String::new(),
            task_creation_focus: TaskCreationField::Name,
            assigned_filter_active: false,
            current_user_id: None,
            cached_list_members: std::collections::HashMap::new(),
            cached_list_statuses: std::collections::HashMap::new(),
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
            pending_delete_task_id: None,
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
            auth_invalid: false,
            manage_stored_token: false,
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
            task_name_input: String::new(),
            task_description_input: String::new(),
            task_creating: false,
            insert_mode_target: None,
            command_menu: None,
            expanded_panel: None,
            description_edit_buffer: String::new(),
            description_scroll_offset: 0,
            mention_members: Vec::new(),
            mention_cursor: 0,
            mention_picker_open: false,
            mention_prev_target: None,
            mention_query: String::new(),
            task_creation_focus: TaskCreationField::Name,
            assigned_filter_active: false,
            current_user_id: None,
            cached_list_members: std::collections::HashMap::new(),
            cached_list_statuses: std::collections::HashMap::new(),
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
            pending_delete_task_id: None,
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
                    AppMessage::WorkspacesVerified(result, token) => {
                        self.loading = false;
                        self.auth_state.loading = false;
                        match result {
                            Ok(workspaces) => {
                                // Token proven working - only now persist it.
                                if self.manage_stored_token {
                                    if let Err(e) = self.auth.save_token(&token) {
                                        tracing::warn!("Failed to save verified token: {}", e);
                                        self.status =
                                            "Authenticated, but failed to save token".to_string();
                                    }
                                }
                                self.auth_invalid = false;
                                self.error = None;
                                self.workspaces = workspaces.clone();
                                let mut items = Vec::new();
                                items.extend(self.workspaces.iter().map(|w| {
                                    SidebarItem::Workspace {
                                        name: w.name.clone(),
                                        id: w.id.clone(),
                                    }
                                }));
                                *self.sidebar.items_mut() = items;
                                self.sidebar.select_first();
                                self.auth_state.clear();
                                self.state = AppState::Main;
                                self.screen = Screen::Workspaces;
                                self.screen_title = generate_screen_title("Workspaces");
                                self.status = format!(
                                    "Authenticated! Loaded {} workspace(s)",
                                    workspaces.len()
                                );
                            }
                            Err(e) => {
                                self.auth_invalid = true;
                                self.state = AppState::Unauthenticated;
                                self.screen = Screen::Auth;
                                self.auth_state.clear();
                                self.auth_state.error = Some(format!(
                                    "Token rejected by ClickUp ({}). Enter a valid personal API token (pk_...).",
                                    e
                                ));
                                self.status = "Authentication failed: token rejected".to_string();
                            }
                        }
                    }
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
                                // A dead stored token must not strand the user on
                                // an error banner: drop it and return to auth.
                                if e.contains("Token invalid or expired") {
                                    self.auth_invalid = true;
                                    if self.manage_stored_token {
                                        let _ = self.auth.clear_token();
                                    }
                                    self.client = None;
                                    self.state = AppState::Unauthenticated;
                                    self.screen = Screen::Auth;
                                    self.auth_state.clear();
                                    self.auth_state.error = Some(
                                        "Stored token was rejected by ClickUp (invalid or expired). Enter a fresh personal API token (pk_...)."
                                            .to_string(),
                                    );
                                    self.status = "Re-authentication required".to_string();
                                } else {
                                    self.error =
                                        Some(format!("Failed to load workspaces: {}", e));
                                    self.status = "Failed to load workspaces".to_string();
                                }
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
                                // Cache any status configuration that came with the lists
                                for l in &self.lists {
                                    if !l.statuses.is_empty() {
                                        self.cached_list_statuses
                                            .insert(l.id.clone(), l.statuses.clone());
                                    }
                                }
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
                                                self.task_list
                                                .rows()
                                                .iter()
                                                .filter(|r| matches!(r, ListRow::Task(_)))
                                                .count()
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
                                            self.task_list
                                            .rows()
                                            .iter()
                                            .filter(|r| matches!(r, ListRow::Task(_)))
                                            .count()
                                        );
                                    }
                                } else {
                                    self.task_list.select_first();
                                    self.status = format!(
                                        "Loaded {} task(s)",
                                        self.task_list
                                        .rows()
                                        .iter()
                                        .filter(|r| matches!(r, ListRow::Task(_)))
                                        .count()
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
                                self.comment_selected_index = 0;
                                self.comment_editing_index = None;
                                self.comment_new_text.clear();
                            }
                        }
                    }
                    AppMessage::CommentCreated(result, comment_meta) => {
                        self.loading = false;
                        match result {
                            Ok(comment) => {
                                self.comments.insert(0, comment);
                                self.comment_new_text.clear();
                                self.comment_editing_index = None;
                                self.status = if comment_meta.is_reply {
                                    "Reply added".to_string()
                                } else {
                                    "Comment added".to_string()
                                };

                                // refetch comments on comment created
                                self.load_comments(comment_meta.task_id);
                            }
                            Err(e) => {
                                self.error = Some(format!(
                                    "Failed to create {}: {}",
                                    if comment_meta.is_reply {
                                        "reply"
                                    } else {
                                        "comment"
                                    },
                                    e
                                ));
                                self.status = format!(
                                    "Failed to create {}",
                                    if comment_meta.is_reply {
                                        "reply"
                                    } else {
                                        "comment"
                                    }
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
                    AppMessage::ListStatusesLoaded(list_id, result) => {
                        self.loading = false;
                        match result {
                            Ok(statuses) => {
                                if !statuses.is_empty() {
                                    self.cached_list_statuses
                                        .insert(list_id.clone(), statuses.clone());
                                }
                                // If the picker is open waiting for statuses, refresh it
                                if self.status_picker_open {
                                    if !statuses.is_empty() {
                                        self.status_picker_statuses = statuses;
                                        if self.status_picker_cursor
                                            >= self.status_picker_statuses.len()
                                        {
                                            self.status_picker_cursor = 0;
                                        }
                                    }
                                    self.status = "Statuses loaded".to_string();
                                }
                            }
                            Err(e) => {
                                // Keep whatever options the picker already has
                                if self.status_picker_open {
                                    self.status = format!(
                                        "Could not load full status list ({}); showing known statuses",
                                        e
                                    );
                                }
                            }
                        }
                    }
                    AppMessage::MembersLoaded(result) => {
                        self.loading = false;
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
                        self.loading = false;
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
                        self.loading = false;
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
                                if let Some(ref original_status) =
                                    self.status_picker_original_status
                                {
                                    if let Some(ref task_id) = self.status_picker_task_id {
                                        for task in &mut self.tasks {
                                            if task.id == *task_id {
                                                task.status = Some(original_status.clone());
                                                break;
                                            }
                                        }
                                        if let Some(detail_task) = &mut self.task_detail.task {
                                            if detail_task.id == *task_id {
                                                detail_task.status = Some(original_status.clone());
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
                    AppMessage::TaskDescriptionUpdated(result) => {
                        self.loading = false;
                        match result {
                            Ok(updated_task) => {
                                for task in &mut self.tasks {
                                    if task.id == updated_task.id {
                                        *task = updated_task.clone();
                                        break;
                                    }
                                }
                                self.rebuild_task_list();
                                self.task_detail.task = Some(updated_task.clone());
                                self.description_edit_buffer.clear();
                                self.status = "Description saved".to_string();
                            }
                            Err(e) => {
                                self.error = Some(format!("Failed to save description: {}", e));
                                self.status = "Failed to save description".to_string();
                            }
                        }
                    }
                    AppMessage::MentionMembersLoaded(result) => match result {
                        Ok(members) => {
                            self.mention_members = members;
                            if self.mention_members.is_empty() {
                                self.status = "No members found for @mentions".to_string();
                            }
                        }
                        Err(e) => {
                            self.mention_picker_open = false;
                            self.insert_mode_target = self.mention_prev_target.take();
                            self.status = format!("Could not load members for @: {}", e);
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
                                if e.contains("Token invalid or expired") {
                                    self.status = "Token invalid - re-authenticate with a fresh ClickUp API token (pk_...)".to_string();
                                } else {
                                    self.status = format!("Resource not found: {}", e);
                                }
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
                                        Err(e) => AppMessage::CommentsLoadedForCommentNavigation(
                                            Err(e.to_string()),
                                            comment_id,
                                        ),
                                    };
                                    let _ = tx.send(msg).await;
                                });
                            }
                            Err(e) => {
                                self.navigating = false;
                                self.navigating_level.clear();
                                if e.contains("Token invalid or expired") {
                                    self.status = "Token invalid - re-authenticate with a fresh ClickUp API token (pk_...)".to_string();
                                } else {
                                    self.status = format!("Resource not found: {}", e);
                                }
                                self.screen = prev_screen;
                            }
                        }
                    }
                    AppMessage::DocumentFetchedForNavigation(result, prev_screen) => match result {
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
                    },
                    AppMessage::CommentsLoadedForCommentNavigation(result, comment_id) => {
                        match result {
                            Ok(comments) => {
                                self.comments = comments;
                                self.comment_view_mode = CommentViewMode::TopLevel;
                                // Find and select the target comment
                                let found = self.comments.iter().position(|c| c.id == comment_id);
                                if let Some(idx) = found {
                                    self.comment_selected_index = idx;
                                    self.navigating = false;
                                    self.navigating_level.clear();
                                    self.status = format!(
                                        "Navigated to comment: {}",
                                        crate::utils::truncate_chars(&self.comments[idx].text, 40)
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
                                self.tasks.retain(|task| task.id != task_id);

                                let deleted_open_task = self
                                    .task_detail
                                    .task
                                    .as_ref()
                                    .is_some_and(|task| task.id == task_id);
                                if deleted_open_task {
                                    self.task_detail.task = None;
                                    self.task_detail.editing = false;
                                    self.task_detail.creating = false;
                                    self.comments.clear();
                                    self.comment_selected_index = 0;
                                    self.comment_editing_index = None;
                                    self.comment_new_text.clear();
                                    self.comment_focus = false;
                                    self.comment_view_mode = CommentViewMode::TopLevel;
                                    if self.screen == Screen::TaskDetail {
                                        self.screen = Screen::Tasks;
                                    }
                                }

                                // Rebuild grouped rows so the deleted task disappears
                                // from the widget, not just from `self.tasks`.
                                self.rebuild_task_list();
                                self.update_screen_title();
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
        // Mention picker is a modal overlay (issue 6): highest priority
        if self.mention_picker_open {
            if let InputEvent::Key(key) = event {
                self.handle_mention_picker_input(key);
            }
            return;
        }

        // Vim-style insert mode (issue 7): while active, all printable keys go
        // into the buffer; normal-mode commands are not interpreted.
        if self.insert_mode_target.is_some() {
            if let InputEvent::Key(key) = event {
                self.handle_insert_mode(key);
            }
            return;
        }

        // SPACEmenu command overlay (issue 7)
        if self.command_menu.is_some() {
            if let InputEvent::Key(key) = event {
                self.handle_command_menu_input(key);
            }
            return;
        }

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
            if key.code == KeyCode::Char('?')
                && !self.is_text_input_active()
                && self.screen != Screen::Auth
            {
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
                        self.pending_delete_task_id = None;
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

            // SPACE opens the command menu (Spacemacs-style SPACEmenu)
            if key.code == KeyCode::Char(' ') && self.screen != Screen::Auth {
                self.open_command_menu();
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
                KeyCode::PageDown => {
                    self.task_list.select_next_page();
                }
                KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.task_list.select_next_page();
                }
                KeyCode::PageUp => {
                    self.task_list.select_previous_page();
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.task_list.select_previous_page();
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
                            TaskCreationField::Name => {
                                self.task_name_input.pop();
                            }
                            TaskCreationField::Description => {
                                self.task_description_input.pop();
                            }
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
                                    match self
                                        .comments
                                        .get(edit_idx)
                                        .map(|comment| comment.id.clone())
                                    {
                                        Some(comment_id) => self.update_comment(comment_id, text),
                                        None => {
                                            self.comment_new_text.clear();
                                            self.comment_editing_index = None;
                                            self.status = "Comment no longer exists".to_string();
                                            return;
                                        }
                                    }
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
                    KeyCode::Enter => {
                        // Newline inside the comment text
                        self.comment_new_text.push('\n');
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
                    // Collapse an expanded panel first (issue 5)
                    if self.expanded_panel.is_some() {
                        self.expanded_panel = None;
                        self.status = "Panel restored".to_string();
                        return;
                    }
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
                KeyCode::Char('j') if self.expanded_panel == Some(ExpandedPanel::Description) => {
                    self.description_scroll_down();
                }
                KeyCode::Char('k') if self.expanded_panel == Some(ExpandedPanel::Description) => {
                    self.description_scroll_up();
                }
                KeyCode::PageDown => {
                    // Page down: expanded panel scrolls a page; comment focus pages selection
                    if self.expanded_panel == Some(ExpandedPanel::Description) {
                        self.description_scroll_offset = self.description_scroll_offset.saturating_add(20);
                    } else if self.comment_focus {
                        self.move_comment_selection(10);
                    }
                }
                KeyCode::PageUp => {
                    // Page up: expanded panel scrolls a page; comment focus pages selection
                    if self.expanded_panel == Some(ExpandedPanel::Description) {
                        self.description_scroll_offset = self.description_scroll_offset.saturating_sub(20);
                    } else if self.comment_focus {
                        self.move_comment_selection(-10);
                    }
                }
                KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.expanded_panel == Some(ExpandedPanel::Description) {
                        self.description_scroll_offset = self.description_scroll_offset.saturating_add(20);
                    } else if self.comment_focus {
                        self.move_comment_selection(10);
                    }
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.expanded_panel == Some(ExpandedPanel::Description) {
                        self.description_scroll_offset = self.description_scroll_offset.saturating_sub(20);
                    } else if self.comment_focus {
                        self.move_comment_selection(-10);
                    }
                }
                KeyCode::Char('j') if self.comment_focus => {
                    self.move_comment_selection(1);
                }
                KeyCode::Char('k') if self.comment_focus => {
                    self.move_comment_selection(-1);
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
                // Modal entry keys: 'n' starts a new comment (INPUT mode) when
                // comment focus is on; 'e' enters description INPUT mode otherwise.
                KeyCode::Char('n') if self.comment_focus => {
                    self.start_comment_compose(ComposeKind::New);
                }
                KeyCode::Char('e') if !self.comment_focus => {
                    if let Some(task) = self.task_detail.task.clone() {
                        self.description_edit_buffer = task
                            .description
                            .as_ref()
                            .map(|d| d.as_text())
                            .unwrap_or_default();
                        self.insert_mode_target = Some(InsertTarget::Description);
                        self.status =
                            "INSERT mode: edit description, Ctrl+S save, Esc normal mode".to_string();
                    } else {
                        self.status = "No task selected".to_string();
                    }
                }
                _ => {}
            }
        }
    }

    /// Indices of comments visible in the current comment view, in display order.
    fn comment_indices_in_view(&self) -> Vec<usize> {
        match &self.comment_view_mode {
            CommentViewMode::TopLevel => self
                .comments
                .iter()
                .enumerate()
                .filter(|(_, comment)| comment.parent_id.is_none())
                .map(|(index, _)| index)
                .collect(),
            CommentViewMode::InThread {
                parent_comment_id, ..
            } => {
                let mut indices = Vec::new();
                if let Some(parent_index) = self
                    .comments
                    .iter()
                    .position(|comment| &comment.id == parent_comment_id)
                {
                    indices.push(parent_index);
                }
                indices.extend(
                    self.comments
                        .iter()
                        .enumerate()
                        .filter(|(_, comment)| {
                            comment.parent_id.as_ref() == Some(parent_comment_id)
                        })
                        .map(|(index, _)| index),
                );
                indices
            }
        }
    }

    /// Move the comment selection within the currently visible comments.
    fn move_comment_selection(&mut self, delta: isize) {
        let indices = self.comment_indices_in_view();
        if indices.is_empty() {
            return;
        }

        let current_position = indices
            .iter()
            .position(|index| *index == self.comment_selected_index);
        let next_position = match current_position {
            Some(position) => {
                (position as isize + delta).rem_euclid(indices.len() as isize) as usize
            }
            None => 0,
        };
        self.comment_selected_index = indices[next_position];
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
                KeyCode::PageDown => {
                    // Page down: roughly a screenful of lines
                    for _ in 0..10 {
                        self.document.scroll_down();
                    }
                }
                KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    for _ in 0..10 {
                        self.document.scroll_down();
                    }
                }
                KeyCode::PageUp => {
                    // Page up: roughly a screenful of lines
                    for _ in 0..10 {
                        self.document.scroll_up();
                    }
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    for _ in 0..10 {
                        self.document.scroll_up();
                    }
                }
                KeyCode::Esc => {
                    self.navigate_back();
                }
                _ => {}
            }
        }
    }

    /// Scroll the expanded description panel down
    fn description_scroll_down(&mut self) {
        self.description_scroll_offset = self.description_scroll_offset.saturating_add(3);
    }

    /// Scroll the expanded description panel up
    fn description_scroll_up(&mut self) {
        self.description_scroll_offset = self.description_scroll_offset.saturating_sub(3);
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
                if let Some(list) = self
                    .current_list_id
                    .as_ref()
                    .and_then(|id| self.lists.iter().find(|list| &list.id == id))
                {
                    self.screen_title = generate_screen_title(&format!("Tasks: {}", list.name));
                } else {
                    self.screen_title = generate_screen_title("Tasks");
                }
            }
            Screen::Document => {
                self.screen = Screen::Tasks;
                self.update_screen_title();
            }
        }
    }

    /// Authenticate with the token typed on the auth screen.
    /// Public so tests can drive the auth flow end-to-end.
    pub fn authenticate(&mut self) {
        let token = self.auth_state.token_input.clone();
        if token.is_empty() {
            self.auth_state.error = Some("Token cannot be empty".to_string());
            self.status = "Authentication failed".to_string();
            return;
        }

        // Reject obviously malformed input before anything else: ClickUp
        // personal API tokens start with pk_. Pasting a task URL (or any
        // other non-token text) previously got saved verbatim and silently
        // broke every API call until the file was fixed manually.
        if !token.starts_with("pk_") {
            self.auth_state.error =
                Some("Token must be a ClickUp API token (starts with pk_)".to_string());
            self.status = "Authentication failed".to_string();
            return;
        }

        self.loading = true;
        self.auth_state.loading = true;
        self.status = "Verifying token...".to_string();

        // Reuse an injected client when present (tests); otherwise create a
        // real HTTP client for the verification call.
        let client = match &self.client {
            Some(c) => c.clone(),
            None => Arc::new(ClickUpClient::new(token.clone())),
        };
        self.client = Some(client.clone());

        // Do NOT persist the token yet. Verify it against the API first so a
        // bad token never overwrites a previously working one on disk. The
        // token travels with the result and is only saved on success.
        let tx = self.message_tx.clone().unwrap();
        tokio::spawn(async move {
            let result = client.get_workspaces().await;
            let msg = match result {
                Ok(workspaces) => AppMessage::WorkspacesVerified(Ok(workspaces), token),
                Err(e) => AppMessage::WorkspacesVerified(Err(e.to_string()), token),
            };
            let _ = tx.send(msg).await;
        });
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

        // Immediate UI feedback while the request is in flight
        self.loading = true;
        self.status = "Saving assignees...".to_string();

        tokio::spawn(async move {
            let result = client.update_task(&task_id, &update).await;
            let msg = match result {
                Ok(task) => AppMessage::AssigneesUpdated(Ok(task)),
                Err(e) => AppMessage::AssigneesUpdated(Err(e.to_string())),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Open the status picker for a task.
    ///
    /// The ClickUp API does not expose list statuses through the endpoints we use
    /// here, so derive the options from statuses already observed on loaded tasks.
    /// This still lets users pick any status actually in use in the list.
    fn open_status_picker(&mut self, task: crate::models::Task) {
        use crate::models::TaskStatus;

        // Store the task ID we're changing status for
        self.status_picker_task_id = Some(task.id.clone());

        // Store original status for potential rollback
        self.status_picker_original_status = task.status.clone();

        let mut statuses: Vec<TaskStatus> = Vec::new();
        let add_status = |status: &TaskStatus, statuses: &mut Vec<TaskStatus>| {
            if !statuses
                .iter()
                .any(|existing| existing.status.eq_ignore_ascii_case(&status.status))
            {
                statuses.push(status.clone());
            }
        };

        // Prefer the list's full status configuration (all available statuses,
        // not just the subset currently in use by loaded tasks).
        if let Some(list_id) = &self.current_list_id {
            if let Some(cached) = self.cached_list_statuses.get(list_id) {
                for status in cached {
                    add_status(status, &mut statuses);
                }
            }
        }
        // Fall back to statuses observed on loaded tasks
        if statuses.is_empty() {
            for loaded_task in &self.tasks {
                if let Some(status) = &loaded_task.status {
                    add_status(status, &mut statuses);
                }
            }
        }
        if let Some(status) = &task.status {
            add_status(status, &mut statuses);
        }

        // Last-resort defaults when no task has any status information.
        if statuses.is_empty() {
            statuses = vec![
                TaskStatus {
                    id: None,
                    status: "To Do".to_string(),
                    color: Some("#8794a6".to_string()),
                    type_field: None,
                    orderindex: Some(0),
                    status_group: Some("todo".to_string()),
                },
                TaskStatus {
                    id: None,
                    status: "In Progress".to_string(),
                    color: Some("#4f46de".to_string()),
                    type_field: None,
                    orderindex: Some(1),
                    status_group: Some("in_progress".to_string()),
                },
                TaskStatus {
                    id: None,
                    status: "Done".to_string(),
                    color: Some("#0f4a58".to_string()),
                    type_field: None,
                    orderindex: Some(2),
                    status_group: Some("done".to_string()),
                },
            ];
        }

        statuses.sort_by(|a, b| {
            a.orderindex
                .unwrap_or(u32::MAX)
                .cmp(&b.orderindex.unwrap_or(u32::MAX))
                .then_with(|| a.status.to_lowercase().cmp(&b.status.to_lowercase()))
        });

        self.status_picker_cursor = task
            .status
            .as_ref()
            .and_then(|current| {
                statuses
                    .iter()
                    .position(|status| status.status.eq_ignore_ascii_case(&current.status))
            })
            .unwrap_or(0);
        self.status_picker_statuses = statuses;
        self.status_picker_open = true;

        // If we only know a subset of the list's statuses, fetch the full
        // configuration in the background and refresh the picker when it lands.
        if let Some(list_id) = &self.current_list_id {
            if !self.cached_list_statuses.contains_key(list_id) {
                let client = match &self.client {
                    Some(c) => c.clone(),
                    None => return,
                };
                let list_id = list_id.clone();
                let tx = self.message_tx.clone().unwrap();
                self.loading = true;
                self.status = "Loading statuses...".to_string();
                tokio::spawn(async move {
                    let result = client.get_list(&list_id).await;
                    let statuses = result.map(|l| l.statuses);
                    let msg = AppMessage::ListStatusesLoaded(
                        list_id,
                        statuses.map_err(|e| e.to_string()),
                    );
                    let _ = tx.send(msg).await;
                });
            }
        }
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
            Some(status) => status.clone(),
            None => {
                self.status_picker_open = false;
                self.status = "No status selected".to_string();
                return;
            }
        };

        // Optimistic UI update: update the task in self.tasks and rebuild grouped list
        for task in &mut self.tasks {
            if task.id == task_id {
                task.status = Some(new_status.clone());
                break;
            }
        }
        if let Some(detail_task) = &mut self.task_detail.task {
            if detail_task.id == task_id {
                detail_task.status = Some(new_status.clone());
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
        let new_status_clone = new_status.status.clone();
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

            let meta = CommentCreatedMessageMeta { is_reply, task_id };
            let msg = match result {
                Ok(comment) => AppMessage::CommentCreated(Ok(comment), meta),
                Err(e) => AppMessage::CommentCreated(Err(e.to_string()), meta),
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

    /// Open the delete confirmation for a specific task.
    fn open_delete_confirmation(&mut self, task_id: Option<String>) {
        match task_id {
            Some(task_id) => {
                self.pending_delete_task_id = Some(task_id);
                self.dialog.show(DialogType::ConfirmDelete);
            }
            None => {
                self.status = "No task selected".to_string();
            }
        }
    }

    /// Delete the task captured when the confirmation dialog was opened.
    fn delete_selected_task(&mut self) {
        let task_id = match self.pending_delete_task_id.take() {
            Some(task_id) => task_id,
            None => {
                // Fallback for callers that show the dialog without going through
                // open_delete_confirmation (e.g. older tests).
                let fallback = if self.screen == Screen::TaskDetail {
                    self.task_detail.task.as_ref().map(|task| task.id.clone())
                } else {
                    self.task_list.selected_task().map(|task| task.id.clone())
                };
                match fallback {
                    Some(task_id) => task_id,
                    None => {
                        self.status = "No task selected".to_string();
                        return;
                    }
                }
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
                if let Some(workspace) = self
                    .current_workspace_id
                    .as_ref()
                    .and_then(|id| self.workspaces.iter().find(|workspace| &workspace.id == id))
                {
                    generate_screen_title(&workspace.name)
                } else {
                    generate_screen_title("Spaces")
                }
            }
            Screen::Folders => {
                if let Some(space) = self
                    .current_space_id
                    .as_ref()
                    .and_then(|id| self.spaces.iter().find(|space| &space.id == id))
                {
                    generate_screen_title(&space.name)
                } else {
                    generate_screen_title("Folders")
                }
            }
            Screen::Lists => {
                if let Some(folder) = self
                    .current_folder_id
                    .as_ref()
                    .and_then(|id| self.folders.iter().find(|folder| &folder.id == id))
                {
                    generate_screen_title(&folder.name)
                } else {
                    generate_screen_title("Lists")
                }
            }
            Screen::Tasks => {
                if let Some(list) = self
                    .current_list_id
                    .as_ref()
                    .and_then(|id| self.lists.iter().find(|list| &list.id == id))
                {
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
                let current_status = self.status_picker_task_id.as_ref().and_then(|task_id| {
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

            // Render SPACEmenu command overlay if open (issue 7)
            if self.command_menu.is_some() {
                let level = self.command_menu.clone().unwrap();
                render_command_menu(frame, area, &level);
            }

            // Render @mention picker overlay if open (issue 6)
            if self.mention_picker_open {
                render_mention_picker(
                    frame,
                    area,
                    &self.mention_members,
                    self.mention_cursor,
                    &self.mention_query,
                );
            }

            // Render help overlay if visible
            let help_context = self.get_help_context();
            render_help(frame, &self.help, &help_context, area);

            // Render status line
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
            layout.render_status(frame, &status);
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
            Screen::Tasks => {
                self.task_list.set_viewport_height(area.height.saturating_sub(2) as usize);
                render_task_list(frame, &self.task_list, area, false)
            }
            Screen::TaskDetail => {
                // Expanded panel takes the full screen (issue 5)
                let (task_detail_area, comments_area) = match self.expanded_panel {
                    Some(ExpandedPanel::Description) => (area, Rect::default()),
                    Some(ExpandedPanel::Comments) => (Rect::default(), area),
                    None => split_task_detail(area),
                };

                if task_detail_area.height > 0 {
                    // Render task detail (30% normally, full screen when expanded)
                    render_task_detail_expanded(
                        frame,
                        &self.task_detail,
                        task_detail_area,
                        &self.task_name_input,
                        &self.task_description_input,
                        &self.task_creation_focus,
                        self.expanded_panel == Some(ExpandedPanel::Description),
                        self.description_scroll_offset,
                    );
                }

                if comments_area.height > 0 {
                    // Render comments (70% normally, full screen when expanded)
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
                ratatui::layout::Constraint::Length(1), // prompt
                ratatui::layout::Constraint::Length(1), // spacer
                ratatui::layout::Constraint::Length(3), // input field
                ratatui::layout::Constraint::Length(2), // error message
                ratatui::layout::Constraint::Length(1), // spacer
                ratatui::layout::Constraint::Length(1), // hints
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
        let cursor_pos = self
            .url_input_cursor
            .min(self.url_input_text.chars().count());
        let input_line: Line = if self.url_input_text.is_empty() {
            Line::from(Span::styled(
                "https://app.clickup.com/...",
                Style::default().fg(ratatui::style::Color::DarkGray),
            ))
        } else {
            // Build the line with cursor position tracking
            let before_cursor: String = self.url_input_text.chars().take(cursor_pos).collect();
            let after_cursor: String = self.url_input_text.chars().skip(cursor_pos).collect();
            let mut spans = vec![Span::styled(
                before_cursor,
                Style::default().fg(ratatui::style::Color::Rgb(205, 214, 244)),
            )];
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
        let hints = Paragraph::new("Enter: Navigate | Esc: Cancel | Ctrl+V: Paste")
            .style(Style::default().fg(ratatui::style::Color::DarkGray));
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
                self.url_input_cursor = url.chars().count();
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
                            let byte_index = char_index_to_byte_index(
                                &self.url_input_text,
                                self.url_input_cursor,
                            );
                            self.url_input_text.insert(byte_index, c);
                            self.url_input_cursor += 1;
                        }
                        self.url_input_error = None;
                    }
                }
            }
            KeyCode::Char(c) => {
                let byte_index =
                    char_index_to_byte_index(&self.url_input_text, self.url_input_cursor);
                self.url_input_text.insert(byte_index, c);
                self.url_input_cursor += 1;
                self.url_input_error = None;
            }
            KeyCode::Backspace => {
                if self.url_input_cursor > 0 {
                    let start =
                        char_index_to_byte_index(&self.url_input_text, self.url_input_cursor - 1);
                    let end = char_index_to_byte_index(&self.url_input_text, self.url_input_cursor);
                    self.url_input_text.replace_range(start..end, "");
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
                if self.url_input_cursor < self.url_input_text.chars().count() {
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
            ParsedUrl::Space {
                workspace_id,
                space_id,
            } => {
                self.navigate_to_space(workspace_id, space_id);
            }
            ParsedUrl::Folder {
                workspace_id,
                folder_id,
            } => {
                self.navigate_to_folder(workspace_id, folder_id);
            }
            ParsedUrl::List {
                workspace_id,
                list_id,
            } => {
                self.navigate_to_list(workspace_id, list_id);
            }
            ParsedUrl::Task { team_id, task_id } => {
                self.navigate_to_task(team_id, task_id);
            }
            ParsedUrl::Comment {
                team_id,
                task_id,
                comment_id,
            } => {
                self.navigate_to_comment(team_id, task_id, comment_id);
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
    fn navigate_to_task(&mut self, team_id: Option<String>, task_id: String) {
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
        let target_team_id = team_id;
        let prev_screen = self.screen.clone();
        tokio::spawn(async move {
            // Fetch task from API to get its context
            let result = client
                .get_task_with_context(&target_task_id, target_team_id.as_deref())
                .await;
            let msg = match result {
                Ok(task) => AppMessage::TaskFetchedForNavigation(Ok(task), prev_screen),
                Err(e) => AppMessage::TaskFetchedForNavigation(Err(e.to_string()), prev_screen),
            };
            let _ = tx.send(msg).await;
        });
    }

    /// Navigate to a comment by task ID + comment ID
    fn navigate_to_comment(&mut self, team_id: Option<String>, task_id: String, comment_id: String) {
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
        let target_team_id = team_id;
        let target_comment_id = comment_id.clone();
        let prev_screen = self.screen.clone();
        tokio::spawn(async move {
            let result = client
                .get_task_with_context(&target_task_id, target_team_id.as_deref())
                .await;
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
                            .send(AppMessage::DocumentFetchedForNavigation(
                                Ok(doc),
                                prev_screen,
                            ))
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

        let mock_client = MockClickUpClient::new().with_tasks(vec![]);

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Set up the app in Tasks screen with a list context
        app.set_screen(Screen::Tasks);
        app.current_list_id = Some("test-list-1".to_string());

        // Direct 'n' key must no longer trigger anything (SPACEmenu-only commands)
        let n_key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
        app.update(InputEvent::Key(n_key));
        assert_eq!(app.screen, Screen::Tasks, "direct n must not open creation");
        assert!(!app.task_creating, "direct n must not set task_creating");

        // SPACEmenu: SPACE, t, n
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char(' '),
            KeyModifiers::NONE,
        )));
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('t'),
            KeyModifiers::NONE,
        )));
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('n'),
            KeyModifiers::NONE,
        )));

        // Should navigate to TaskDetail screen
        assert_eq!(
            app.screen,
            Screen::TaskDetail,
            "Should navigate to TaskDetail"
        );

        // Should be in creation mode
        assert!(app.task_creating, "task_creating should be true");
        assert!(
            app.task_detail.creating,
            "task_detail.creating should be true"
        );

        // Input fields should be empty
        assert_eq!(app.task_name_input, "", "Name input should be empty");
        assert_eq!(
            app.task_description_input, "",
            "Description input should be empty"
        );

        // Focus should be on name field
        assert_eq!(
            app.task_creation_focus,
            TaskCreationField::Name,
            "Focus should be on Name"
        );

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

    fn make_task(id: &str, name: &str) -> Task {
        Task {
            id: id.to_string(),
            name: name.to_string(),
            ..Default::default()
        }
    }

    fn make_comment(id: &str, text: &str, parent_id: Option<&str>) -> crate::models::Comment {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "comment_text": text,
            "parent_id": parent_id,
        }))
            .expect("comment fixture should deserialize")
    }

    #[tokio::test]
    async fn test_delete_confirmation_deletes_captured_task_not_current_selection() {
        use std::time::Duration;

        let first = make_task("task-1", "First");
        let second = make_task("task-2", "Second");
        let mock_client = MockClickUpClient::new()
            .with_tasks(vec![first.clone(), second.clone()])
            .with_delete_task_success();
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.tasks = vec![first.clone(), second.clone()];
        app.task_list = GroupedTaskList::from_tasks(app.tasks.clone());
        app.task_list.select_first();
        app.open_delete_confirmation(Some(first.id.clone()));
        let second_index = app
            .task_list
            .rows()
            .iter()
            .position(|row| matches!(row, ListRow::Task(task) if task.id == second.id))
            .expect("second task row should exist");
        app.task_list.select(Some(second_index));
        assert_eq!(
            app.task_list.selected_task().map(|task| task.id.as_str()),
            Some("task-2")
        );

        app.delete_selected_task();
        tokio::time::sleep(Duration::from_millis(20)).await;
        app.process_async_messages();

        assert_eq!(
            app.tasks
            .iter()
            .map(|task| task.id.as_str())
            .collect::<Vec<_>>(),
            vec!["task-2"]
        );
        assert_eq!(
            app.task_list
            .rows()
            .iter()
            .filter(|row| matches!(row, ListRow::Task(_)))
            .count(),
            1
        );
        assert!(app.pending_delete_task_id.is_none());
    }

    #[test]
    fn test_task_deleted_message_rebuilds_grouped_list_and_leaves_detail() {
        let first = make_task("task-1", "First");
        let second = make_task("task-2", "Second");
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.tasks = vec![first.clone(), second.clone()];
        app.task_list = GroupedTaskList::from_tasks(app.tasks.clone());
        app.screen = Screen::TaskDetail;
        app.task_detail.task = Some(first.clone());

        let tx = app.message_tx.clone().unwrap();
        tx.try_send(AppMessage::TaskDeleted(Ok(first.id.clone())))
            .unwrap();
        app.process_async_messages();

        assert_eq!(app.tasks.len(), 1);
        assert_eq!(
            app.task_list
            .rows()
            .iter()
            .filter(|row| matches!(row, ListRow::Task(_)))
            .count(),
            1
        );
        assert!(app.task_detail.task.is_none());
        assert_eq!(app.screen, Screen::Tasks);
    }

    #[test]
    fn test_question_mark_does_not_toggle_help_in_text_inputs_or_auth() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        let question = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE);

        app.url_input_open = true;
        app.update(InputEvent::Key(question));
        assert!(!app.is_help_visible());

        app.url_input_open = false;
        app.screen = Screen::TaskDetail;
        app.task_detail.task = Some(make_task("task-1", "Task"));
        app.comments = vec![make_comment("comment-1", "Body", None)];
        app.comment_editing_index = Some(0);
        app.update(InputEvent::Key(question));
        assert!(!app.is_help_visible());

        app.comment_editing_index = None;
        app.comment_new_text.clear();
        app.screen = Screen::Auth;
        app.update(InputEvent::Key(question));
        assert!(!app.is_help_visible());
    }

    #[test]
    fn test_url_input_handles_multibyte_cursor_positions() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        app.url_input_text = "é".to_string();
        app.url_input_cursor = 1;

        app.handle_url_input(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
        assert_eq!(app.url_input_text, "éx");
        assert_eq!(app.url_input_cursor, 2);

        app.handle_url_input(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(app.url_input_text, "é");
        assert_eq!(app.url_input_cursor, 1);
    }

    #[test]
    fn test_comment_navigation_skips_hidden_replies_and_wraps() {
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        app.comments = vec![
            make_comment("1", "parent", None),
            make_comment("2", "reply one", Some("1")),
            make_comment("3", "reply two", Some("1")),
            make_comment("4", "other", None),
        ];
        app.comment_view_mode = CommentViewMode::TopLevel;
        app.comment_selected_index = 0;

        app.move_comment_selection(-1);
        assert_eq!(app.comment_selected_index, 3);

        app.comment_view_mode = CommentViewMode::InThread {
            parent_comment_id: "1".to_string(),
            parent_author: "author".to_string(),
        };
        app.comment_selected_index = 2;
        app.move_comment_selection(1);
        assert_eq!(app.comment_selected_index, 0);
    }

    #[test]
    fn test_stale_comment_edit_index_does_not_panic() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        app.task_detail.task = Some(make_task("task-1", "Task"));
        app.comments.clear();
        app.comment_editing_index = Some(7);
        app.comment_new_text = "stale edit".to_string();

        app.handle_comment_input(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));

        assert!(app.comment_editing_index.is_none());
        assert!(app.comment_new_text.is_empty());
        assert_eq!(app.status(), "Comment no longer exists");
    }

    #[tokio::test]
    async fn test_e_key_loads_selected_task_into_detail() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let task = make_task("task-1", "Selected");
        let mock_client = MockClickUpClient::new().with_tasks(vec![task.clone()]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        app.tasks = vec![task.clone()];
        app.task_list = GroupedTaskList::from_tasks(app.tasks.clone());
        app.task_list.select_first();
        app.screen = Screen::Tasks;
        app.current_list_id = Some("list-1".to_string());

        // Direct 'e' key must no longer trigger anything (SPACEmenu-only commands)
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('e'),
            KeyModifiers::NONE,
        )));
        assert_eq!(app.screen, Screen::Tasks, "direct e must not open editor");

        // SPACEmenu: SPACE, t, e
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char(' '),
            KeyModifiers::NONE,
        )));
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('t'),
            KeyModifiers::NONE,
        )));
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('e'),
            KeyModifiers::NONE,
        )));

        assert_eq!(app.screen, Screen::TaskDetail);
        assert!(app.task_detail.editing);
        assert_eq!(
            app.task_detail.task.as_ref().map(|task| task.id.as_str()),
            Some("task-1")
        );
    }

    #[tokio::test]
    async fn test_page_down_moves_task_selection_by_page() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let tasks: Vec<_> = (0..30)
            .map(|i| make_task(&format!("task-{}", i), &format!("Task {}", i)))
            .collect();
        let mock_client = MockClickUpClient::new().with_tasks(tasks.clone());
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        app.tasks = tasks;
        app.task_list = GroupedTaskList::from_tasks(app.tasks.clone());
        app.task_list.select_first();
        app.screen = Screen::Tasks;
        // Simulate a 20-row viewport (render records height - 2 border rows)
        app.task_list.set_viewport_height(18);

        let start = app.task_list_for_test().selected_task().map(|t| t.id.clone());
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::PageDown,
            KeyModifiers::NONE,
        )));
        let after_down = app.task_list_for_test().selected_task().map(|t| t.id.clone());
        assert_ne!(
            after_down, start,
            "PageDown must move the selection forward"
        );

        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('u'),
            KeyModifiers::CONTROL,
        )));
        let after_up = app.task_list_for_test().selected_task().map(|t| t.id.clone());
        assert_ne!(
            after_up, after_down,
            "Ctrl+U must move the selection back"
        );
    }

    #[test]
    fn test_space_menu_direct_command_keys_are_noops_in_normal_mode() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let tasks = vec![make_task("task-1", "Selected")];
        let mock_client = MockClickUpClient::new().with_tasks(tasks.clone());
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        app.tasks = tasks;
        app.task_list = GroupedTaskList::from_tasks(app.tasks.clone());
        app.task_list.select_first();
        app.screen = Screen::Tasks;
        app.current_list_id = Some("list-1".to_string());

        // Direct command keys must be no-ops in NORMAL mode
        for key in ['d', 's', 'a', 'u'] {
            app.update(InputEvent::Key(KeyEvent::new(
                KeyCode::Char(key),
                KeyModifiers::NONE,
            )));
        }
        assert!(
            !app.dialog.is_visible(),
            "direct d must not open delete dialog"
        );
        assert!(
            !app.status_picker_open,
            "direct s must not open status picker"
        );

        // SPACE opens the transient menu
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char(' '),
            KeyModifiers::NONE,
        )));
        assert!(app.command_menu.is_some(), "SPACE must open SPACEmenu");

        // SPACE inside INPUT mode must not open the menu
        app.command_menu = None;
        app.comment_focus = true;
        app.comment_new_text.clear();
        app.comment_editing_index = Some(usize::MAX);
        app.insert_mode_target = Some(InsertTarget::Comment);
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char(' '),
            KeyModifiers::NONE,
        )));
        assert!(
            app.command_menu.is_none(),
            "SPACE in INPUT mode must be text, not menu"
        );
    }

    #[test]
    fn test_spc_q_quit_sequence() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let mock_client = MockClickUpClient::new();
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        app.screen = Screen::Workspaces;

        // Plain q in normal mode must not quit
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
        )));
        assert!(
            !app.dialog.is_visible(),
            "plain q must not open quit dialog"
        );

        // SPC q opens the ConfirmQuit dialog and closes the menu
        for key in [' ', 'q'] {
            app.update(InputEvent::Key(KeyEvent::new(
                KeyCode::Char(key),
                KeyModifiers::NONE,
            )));
        }
        assert!(
            app.dialog.is_visible(),
            "SPC q must open the quit confirmation dialog"
        );
        assert!(
            app.command_menu.is_none(),
            "SPACEmenu must close after SPC q"
        );

        // Esc cancels the quit
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));
        assert!(!app.dialog.is_visible(), "Esc must cancel quit");
        assert_ne!(app.state, AppState::Quitting, "quit must be cancelled");
    }

    #[test]
    fn test_space_chords_fire_comment_commands() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let tasks = vec![make_task("task-1", "Detail")];
        let mock_client = MockClickUpClient::new().with_tasks(tasks.clone());
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        app.screen = Screen::TaskDetail;
        app.task_detail.task = Some(tasks[0].clone());
        app.comment_focus = true;

        // SPACE c n starts a new comment in INSERT mode
        for key in [' ', 'c', 'n'] {
            app.update(InputEvent::Key(KeyEvent::new(
                KeyCode::Char(key),
                KeyModifiers::NONE,
            )));
        }
        assert_eq!(
            app.insert_mode_target,
            Some(InsertTarget::Comment),
            "SPC c n must enter comment INSERT mode"
        );
    }

    #[test]
    fn test_status_line_renders_without_hints() {
        use ratatui::{backend::TestBackend, Terminal};
        use crate::tui::layout::TuiLayout;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let layout = TuiLayout::new(frame.area());
                layout.render_status(frame, "Ready");
            })
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        // Bottom row must show the status message only
        let bottom: String = (0..80)
            .map(|x| buffer[(x, 23)].symbol().to_string())
            .collect();
        assert!(bottom.contains("Ready"), "status message rendered: {}", bottom);
        assert!(
            !bottom.contains("Navigate"),
            "no key hints on status row: {}",
            bottom
        );
    }

    #[test]
    fn test_screen_titles_use_current_hierarchy_ids() {
        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.workspaces = serde_json::from_value(serde_json::json!([
            {"id": "w1", "name": "Alpha"},
            {"id": "w2", "name": "Beta"}
        ]))
            .unwrap();
        app.spaces = serde_json::from_value(serde_json::json!([
            {"id": "s1", "name": "Space One"},
            {"id": "s2", "name": "Space Two"}
        ]))
            .unwrap();
        app.folders = serde_json::from_value(serde_json::json!([
            {"id": "f1", "name": "Folder One"},
            {"id": "f2", "name": "Folder Two"}
        ]))
            .unwrap();
        app.lists = serde_json::from_value(serde_json::json!([
            {"id": "l1", "name": "List One"},
            {"id": "l2", "name": "List Two"}
        ]))
            .unwrap();

        app.current_workspace_id = Some("w2".to_string());
        app.screen = Screen::Spaces;
        app.update_screen_title();
        assert!(app.screen_title.contains("Beta"));

        app.current_space_id = Some("s2".to_string());
        app.screen = Screen::Folders;
        app.update_screen_title();
        assert!(app.screen_title.contains("Space Two"));

        app.current_folder_id = Some("f2".to_string());
        app.screen = Screen::Lists;
        app.update_screen_title();
        assert!(app.screen_title.contains("Folder Two"));

        app.current_list_id = Some("l2".to_string());
        app.screen = Screen::Tasks;
        app.update_screen_title();
        assert!(app.screen_title.contains("List Two"));
    }

    #[test]
    fn test_status_picker_uses_statuses_present_in_loaded_tasks() {
        use crate::models::TaskStatus;

        let status = |name: &str, order: u32| TaskStatus {
            id: None,
            status: name.to_string(),
            color: Some("#123456".to_string()),
            type_field: None,
            orderindex: Some(order),
            status_group: None,
        };
        let todo = status("To Do", 0);
        let in_progress = status("In Progress", 1);
        let review = status("Review", 2);

        let mut todo_task = make_task("task-1", "Todo task");
        todo_task.status = Some(todo.clone());
        let mut progress_task = make_task("task-2", "Progress task");
        progress_task.status = Some(in_progress.clone());
        let mut review_task = make_task("task-3", "Review task");
        review_task.status = Some(review.clone());

        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        app.tasks = vec![todo_task.clone(), progress_task, review_task.clone()];

        app.open_status_picker(review_task);

        assert_eq!(app.status_picker_statuses.len(), 3);
        assert_eq!(app.status_picker_cursor, 2);
        assert_eq!(
            app.status_picker_original_status
            .as_ref()
            .map(|status| status.status.as_str()),
            Some("Review")
        );
        assert_eq!(app.status_picker_statuses[2].status, "Review");
        assert_eq!(app.status_picker_statuses[0].status, "To Do");
    }
}
