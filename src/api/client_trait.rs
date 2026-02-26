//! Trait definition for ClickUp API client to enable mocking

use crate::models::{
    Workspace, ClickUpSpace, Folder, List, Task, TaskFilters,
    CreateTaskRequest, UpdateTaskRequest, Document, Page, DocumentFilters,
    Comment, CreateCommentRequest, UpdateCommentRequest,
};
use anyhow::Result;

/// Authentication result containing the API token
#[derive(Clone)]
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

    // ==================== Spaces ====================

    /// Get all spaces in a team/workspace
    async fn get_spaces(&self, team_id: &str) -> Result<Vec<ClickUpSpace>>;

    /// Get a single space
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
    async fn get_lists_in_space(
        &self,
        space_id: &str,
        archived: Option<bool>,
    ) -> Result<Vec<List>>;

    // ==================== Tasks ====================

    /// Get all tasks in a list
    async fn get_tasks(&self, list_id: &str, filters: &TaskFilters) -> Result<Vec<Task>>;

    /// Get a single task
    async fn get_task(&self, task_id: &str) -> Result<Task>;

    /// Create a new task
    async fn create_task(&self, list_id: &str, task: &CreateTaskRequest) -> Result<Task>;

    /// Update a task
    async fn update_task(&self, task_id: &str, task: &UpdateTaskRequest) -> Result<Task>;

    /// Delete a task
    async fn delete_task(&self, task_id: &str) -> Result<()>;

    // ==================== Documents ====================

    /// Search documents
    async fn search_docs(&self, filters: &DocumentFilters) -> Result<Vec<Document>>;

    /// Get all pages in a document
    async fn get_doc_pages(&self, doc_id: &str) -> Result<Vec<Page>>;

    /// Get a single page
    async fn get_page(&self, page_id: &str) -> Result<Page>;

    // ==================== Comments ====================

    /// Get all comments for a task (top-level only)
    async fn get_task_comments(&self, task_id: &str) -> Result<Vec<Comment>>;

    /// Get replies to a specific comment (threaded comments)
    async fn get_comment_replies(&self, comment_id: &str) -> Result<Vec<Comment>>;

    /// Create a new comment on a task (top-level)
    async fn create_comment(&self, task_id: &str, comment: &CreateCommentRequest) -> Result<Comment>;

    /// Create a reply to an existing comment (threaded)
    async fn create_comment_reply(&self, parent_comment_id: &str, comment: &CreateCommentRequest) -> Result<Comment>;

    /// Update a comment
    async fn update_comment(&self, comment_id: &str, comment: &UpdateCommentRequest) -> Result<Comment>;
}
