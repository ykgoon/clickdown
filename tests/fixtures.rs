//! Test fixtures for ClickUp API mock data

use clickdown::models::{
    Workspace, ClickUpSpace, Folder, List, Task, TaskStatus,
    Document, Page, Priority,
};
use clickdown::models::task::User;
use clickdown::models::document::User as DocumentUser;

/// Create a test workspace
pub fn test_workspace() -> Workspace {
    Workspace {
        id: "test-workspace-1".to_string(),
        name: "Test Workspace".to_string(),
        color: Some("#7B68EE".to_string()),
        avatar: None,
        member_count: Some(5),
    }
}

/// Create a test space
pub fn test_space() -> ClickUpSpace {
    ClickUpSpace {
        id: "test-space-1".to_string(),
        name: "Test Space".to_string(),
        color: Some("#4CAF50".to_string()),
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
        color: Some("#FF9800".to_string()),
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
        description: Some("A test list for testing".to_string()),
        archived: false,
        hidden: false,
        orderindex: Some(0),
        space: None,
        folder: None,
        status: None,
        priority: None,
    }
}

/// Create a test user
pub fn test_user() -> User {
    User {
        id: 12345,
        username: "testuser".to_string(),
        color: Some("#2196F3".to_string()),
        email: Some("test@example.com".to_string()),
        profile_picture: None,
    }
}

/// Create a test document user
pub fn test_document_user() -> DocumentUser {
    DocumentUser {
        id: 12345,
        username: "testuser".to_string(),
        email: Some("test@example.com".to_string()),
        profile_picture: None,
    }
}

/// Create a test task status
pub fn test_task_status() -> TaskStatus {
    TaskStatus {
        status: "open".to_string(),
        color: Some("#4CAF50".to_string()),
        type_field: None,
        orderindex: Some(0),
    }
}

/// Create a test task
pub fn test_task() -> Task {
    Task {
        id: "test-task-1".to_string(),
        name: "Test Task".to_string(),
        description: Some("This is a test task".to_string()),
        status: Some(test_task_status()),
        orderindex: Some("0".to_string()),
        content: None,
        created_at: Some(1700000000000),
        updated_at: None,
        closed_at: None,
        creator: Some(test_user()),
        assignees: vec![test_user()],
        checklists: vec![],
        tags: vec![],
        parent: None,
        priority: Some(Priority {
            priority: "high".to_string(),
            color: Some("#F44336".to_string()),
        }),
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

/// Create multiple test tasks
pub fn test_tasks() -> Vec<Task> {
    vec![
        test_task(),
        Task {
            id: "test-task-2".to_string(),
            name: "Another Test Task".to_string(),
            description: Some("Second test task".to_string()),
            status: Some(TaskStatus {
                status: "in progress".to_string(),
                color: Some("#FFC107".to_string()),
                type_field: None,
                orderindex: Some(1),
            }),
            orderindex: Some("1".to_string()),
            content: None,
            created_at: Some(1700000000000),
            updated_at: None,
            closed_at: None,
            creator: Some(test_user()),
            assignees: vec![],
            checklists: vec![],
            tags: vec![],
            parent: None,
            priority: Some(Priority {
                priority: "medium".to_string(),
                color: Some("#FF9800".to_string()),
            }),
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
        },
        Task {
            id: "test-task-3".to_string(),
            name: "Completed Task".to_string(),
            description: None,
            status: Some(TaskStatus {
                status: "complete".to_string(),
                color: Some("#8BC34A".to_string()),
                type_field: None,
                orderindex: Some(2),
            }),
            orderindex: Some("2".to_string()),
            content: None,
            created_at: Some(1700000000000),
            updated_at: None,
            closed_at: Some(1700100000000),
            creator: Some(test_user()),
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
        },
    ]
}

/// Create a test document
pub fn test_document() -> Document {
    Document {
        id: "test-doc-1".to_string(),
        name: "Test Document".to_string(),
        created_at: Some(1700000000000),
        updated_at: None,
        created_by: Some(test_document_user()),
        updated_by: None,
        space: None,
        folder: None,
        url: None,
        pages: vec![],
    }
}

/// Create a test page
pub fn test_page() -> Page {
    Page {
        id: "test-page-1".to_string(),
        name: "Test Page".to_string(),
        content: Some("<h1>Test Content</h1><p>This is test content.</p>".to_string()),
        content_markdown: Some("# Test Content\n\nThis is test content.".to_string()),
        order: Some(0),
        created_at: Some(1700000000000),
        updated_at: None,
        children: vec![],
    }
}
