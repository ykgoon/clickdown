//! Integration tests for ClickDown TUI application
//!
//! These tests verify the TUI application functionality.

mod fixtures;

use clickdown::tui::app::TuiApp;

/// Test that the TUI app initializes correctly
#[test]
fn test_tui_app_initialization() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let app = rt.block_on(async { TuiApp::new() });

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

    assert_ne!(
        auth_title, workspace_title,
        "Auth and Workspace titles should be different"
    );
    assert_ne!(
        auth_title, tasks_title,
        "Auth and Tasks titles should be different"
    );
    assert_ne!(
        workspace_title, tasks_title,
        "Workspace and Tasks titles should be different"
    );
}

/// Test that minimum terminal size constants are defined
#[test]
fn test_minimum_terminal_size() {
    use clickdown::tui::layout::{MIN_HEIGHT, MIN_WIDTH};

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

    assert!(
        !layout.too_small,
        "Layout should not be too small for 100x30 terminal"
    );
    assert!(layout.title_area.height == 1, "Title area should be 1 row");
    assert!(
        layout.status_area.height == 1,
        "Status area should be 1 row (hint bar removed)"
    );
}

/// Test that layout detects small terminals
#[test]
fn test_layout_small_terminal() {
    use clickdown::tui::layout::TuiLayout;
    use ratatui::prelude::Rect;

    let area = Rect::new(0, 0, 60, 20);
    let layout = TuiLayout::new(area);

    assert!(
        layout.too_small,
        "Layout should be too small for 60x20 terminal"
    );
}

/// Test input event types
#[test]
fn test_input_event_types() {
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent};

    let key_event = KeyEvent::new(KeyCode::Char('a'), crossterm::event::KeyModifiers::NONE);
    let input_event = InputEvent::Key(key_event);

    match input_event {
        InputEvent::Key(_) => (),
        _ => panic!("Expected Key event"),
    }
}

/// Test that plain q and plain space do NOT quit (SPC q sequence only, via dialog)
#[test]
fn test_quit_detection() {
    use clickdown::tui::app::TuiApp;
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut app = TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap();

        // Plain q in normal mode must not open the quit dialog
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
        )));
        assert!(!app.is_dialog_visible(), "plain q should not quit");

        // Plain space alone must not open the quit dialog
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char(' '),
            KeyModifiers::NONE,
        )));
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));
        assert!(!app.is_dialog_visible(), "plain space should not quit");
    });
}

/// Test dialog state
#[test]
fn test_dialog_state() {
    use clickdown::tui::widgets::{DialogState, DialogType};

    let mut dialog = DialogState::new();

    // Dialog should start hidden
    assert!(!dialog.is_visible(), "Dialog should start hidden");

    // Show dialog
    dialog.show(DialogType::ConfirmDelete);
    assert!(dialog.is_visible(), "Dialog should be visible after show");

    // Toggle selection
    assert!(!dialog.confirmed(), "Dialog should start with No selected");
    dialog.toggle();
    assert!(
        dialog.confirmed(),
        "Dialog should have Yes selected after toggle"
    );

    // Hide dialog
    dialog.hide();
    assert!(!dialog.is_visible(), "Dialog should be hidden after hide");
}

/// Test auth state
#[test]
fn test_auth_state() {
    use clickdown::tui::widgets::AuthState;

    let mut auth = AuthState::new();

    // Auth should start empty
    assert!(
        auth.token_input.is_empty(),
        "Token input should start empty"
    );
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
    assert!(
        auth.token_input.is_empty(),
        "Token input should be empty after clear"
    );
    assert_eq!(auth.cursor_pos, 0, "Cursor should be at 0 after clear");
}

/// Test sidebar state
#[test]
fn test_sidebar_state() {
    use clickdown::tui::widgets::{SidebarItem, SidebarState};

    let mut sidebar = SidebarState::new();

    // Sidebar should start empty
    assert!(sidebar.items().is_empty(), "Sidebar should start empty");

    // Add items
    sidebar.items_mut().push(SidebarItem::Workspace {
        name: "Test Workspace".to_string(),
        id: "ws-1".to_string(),
    });
    sidebar.items_mut().push(SidebarItem::Space {
        name: "Test Space".to_string(),
        id: "sp-1".to_string(),
    });

    // Select first
    sidebar.select_first();
    assert_eq!(
        sidebar.state().selected(),
        Some(0),
        "First item should be selected"
    );

    // Select next
    sidebar.select_next();
    assert_eq!(
        sidebar.state().selected(),
        Some(1),
        "Second item should be selected"
    );

    // Select next (wrap around)
    sidebar.select_next();
    assert_eq!(
        sidebar.state().selected(),
        Some(0),
        "Should wrap to first item"
    );

    // Select previous
    sidebar.select_previous();
    assert_eq!(
        sidebar.state().selected(),
        Some(1),
        "Should go back to second item"
    );
}

