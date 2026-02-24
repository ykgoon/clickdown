//! Test fixtures for ClickDown tests

use clickdown::models::workspace::{Workspace, Space, Folder, List};
use clickdown::models::task::Task;
use clickdown::models::document::Document;

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
        name: "Test Task".to_string(),
        description: None,
        status: None,
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
