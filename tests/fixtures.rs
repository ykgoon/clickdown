//! Test fixtures for ClickDown tests

use clickdown::models::workspace::{Workspace, Space, Folder, List};
use clickdown::models::task::Task;
use clickdown::models::document::Document;
use clickdown::models::CommentUser;
use clickdown::models::comment::Comment;

/// Create a test workspace
pub fn test_workspace() -> Workspace {
    Workspace {
        id: "test-ws-1".to_string(),
        name: "Test Workspace".to_string(),
        color: Some("#1abc9c".to_string()),
        avatar: None,
        member_count: Some(5),
    }
}

/// Create a test space
pub fn test_space() -> Space {
    Space {
        id: "test-space-1".to_string(),
        name: "Test Space".to_string(),
        color: Some("#3498db".to_string()),
        private: false,
        status: None,
        folders: vec![],
        lists: vec![],
    }
}

/// Create a test folder
pub fn test_folder() -> Folder {
    Folder {
        id: "test-folder-1".to_string(),
        name: "Test Folder".to_string(),
        color: Some("#e74c3c".to_string()),
        private: false,
        space: None,
        lists: vec![],
    }
}

/// Create a test list
pub fn test_list() -> List {
    List {
        id: "test-list-1".to_string(),
        name: "Test List".to_string(),
        content: None,
        description: None,
        archived: false,
        hidden: false,
        orderindex: Some(0),
        space: None,
        folder: None,
        status: None,
        priority: None,
    }
}

/// Create a test task
pub fn test_task() -> Task {
    Task {
        id: "test-task-1".to_string(),
        custom_id: None,
        custom_item_id: None,
        name: "Test Task".to_string(),
        text_content: None,
        description: None,
        markdown_description: None,
        status: None,
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
    }
}

/// Create a test document
pub fn test_document() -> Document {
    Document {
        id: "test-doc-1".to_string(),
        name: "Test Document".to_string(),
        created_at: None,
        updated_at: None,
        created_by: None,
        updated_by: None,
        space: None,
        folder: None,
        url: None,
        pages: vec![],
    }
}

/// Create a test comment user
pub fn test_comment_user() -> CommentUser {
    CommentUser {
        id: 123,
        username: "testuser".to_string(),
        color: None,
        email: Some("test@example.com".to_string()),
        profile_picture: None,
        initials: Some("TU".to_string()),
    }
}

/// Create a test comment
pub fn test_comment() -> Comment {
    Comment {
        id: "test-comment-1".to_string(),
        text: "This is a test comment".to_string(),
        text_preview: "This is a...".to_string(),
        commenter: Some(test_comment_user()),
        created_at: Some(1234567890000),
        updated_at: None,
        assigned_commenter: None,
        assigned_by: None,
        assigned: false,
        reaction: String::new(),
        parent_id: None,
    }
}

/// Create a test comment with update timestamp
pub fn test_comment_edited() -> Comment {
    Comment {
        id: "test-comment-2".to_string(),
        text: "This comment was edited".to_string(),
        text_preview: "This comment...".to_string(),
        commenter: Some(test_comment_user()),
        created_at: Some(1234567890000),
        updated_at: Some(1234567900000),
        assigned_commenter: None,
        assigned_by: None,
        assigned: false,
        reaction: String::new(),
        parent_id: None,
    }
}

/// Create multiple test comments
pub fn test_comments() -> Vec<Comment> {
    vec![test_comment(), test_comment_edited()]
}
