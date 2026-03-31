//! Assigned items models for unified "Assigned to Me" view
//!
//! This module provides types for representing both assigned tasks and assigned comments
//! in a unified view, enabling users to see all work assigned to them in one place.

use crate::models::{Comment, Task, TaskReference};
use serde::{Deserialize, Serialize};

/// Represents a comment that has been assigned to the current user
/// 
/// This struct wraps a comment with its parent task reference for display
/// in the unified assigned items view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignedComment {
    /// The comment content and metadata
    pub comment: Comment,
    /// Reference to the parent task (for navigation context)
    pub task: TaskReference,
    /// When the comment was assigned to the user (if available)
    #[serde(default, deserialize_with = "crate::utils::deserializers::flexible_timestamp")]
    pub assigned_at: Option<i64>,
}

impl AssignedComment {
    /// Get the updated_at timestamp for sorting
    /// Uses comment's updated_at, falling back to created_at
    pub fn updated_at(&self) -> i64 {
        self.comment.updated_at.unwrap_or(self.comment.created_at.unwrap_or(0))
    }

    /// Get the comment preview text (truncated if needed)
    pub fn preview(&self, max_len: usize) -> String {
        let text = &self.comment.text;
        if text.len() <= max_len {
            text.clone()
        } else {
            format!("{}...", &text[..max_len - 3])
        }
    }
}

/// Represents an assigned work item - either a task or a comment
/// 
/// This enum allows the unified "Assigned to Me" view to display
/// both tasks and comments in a single sorted list while maintaining
/// type safety.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssignedItem {
    /// An assigned task
    Task(Task),
    /// An assigned comment
    AssignedComment(AssignedComment),
}

impl AssignedItem {
    /// Get the unique identifier for this item
    pub fn id(&self) -> String {
        match self {
            AssignedItem::Task(task) => task.id.clone(),
            AssignedItem::AssignedComment(ac) => format!("comment_{}", ac.comment.id),
        }
    }

    /// Get the display name/title for this item
    pub fn name(&self) -> String {
        match self {
            AssignedItem::Task(task) => task.name.clone(),
            AssignedItem::AssignedComment(ac) => ac.preview(50),
        }
    }

    /// Get the updated_at timestamp for sorting
    pub fn updated_at(&self) -> i64 {
        match self {
            AssignedItem::Task(task) => task.updated_at.unwrap_or(0),
            AssignedItem::AssignedComment(ac) => ac.updated_at(),
        }
    }

    /// Get the parent task reference (for navigation)
    pub fn parent_task(&self) -> Option<TaskReference> {
        match self {
            AssignedItem::Task(task) => Some(TaskReference {
                id: task.id.clone(),
                name: Some(task.name.clone()),
            }),
            AssignedItem::AssignedComment(ac) => Some(ac.task.clone()),
        }
    }

    /// Check if this item is a task
    pub fn is_task(&self) -> bool {
        matches!(self, AssignedItem::Task(_))
    }

    /// Check if this item is a comment
    pub fn is_comment(&self) -> bool {
        matches!(self, AssignedItem::AssignedComment(_))
    }
}

/// Filter for assigned items view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssignedItemsFilter {
    /// Show all assigned items (tasks and comments)
    #[default]
    All,
    /// Show only assigned tasks
    TasksOnly,
    /// Show only assigned comments
    CommentsOnly,
}

impl AssignedItemsFilter {
    /// Filter a list of assigned items based on the current filter setting
    pub fn apply(&self, items: &[AssignedItem]) -> Vec<AssignedItem> {
        match self {
            AssignedItemsFilter::All => items.to_vec(),
            AssignedItemsFilter::TasksOnly => {
                items.iter().filter(|item| item.is_task()).cloned().collect()
            }
            AssignedItemsFilter::CommentsOnly => {
                items.iter().filter(|item| item.is_comment()).cloned().collect()
            }
        }
    }

    /// Get the display label for this filter
    pub fn label(&self) -> &'static str {
        match self {
            AssignedItemsFilter::All => "All",
            AssignedItemsFilter::TasksOnly => "Tasks",
            AssignedItemsFilter::CommentsOnly => "Comments",
        }
    }

    /// Cycle to the next filter in sequence
    pub fn next(&self) -> Self {
        match self {
            AssignedItemsFilter::All => AssignedItemsFilter::TasksOnly,
            AssignedItemsFilter::TasksOnly => AssignedItemsFilter::CommentsOnly,
            AssignedItemsFilter::CommentsOnly => AssignedItemsFilter::All,
        }
    }
}

