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
use crate::config;
use crate::models::{
    ClickUpSpace, Comment, CreateCommentRequest, CreateTaskRequest, Document, Folder, List,
    SessionState, Task, UpdateCommentRequest, User, Workspace,
};
use crate::tui::widgets::SidebarItem;
use crate::utils::{ClickUpUrlGenerator, ClipboardService, UrlGenerator};

use super::input::InputEvent;
use super::layout::{generate_screen_title, split_task_detail_focus_comments, split_task_detail_focus_description, TuiLayout};
use super::terminal;
use super::widgets::{
    render_assignee_picker, render_auth, render_command_menu,
    render_comments, render_mention_picker,
    render_dialog, render_document, render_filter_menu, render_inbox, render_sidebar,
    render_status_picker,
    render_task_detail_with_editor, render_task_list, AuthState, DialogState, DialogType, DocumentState,
    GroupedTaskList, ListRow, SidebarState, TaskDetailState,
};

/// Application screens
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Auth,
    /// Workspace inbox: items needing attention, reachable only from the root.
    Inbox,
    /// Workspace-wide "Assigned to Me" view, reachable only from the root.
    Assigned,
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

/// Focus field for task creation form
#[derive(Debug, Clone, PartialEq)]
pub enum TaskCreationField {
    Name,
    Description,
}

/// The single reaction this client exposes: a thumbs-up on a comment.
///
/// ClickUp takes CLDR short names on the wire (`thumbs_up`) while the UI shows
/// the 👍 glyph; keeping both here avoids drift between the value we send and
/// the value comment rows look for.
pub const THUMBS_UP_REACTION: &str = crate::models::comment::THUMBS_UP;

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
}

/// What the status overlay is being used for.
///
/// The overlay serves two unrelated jobs. Overloading `status_picker_task_id`
/// as the mode flag made a filter picker inherit a previous status-change
/// selection and silently mutate a task instead of filtering the list, so the
/// mode is explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusPickerMode {
    /// Pick a new status for one task (`SPC t s`)
    ChangeTask,
    /// Pick a status to filter the visible task list by (`SPC t f` -> `s`)
    FilterList,
}

/// Command menu state: which command group is shown.
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

