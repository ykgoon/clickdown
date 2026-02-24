//! Integration tests for ClickDown TUI application
//!
//! These tests verify the TUI application functionality.

use clickdown::tui::app::TuiApp;

/// Test that the TUI app initializes correctly
#[test]
fn test_tui_app_initialization() {
    let app = TuiApp::new();
    
    // App should initialize successfully
    assert!(app.is_ok(), "TUI App failed to initialize: {:?}", app.err());
}

/// Test that screen titles are generated correctly
#[test]
fn test_screen_title_generation() {
    use clickdown::tui::layout::generate_screen_title;
    
    let title = generate_screen_title("Authentication");
    assert_eq!(title, "ClickDown - Authentication");
    
    let title = generate_screen_title("Workspaces");
    assert_eq!(title, "ClickDown - Workspaces");
    
    let title = generate_screen_title("Tasks: My List");
    assert_eq!(title, "ClickDown - Tasks: My List");
}

/// Test that screen titles are unique for different screens
#[test]
fn test_screen_titles_are_unique() {
    use clickdown::tui::layout::generate_screen_title;
    
    let auth_title = generate_screen_title("Authentication");
    let workspace_title = generate_screen_title("Workspaces");
    let tasks_title = generate_screen_title("Tasks");
    
    assert_ne!(auth_title, workspace_title, "Auth and Workspace titles should be different");
    assert_ne!(auth_title, tasks_title, "Auth and Tasks titles should be different");
    assert_ne!(workspace_title, tasks_title, "Workspace and Tasks titles should be different");
}

/// Test that minimum terminal size constants are defined
#[test]
fn test_minimum_terminal_size() {
    use clickdown::tui::layout::{MIN_WIDTH, MIN_HEIGHT};
    
    assert_eq!(MIN_WIDTH, 80, "Minimum width should be 80");
    assert_eq!(MIN_HEIGHT, 24, "Minimum height should be 24");
}

/// Test that layout is created correctly
#[test]
fn test_layout_creation() {
    use clickdown::tui::layout::TuiLayout;
    use ratatui::prelude::Rect;
    
    let area = Rect::new(0, 0, 100, 30);
    let layout = TuiLayout::new(area);
    
    assert!(!layout.too_small, "Layout should not be too small for 100x30 terminal");
    assert!(layout.title_area.height == 1, "Title area should be 1 row");
    assert!(layout.status_area.height == 3, "Status area should be 3 rows");
}

/// Test that layout detects small terminals
#[test]
fn test_layout_small_terminal() {
    use clickdown::tui::layout::TuiLayout;
    use ratatui::prelude::Rect;
    
    let area = Rect::new(0, 0, 60, 20);
    let layout = TuiLayout::new(area);
    
    assert!(layout.too_small, "Layout should be too small for 60x20 terminal");
}

/// Test input event types
#[test]
fn test_input_event_types() {
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyEvent, KeyCode};
    
    let key_event = KeyEvent::new(KeyCode::Char('a'), crossterm::event::KeyModifiers::NONE);
    let input_event = InputEvent::Key(key_event);
    
    match input_event {
        InputEvent::Key(_) => (),
        _ => panic!("Expected Key event"),
    }
}

/// Test quit detection
#[test]
fn test_quit_detection() {
    use clickdown::tui::input::is_quit;
    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
    
    // Ctrl+Q should be detected as quit
    let ctrl_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
    assert!(is_quit(ctrl_q), "Ctrl+Q should be detected as quit");
    
    // Capital Q should be detected as quit
    let capital_q = KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::NONE);
    assert!(is_quit(capital_q), "Q should be detected as quit");
    
    // Other keys should not be detected as quit
    let other = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
    assert!(!is_quit(other), "Other keys should not be detected as quit");
}

/// Test escape detection
#[test]
fn test_escape_detection() {
    use clickdown::tui::input::is_escape;
    use crossterm::event::{KeyEvent, KeyCode};
    
    let esc = KeyEvent::new(KeyCode::Esc, crossterm::event::KeyModifiers::NONE);
    assert!(is_escape(esc), "Esc should be detected");
    
    let other = KeyEvent::new(KeyCode::Char('a'), crossterm::event::KeyModifiers::NONE);
    assert!(!is_escape(other), "Other keys should not be detected as escape");
}

/// Test enter detection
#[test]
fn test_enter_detection() {
    use clickdown::tui::input::is_enter;
    use crossterm::event::{KeyEvent, KeyCode};
    
    let enter = KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE);
    assert!(is_enter(enter), "Enter should be detected");
    
    let other = KeyEvent::new(KeyCode::Char('a'), crossterm::event::KeyModifiers::NONE);
    assert!(!is_enter(other), "Other keys should not be detected as enter");
}