/// Test task list state
#[test]
fn test_task_list_state() {
    use clickdown::models::{Task, TaskStatus};
    use clickdown::tui::widgets::GroupedTaskList;

    // Create tasks with status_group for grouped rendering
    let tasks = vec![Task {
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
            status_group: Some("todo".to_string()),
        }),
        orderindex: None,
        content: None,
        created_at: None,
        updated_at: Some(1000),
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
    }];

    let mut task_list = GroupedTaskList::from_tasks(tasks);

    // from_tasks selects first task automatically
    assert!(
        task_list.selected_task().is_some(),
        "Should have a task selected"
    );

    // Get selected task
    let selected = task_list.selected_task();
    assert!(selected.is_some(), "Should have selected task");
    assert_eq!(
        selected.unwrap().name,
        "Task 1",
        "Selected task should be Task 1"
    );
}

/// Test that Ctrl+Shift+V does NOT trigger quit
/// (quit is now the SPC q sequence via the command menu, so modifier combos are inert)
#[test]
fn test_ctrl_shift_v_does_not_quit() {
    use clickdown::tui::app::TuiApp;
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut app = TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap();

        // Ctrl+Shift+V should NOT open the quit dialog
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('v'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        )));
        assert!(!app.is_dialog_visible(), "Ctrl+Shift+V should NOT quit");

        // Ctrl+Shift+Q should NOT open the quit dialog
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('q'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        )));
        assert!(!app.is_dialog_visible(), "Ctrl+Shift+Q should NOT quit");
    });
}

/// Test that the SPC q sequence quits immediately (no confirmation dialog)
#[test]
fn test_spc_q_sequence_quits_immediately() {
    use clickdown::tui::app::TuiApp;
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut app = TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap();

        // SPC opens the command menu, then q at the root quits right away
        for key in [' ', 'q'] {
            app.update(InputEvent::Key(KeyEvent::new(
                KeyCode::Char(key),
                KeyModifiers::NONE,
            )));
        }

        assert!(
            !app.is_dialog_visible(),
            "SPC q must not show a confirmation dialog"
        );
        assert!(
            app.is_quitting_for_test(),
            "SPC q must quit immediately"
        );
        assert!(!app.command_menu_open_for_test(), "menu should close after SPC q");
    });
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
    assert_eq!(
        display, "abcd",
        "4-char token should show all chars unmasked"
    );

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
            display.push('█'); // Block cursor
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
            display.push('█'); // Block cursor
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
            display.push('█'); // Block cursor
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
            display.push('█'); // Block cursor
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
            display.push('█'); // Block cursor
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
    assert_eq!(
        display, "test••••••••••••••••█",
        "Display should show first 4 chars + bullets + cursor"
    );
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
            display.push('█'); // Block cursor
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
    assert_eq!(
        display, "abcd•█",
        "Display should show abcd + bullet + cursor"
    );
}

// ==================== Comment API Tests ====================

/// Test that mock client can be configured with comments
#[test]
fn test_mock_client_with_comments() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::api::ClickUpApi;
    use clickdown::models::comment::Comment;
    use std::sync::Arc;
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

        let mock_client = MockClickUpClient::new().with_task_comments(comments);

        let comments = mock_client.get_task_comments("task-123").await.unwrap();
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].id, "test-comment-1");
    });
}

/// Test that mock client create comment works
#[test]
fn test_mock_client_create_comment() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::api::ClickUpApi;
    use clickdown::models::comment::Comment;
    use clickdown::models::CreateCommentRequest;
    use std::sync::Arc;
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

        let mock_client =
            MockClickUpClient::new().with_create_comment_response(new_comment.clone());

        let request = CreateCommentRequest {
            comment: None,
            comment_text: "New comment".to_string(),
            assignee: None,
            assigned_commenter: None,
            parent_id: None,
        };

        let result = mock_client
            .create_comment("task-123", &request)
            .await
            .unwrap();
        assert_eq!(result.id, "test-comment-1");
        assert_eq!(result.text, "This is a test comment");
    });
}

