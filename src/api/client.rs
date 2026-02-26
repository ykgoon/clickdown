//! ClickUp API HTTP client
//!
//! **Note on Authentication**: ClickUp does not support username/password authentication
//! for obtaining API tokens programmatically. This client uses Personal API Tokens that
//! must be manually generated from the ClickUp web UI (Settings → Apps → ClickUp API).
//! OAuth 2.0 is available for multi-user applications but requires app registration and
//! a browser-based authorization flow.

use crate::models::{WorkspacesResponse, SpacesResponse, FoldersResponse, ListsResponse, Workspace, ClickUpSpace as Space, Folder, List, Task, DocumentsResponse, DocumentPagesResponse, PageResponse, TasksResponse, CreateTaskRequest, UpdateTaskRequest, DocumentFilters, Document, Page, CommentsResponse, Comment, CreateCommentRequest, UpdateCommentRequest};
use crate::models::TaskFilters;
use crate::api::endpoints::ApiEndpoints;
use crate::api::client_trait::ClickUpApi;
use anyhow::{Context, Result};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use async_trait::async_trait;

/// ClickUp API client
pub struct ClickUpClient {
    client: Client,
    token: String,
}

impl ClickUpClient {
    /// Create a new ClickUp client with the given API token
    pub fn new(token: String) -> Self {
        let client = Client::builder()
            .user_agent("ClickDown/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client, token }
    }

    /// Build a request with authentication headers
    fn request(&self, method: reqwest::Method, url: String) -> reqwest::RequestBuilder {
        self.client
            .request(method, &url)
            .header("Authorization", &self.token)
            .header("Accept", "application/json")
    }

    /// Execute a request and parse the response
    async fn execute<T: DeserializeOwned>(&self, request: reqwest::RequestBuilder) -> Result<T> {
        let response = request.send().await.context("Request failed")?;
        self.parse_response(response).await
    }

    /// Parse an API response, handling errors
    async fn parse_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API error ({}): {}", status, error_text);
        }

        // Get response body as text first for better error messages
        let body = response.text().await.context("Failed to read response body")?;

        // Log the raw response for debugging (but not for comments to avoid logging sensitive data)
        tracing::debug!("API response body: {}", body);