/// Which element on the task screen currently holds TAB focus.
///
/// Focus traversal (TAB): Description → each comment in view order →
/// Description… The focused element is enlarged so more of it is visible,
/// and its content can be scrolled with Up/Down, PageUp/PageDown,
/// Ctrl+F/Ctrl+U when it overflows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskScreenFocus {
    Description,
    Comment(usize),
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
    /// A page of workspace-wide assigned tasks, plus the page it came from
    AssignedTasksLoaded(Result<Vec<Task>, String>, u32),
    /// Inbox contents for the workspace
    InboxLoaded(Result<crate::models::Inbox, String>),
    CommentsLoaded(Result<Vec<Comment>, String>),
    /// A `SPC c u` reaction call finished: (comment_id, reaction, added)
    CommentReactionUpdated(Result<(), String>, String, String, bool),
    /// A task re-fetched by `SPC r` while its detail view is open
    TaskRefreshed(Result<Task, String>),
    /// A document re-fetched by `SPC r`
    DocumentRefreshed(Result<Document, String>),
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
    /// Cursor position (char index) inside `comment_new_text`
    comment_cursor: usize,
    comment_focus: bool, // true = focus on comments, false = focus on task form
    /// Manual scroll offset of the comments pane while a comment is focused
    comment_pane_scroll: usize,

    /// Vim-style modal input state (issue 7):
    /// While Some(target), all printable keys are inserted into the target
    /// buffer instead of being interpreted as commands.
    insert_mode_target: Option<InsertTarget>,
    /// Command menu state: None = closed
    command_menu: Option<CommandMenuLevel>,

    /// Full-screen expansion state (issue 5): which panel is expanded
    expanded_panel: Option<ExpandedPanel>,
    /// Edit buffer for the task description (insert mode, issue 7)
    description_edit_buffer: String,
    /// Cursor position (char index) inside `description_edit_buffer`
    description_edit_cursor: usize,
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
    /// Char index of the @ that opened the picker, so a selection replaces
    /// exactly that character instead of guessing with rfind.
    mention_at: Option<usize>,
    /// People that `@Name` in the current buffers may resolve to.
    mention_registry: Vec<crate::tui::mention::MentionEntry>,

    /// Comment selected before a refresh, so a reload can reselect it
    pending_comment_selection: Option<String>,

    /// Task selected before a task-list refresh, so the reload can reselect it
    pending_task_selection: Option<String>,

    /// True while a `SPC r` on task detail is still reloading its comments,
    /// so the follow-up comment load reports the refresh instead of a plain
    /// "Loaded N comments".
    refresh_reloading_comments: bool,

    /// Comment thread navigation state
    comment_view_mode: CommentViewMode,
    comment_previous_selection: Option<usize>, // Store selection when entering thread

    /// Task creation form state
    task_name_input: String,
    task_name_cursor: usize,
    task_description_input: String,
    task_description_cursor: usize,
    task_creating: bool,
    task_creation_focus: TaskCreationField,

    /// Per-list assigned filter state
    assigned_filter_active: bool,

    /// Workspace id used for the root "Assigned to Me" query
    current_team_id: Option<String>,
    /// Page currently being fetched while paging through assigned tasks
    assigned_page_in_flight: Option<u32>,
    /// True while the Assigned view entry itself is being resolved
    assigned_loading: bool,
    /// Screen a task detail view was opened from, so Esc returns there
    task_origin_screen: Screen,

    /// Inbox entries currently listed (dismissed ones filtered out)
    inbox_tasks: Vec<crate::models::InboxTask>,
    /// Inbox ids the user has already seen, tracked locally
    inbox_read_ids: std::collections::HashSet<String>,
    /// Inbox ids the user dismissed, tracked locally
    inbox_dismissed_ids: std::collections::HashSet<String>,
    /// Selection index within the rendered inbox rows
    inbox_selected: usize,
    /// True while an inbox fetch is in flight
    inbox_loading: bool,

    /// User identity for assignee filtering
    current_user_id: Option<i32>,

    /// In-memory cache for list members (keyed by list ID)
    cached_list_members: std::collections::HashMap<String, Vec<User>>,
    /// List the currently loaded `mention_members` came from, so switching to
    /// a task in another list refetches instead of offering the wrong people.
    mention_members_list_id: Option<String>,
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

    /// Active task-list status filters. Empty shows every status. More than
    /// one entry means "any of these", which ClickUp expresses as a repeated
    /// `statuses[]` parameter.
    status_filters: Vec<String>,

    /// Every status this session has observed.
    ///
    /// Needed because the list shown is the *filtered* result: applying a
    /// status filter replaces `tasks` with rows of that one status, so
    /// deriving the menu from loaded tasks collapsed it to a single entry and
    /// made an active filter impossible to change. This registry only grows,
    /// so it survives filtering.
    ///
    /// Full `TaskStatus` values are kept, not just names, so the picker can
    /// still order options by the workspace's configured `orderindex` instead
    /// of the order they happened to be seen in.
    known_statuses: Vec<crate::models::TaskStatus>,

    /// Statuses ticked in the filter picker (multi-select)
    status_picker_selected: Vec<String>,

    /// Filter menu (`SPC t f`) open state and cursor
    filter_menu_open: bool,
    filter_menu_cursor: usize,

    /// Which filters the current screen can actually apply. The Assigned view
    /// has no list context, so a status filter is unavailable there.
    filter_can_status: bool,

    /// Why the status overlay is open (task change vs. list filter)
    status_picker_mode: StatusPickerMode,

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

    /// Which screen the saved session should end on, when that screen is a
    /// workspace-wide query (`Assigned`). The hierarchy replay cannot reach
    /// such a screen because it has no list to navigate to, so it is applied
    /// separately once the user id the query needs has loaded.
    restored_screen: Option<Screen>,
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