/// Test that mock client update comment works
#[test]
fn test_mock_client_update_comment() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::api::ClickUpApi;
    use clickdown::models::comment::Comment;
    use clickdown::models::UpdateCommentRequest;
    use std::sync::Arc;
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

        let mock_client =
            MockClickUpClient::new().with_update_comment_response(updated_comment.clone());

        let request = UpdateCommentRequest {
            comment_text: Some("Updated text".to_string()),
            assigned: None,
            assignee: None,
            assigned_commenter: None,
        };

        let result = mock_client
            .update_comment("comment-123", &request)
            .await
            .unwrap();
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
    let cached = cache.get_comments("task-123").unwrap();
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
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

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
    assert_eq!(cache.get_comments("task-123").unwrap().len(), 2);

    // Clear comments for this task
    cache.clear_comments("task-123").unwrap();

    // Should be empty
    assert_eq!(cache.get_comments("task-123").unwrap().len(), 0);
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
    let task1_comments = cache.get_comments("task-1").unwrap();
    let task2_comments = cache.get_comments("task-2").unwrap();

    assert_eq!(task1_comments.len(), 1);
    assert_eq!(task1_comments[0].id, "test-comment-1");

    assert_eq!(task2_comments.len(), 1);
    assert_eq!(task2_comments[0].id, "test-comment-2");
}

// ==================== Reply Creation Tests ====================

/// Test that mock client can create comment replies
#[test]
fn test_mock_client_create_comment_reply() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::api::ClickUpApi;
    use clickdown::models::comment::Comment;
    use clickdown::models::CreateCommentRequest;
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let reply_comment = Comment {
            id: "test-reply-1".to_string(),
            text: "This is a test reply".to_string(),
            text_preview: "This is a...".to_string(),
            commenter: None,
            created_at: Some(1234567890000),
            updated_at: None,
            assigned_commenter: None,
            assigned_by: None,
            assigned: false,
            reaction: String::new(),
            parent_id: Some("parent-comment-123".to_string()),
        };

        let mock_client =
            MockClickUpClient::new().with_create_comment_reply_response(reply_comment.clone());

        let request = CreateCommentRequest {
            comment: None,
            comment_text: "This is a test reply".to_string(),
            assignee: None,
            assigned_commenter: None,
            parent_id: Some("parent-comment-123".to_string()),
        };

        let result = mock_client
            .create_comment_reply("parent-comment-123", &request)
            .await
            .unwrap();
        assert_eq!(result.id, "test-reply-1");
        assert_eq!(result.parent_id, Some("parent-comment-123".to_string()));
    });
}

/// Test that reply request includes parent_id
#[test]
fn test_create_comment_request_with_parent_id() {
    use clickdown::models::CreateCommentRequest;

    let request = CreateCommentRequest {
        comment: None,
        comment_text: "Reply text".to_string(),
        assignee: None,
        assigned_commenter: None,
        parent_id: Some("parent-123".to_string()),
    };

    // Verify parent_id is set correctly
    assert_eq!(request.parent_id, Some("parent-123".to_string()));
    assert_eq!(request.comment_text, "Reply text");
}

/// Test that CreateCommentRequest serializes parent_id correctly
#[test]
fn test_create_comment_request_serialization() {
    use clickdown::models::CreateCommentRequest;
    use serde_json;

    let request = CreateCommentRequest {
        comment: None,
        comment_text: "Reply content".to_string(),
        assignee: None,
        assigned_commenter: None,
        parent_id: Some("parent-456".to_string()),
    };

    let json = serde_json::to_string(&request).unwrap();

    // Verify JSON contains parent_id
    assert!(json.contains("parent_id"));
    assert!(json.contains("parent-456"));
    assert!(json.contains("Reply content"));
}

/// Test that empty reply validation works
#[test]
fn test_empty_reply_validation() {
    // Test that empty string is detected
    let empty = "";
    assert!(empty.trim().is_empty());

    // Test that whitespace-only string is detected
    let whitespace = "   \n\t  ";
    assert!(whitespace.trim().is_empty());

    // Test that valid text passes validation
    let valid = "This is a valid reply";
    assert!(!valid.trim().is_empty());
}

