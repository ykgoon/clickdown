//! Trait definition for ClickUp API client to enable mocking

use crate::models::{
    ClickUpSpace, Comment, CreateCommentRequest, CreateTaskRequest, Document, DocumentFilters,
    Folder, List, Notification, Page, Task, TaskFilters, UpdateCommentRequest, UpdateTaskRequest,
    User, Workspace,
};
use anyhow::Result;

/// Authentication result containing the API token
#[derive(Clone)]
#[allow(dead_code)]
pub struct AuthToken {
    pub token: String,
}

/// Trait defining the ClickUp API interface for dependency injection
/// This enables mocking the API for headless testing
#[async_trait::async_trait]
pub trait ClickUpApi: Send + Sync {
    // ==================== Workspace/Team ====================

    /// Get all authorized workspaces
    async fn get_workspaces(&self) -> Result<Vec<Workspace>>;

    // ==================== User ====================

    /// Get the current authenticated user's profile
    async fn get_current_user(&self) -> Result<User>;

    // ==================== Spaces ====================

    /// Get all spaces in a team/workspace
    async fn get_spaces(&self, team_id: &str) -> Result<Vec<ClickUpSpace>>;

    /// Get a single space
    #[allow(dead_code)]
    async fn get_space(&self, space_id: &str) -> Result<ClickUpSpace>;

    // ==================== Folders ====================

    /// Get all folders in a space
    async fn get_folders(&self, space_id: &str) -> Result<Vec<Folder>>;

    // ==================== Lists ====================

    /// Get all lists in a folder
    async fn get_lists_in_folder(
        &self,
        folder_id: &str,
        archived: Option<bool>,
    ) -> Result<Vec<List>>;

    /// Get all lists in a space (folderless lists)
    async fn get_lists_in_space(&self, space_id: &str, archived: Option<bool>)
        -> Result<Vec<List>>;

    // ==================== Tasks ====================

    /// Get all tasks in a list
    async fn get_tasks(&self, list_id: &str, filters: &TaskFilters) -> Result<Vec<Task>>;

    /// Get a single task
    async fn get_task(&self, task_id: &str) -> Result<Task>;

    /// Create a new task
    #[allow(dead_code)]
    async fn create_task(&self, list_id: &str, task: &CreateTaskRequest) -> Result<Task>;

    /// Update a task
    #[allow(dead_code)]
    async fn update_task(&self, task_id: &str, task: &UpdateTaskRequest) -> Result<Task>;

    /// Delete a task
    #[allow(dead_code)]
    async fn delete_task(&self, task_id: &str) -> Result<()>;

    // ==================== Documents ====================

    /// Search documents
    async fn search_docs(&self, filters: &DocumentFilters) -> Result<Vec<Document>>;

    /// Get all pages in a document
    #[allow(dead_code)]
    async fn get_doc_pages(&self, doc_id: &str) -> Result<Vec<Page>>;

    /// Get a single page
    #[allow(dead_code)]
    async fn get_page(&self, page_id: &str) -> Result<Page>;

    // ==================== Comments ====================

    /// Get all comments for a task (top-level only)
    #[allow(dead_code)]
    async fn get_task_comments(&self, task_id: &str) -> Result<Vec<Comment>>;

    /// Get replies to a specific comment (threaded comments)
    #[allow(dead_code)]
    async fn get_comment_replies(&self, comment_id: &str) -> Result<Vec<Comment>>;

    /// Create a new comment on a task (top-level)
    async fn create_comment(
        &self,
        task_id: &str,
        comment: &CreateCommentRequest,
    ) -> Result<Comment>;

    /// Create a reply to an existing comment (threaded)
    #[allow(dead_code)]
    async fn create_comment_reply(
        &self,
        parent_comment_id: &str,
        comment: &CreateCommentRequest,
    ) -> Result<Comment>;

    /// Update a comment
    async fn update_comment(
        &self,
        comment_id: &str,
        comment: &UpdateCommentRequest,
    ) -> Result<Comment>;

    // ==================== Notifications ====================

    /// Get notifications for a workspace
    /// Note: This endpoint doesn't exist in ClickUp API v2 - kept for backward compatibility
    /// Use get_inbox_activity() instead for the smart inbox feature
    #[allow(dead_code)]
    async fn get_notifications(&self, workspace_id: &str) -> Result<Vec<Notification>>;

    // ==================== Smart Inbox / Activity Feed ====================

    /// Get tasks assigned to a user with optional date filter
    #[allow(dead_code)]
    async fn get_tasks_assigned_to_user(
        &self,
        team_id: &str,
        user_id: i32,
        date_updated_gt: Option<i64>,
    ) -> Result<Vec<Task>>;

    /// Get comments for multiple tasks (batched fetch)
    #[allow(dead_code)]
    async fn get_comments_for_tasks(
        &self,
        task_ids: &[String],
        date_created_gt: Option<i64>,
    ) -> Result<Vec<Comment>>;

    /// Get tasks with due dates before a specified timestamp
    #[allow(dead_code)]
    async fn get_tasks_with_due_dates(
        &self,
        team_id: &str,
        due_date_before: i64,
        date_updated_gt: Option<i64>,
    ) -> Result<Vec<Task>>;

    /// Get inbox activity by aggregating assignments, comments, status changes, and due dates
    async fn get_inbox_activity(
        &self,
        team_id: &str,
        user_id: i32,
        since_timestamp: Option<i64>,
    ) -> Result<crate::models::InboxActivityResponse>;

    // ==================== Assigned Tasks ====================

    /// Get all lists accessible to the user (for fetching assigned tasks)
    async fn get_all_accessible_lists(&self) -> Result<Vec<List>>;

    /// Get tasks assigned to a specific user from a list
    async fn get_tasks_with_assignee(
        &self,
        list_id: &str,
        user_id: i32,
        limit: Option<i32>,
    ) -> Result<Vec<Task>>;

    // ==================== Assigned Comments ====================

    /// Get comments assigned to a specific user from a task
    async fn get_comments_with_assigned_commenter(
        &self,
        task_id: &str,
        user_id: i32,
    ) -> Result<Vec<Comment>>;

    /// Get all comments assigned to a user across all accessible lists
    async fn get_assigned_comments(&self, user_id: i32) -> Result<Vec<crate::models::AssignedComment>>;
}