impl TuiApp {
    pub fn new() -> Result<Self> {
        let auth = AuthManager::new().unwrap_or_default();
        let cache = CacheManager::new(config::database_path()?)?;

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
            comment_cursor: 0,
            comment_focus: false,
            comment_pane_scroll: 0,
            comment_view_mode: CommentViewMode::TopLevel,
            comment_previous_selection: None,
            pending_comment_selection: None,
            pending_task_selection: None,
            refresh_reloading_comments: false,
            task_name_input: String::new(),
            task_name_cursor: 0,
            task_description_input: String::new(),
            task_description_cursor: 0,
            task_creating: false,
            insert_mode_target: None,
            command_menu: None,
            expanded_panel: None,
            description_edit_buffer: String::new(),
            description_edit_cursor: 0,
            description_scroll_offset: 0,
            mention_members: Vec::new(),
            mention_cursor: 0,
            mention_picker_open: false,
            mention_prev_target: None,
            mention_query: String::new(),
            mention_at: None,
            mention_registry: Vec::new(),
            task_creation_focus: TaskCreationField::Name,
            assigned_filter_active: false,
            current_team_id: None,
            assigned_page_in_flight: None,
            assigned_loading: false,
            task_origin_screen: Screen::Tasks,
            inbox_tasks: Vec::new(),
            inbox_read_ids: std::collections::HashSet::new(),
            inbox_dismissed_ids: std::collections::HashSet::new(),
            inbox_selected: 0,
            inbox_loading: false,
            current_user_id: None,
            cached_list_members: std::collections::HashMap::new(),
            mention_members_list_id: None,
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
            status_filters: Vec::new(),
            known_statuses: Vec::new(),
            status_picker_selected: Vec::new(),
            filter_menu_open: false,
            filter_menu_cursor: 0,
            filter_can_status: false,
            status_picker_mode: StatusPickerMode::ChangeTask,
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
            restored_screen: None,
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
    pub fn with_client(client: Arc<dyn ClickUpApi>) -> Result<Self> {
        // Delegate so each app gets its own temporary database file.
        //
        // This used to open `config::database_path()`, i.e. the user's
        // real cache.db. cargo runs test binaries in parallel, so dozens of
        // tests opened one shared SQLite file and intermittently failed with
        // "database is locked". Tests must never touch the real user cache.
        Self::with_client_and_test_cache(client)
    }

    /// Create a new TUI app with a custom client and in-memory cache (for testing)
    pub fn with_client_and_test_cache(client: Arc<dyn ClickUpApi>) -> Result<Self> {
        use std::env;

        // Use a unique temporary database file for tests to avoid FK conflicts.
        // The counter keeps concurrently running tests in one binary from
        // sharing a file: cached state (inbox read/dismiss markers) leaking
        // across tests makes them order-dependent.
        use std::sync::atomic::{AtomicU32, Ordering};
        static TEST_DB_SEQ: AtomicU32 = AtomicU32::new(0);
        let temp_dir = env::temp_dir();
        let db_path = temp_dir.join(format!(
            "clickdown_test_{}_{}.db",
            std::process::id(),
            TEST_DB_SEQ.fetch_add(1, Ordering::Relaxed)
        ));

        // Remove existing file if present
        let _ = std::fs::remove_file(&db_path);

        // Sweep databases left behind by earlier runs. Each app instance gets
        // its own file, so a full suite creates hundreds per process and they
        // would sit in the temp dir forever. The pid is part of the name, so
        // anything whose owning process is gone is safe to delete.
        use std::sync::Once;
        static SWEEP: Once = Once::new();
        SWEEP.call_once(|| {
            let Ok(entries) = std::fs::read_dir(env::temp_dir()) else {
                return;
            };
            for entry in entries.flatten() {
                let name = entry.file_name();
                let Some(name) = name.to_str() else { continue };
                let Some(rest) = name.strip_prefix("clickdown_test_") else {
                    continue;
                };
                let Some((pid, _)) = rest.split_once('_') else {
                    continue;
                };
                let Ok(pid) = pid.parse::<u32>() else { continue };
                if pid == std::process::id() {
                    continue; // ours: still in use
                }
                let owner_alive = if std::path::Path::new("/proc").is_dir() {
                    // Linux: the pid is authoritative.
                    std::path::Path::new(&format!("/proc/{pid}")).exists()
                } else {
                    // Elsewhere fall back to age: only reclaim clearly old files.
                    entry
                        .metadata()
                        .ok()
                        .and_then(|meta| meta.modified().ok())
                        .and_then(|modified| modified.elapsed().ok())
                        .map(|age| age < std::time::Duration::from_secs(3600))
                        .unwrap_or(false)
                };
                if owner_alive {
                    continue;
                }
                let _ = std::fs::remove_file(entry.path());
            }
        });

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
            comment_cursor: 0,
            comment_focus: false,
            comment_pane_scroll: 0,
            comment_view_mode: CommentViewMode::TopLevel,
            comment_previous_selection: None,
            pending_comment_selection: None,
            pending_task_selection: None,
            refresh_reloading_comments: false,
            task_name_input: String::new(),
            task_name_cursor: 0,
            task_description_input: String::new(),
            task_description_cursor: 0,
            task_creating: false,
            insert_mode_target: None,
            command_menu: None,
            expanded_panel: None,
            description_edit_buffer: String::new(),
            description_edit_cursor: 0,
            description_scroll_offset: 0,
            mention_members: Vec::new(),
            mention_cursor: 0,
            mention_picker_open: false,
            mention_prev_target: None,
            mention_query: String::new(),
            mention_at: None,
            mention_registry: Vec::new(),
            task_creation_focus: TaskCreationField::Name,
            assigned_filter_active: false,
            current_team_id: None,
            assigned_page_in_flight: None,
            assigned_loading: false,
            task_origin_screen: Screen::Tasks,
            inbox_tasks: Vec::new(),
            inbox_read_ids: std::collections::HashSet::new(),
            inbox_dismissed_ids: std::collections::HashSet::new(),
            inbox_selected: 0,
            inbox_loading: false,
            current_user_id: None,
            cached_list_members: std::collections::HashMap::new(),
            mention_members_list_id: None,
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
            status_filters: Vec::new(),
            known_statuses: Vec::new(),
            status_picker_selected: Vec::new(),
            filter_menu_open: false,
            filter_menu_cursor: 0,
            filter_can_status: false,
            status_picker_mode: StatusPickerMode::ChangeTask,
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
            restored_screen: None,
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
}

mod update;
mod input;
mod navigate;
mod inbox;
mod assigned;
mod auth;
mod filters;
mod refresh;
mod render;
mod url;
mod reactions;
mod session;
mod input_engine;
mod test_support;

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

        let state = SessionState {
            screen: "Tasks".to_string(),
            workspace_id: Some("ws-123".to_string()),
            user_id: Some(456),
            ..Default::default()
        };

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
        app.task_list = GroupedTaskList::from_tasks_with_folds(app.tasks.clone(), std::collections::HashSet::new());

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

        // Direct 'n' key must no longer trigger anything (menu-only commands)
        let n_key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
        app.update(InputEvent::Key(n_key));
        assert_eq!(app.screen, Screen::Tasks, "direct n must not open creation");
        assert!(!app.task_creating, "direct n must not set task_creating");

        // Command menu: SPACE, t, n
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
        app.task_list = GroupedTaskList::from_tasks_with_folds(app.tasks.clone(), std::collections::HashSet::new());
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
        app.task_list = GroupedTaskList::from_tasks_with_folds(app.tasks.clone(), std::collections::HashSet::new());
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
    fn test_question_mark_is_a_noop_everywhere() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        let question = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE);