/// Test that reply comment has parent_id set
#[test]
fn test_reply_comment_parent_id() {
    use clickdown::models::comment::Comment;

    let reply = Comment {
        id: "reply-1".to_string(),
        text: "This is a reply".to_string(),
        text_preview: "This is...".to_string(),
        commenter: None,
        created_at: Some(1234567890000),
        updated_at: None,
        assigned_commenter: None,
        assigned_by: None,
        assigned: false,
        reaction: String::new(),
        parent_id: Some("parent-123".to_string()),
    };

    assert_eq!(reply.parent_id, Some("parent-123".to_string()));
}

/// Test that top-level comment has no parent_id
#[test]
fn test_top_level_comment_no_parent_id() {
    use clickdown::models::comment::Comment;

    let top_level = Comment {
        id: "top-level-1".to_string(),
        text: "This is a top-level comment".to_string(),
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

    assert_eq!(top_level.parent_id, None);
}

fn key_event(c: char) -> clickdown::tui::input::InputEvent {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    clickdown::tui::input::InputEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}

/// Test that SPC t s opens the status picker; bare 's' is a NORMAL-mode no-op.
#[test]
fn test_s_key_opens_status_picker() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::models::{Task, TaskStatus};
    use clickdown::tui::app::Screen;
    use clickdown::tui::app::TuiApp;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Create task with a status
    let task = Task {
        id: "task-1".to_string(),
        custom_id: None,
        custom_item_id: None,
        name: "Test Task".to_string(),
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
    };

    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![])
        .with_tasks(vec![task.clone()]);

    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(mock_client)) })
        .unwrap();

    // Manually navigate to Tasks screen (bypass async loading)
    app.set_screen(Screen::Tasks);
    app.tasks_mut_for_test().push(task.clone());
    app.rebuild_task_list_for_test();

    // Select the task
    app.task_list().select_first();

    // Verify task is selected
    assert!(
        app.task_list().selected_task().is_some(),
        "Task should be selected before pressing 's'"
    );

    // Press SPC t s chord (bare 's' is a no-op in NORMAL mode now)
    app.update(key_event(' '));
    app.update(key_event('t'));
    app.update(key_event('s'));

    // Status picker should be open
    assert!(
        app.is_status_picker_open(),
        "SPC t s with a selected task should open status picker"
    );

    // Bare 's' in NORMAL mode must be a no-op (fresh app, same setup)
    let mock2 = MockClickUpClient::new()
        .with_workspaces(vec![])
        .with_tasks(vec![task.clone()]);
    let mut app2 = rt
        .block_on(async { TuiApp::with_client(Arc::new(mock2)) })
        .unwrap();
    app2.set_screen(Screen::Tasks);
    app2.tasks_mut_for_test().push(task.clone());
    app2.rebuild_task_list_for_test();
    app2.task_list().select_first();
    app2.update(InputEvent::Key(KeyEvent::new(
        KeyCode::Char('s'),
        crossterm::event::KeyModifiers::NONE,
    )));
    assert!(
        !app2.is_status_picker_open(),
        "Bare 's' in NORMAL mode must not open status picker"
    );
}

/// Test that pressing 's' in task list without a selected task shows feedback
#[test]
fn test_s_key_no_task_selected() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::Screen;
    use clickdown::tui::app::TuiApp;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();

    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![])
        .with_tasks(vec![]);

    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(mock_client)) })
        .unwrap();

    // Navigate to Tasks screen
    app.set_screen(Screen::Tasks);

    // Press 's' key without any task selected
    let s_key = KeyEvent::new(KeyCode::Char('s'), crossterm::event::KeyModifiers::NONE);
    app.update(InputEvent::Key(s_key));

    // Status picker should NOT be open
    assert!(
        !app.is_status_picker_open(),
        "Pressing 's' without a selected task should NOT open status picker"
    );
}