        // Parse the JSON with enhanced error reporting
        // Use serde_path_to_error to get field-level diagnostics
        let mut deserializer = serde_json::Deserializer::from_str(&body);
        serde_path_to_error::deserialize(&mut deserializer)
            .context(format!("Failed to parse response: {}", body.chars().take(200).collect::<String>()))
    }

    // ==================== Workspace/Team ====================

    /// Get all authorized workspaces
    pub async fn get_workspaces(&self) -> Result<Vec<Workspace>> {
        let url = ApiEndpoints::teams();
        let response = self.execute::<WorkspacesResponse>(
            self.request(reqwest::Method::GET, url)
        ).await?;
        Ok(response.teams)
    }

    // ==================== Spaces ====================

    /// Get all spaces in a team/workspace
    pub async fn get_spaces(&self, team_id: &str) -> Result<Vec<Space>> {
        let url = ApiEndpoints::spaces(team_id);
        let response = self.execute::<SpacesResponse>(
            self.request(reqwest::Method::GET, url)
        ).await?;
        Ok(response.spaces)
    }

    /// Get a single space
    pub async fn get_space(&self, space_id: &str) -> Result<Space> {
        let url = ApiEndpoints::space(space_id);
        self.execute::<Space>(
            self.request(reqwest::Method::GET, url)
        ).await
    }

    // ==================== Folders ====================

    /// Get all folders in a space
    pub async fn get_folders(&self, space_id: &str) -> Result<Vec<Folder>> {
        let url = ApiEndpoints::folders(space_id);
        let response = self.execute::<FoldersResponse>(
            self.request(reqwest::Method::GET, url)
        ).await?;
        Ok(response.folders)
    }

    // ==================== Lists ====================

    /// Get all lists in a folder
    pub async fn get_lists_in_folder(&self, folder_id: &str, archived: Option<bool>) -> Result<Vec<List>> {
        let mut url = ApiEndpoints::lists_in_folder(folder_id);
        if let Some(archived) = archived {
            url.push_str(&format!("?archived={}", archived));
        }
        let response = self.execute::<ListsResponse>(
            self.request(reqwest::Method::GET, url)
        ).await?;
        Ok(response.lists)
    }

    /// Get all lists in a space (folderless lists)
    pub async fn get_lists_in_space(&self, space_id: &str, archived: Option<bool>) -> Result<Vec<List>> {
        let mut url = ApiEndpoints::lists_in_space(space_id);
        if let Some(archived) = archived {
            url.push_str(&format!("?archived={}", archived));
        }
        let response = self.execute::<ListsResponse>(
            self.request(reqwest::Method::GET, url)
        ).await?;
        Ok(response.lists)
    }

    // ==================== Tasks ====================

    /// Get all tasks in a list
    pub async fn get_tasks(&self, list_id: &str, filters: &TaskFilters) -> Result<Vec<Task>> {
        let query = filters.to_query_string();
        let url = ApiEndpoints::tasks_in_list(list_id, &query);
        let response = self.execute::<TasksResponse>(
            self.request(reqwest::Method::GET, url)
        ).await?;
        Ok(response.tasks)
    }

    /// Get a single task
    pub async fn get_task(&self, task_id: &str) -> Result<Task> {
        let url = ApiEndpoints::task(task_id);
        self.execute::<Task>(
            self.request(reqwest::Method::GET, url)
        ).await
    }

    /// Create a new task
    pub async fn create_task(&self, list_id: &str, task: &CreateTaskRequest) -> Result<Task> {
        let url = ApiEndpoints::tasks_in_list(list_id, "");
        self.execute::<Task>(
            self.request(reqwest::Method::POST, url)
                .json(task)
        ).await
    }

    /// Update a task
    pub async fn update_task(&self, task_id: &str, task: &UpdateTaskRequest) -> Result<Task> {
        let url = ApiEndpoints::task(task_id);
        self.execute::<Task>(
            self.request(reqwest::Method::PUT, url)
                .json(task)
        ).await
    }

    /// Delete a task
    pub async fn delete_task(&self, task_id: &str) -> Result<()> {
        let url = ApiEndpoints::task(task_id);
        let response = self.request(reqwest::Method::DELETE, url)
            .send()
            .await
            .context("Request failed")?;
        
        self.parse_response::<serde_json::Value>(response).await?;
        Ok(())
    }

    // ==================== Documents ====================

    /// Search documents
    pub async fn search_docs(&self, filters: &DocumentFilters) -> Result<Vec<Document>> {
        let query = filters.to_query_string();
        let url = ApiEndpoints::docs(&query);
        let response = self.execute::<DocumentsResponse>(
            self.request(reqwest::Method::GET, url)
        ).await?;
        Ok(response.docs)
    }

    /// Get all pages in a document
    pub async fn get_doc_pages(&self, doc_id: &str) -> Result<Vec<Page>> {
        let url = ApiEndpoints::doc_pages(doc_id);
        let response = self.execute::<DocumentPagesResponse>(
            self.request(reqwest::Method::GET, url)
        ).await?;
        Ok(response.pages)
    }

    /// Get a single page
    pub async fn get_page(&self, page_id: &str) -> Result<Page> {
        let url = ApiEndpoints::page(page_id);
        let response = self.execute::<PageResponse>(
            self.request(reqwest::Method::GET, url)
        ).await?;
        Ok(response.page)
    }

    // ==================== Comments ====================

    /// Get all comments for a task (top-level only)
    pub async fn get_task_comments(&self, task_id: &str) -> Result<Vec<Comment>> {
        let url = ApiEndpoints::task_comments(task_id);
        let response = self.execute::<CommentsResponse>(
            self.request(reqwest::Method::GET, url)
        ).await?;
        Ok(response.comments)
    }

    /// Get replies to a specific comment (threaded comments)
    pub async fn get_comment_replies(&self, comment_id: &str) -> Result<Vec<Comment>> {
        let url = ApiEndpoints::comment_replies(comment_id);
        let response = self.execute::<CommentsResponse>(
            self.request(reqwest::Method::GET, url)
        ).await?;
        Ok(response.comments)
    }

    /// Create a new comment on a task (top-level)
    pub async fn create_comment(&self, task_id: &str, comment: &CreateCommentRequest) -> Result<Comment> {
        let url = ApiEndpoints::task_comments(task_id);
        let response = self.execute::<Comment>(
            self.request(reqwest::Method::POST, url)
                .json(comment)
        ).await?;
        Ok(response)
    }

    /// Create a reply to an existing comment (threaded)
    pub async fn create_comment_reply(&self, parent_comment_id: &str, comment: &CreateCommentRequest) -> Result<Comment> {
        let url = ApiEndpoints::comment_replies(parent_comment_id);
        let response = self.execute::<Comment>(
            self.request(reqwest::Method::POST, url)
                .json(comment)
        ).await?;
        Ok(response)
    }

    /// Update a comment
    pub async fn update_comment(&self, comment_id: &str, comment: &UpdateCommentRequest) -> Result<Comment> {
        let url = ApiEndpoints::comment(comment_id);
        let response = self.execute::<Comment>(
            self.request(reqwest::Method::PUT, url)
                .json(comment)
        ).await?;
        Ok(response)
    }
}