/// Test dialog state
#[test]
fn test_dialog_state() {
    use clickdown::tui::widgets::{DialogState, DialogType};
    
    let mut dialog = DialogState::new();
    
    // Dialog should start hidden
    assert!(!dialog.is_visible(), "Dialog should start hidden");
    
    // Show dialog
    dialog.show(DialogType::ConfirmQuit);
    assert!(dialog.is_visible(), "Dialog should be visible after show");
    
    // Toggle selection
    assert!(!dialog.confirmed(), "Dialog should start with No selected");
    dialog.toggle();
    assert!(dialog.confirmed(), "Dialog should have Yes selected after toggle");
    
    // Hide dialog
    dialog.hide();
    assert!(!dialog.is_visible(), "Dialog should be hidden after hide");
}

/// Test help state
#[test]
fn test_help_state() {
    use clickdown::tui::widgets::HelpState;
    
    let mut help = HelpState::new();
    
    // Help should start hidden
    assert!(!help.visible, "Help should start hidden");
    
    // Toggle help
    help.toggle();
    assert!(help.visible, "Help should be visible after toggle");
    
    help.toggle();
    assert!(!help.visible, "Help should be hidden after second toggle");
}

/// Test auth state
#[test]
fn test_auth_state() {
    use clickdown::tui::widgets::AuthState;
    
    let mut auth = AuthState::new();
    
    // Auth should start empty
    assert!(auth.token_input.is_empty(), "Token input should start empty");
    assert_eq!(auth.cursor_pos, 0, "Cursor should start at 0");
    
    // Add characters
    auth.add_char('t');
    auth.add_char('e');
    auth.add_char('s');
    auth.add_char('t');
    
    assert_eq!(auth.token_input, "test", "Token input should be 'test'");
    assert_eq!(auth.cursor_pos, 4, "Cursor should be at 4");
    
    // Remove character
    auth.remove_char();
    assert_eq!(auth.token_input, "tes", "Token input should be 'tes'");
    assert_eq!(auth.cursor_pos, 3, "Cursor should be at 3");
    
    // Clear
    auth.clear();
    assert!(auth.token_input.is_empty(), "Token input should be empty after clear");
    assert_eq!(auth.cursor_pos, 0, "Cursor should be at 0 after clear");
}

/// Test sidebar state
#[test]
fn test_sidebar_state() {
    use clickdown::tui::widgets::{SidebarState, SidebarItem};
    
    let mut sidebar = SidebarState::new();
    
    // Sidebar should start empty
    assert!(sidebar.items.is_empty(), "Sidebar should start empty");
    
    // Add items
    sidebar.items.push(SidebarItem::Workspace {
        name: "Test Workspace".to_string(),
        id: "ws-1".to_string(),
    });
    sidebar.items.push(SidebarItem::Space {
        name: "Test Space".to_string(),
        id: "sp-1".to_string(),
        indent: 1,
    });
    
    // Select first
    sidebar.select_first();
    assert_eq!(sidebar.selected.selected(), Some(0), "First item should be selected");
    
    // Select next
    sidebar.select_next();
    assert_eq!(sidebar.selected.selected(), Some(1), "Second item should be selected");
    
    // Select next (wrap around)
    sidebar.select_next();
    assert_eq!(sidebar.selected.selected(), Some(0), "Should wrap to first item");
    
    // Select previous
    sidebar.select_previous();
    assert_eq!(sidebar.selected.selected(), Some(1), "Should go back to second item");
}

/// Test task list state
#[test]
fn test_task_list_state() {
    use clickdown::tui::widgets::TaskListState;
    use clickdown::models::{Task, TaskStatus};
    
    let mut task_list = TaskListState::new();
    
    // Task list should start empty
    assert!(task_list.tasks.is_empty(), "Task list should start empty");
    
    // Add tasks
    task_list.tasks.push(Task {
        id: "task-1".to_string(),
        name: "Task 1".to_string(),
        description: None,
        status: Some(TaskStatus {
            status: "todo".to_string(),
            color: None,
            type_field: None,
            orderindex: None,
        }),
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
    });
    
    // Select first
    task_list.select_first();
    assert_eq!(task_list.selected.selected(), Some(0), "First task should be selected");
    
    // Get selected task
    let selected = task_list.selected_task();
    assert!(selected.is_some(), "Should have selected task");
    assert_eq!(selected.unwrap().name, "Task 1", "Selected task should be Task 1");
}