/// Test that pressing 's' in Task Detail view should respond (not silently ignored)
///
/// BUG REPRODUCTION: When the user is in Task Detail view (after pressing Enter on a task),
/// pressing 's' should do something (e.g., open status picker or show a message).
/// Currently, 's' is silently ignored - no status message, no visual feedback.
#[test]
fn test_s_key_in_task_detail_should_respond() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::{Screen, TuiApp};
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();

    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![])
        .with_tasks(vec![]);

    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(mock_client)) })
        .unwrap();

    // Navigate to Task Detail screen
    app.set_screen(Screen::TaskDetail);

    // Set a task in the detail view
    let task = fixtures::test_task();
    app.task_detail().task = Some(task);

    // Bare 's' in NORMAL mode is a no-op now
    let status_before = app.status().to_string();
    app.update(InputEvent::Key(KeyEvent::new(
        KeyCode::Char('s'),
        crossterm::event::KeyModifiers::NONE,
    )));
    assert_eq!(
        app.status(),
        status_before,
        "Bare 's' in NORMAL mode should be a no-op"
    );
    assert!(
        !app.is_status_picker_open(),
        "Bare 's' must not open the status picker"
    );

    // SPC t s chord opens the status picker with feedback
    app.update(key_event(' '));
    app.update(key_event('t'));
    app.update(key_event('s'));
    assert!(
        app.is_status_picker_open(),
        "SPC t s in Task Detail view should open the status picker"
    );
}

// ==================== URL Navigation Integration Tests ====================

/// Test that the URL parser correctly handles all URL patterns
#[test]
fn test_url_parser_roundtrip() {
    use clickdown::utils::{ClickUpUrlGenerator, ParsedUrl, UrlGenerator, UrlParser};

    // Test workspace URL roundtrip
    let generated = ClickUpUrlGenerator::workspace_url("ws1").unwrap();
    let parsed = UrlParser::parse(&generated).unwrap();
    assert!(matches!(parsed, ParsedUrl::Workspace { ref workspace_id } if workspace_id == "ws1"));

    // Test task URL roundtrip
    let generated = ClickUpUrlGenerator::task_url("", "", "task123").unwrap();
    let parsed = UrlParser::parse(&generated).unwrap();
    assert!(matches!(parsed, ParsedUrl::Task { ref task_id, .. } if task_id == "task123"));

    // Test document URL roundtrip
    let generated = ClickUpUrlGenerator::document_url("", "doc1").unwrap();
    let parsed = UrlParser::parse(&generated).unwrap();
    assert!(matches!(parsed, ParsedUrl::Document { ref doc_id } if doc_id == "doc1"));

    // Test comment URL roundtrip
    let generated = ClickUpUrlGenerator::comment_url("", "", "task123", "cmt1").unwrap();
    let parsed = UrlParser::parse(&generated).unwrap();
    assert!(
        matches!(parsed, ParsedUrl::Comment { ref task_id, ref comment_id, .. } if task_id == "task123" && comment_id == "cmt1")
    );
}

/// Test that the URL input dialog opens with g → u chord
#[test]
fn test_g_u_opens_url_input_dialog() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::TuiApp;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new().with_workspaces(vec![fixtures::test_workspace()]);

    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(mock_client)) })
        .unwrap();

    // Verify dialog is not open initially
    assert!(
        !app.is_url_input_open(),
        "URL input dialog should not be open initially"
    );

    // Press 'g' key
    let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    app.update(InputEvent::Key(g_key));

    // Dialog should NOT be open yet (waiting for second key)
    assert!(
        !app.is_url_input_open(),
        "URL input dialog should not be open after just 'g'"
    );

    // Press 'u' key
    let u_key = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE);
    app.update(InputEvent::Key(u_key));

    // Dialog should now be open
    assert!(
        app.is_url_input_open(),
        "URL input dialog should be open after 'g' then 'u'"
    );
    assert_eq!(app.url_input_text(), "", "URL input text should be empty");
}

/// Test that invalid URL shows error in dialog
#[test]
fn test_invalid_url_shows_error() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::TuiApp;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new().with_workspaces(vec![fixtures::test_workspace()]);

    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(mock_client)) })
        .unwrap();

    // Open dialog
    let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    app.update(InputEvent::Key(g_key));
    let u_key = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE);
    app.update(InputEvent::Key(u_key));
    assert!(app.is_url_input_open());

    // Type an invalid URL
    for c in "not-a-url".chars() {
        let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
        app.handle_url_input(key);
    }

    // Submit with Enter
    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.update(InputEvent::Key(enter_key));

    // Dialog should remain open with error
    assert!(
        app.is_url_input_open(),
        "Dialog should stay open for invalid URL"
    );
    assert!(
        app.url_input_error().is_some(),
        "Error should be shown for invalid URL"
    );
}

