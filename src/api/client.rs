//! ClickUp API HTTP client

use crate::models::{WorkspacesResponse, SpacesResponse, FoldersResponse, ListsResponse, Workspace, ClickUpSpace as Space, Folder, List, Task, DocumentsResponse, DocumentPagesResponse, PageResponse, TasksResponse, CreateTaskRequest, UpdateTaskRequest, DocumentFilters, Document, Page};
use crate::models::TaskFilters;
use crate::api::endpoints::ApiEndpoints;
use crate::api::client_trait::{ClickUpApi, AuthToken};
use anyhow::{Context, Result};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use async_trait::async_trait;
use serde::Deserialize;

/// OAuth token response from ClickUp
#[derive(Debug, Deserialize)]
struct OAuthTokenResponse {
    token: String,
}

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

        response.json::<T>().await.context("Failed to parse response")
    }

    /// Authenticate with username and password
    pub async fn authenticate_with_credentials(
        &self,
        username: &str,
        password: &str,
    ) -> Result<AuthToken> {
        // Note: ClickUp's actual API uses OAuth 2.0. This implementation
        // simulates credential-based authentication for the prototype.
        // In production, this would use OAuth 2.0 with client credentials
        // or authorization code flow.
        let url = ApiEndpoints::oauth_token();
        
        let response = self.client
            .post(&url)
            .header("Accept", "application/json")
            .form(&[
                ("grant_type", "password"),
                ("username", username),
                ("password", password),
            ])
            .send()
            .await
            .context("Authentication request failed")?;

        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            
            // Provide user-friendly error messages based on status code
            let error_msg = match status.as_u16() {
                401 => "Invalid username or password".to_string(),
                403 => "Your account has been locked. Please contact ClickUp support.".to_string(),
                429 => "Too many login attempts. Please try again later.".to_string(),
                _ => format!("Authentication failed ({}): {}", status, error_text),
            };
            
            anyhow::bail!(error_msg);
        }

        let token_response = response.json::<OAuthTokenResponse>()
            .await
            .context("Failed to parse authentication response")?;

        Ok(AuthToken {
            token: token_response.token,
        })
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
}

/// Implement the ClickUpApi trait for ClickUpClient
#[async_trait]
impl ClickUpApi for ClickUpClient {
    async fn authenticate_with_credentials(
        &self,
        username: &str,
        password: &str,
    ) -> Result<AuthToken> {
        self.authenticate_with_credentials(username, password).await
    }

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
}