        // No help overlay exists any more: '?' must not open one, and the
        // text-input contexts must still receive it as text.
        app.url_input_open = true;
        app.update(InputEvent::Key(question));
        assert_eq!(app.url_input_text(), "?");

        app.url_input_open = false;
        app.screen = Screen::TaskDetail;
        app.task_detail.task = Some(make_task("task-1", "Task"));
        app.comments = vec![make_comment("comment-1", "Body", None)];
        app.comment_editing_index = Some(0);
        app.update(InputEvent::Key(question));
        assert_eq!(app.comment_new_text(), "?");

        app.comment_editing_index = None;
        app.comment_new_text.clear();
        app.screen = Screen::Auth;
        app.update(InputEvent::Key(question));
        assert_eq!(app.auth_state.token_input, "?");
    }

    #[test]
    fn test_spc_h_is_not_a_command_any_more() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        app.screen = Screen::Workspaces;

        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char(' '),
            KeyModifiers::NONE,
        )));
        assert!(
            app.command_menu.is_some(),
            "SPACE must open the command menu"
        );

        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('h'),
            KeyModifiers::NONE,
        )));
        assert!(
            app.command_menu.is_none(),
            "command menu must close on an unknown group key"
        );
        assert_eq!(app.status(), "No command group 'h'");
        assert_eq!(app.screen, Screen::Workspaces, "no overlay opened");
    }

    #[test]
    fn test_esc_from_tasks_restores_previous_list_selection() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mk_list = |id: &str, name: &str| List {
            id: id.to_string(),
            name: name.to_string(),
            content: None,
            description: None,
            archived: false,
            hidden: false,
            orderindex: None,
            space: None,
            folder: None,
            status: None,
            priority: None,
            statuses: Vec::new(),
        };

        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Simulate state after navigating into "list-2": the sidebar shows the
        // folder's lists with list-2 selected, and the Tasks screen is active.
        app.lists = vec![
            mk_list("list-1", "Alpha"),
            mk_list("list-2", "Beta"),
            mk_list("list-3", "Gamma"),
        ];
        *app.sidebar.items_mut() = app
            .lists
            .iter()
            .map(|l| SidebarItem::List {
                name: l.name.clone(),
                id: l.id.clone(),
            })
            .collect();
        app.sidebar.select(Some(1));
        app.current_folder_id = Some("folder-1".to_string());
        app.current_list_id = Some("list-2".to_string());
        app.screen = Screen::Tasks;

        // Esc goes back to the Lists screen
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));

        assert_eq!(app.screen, Screen::Lists, "Esc must return to Lists screen");
        let selected = app
            .sidebar
            .selected_item()
            .map(|item| item.id().to_string());
        assert_eq!(
            selected.as_deref(),
            Some("list-2"),
            "Esc from Tasks must restore focus to the previously opened list"
        );
    }

    #[test]
    fn test_esc_from_tasks_falls_back_to_first_when_list_missing() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mk_list = |id: &str, name: &str| List {
            id: id.to_string(),
            name: name.to_string(),
            content: None,
            description: None,
            archived: false,
            hidden: false,
            orderindex: None,
            space: None,
            folder: None,
            status: None,
            priority: None,
            statuses: Vec::new(),
        };

        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        app.lists = vec![mk_list("list-1", "Alpha"), mk_list("list-2", "Beta")];
        *app.sidebar.items_mut() = app
            .lists
            .iter()
            .map(|l| SidebarItem::List {
                name: l.name.clone(),
                id: l.id.clone(),
            })
            .collect();
        app.sidebar.select(Some(1));
        app.current_folder_id = Some("folder-1".to_string());
        // The previously opened list no longer exists in the folder
        app.current_list_id = Some("list-gone".to_string());
        app.screen = Screen::Tasks;

        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));

        assert_eq!(app.screen, Screen::Lists, "Esc must return to Lists screen");
        let selected = app
            .sidebar
            .selected_item()
            .map(|item| item.id().to_string());
        assert_eq!(
            selected.as_deref(),
            Some("list-1"),
            "missing list must fall back to the first entry"
        );
        assert_eq!(
            app.status(),
            "Saved list not found, showing lists",
            "fallback must surface a status message"
        );
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
        app.task_list = GroupedTaskList::from_tasks_with_folds(app.tasks.clone(), std::collections::HashSet::new());
        app.task_list.select_first();
        app.screen = Screen::Tasks;
        app.current_list_id = Some("list-1".to_string());

        // Direct 'e' key must no longer trigger anything (menu-only commands)
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('e'),
            KeyModifiers::NONE,
        )));
        assert_eq!(app.screen, Screen::Tasks, "direct e must not open editor");

        // Command menu: SPACE, t, e
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
        app.task_list = GroupedTaskList::from_tasks_with_folds(app.tasks.clone(), std::collections::HashSet::new());
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
    fn test_command_menu_direct_command_keys_are_noops_in_normal_mode() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let tasks = vec![make_task("task-1", "Selected")];
        let mock_client = MockClickUpClient::new().with_tasks(tasks.clone());
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        app.tasks = tasks;
        app.task_list = GroupedTaskList::from_tasks_with_folds(app.tasks.clone(), std::collections::HashSet::new());
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
        assert!(
            app.command_menu.is_some(),
            "SPACE must open the command menu"
        );

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
        assert_ne!(app.state, AppState::Quitting, "plain q must not quit");

        // SPC q quits immediately: no confirmation dialog, menu closes
        for key in [' ', 'q'] {
            app.update(InputEvent::Key(KeyEvent::new(
                KeyCode::Char(key),
                KeyModifiers::NONE,
            )));
        }
        assert!(
            !app.dialog.is_visible(),
            "SPC q must not show any confirmation dialog"
        );
        assert_eq!(
            app.state,
            AppState::Quitting,
            "SPC q must request an immediate quit"
        );
        assert!(
            app.command_menu.is_none(),
            "command menu must close after SPC q"
        );
    }

    #[test]
    fn test_window_group_has_no_quit_action() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mock_client = MockClickUpClient::new().with_tasks(vec![]);
        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();
        app.screen = Screen::Workspaces;

        for key in [' ', 'w', 'q'] {
            app.update(InputEvent::Key(KeyEvent::new(
                KeyCode::Char(key),
                KeyModifiers::NONE,
            )));
        }
        assert_ne!(app.state, AppState::Quitting, "w q must not quit");
        assert!(!app.dialog.is_visible(), "w q must not open a dialog");
        assert_eq!(app.status(), "No command 'wq'");
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
        // Bottom row carries the status message plus the version indicator
        let bottom: String = (0..80)
            .map(|x| buffer[(x, 23)].symbol().to_string())
            .collect();
        assert!(bottom.contains("Ready"), "status message rendered: {}", bottom);
        assert!(
            !bottom.contains("Navigate"),
            "no key hints on status row: {}",
            bottom
        );
        assert!(
            bottom
                .trim_end()
                .ends_with(concat!("v", env!("CARGO_PKG_VERSION"))),
            "version indicator sits at the right edge: {}",
            bottom
        );
    }

    /// The version indicator is pinned to the last column of the status row and
    /// states the crate version, so a release cannot ship an unlabelled binary.
    #[test]
    fn test_status_row_shows_version_in_bottom_right_corner() {
        use ratatui::{backend::TestBackend, Terminal};

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let layout = crate::tui::layout::TuiLayout::new(frame.area());
                layout.render_status(frame, "");
            })
            .unwrap();
        let buffer = terminal.backend().buffer().clone();

        let expected = format!("v{}", env!("CARGO_PKG_VERSION"));
        let bottom: String = (0..80)
            .map(|x| buffer[(x, 23)].symbol().to_string())
            .collect();

        assert!(
            bottom.ends_with(&expected),
            "bottom row must end with the version: {bottom:?}"
        );
        // The label is derived from `CARGO_PKG_VERSION`, i.e. Cargo.toml, so it
        // tracks a version bump instead of needing a matching edit here.
        assert!(expected.starts_with('v'), "label is prefixed: {expected:?}");

        // Nothing is painted right of the label, i.e. it is flush right.
        let label_start = 80 - expected.chars().count() as u16;
        let painted: String = (label_start..80)
            .map(|x| buffer[(x, 23)].symbol().to_string())
            .collect();
        assert_eq!(painted, expected, "version occupies the last columns");
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
