//! Integration tests for ClickDown TUI application
//!
//! These tests verify the TUI application functionality.

mod fixtures;

use clickdown::tui::app::TuiApp;

/// Test that the TUI app initializes correctly
#[test]
fn test_tui_app_initialization() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let app = rt.block_on(async {
        TuiApp::new()
    });

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
        custom_id: None,
        custom_item_id: None,
        name: "Task 1".to_string(),
        text_content: None,
        description: None,
        markdown_description: None,
        status: Some(TaskStatus {
            id: None,
            status: "todo".to_string(),
            color: None,
            type_field: None,
            orderindex: None,
            status_group: None,
        }),
        orderindex: None,
        content: None,
        created_at: None,
        updated_at: None,
        closed_at: None,
        done_at: None,
        archived: None,
        creator: None,
        assignees: vec![],
        group_assignees: vec![],
        watchers: vec![],
        checklists: vec![],
        tags: vec![],
        parent: None,
        top_level_parent: None,
        priority: None,
        due_date: None,
        start_date: None,
        points: None,
        custom_fields: vec![],
        attachments: vec![],
        dependencies: vec![],
        linked_tasks: vec![],
        locations: vec![],
        list: None,
        folder: None,
        space: None,
        project: None,
        url: None,
        team_id: None,
        sharing: None,
        permission_level: None,
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

/// Test that Ctrl+Shift+V does NOT trigger quit
#[test]
fn test_ctrl_shift_v_does_not_quit() {
    use clickdown::tui::input::is_quit;
    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};

    // Ctrl+Shift+V should NOT be detected as quit
    let ctrl_shift_v = KeyEvent::new(
        KeyCode::Char('v'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT
    );
    assert!(!is_quit(ctrl_shift_v), "Ctrl+Shift+V should NOT trigger quit");

    // Ctrl+Shift+Q should NOT be detected as quit (only exact Ctrl+Q)
    let ctrl_shift_q = KeyEvent::new(
        KeyCode::Char('q'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT
    );
    assert!(!is_quit(ctrl_shift_q), "Ctrl+Shift+Q should NOT trigger quit");
}

/// Test that exact Ctrl+Q triggers quit
#[test]
fn test_exact_ctrl_q_triggers_quit() {
    use clickdown::tui::input::is_quit;
    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};

    // Exact Ctrl+Q (no Shift) should trigger quit
    let ctrl_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
    assert!(is_quit(ctrl_q), "Exact Ctrl+Q should trigger quit");
}

/// Test partial token masking display
#[test]
fn test_partial_token_masking() {
    use clickdown::tui::widgets::AuthState;

    // Test empty token
    let mut auth = AuthState::new();
    assert!(auth.token_input.is_empty(), "Token should start empty");

    // Test short token (< 4 chars) - all visible
    auth.add_char('a');
    auth.add_char('b');
    auth.add_char('c');
    
    // Build display string (mimicking render_auth logic)
    let visible_chars = 4;
    let mut display = String::new();
    for (i, c) in auth.token_input.chars().enumerate() {
        if i < visible_chars {
            display.push(c);
        } else {
            display.push('•');
        }
    }
    assert_eq!(display, "abc", "Short token should show all chars unmasked");

    // Test exactly 4 chars - all visible
    auth.add_char('d');
    display.clear();
    for (i, c) in auth.token_input.chars().enumerate() {
        if i < visible_chars {
            display.push(c);
        } else {
            display.push('•');
        }
    }
    assert_eq!(display, "abcd", "4-char token should show all chars unmasked");

    // Test long token (> 4 chars) - first 4 visible, rest masked
    auth.add_char('e');
    auth.add_char('f');
    auth.add_char('g');
    display.clear();
    for (i, c) in auth.token_input.chars().enumerate() {
        if i < visible_chars {
            display.push(c);
        } else {
            display.push('•');
        }
    }
    assert_eq!(display, "abcd•••", "Long token should mask chars after 4th");
}

