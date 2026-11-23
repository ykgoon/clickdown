//! Trait definition for ClickUp API client to enable mocking

use crate::models::{
    ClickUpSpace, Comment, CreateCommentRequest, CreateTaskRequest, Document, DocumentFilters,
    Folder, List, Task, TaskFilters, UpdateCommentRequest, UpdateTaskRequest, User, Workspace,
};
use anyhow::Result;

/// Trait defining the ClickUp API interface for dependency injection
/// This enables mocking the API for headless testing
// `async_trait` re-emits `#[must_use]` on every generated future, which the
// `must_use` lint then flags once per method. The attribute comes from the
// macro, not this crate, so the lint is silenced here rather than 25 times.
#[allow(clippy::double_must_use)]
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

    /// Get a single list, including its full status configuration
    async fn get_list(&self, list_id: &str) -> Result<List>;

    // ==================== Tasks ====================

    /// Get all tasks in a list
    async fn get_tasks(&self, list_id: &str, filters: &TaskFilters) -> Result<Vec<Task>>;

    /// Get a single task
    async fn get_task(&self, task_id: &str) -> Result<Task>;

    /// Get a single task, optionally using team context to resolve custom
    /// task IDs when the plain lookup fails. Default: plain lookup only.
    async fn get_task_with_context(&self, task_id: &str, team_id: Option<&str>) -> Result<Task> {
        let _ = team_id;
        self.get_task(task_id).await
    }

    /// Create a new task
    async fn create_task(&self, list_id: &str, task: &CreateTaskRequest) -> Result<Task>;

    /// Update a task
    async fn update_task(&self, task_id: &str, task: &UpdateTaskRequest) -> Result<Task>;

    /// Delete a task
    async fn delete_task(&self, task_id: &str) -> Result<()>;

    // ==================== Members ====================

    /// Get all members who can access a list
    async fn get_list_members(&self, list_id: &str) -> Result<Vec<User>>;

    // ==================== Documents ====================

    /// Search documents
    async fn search_docs(&self, filters: &DocumentFilters) -> Result<Vec<Document>>;

    // ==================== Comments ====================

    /// Get all comments for a task (top-level only)
    async fn get_task_comments(&self, task_id: &str) -> Result<Vec<Comment>>;

    /// Get replies to a specific comment (threaded comments)
    async fn get_comment_replies(&self, comment_id: &str) -> Result<Vec<Comment>>;

    /// Create a new comment on a task (top-level)
    async fn create_comment(
        &self,
        task_id: &str,
        comment: &CreateCommentRequest,
    ) -> Result<Comment>;

    /// Create a reply to an existing comment (threaded)
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

    /// Add a reaction (e.g. `thumbsup`) to a comment
    async fn add_comment_reaction(&self, comment_id: &str, reaction: &str) -> Result<()>;

    /// Remove a reaction from a comment
    async fn remove_comment_reaction(&self, comment_id: &str, reaction: &str) -> Result<()>;

    // ==================== Inbox ====================

    /// The workspace inbox: items needing the current user's attention.
    ///
    /// Backed by the undocumented `GET /team/{id}/inbox`. Callers must treat
    /// an error as "inbox unavailable" rather than fatal, since the endpoint
    /// is not part of the published API.
    async fn get_inbox(&self, team_id: &str) -> Result<crate::models::Inbox>;

    // ==================== Assigned Tasks ====================

    /// Tasks assigned to a user across the whole Workspace.
    ///
    /// Backed by Get Filtered Team Tasks, which is the only endpoint that
    /// filters by assignee without being scoped to a single list.
    async fn get_team_tasks_with_assignee(
        &self,
        team_id: &str,
        user_id: i32,
        page: u32,
        statuses: Option<Vec<String>>,
    ) -> Result<Vec<Task>>;

    /// Get tasks assigned to a specific user from a list
    async fn get_tasks_with_assignee(
        &self,
        list_id: &str,
        user_id: i32,
        limit: Option<i32>,
    ) -> Result<Vec<Task>>;
}
