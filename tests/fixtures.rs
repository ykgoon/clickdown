//! Test fixtures for ClickDown tests

use clickdown::models::comment::Comment;
use clickdown::models::document::Document;
use clickdown::models::notification::Notification;
use clickdown::models::inbox_activity::{InboxActivity, ActivityType};
use clickdown::models::task::Task;
use clickdown::models::workspace::{Folder, List, Space, Workspace};

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
pub fn test_comment_user() -> clickdown::models::User {
    clickdown::models::User {
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

/// Create a test user (for task assignees)
pub fn test_user() -> clickdown::models::User {
    clickdown::models::User {
        id: 123,
        username: "testuser".to_string(),
        color: None,
        email: Some("test@example.com".to_string()),
        profile_picture: None,
        initials: Some("TU".to_string()),
    }
}

/// Create a test task with assignees
pub fn test_task_with_assignee() -> Task {
    let mut task = test_task();
    task.id = "test-task-assigned-1".to_string();
    task.name = "Task assigned to test user".to_string();
    task.assignees = vec![test_user()];
    task
}

/// Create multiple test tasks with assignees
pub fn test_tasks_with_assignees() -> Vec<Task> {
    vec![
        {
            let mut task = test_task();
            task.id = "assigned-task-1".to_string();
            task.name = "Review Q2 planning doc".to_string();
            task.assignees = vec![test_user()];
            task
        },
        {
            let mut task = test_task();
            task.id = "assigned-task-2".to_string();
            task.name = "Fix bug in task filtering".to_string();
            task.assignees = vec![test_user()];
            task
        },
        {
            let mut task = test_task();
            task.id = "assigned-task-3".to_string();
            task.name = "Update API documentation".to_string();
            task.assignees = vec![test_user()];
            task
        },
    ]
}

/// Create a test notification
pub fn test_notification() -> Notification {
    Notification {
        id: "notif-1".to_string(),
        workspace_id: "ws-1".to_string(),
        title: "Task assigned to you".to_string(),
        description: "You were assigned to 'Review pull request'".to_string(),
        created_at: Some(1704067200000),
        read_at: None,
    }
}

/// Create a test notification that has been read
pub fn test_notification_read() -> Notification {
    Notification {
        id: "notif-2".to_string(),
        workspace_id: "ws-1".to_string(),
        title: "Status change".to_string(),
        description: "Task status changed to Complete".to_string(),
        created_at: Some(1704153600000),
        read_at: Some(1704240000000),
    }
}

/// Create multiple test notifications
pub fn test_notifications() -> Vec<Notification> {
    vec![
        Notification {
            id: "notif-1".to_string(),
            workspace_id: "ws-1".to_string(),
            title: "Task assigned to you".to_string(),
            description: "You were assigned to 'Review pull request'".to_string(),
            created_at: Some(1704067200000),
            read_at: None,
        },
        Notification {
            id: "notif-2".to_string(),
            workspace_id: "ws-1".to_string(),
            title: "Comment on task".to_string(),
            description: "New comment on 'Deploy to production'".to_string(),
            created_at: Some(1704153600000),
            read_at: None,
        },
        Notification {
            id: "notif-3".to_string(),
            workspace_id: "ws-1".to_string(),
            title: "Status change".to_string(),
            description: "Task status changed to Complete".to_string(),
            created_at: Some(1704240000000),
            read_at: None,
        },
    ]
}

// ============================================================================
// Inbox Activity Fixtures
// ============================================================================

/// Create a test inbox activity (assignment)
pub fn test_inbox_activity_assignment() -> InboxActivity {
    InboxActivity {
        id: "activity-1".to_string(),
        activity_type: ActivityType::Assignment,
        title: "Task assigned to you".to_string(),
        description: "You were assigned to 'Review pull request'".to_string(),
        timestamp: 1704067200000,
        task_id: Some("task-123".to_string()),
        comment_id: None,
        workspace_id: "ws-1".to_string(),
        task_name: "Review pull request".to_string(),
        previous_status: None,
        new_status: None,
        due_date: None,
    }
}

/// Create a test inbox activity (comment)
pub fn test_inbox_activity_comment() -> InboxActivity {
    InboxActivity {
        id: "activity-2".to_string(),
        activity_type: ActivityType::Comment,
        title: "New comment on task".to_string(),
        description: "John added a comment: 'Looks good to me!'".to_string(),
        timestamp: 1704153600000,
        task_id: Some("task-456".to_string()),
        comment_id: Some("comment-789".to_string()),
        workspace_id: "ws-1".to_string(),
        task_name: "Deploy to production".to_string(),
        previous_status: None,
        new_status: None,
        due_date: None,
    }
}

/// Create a test inbox activity (status change)
pub fn test_inbox_activity_status_change() -> InboxActivity {
    InboxActivity {
        id: "activity-3".to_string(),
        activity_type: ActivityType::StatusChange,
        title: "Status changed".to_string(),
        description: "Task status changed from 'In Progress' to 'Complete'".to_string(),
        timestamp: 1704240000000,
        task_id: Some("task-789".to_string()),
        comment_id: None,
        workspace_id: "ws-1".to_string(),
        task_name: "Update documentation".to_string(),
        previous_status: Some("In Progress".to_string()),
        new_status: Some("Complete".to_string()),
        due_date: None,
    }
}

/// Create a test inbox activity (due date approaching)
pub fn test_inbox_activity_due_date() -> InboxActivity {
    InboxActivity {
        id: "activity-4".to_string(),
        activity_type: ActivityType::DueDate,
        title: "Due date approaching".to_string(),
        description: "Task is due in 2 days".to_string(),
        timestamp: 1704326400000,
        task_id: Some("task-999".to_string()),
        comment_id: None,
        workspace_id: "ws-1".to_string(),
        task_name: "Submit quarterly report".to_string(),
        previous_status: None,
        new_status: None,
        due_date: Some(1704499200000),
    }
}

/// Create multiple test inbox activities
pub fn test_inbox_activities() -> Vec<InboxActivity> {
    vec![
        test_inbox_activity_assignment(),
        test_inbox_activity_comment(),
        test_inbox_activity_status_change(),
        test_inbox_activity_due_date(),
    ]
}