/// Test cursor position indicator
#[test]
fn test_cursor_indicator() {
    use clickdown::tui::widgets::AuthState;

    let mut auth = AuthState::new();
    auth.add_char('t');
    auth.add_char('e');
    auth.add_char('s');
    auth.add_char('t');
    auth.add_char('1');
    auth.add_char('2');

    // Cursor at position 0 (beginning)
    auth.cursor_pos = 0;
    let visible_chars = 4;
    let token_chars: Vec<char> = auth.token_input.chars().collect();
    let mut display = String::new();
    for i in 0..=token_chars.len() {
        if i == auth.cursor_pos {
            display.push('█');  // Block cursor
        }
        if i < token_chars.len() {
            if i < visible_chars {
                display.push(token_chars[i]);
            } else {
                display.push('•');
            }
        }
    }
    assert_eq!(display, "█test••", "Cursor at start should show █test••");

    // Cursor at position 2 (in visible region)
    auth.cursor_pos = 2;
    display.clear();
    for i in 0..=token_chars.len() {
        if i == auth.cursor_pos {
            display.push('█');  // Block cursor
        }
        if i < token_chars.len() {
            if i < visible_chars {
                display.push(token_chars[i]);
            } else {
                display.push('•');
            }
        }
    }
    assert_eq!(display, "te█st••", "Cursor at 2 should show te█st••");

    // Cursor at position 5 (in masked region)
    auth.cursor_pos = 5;
    display.clear();
    for i in 0..=token_chars.len() {
        if i == auth.cursor_pos {
            display.push('█');  // Block cursor
        }
        if i < token_chars.len() {
            if i < visible_chars {
                display.push(token_chars[i]);
            } else {
                display.push('•');
            }
        }
    }
    assert_eq!(display, "test•█•", "Cursor at 5 should show test•█•");

    // Cursor at position 6 (at end)
    auth.cursor_pos = 6;
    display.clear();
    for i in 0..=token_chars.len() {
        if i == auth.cursor_pos {
            display.push('█');  // Block cursor
        }
        if i < token_chars.len() {
            if i < visible_chars {
                display.push(token_chars[i]);
            } else {
                display.push('•');
            }
        }
    }
    assert_eq!(display, "test••█", "Cursor at end should show test••█");
}

/// Test that auth display shows token after paste
#[test]
fn test_auth_display_after_paste() {
    use clickdown::tui::widgets::AuthState;

    let mut auth = AuthState::new();
    
    // Simulate paste: add multiple characters at once
    let pasted_text = "test_api_token_12345";
    for c in pasted_text.chars() {
        auth.add_char(c);
    }
    
    // Verify token was added
    assert_eq!(auth.token_input, "test_api_token_12345");
    assert_eq!(auth.cursor_pos, 20);
    
    // Build display string (mimicking render_auth logic)
    let visible_chars = 4;
    let token_chars: Vec<char> = auth.token_input.chars().collect();
    let mut display = String::new();
    for i in 0..=token_chars.len() {
        if i == auth.cursor_pos {
            display.push('█');  // Block cursor
        }
        if i < token_chars.len() {
            if i < visible_chars {
                display.push(token_chars[i]);
            } else {
                display.push('•');
            }
        }
    }
    
    // Should show "test" + 16 bullets + cursor at end
    assert_eq!(display, "test••••••••••••••••█", "Display should show first 4 chars + bullets + cursor");
}

/// Test that auth display shows token after typing
#[test]
fn test_auth_display_after_typing() {
    use clickdown::tui::widgets::AuthState;

    let mut auth = AuthState::new();
    
    // Simulate typing character by character
    auth.add_char('a');
    assert_eq!(auth.token_input, "a");
    
    auth.add_char('b');
    assert_eq!(auth.token_input, "ab");
    
    auth.add_char('c');
    assert_eq!(auth.token_input, "abc");
    
    auth.add_char('d');
    assert_eq!(auth.token_input, "abcd");
    
    auth.add_char('e');
    assert_eq!(auth.token_input, "abcde");
    
    // Build display string
    let visible_chars = 4;
    let token_chars: Vec<char> = auth.token_input.chars().collect();
    let mut display = String::new();
    for i in 0..=token_chars.len() {
        if i == auth.cursor_pos {
            display.push('█');  // Block cursor
        }
        if i < token_chars.len() {
            if i < visible_chars {
                display.push(token_chars[i]);
            } else {
                display.push('•');
            }
        }
    }

    // Should show "abcd" + 1 bullet + cursor at end
    assert_eq!(display, "abcd•█", "Display should show abcd + bullet + cursor");
}

// ==================== Comment API Tests ====================