/// Test that Esc cancels the g leader key
#[test]
fn test_esc_cancels_g_leader() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::TuiApp;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new().with_workspaces(vec![fixtures::test_workspace()]);

    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(mock_client)) })
        .unwrap();

    // Press 'g'
    let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    app.update(InputEvent::Key(g_key));

    // Press Esc to cancel
    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.update(InputEvent::Key(esc_key));

    // Now press 'u' - this should trigger URL copy, not URL input dialog
    let u_key = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE);
    app.update(InputEvent::Key(u_key));

    // URL input dialog should NOT be open (Esc canceled the chord)
    assert!(
        !app.is_url_input_open(),
        "URL input dialog should not open after Esc cancels chord"
    );
}

/// Test that 'g' followed by non-'u' passes through the second key
#[test]
fn test_g_followed_by_non_u_passes_through() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::TuiApp;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_client = MockClickUpClient::new().with_workspaces(vec![fixtures::test_workspace()]);

    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(mock_client)) })
        .unwrap();

    // Press 'g'
    let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    app.update(InputEvent::Key(g_key));

    // Press 'j' - should pass through as normal navigation (move selection down)
    let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    app.update(InputEvent::Key(j_key));

    // URL input dialog should NOT be open
    assert!(
        !app.is_url_input_open(),
        "URL input dialog should not open for 'g' then 'j'"
    );
}

/// Test that confirming the delete dialog actually deletes the selected task
#[test]
fn test_delete_task_on_confirm() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::input::InputEvent;
    use clickdown::tui::widgets::DialogType;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let task = fixtures::test_task();

        let mock_client = MockClickUpClient::new()
            .with_tasks(vec![task.clone()])
            .with_delete_task_success();

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Manually populate the task list (simulating tasks loaded from API)
        // Both TuiApp.tasks and task_list's internal list need to be populated
        app.tasks_mut_for_test().push(task.clone());
        app.rebuild_task_list_for_test();
        app.task_list_mut_for_test().select_first();

        // Verify initial state: task is present
        assert_eq!(app.task_count(), 1, "Should have 1 task");
        assert!(
            app.task_list_for_test().selected_task().is_some(),
            "A task should be selected"
        );

        // Show the delete dialog and confirm
        app.dialog_mut_for_test().show(DialogType::ConfirmDelete);
        app.dialog_mut_for_test().toggle(); // Switch from "No" to "Yes"
        assert!(app.is_dialog_confirmed(), "Dialog should be confirmed");

        // Simulate pressing Enter — this should trigger delete_selected_task()
        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        app.update(InputEvent::Key(enter_key));

        // Dialog should be hidden
        assert!(
            !app.is_dialog_visible(),
            "Dialog should be hidden after confirm"
        );

        // Give the async delete task time to complete
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Process the async message (simulates what run_loop does)
        app.process_async_messages();

        // Task should be removed from the local list
        assert_eq!(
            app.task_count(),
            0,
            "Task should be removed after successful deletion"
        );
        assert!(
            app.task_list_for_test().selected_task().is_none(),
            "No task should be selected after deletion"
        );
    });
}

/// Test that canceling the delete dialog does NOT delete the task
#[test]
fn test_delete_task_on_cancel() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::input::InputEvent;
    use clickdown::tui::widgets::DialogType;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let task = fixtures::test_task();

        let mock_client = MockClickUpClient::new()
            .with_tasks(vec![task.clone()])
            .with_delete_task_success();

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Manually populate the task list
        app.tasks_mut_for_test().push(task.clone());
        app.rebuild_task_list_for_test();
        app.task_list_mut_for_test().select_first();

        // Verify initial state
        assert_eq!(app.task_count(), 1, "Should have 1 task");

        // Show the delete dialog and cancel with Esc
        app.dialog_mut_for_test().show(DialogType::ConfirmDelete);
        let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        app.update(InputEvent::Key(esc_key));

        // Dialog should be hidden, task should remain
        assert!(
            !app.is_dialog_visible(),
            "Dialog should be hidden after cancel"
        );
        assert_eq!(
            app.task_count(),
            1,
            "Task should NOT be removed after cancel"
        );
    });
}

/// Test that pressing 'd' with no task selected does NOT show the dialog
#[test]
fn test_delete_no_task_selected() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::app::Screen;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let mock_client = MockClickUpClient::new();

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Navigate to task list screen (Tasks screen is needed for 'd' to work)
        app.set_screen_for_test(Screen::Tasks);

        // Press 'd' with no task selected
        let d_key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        app.update(InputEvent::Key(d_key));

        // Dialog should NOT be shown
        assert!(
            !app.is_dialog_visible(),
            "Dialog should NOT appear when no task is selected"
        );
    });
}