/// Sort assigned items by updated_at descending (most recent first)
pub fn sort_assigned_items(mut items: Vec<AssignedItem>) -> Vec<AssignedItem> {
    items.sort_by(|a, b| {
        let updated_a = a.updated_at();
        let updated_b = b.updated_at();
        // Descending order - most recent first
        updated_b.cmp(&updated_a)
    });
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Comment, Task, TaskReference};

    fn create_test_task(id: &str, name: &str, updated_at: Option<i64>) -> Task {
        Task {
            id: id.to_string(),
            name: name.to_string(),
            updated_at,
            ..Default::default()
        }
    }

    fn create_test_comment(id: &str, text: &str, updated_at: Option<i64>) -> AssignedComment {
        AssignedComment {
            comment: Comment {
                id: id.to_string(),
                text: text.to_string(),
                text_preview: String::new(),
                commenter: None,
                created_at: None,
                updated_at,
                assigned_commenter: None,
                assigned_by: None,
                assigned: false,
                reaction: String::new(),
                parent_id: None,
            },
            task: TaskReference {
                id: "task123".to_string(),
                name: Some("Parent Task".to_string()),
            },
            assigned_at: None,
        }
    }

    #[test]
    fn test_assigned_item_id() {
        let task = create_test_task("task1", "Test Task", Some(1000));
        let item = AssignedItem::Task(task);
        assert_eq!(item.id(), "task1");

        let comment = create_test_comment("comment1", "Test Comment", Some(1000));
        let item = AssignedItem::AssignedComment(comment);
        assert_eq!(item.id(), "comment_comment1");
    }

    #[test]
    fn test_assigned_item_name() {
        let task = create_test_task("task1", "Test Task", Some(1000));
        let item = AssignedItem::Task(task);
        assert_eq!(item.name(), "Test Task");

        let comment = create_test_comment("comment1", "Test Comment Text", Some(1000));
        let item = AssignedItem::AssignedComment(comment);
        assert_eq!(item.name(), "Test Comment Text");
    }

    #[test]
    fn test_assigned_item_is_task_is_comment() {
        let task_item = AssignedItem::Task(create_test_task("task1", "Test", Some(1000)));
        assert!(task_item.is_task());
        assert!(!task_item.is_comment());

        let comment_item = AssignedItem::AssignedComment(create_test_comment("c1", "Test", Some(1000)));
        assert!(comment_item.is_comment());
        assert!(!comment_item.is_task());
    }

    #[test]
    fn test_assigned_item_parent_task() {
        let task = create_test_task("task1", "Test Task", Some(1000));
        let item = AssignedItem::Task(task);
        let parent = item.parent_task();
        assert!(parent.is_some());
        assert_eq!(parent.unwrap().id, "task1");

        let comment = create_test_comment("comment1", "Test", Some(1000));
        let item = AssignedItem::AssignedComment(comment);
        let parent = item.parent_task();
        assert!(parent.is_some());
        assert_eq!(parent.unwrap().id, "task123");
    }

    #[test]
    fn test_sort_assigned_items() {
        let items = vec![
            AssignedItem::Task(create_test_task("old", "Old Task", Some(1000))),
            AssignedItem::Task(create_test_task("new", "New Task", Some(3000))),
            AssignedItem::AssignedComment(create_test_comment("mid", "Mid Comment", Some(2000))),
        ];

        let sorted = sort_assigned_items(items);

        assert_eq!(sorted[0].id(), "new");
        assert_eq!(sorted[1].id(), "comment_mid");
        assert_eq!(sorted[2].id(), "old");
    }

    #[test]
    fn test_assigned_items_filter_apply() {
        let items = vec![
            AssignedItem::Task(create_test_task("task1", "Task 1", Some(1000))),
            AssignedItem::AssignedComment(create_test_comment("comment1", "Comment 1", Some(1000))),
            AssignedItem::Task(create_test_task("task2", "Task 2", Some(1000))),
        ];

        let all = AssignedItemsFilter::All.apply(&items);
        assert_eq!(all.len(), 3);

        let tasks = AssignedItemsFilter::TasksOnly.apply(&items);
        assert_eq!(tasks.len(), 2);
        assert!(tasks.iter().all(|i| i.is_task()));

        let comments = AssignedItemsFilter::CommentsOnly.apply(&items);
        assert_eq!(comments.len(), 1);
        assert!(comments.iter().all(|i| i.is_comment()));
    }

    #[test]
    fn test_assigned_items_filter_next() {
        assert_eq!(AssignedItemsFilter::All.next(), AssignedItemsFilter::TasksOnly);
        assert_eq!(AssignedItemsFilter::TasksOnly.next(), AssignedItemsFilter::CommentsOnly);
        assert_eq!(AssignedItemsFilter::CommentsOnly.next(), AssignedItemsFilter::All);
    }

    #[test]
    fn test_assigned_items_filter_label() {
        assert_eq!(AssignedItemsFilter::All.label(), "All");
        assert_eq!(AssignedItemsFilter::TasksOnly.label(), "Tasks");
        assert_eq!(AssignedItemsFilter::CommentsOnly.label(), "Comments");
    }

    #[test]
    fn test_assigned_comment_preview() {
        let comment = create_test_comment("c1", "This is a short comment", None);
        assert_eq!(comment.preview(50), "This is a short comment");

        let long_comment = create_test_comment(
            "c2",
            "This is a very long comment that should be truncated",
            None,
        );
        assert_eq!(
            long_comment.preview(20),
            "This is a very lo..."
        );
    }
}