/// Test that mock client can be configured with comments
#[test]
fn test_mock_client_with_comments() {
    use clickdown::api::MockClickUpClient;
    use clickdown::api::ClickUpApi;
    use clickdown::models::comment::Comment;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();
    
    rt.block_on(async {
        let comments = vec![
            Comment {
                id: "test-comment-1".to_string(),
                text: "First comment".to_string(),
                text_preview: "First...".to_string(),
                commenter: None,
                created_at: Some(1234567890000),
                updated_at: None,
                assigned_commenter: None,
                assigned_by: None,
                assigned: false,
                reaction: String::new(),
            parent_id: None,
            },
            Comment {
                id: "test-comment-2".to_string(),
                text: "Second comment".to_string(),
                text_preview: "Second...".to_string(),
                commenter: None,
                created_at: Some(1234567899000),
                updated_at: Some(1234567900000),
                assigned_commenter: None,
                assigned_by: None,
                assigned: false,
                reaction: String::new(),
            parent_id: None,
            },
        ];
        
        let mock_client = MockClickUpClient::new()
            .with_task_comments(comments);

        let comments = mock_client.get_task_comments("task-123").await.unwrap();
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].id, "test-comment-1");
    });
}

/// Test that mock client create comment works
#[test]
fn test_mock_client_create_comment() {
    use clickdown::api::MockClickUpClient;
    use clickdown::api::ClickUpApi;
    use clickdown::models::CreateCommentRequest;
    use clickdown::models::comment::Comment;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();
    
    rt.block_on(async {
        let new_comment = Comment {
            id: "test-comment-1".to_string(),
            text: "This is a test comment".to_string(),
            text_preview: "This is a...".to_string(),
            commenter: None,
            created_at: Some(1234567890000),
            updated_at: None,
            assigned_commenter: None,
            assigned_by: None,
            assigned: false,
            reaction: String::new(),
            parent_id: None,
        };
        
        let mock_client = MockClickUpClient::new()
            .with_create_comment_response(new_comment.clone());

        let request = CreateCommentRequest {
            comment_text: "New comment".to_string(),
            assignee: None,
            assigned_commenter: None,
            parent_id: None,
        };

        let result = mock_client.create_comment("task-123", &request).await.unwrap();
        assert_eq!(result.id, "test-comment-1");
        assert_eq!(result.text, "This is a test comment");
    });
}

/// Test that mock client update comment works
#[test]
fn test_mock_client_update_comment() {
    use clickdown::api::MockClickUpClient;
    use clickdown::api::ClickUpApi;
    use clickdown::models::UpdateCommentRequest;
    use clickdown::models::comment::Comment;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();
    
    rt.block_on(async {
        let updated_comment = Comment {
            id: "test-comment-2".to_string(),
            text: "This comment was edited".to_string(),
            text_preview: "This comment...".to_string(),
            commenter: None,
            created_at: Some(1234567890000),
            updated_at: Some(1234567900000),
            assigned_commenter: None,
            assigned_by: None,
            assigned: false,
            reaction: String::new(),
            parent_id: None,
        };
        
        let mock_client = MockClickUpClient::new()
            .with_update_comment_response(updated_comment.clone());

        let request = UpdateCommentRequest {
            comment_text: Some("Updated text".to_string()),
            assigned: None,
            assignee: None,
            assigned_commenter: None,
        };

        let result = mock_client.update_comment("comment-123", &request).await.unwrap();
        assert_eq!(result.id, "test-comment-2");
        assert_eq!(result.text, "This comment was edited");
    });
}

// ==================== Comment Cache Tests ====================

/// Test that comments can be cached and retrieved
#[test]
fn test_cache_comments() {
    use clickdown::cache::CacheManager;
    use clickdown::models::comment::Comment;
    use std::path::PathBuf;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let db_path = PathBuf::from(temp_dir.path()).join("cache.db");
    
    let mut cache = CacheManager::new(db_path).unwrap();
    
    // Create test comments
    let comments = vec![
        Comment {
            id: "test-comment-1".to_string(),
            text: "First comment".to_string(),
            text_preview: "First...".to_string(),
            commenter: None,
            created_at: Some(1234567890000),
            updated_at: None,
            assigned_commenter: None,
            assigned_by: None,
            assigned: false,
            reaction: String::new(),
            parent_id: None,
        },
        Comment {
            id: "test-comment-2".to_string(),
            text: "Second comment".to_string(),
            text_preview: "Second...".to_string(),
            commenter: None,
            created_at: Some(1234567899000),
            updated_at: Some(1234567900000),
            assigned_commenter: None,
            assigned_by: None,
            assigned: false,
            reaction: String::new(),
            parent_id: None,
        },
    ];
    
    // Cache comments
    cache.cache_comments("task-123", &comments).unwrap();

    // Retrieve comments (ordered by created_at DESC, so comment-2 comes first)
    let cached = cache.get_cached_comments("task-123").unwrap();
    assert_eq!(cached.len(), 2);
    assert_eq!(cached[0].id, "test-comment-2");
    assert_eq!(cached[1].id, "test-comment-1");
}