/// Test that an empty API response body on delete causes a parse error
/// and the task remains in the UI — reproduces "failed to parse response" bug
#[test]
fn test_delete_task_empty_response_fails_to_parse() {
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::input::InputEvent;
    use clickdown::tui::widgets::DialogType;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let task = fixtures::test_task();

        // Simulate ClickUp returning an empty body for DELETE
        let mock_client = MockClickUpClient::new()
            .with_tasks(vec![task.clone()])
            .with_delete_task_json("");

        let mut app = TuiApp::with_client(Arc::new(mock_client)).unwrap();

        // Manually populate the task list
        app.tasks_mut_for_test().push(task.clone());
        app.rebuild_task_list_for_test();
        app.task_list_mut_for_test().select_first();

        // Verify initial state: task is present
        assert_eq!(app.task_count(), 1, "Should have 1 task");

        // Show the delete dialog and confirm
        app.dialog_mut_for_test().show(DialogType::ConfirmDelete);
        app.dialog_mut_for_test().toggle();
        assert!(app.is_dialog_confirmed(), "Dialog should be confirmed");

        // Simulate pressing Enter
        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        app.update(InputEvent::Key(enter_key));

        // Give the async delete task time to complete
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Process the async message
        app.process_async_messages();

        // BUG REPRODUCTION: Task should still be in the list because empty response
        // causes "failed to parse response" error, treated as deletion failure
        assert_eq!(
            app.task_count(),
            1,
            "Task should remain in list because empty response causes parse error"
        );
    });
}

/// SPACEmenu minibuffer: Esc at a group level walks back to the root menu
/// (Doom-style hierarchy), instead of closing the whole menu.
#[test]
fn test_spc_menu_esc_at_group_returns_to_root() {
    use clickdown::tui::app::{CommandMenuLevel, TuiApp};
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut app = TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap();

        // SPC opens root, 't' enters the Task group
        for k in [' ', 't'] {
            app.update(InputEvent::Key(KeyEvent::new(
                KeyCode::Char(k),
                KeyModifiers::NONE,
            )));
        }
        assert!(app.command_menu_open_for_test(), "menu open at group level");
        assert_eq!(
            app.command_menu_level_for_test(),
            Some(CommandMenuLevel::Group('t')),
            "'t' should enter the Task group"
        );

        // Esc returns to the root level, keeping the menu open
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));
        assert!(app.command_menu_open_for_test(), "menu stays open at root");
        assert_eq!(
            app.command_menu_level_for_test(),
            Some(CommandMenuLevel::Root),
            "Esc at group level should return to root"
        );

        // Esc at root closes the menu entirely
        app.update(InputEvent::Key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));
        assert!(!app.command_menu_open_for_test(), "Esc at root closes the menu");
    });
}

/// SPACEmenu minibuffer: every root group key has a single-key display,
/// and unknown group keys close the menu with feedback (one key per level).
#[test]
fn test_spc_menu_root_keys_enter_groups() {
    use clickdown::tui::app::{CommandMenuLevel, TuiApp};
    use clickdown::api::mock_client::MockClickUpClient;
    use clickdown::tui::input::InputEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut app = TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap();

        for (open_key, expected) in [
            ('g', CommandMenuLevel::Group('g')),
            ('t', CommandMenuLevel::Group('t')),
            ('c', CommandMenuLevel::Group('c')),
            ('y', CommandMenuLevel::Group('y')),
            ('w', CommandMenuLevel::Group('w')),
        ] {
            app.update(InputEvent::Key(KeyEvent::new(
                KeyCode::Char(' '),
                KeyModifiers::NONE,
            )));
            app.update(InputEvent::Key(KeyEvent::new(
                KeyCode::Char(open_key),
                KeyModifiers::NONE,
            )));
            assert_eq!(
                app.command_menu_level_for_test(),
                Some(expected),
                "SPC {} should enter group level with a single key",
                open_key
            );
            // close via Esc-at-root for the next iteration
            app.update(InputEvent::Key(KeyEvent::new(
                KeyCode::Esc,
                KeyModifiers::NONE,
            )));
            app.update(InputEvent::Key(KeyEvent::new(
                KeyCode::Esc,
                KeyModifiers::NONE,
            )));
            assert!(!app.command_menu_open_for_test());
        }
    });
}
