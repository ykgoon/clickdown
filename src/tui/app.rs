//! Main TUI application

use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use arboard::Clipboard;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{Frame, prelude::Rect};
use tokio::sync::mpsc;

use crate::api::{ClickUpApi, AuthManager, ClickUpClient};
use crate::config::ConfigManager;
use crate::models::{Workspace, ClickUpSpace, Folder, List, Task, Document};

use super::terminal;
use super::layout::{TuiLayout, generate_screen_title};
use super::input::{InputEvent, is_quit, is_enter, is_escape};
use super::widgets::{
    SidebarState, SidebarItem, render_sidebar, get_sidebar_hints,
    TaskListState, render_task_list, get_task_list_hints,
    TaskDetailState, render_task_detail, get_task_detail_hints,
    AuthState, render_auth, get_auth_hints,
    DocumentState, render_document, get_document_hints,
    DialogState, DialogType, render_dialog, get_dialog_hints,
    HelpState, render_help,
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

/// Async messages for API results
#[derive(Debug, Clone)]
pub enum AppMessage {
    WorkspacesLoaded(Result<Vec<Workspace>, String>),
    SpacesLoaded(Result<Vec<ClickUpSpace>, String>),
    FoldersLoaded(Result<Vec<Folder>, String>),
    ListsLoaded(Result<Vec<List>, String>),
    TasksLoaded(Result<Vec<Task>, String>),
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

    /// Async message receiver
    message_rx: Option<mpsc::Receiver<AppMessage>>,

    /// Async message sender
    message_tx: Option<mpsc::Sender<AppMessage>>,
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
            message_rx: Some(message_rx),
            message_tx: Some(message_tx.clone()),
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
                                // Update task_list.tasks for rendering (not self.tasks)
                                self.task_list.tasks = tasks.clone();
                                self.tasks = tasks;
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
                        self.task_detail.task = Some(task);
                        self.screen = Screen::TaskDetail;
                        self.update_screen_title();
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
            match key.code {
                KeyCode::Esc => {
                    self.task_detail.editing = false;
                    self.screen = Screen::Tasks;
                    self.update_screen_title();
                }
                KeyCode::Char('e') => {
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
                    self.load_spaces(id.clone());
                    self.screen = Screen::Spaces;
                    self.screen_title = generate_screen_title(&name);
                }
            }
            Screen::Spaces => {
                if let Some(SidebarItem::Space { id, name, .. }) = selected_item {
                    self.load_folders(id.clone());
                    self.screen = Screen::Folders;
                    self.screen_title = generate_screen_title(&name);
                }
            }
            Screen::Folders => {
                if let Some(SidebarItem::Folder { id, name, .. }) = selected_item {
                    self.load_lists(id.clone());
                    self.screen = Screen::Lists;
                    self.screen_title = generate_screen_title(&name);
                }
            }
            Screen::Lists => {
                if let Some(SidebarItem::List { id, name, .. }) = selected_item {
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
                self.screen = Screen::Workspaces;
                self.screen_title = generate_screen_title("Workspaces");
            }
            Screen::Folders => {
                self.screen = Screen::Spaces;
                if let Some(space) = self.spaces.first() {
                    self.screen_title = generate_screen_title(&space.name);
                }
            }
            Screen::Lists => {
                self.screen = Screen::Folders;
                if let Some(folder) = self.folders.first() {
                    self.screen_title = generate_screen_title(&folder.name);
                }
            }
            Screen::Tasks => {
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
            let status = if self.loading {
                "Loading...".to_string()
            } else if let Some(ref error) = self.error {
                error.clone()
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
            Screen::TaskDetail => render_task_detail(frame, &self.task_detail, content_area),
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
            Screen::TaskDetail => render_task_detail(frame, &self.task_detail, area),
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
                Screen::TaskDetail => get_task_detail_hints(),
                Screen::Document => get_document_hints(),
                _ => get_sidebar_hints(),
            }
        }
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new().expect("Failed to create TUI app")
    }
}