/// Implement the ClickUpApi trait for ClickUpClient
#[async_trait]
impl ClickUpApi for ClickUpClient {
    async fn get_workspaces(&self) -> Result<Vec<Workspace>> {
        self.get_workspaces().await
    }

    async fn get_spaces(&self, team_id: &str) -> Result<Vec<Space>> {
        self.get_spaces(team_id).await
    }

    async fn get_space(&self, space_id: &str) -> Result<Space> {
        self.get_space(space_id).await
    }

    async fn get_folders(&self, space_id: &str) -> Result<Vec<Folder>> {
        self.get_folders(space_id).await
    }

    async fn get_lists_in_folder(
        &self,
        folder_id: &str,
        archived: Option<bool>,
    ) -> Result<Vec<List>> {
        self.get_lists_in_folder(folder_id, archived).await
    }

    async fn get_lists_in_space(
        &self,
        space_id: &str,
        archived: Option<bool>,
    ) -> Result<Vec<List>> {
        self.get_lists_in_space(space_id, archived).await
    }

    async fn get_tasks(&self, list_id: &str, filters: &TaskFilters) -> Result<Vec<Task>> {
        self.get_tasks(list_id, filters).await
    }

    async fn get_task(&self, task_id: &str) -> Result<Task> {
        self.get_task(task_id).await
    }

    async fn create_task(&self, list_id: &str, task: &CreateTaskRequest) -> Result<Task> {
        self.create_task(list_id, task).await
    }

    async fn update_task(&self, task_id: &str, task: &UpdateTaskRequest) -> Result<Task> {
        self.update_task(task_id, task).await
    }

    async fn delete_task(&self, task_id: &str) -> Result<()> {
        self.delete_task(task_id).await
    }

    async fn search_docs(&self, filters: &DocumentFilters) -> Result<Vec<Document>> {
        self.search_docs(filters).await
    }

    async fn get_doc_pages(&self, doc_id: &str) -> Result<Vec<Page>> {
        self.get_doc_pages(doc_id).await
    }

    async fn get_page(&self, page_id: &str) -> Result<Page> {
        self.get_page(page_id).await
    }

    async fn get_task_comments(&self, task_id: &str) -> Result<Vec<Comment>> {
        self.get_task_comments(task_id).await
    }

    async fn get_comment_replies(&self, comment_id: &str) -> Result<Vec<Comment>> {
        self.get_comment_replies(comment_id).await
    }

    async fn create_comment(&self, task_id: &str, comment: &CreateCommentRequest) -> Result<Comment> {
        self.create_comment(task_id, comment).await
    }

    async fn create_comment_reply(&self, parent_comment_id: &str, comment: &CreateCommentRequest) -> Result<Comment> {
        self.create_comment_reply(parent_comment_id, comment).await
    }

    async fn update_comment(&self, comment_id: &str, comment: &UpdateCommentRequest) -> Result<Comment> {
        self.update_comment(comment_id, comment).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_error_includes_field_path() {
        // This test demonstrates that serde_path_to_error provides field-level diagnostics
        // We simulate what would happen with malformed comment data
        
        let malformed_json = json!({
            "id": "123",
            "comment_text": "Test",
            "date": 1234567890.5  // Float instead of int/string
        });

        let body = malformed_json.to_string();
        let mut deserializer = serde_json::Deserializer::from_str(&body);
        let result: Result<Comment, _> = serde_path_to_error::deserialize(&mut deserializer);
        
        assert!(result.is_err(), "Float timestamp should fail to parse");
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        
        // Error message should include field path information
        // serde_path_to_error formats errors as: "path.to.field: error message"
        assert!(err_msg.contains("date") || err_msg.contains("invalid") || err_msg.contains("type"),
                "Error should mention the problematic field: {}", err_msg);
    }

    #[test]
    fn test_parse_error_with_nested_field() {
        // Test error reporting for nested fields (e.g., in User object)
        let malformed_json = json!({
            "id": "123",
            "comment_text": "Test",
            "user": {
                "id": "not-a-number",  // Should be i64
                "username": "test"
            }
        });

        let body = malformed_json.to_string();
        let mut deserializer = serde_json::Deserializer::from_str(&body);
        let result: Result<Comment, _> = serde_path_to_error::deserialize(&mut deserializer);
        
        assert!(result.is_err(), "String user id should fail to parse");
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        
        // Error should indicate the nested field path
        assert!(err_msg.contains("user") || err_msg.contains("id") || err_msg.contains("invalid"),
                "Error should mention the nested field: {}", err_msg);
    }
}