/// Test that cache validity check works
#[test]
fn test_cache_validity() {
    use clickdown::cache::CacheManager;
    use clickdown::models::comment::Comment;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use std::thread;
    use std::time::Duration;

    let temp_dir = TempDir::new().unwrap();
    let db_path = PathBuf::from(temp_dir.path()).join("cache.db");
    
    let mut cache = CacheManager::new(db_path).unwrap();
    
    // Create test comments
    let comments = vec![Comment {
        id: "test-comment-1".to_string(),
        text: "Test comment".to_string(),
        text_preview: "Test...".to_string(),
        commenter: None,
        created_at: Some(1234567890000),
        updated_at: None,
        assigned_commenter: None,
        assigned_by: None,
        assigned: false,
        reaction: String::new(),
            parent_id: None,
    }];
    
    // Cache comments
    cache.cache_comments("task-123", &comments).unwrap();
    
    // Should be valid immediately
    assert!(cache.is_cache_valid("task-123", 300).unwrap()); // 5 min TTL
    
    // Wait a bit and check again (still valid)
    thread::sleep(Duration::from_millis(100));
    assert!(cache.is_cache_valid("task-123", 300).unwrap());
    
    // Check with very short TTL (should be invalid)
    assert!(!cache.is_cache_valid("task-123", 0).unwrap());
}

/// Test that clearing comments works
#[test]
fn test_clear_comments() {
    use clickdown::cache::CacheManager;
    use clickdown::models::comment::Comment;
    use std::path::PathBuf;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let db_path = PathBuf::from(temp_dir.path()).join("cache.db");
    
    let mut cache = CacheManager::new(db_path).unwrap();
    
    // Create test comments
    let comments = vec![
        Comment {
            id: "test-comment-1".to_string(),
            text: "First".to_string(),
            text_preview: "First...".to_string(),
            commenter: None,
            created_at: Some(1234567890000),
            updated_at: None,
            assigned_commenter: None,
            assigned_by: None,
            assigned: false,
            reaction: String::new(),
            parent_id: None,
        },
        Comment {
            id: "test-comment-2".to_string(),
            text: "Second".to_string(),
            text_preview: "Second...".to_string(),
            commenter: None,
            created_at: Some(1234567899000),
            updated_at: None,
            assigned_commenter: None,
            assigned_by: None,
            assigned: false,
            reaction: String::new(),
            parent_id: None,
        },
    ];
    
    // Cache comments
    cache.cache_comments("task-123", &comments).unwrap();
    
    // Verify cached
    assert_eq!(cache.get_cached_comments("task-123").unwrap().len(), 2);
    
    // Clear comments for this task
    cache.clear_comments("task-123").unwrap();
    
    // Should be empty
    assert_eq!(cache.get_cached_comments("task-123").unwrap().len(), 0);
}

/// Test that comments for different tasks are isolated
#[test]
fn test_comments_isolated_by_task() {
    use clickdown::cache::CacheManager;
    use clickdown::models::comment::Comment;
    use std::path::PathBuf;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let db_path = PathBuf::from(temp_dir.path()).join("cache.db");
    
    let mut cache = CacheManager::new(db_path).unwrap();
    
    // Create different comments for different tasks
    let comment1 = Comment {
        id: "test-comment-1".to_string(),
        text: "Task 1 comment".to_string(),
        text_preview: "Task 1...".to_string(),
        commenter: None,
        created_at: Some(1234567890000),
        updated_at: None,
        assigned_commenter: None,
        assigned_by: None,
        assigned: false,
        reaction: String::new(),
            parent_id: None,
    };
    
    let comment2 = Comment {
        id: "test-comment-2".to_string(),
        text: "Task 2 comment".to_string(),
        text_preview: "Task 2...".to_string(),
        commenter: None,
        created_at: Some(1234567890000),
        updated_at: Some(1234567900000),
        assigned_commenter: None,
        assigned_by: None,
        assigned: false,
        reaction: String::new(),
            parent_id: None,
    };
    
    // Cache different comments for different tasks
    cache.cache_comments("task-1", &[comment1]).unwrap();
    cache.cache_comments("task-2", &[comment2]).unwrap();
    
    // Verify isolation
    let task1_comments = cache.get_cached_comments("task-1").unwrap();
    let task2_comments = cache.get_cached_comments("task-2").unwrap();
    
    assert_eq!(task1_comments.len(), 1);
    assert_eq!(task1_comments[0].id, "test-comment-1");
    
    assert_eq!(task2_comments.len(), 1);
    assert_eq!(task2_comments[0].id, "test-comment-2");
}
