//! Mock ClickUp API client for testing

use crate::api::client_trait::{ClickUpApi, AuthToken};
use crate::models::{
    Workspace, ClickUpSpace, Folder, List, Task, TaskFilters,
    CreateTaskRequest, UpdateTaskRequest, Document, Page, DocumentFilters,
};
use anyhow::{Result, anyhow};

/// Mock ClickUp API client for headless testing
///
/// This client implements the ClickUpApi trait and can be configured
/// to return predefined responses for testing without making actual
/// network calls.
#[derive(Default)]
pub struct MockClickUpClient {
    /// Override for authenticate_with_credentials response
    pub auth_credentials_response: Option<Result<AuthToken>>,
    /// Override for get_workspaces response
    pub workspaces_response: Option<Result<Vec<Workspace>>>,
    /// Override for get_spaces response
    pub spaces_response: Option<Result<Vec<ClickUpSpace>>>,
    /// Override for get_folders response
    pub folders_response: Option<Result<Vec<Folder>>>,
    /// Override for get_lists_in_folder response
    pub lists_in_folder_response: Option<Result<Vec<List>>>,
    /// Override for get_lists_in_space response
    pub lists_in_space_response: Option<Result<Vec<List>>>,
    /// Override for get_tasks response
    pub tasks_response: Option<Result<Vec<Task>>>,
    /// Override for get_task response
    pub task_response: Option<Result<Task>>,
    /// Override for create_task response
    pub create_task_response: Option<Result<Task>>,
    /// Override for update_task response
    pub update_task_response: Option<Result<Task>>,
    /// Override for delete_task response
    pub delete_task_response: Option<Result<()>>,
    /// Override for search_docs response
    pub search_docs_response: Option<Result<Vec<Document>>>,
    /// Override for get_doc_pages response
    pub doc_pages_response: Option<Result<Vec<Page>>>,
    /// Override for get_page response
    pub page_response: Option<Result<Page>>,
}

impl MockClickUpClient {
    /// Create a new mock client with default (empty) responses
    pub fn new() -> Self {
        Self {
            auth_credentials_response: None,
            workspaces_response: None,
            spaces_response: None,
            folders_response: None,
            lists_in_folder_response: None,
            lists_in_space_response: None,
            tasks_response: None,
            task_response: None,
            create_task_response: None,
            update_task_response: None,
            delete_task_response: None,
            search_docs_response: None,
            doc_pages_response: None,
            page_response: None,
        }
    }

    /// Set successful credential authentication response
    pub fn with_auth_success(mut self, token: &str) -> Self {
        self.auth_credentials_response = Some(Ok(AuthToken {
            token: token.to_string(),
        }));
        self
    }

    /// Set credential authentication error
    pub fn with_auth_error(mut self, error: String) -> Self {
        self.auth_credentials_response = Some(Err(anyhow!(error)));
        self
    }

    /// Set the workspaces response
    pub fn with_workspaces(mut self, workspaces: Vec<Workspace>) -> Self {
        self.workspaces_response = Some(Ok(workspaces));
        self
    }

    /// Set the workspaces error
    pub fn with_workspaces_error(mut self, error: String) -> Self {
        self.workspaces_response = Some(Err(anyhow!(error)));
        self
    }

    /// Set the spaces response
    pub fn with_spaces(mut self, spaces: Vec<ClickUpSpace>) -> Self {
        self.spaces_response = Some(Ok(spaces));
        self
    }

    /// Set the folders response
    pub fn with_folders(mut self, folders: Vec<Folder>) -> Self {
        self.folders_response = Some(Ok(folders));
        self
    }

    /// Set the lists in folder response
    pub fn with_lists_in_folder(mut self, lists: Vec<List>) -> Self {
        self.lists_in_folder_response = Some(Ok(lists));
        self
    }

    /// Set the tasks response
    pub fn with_tasks(mut self, tasks: Vec<Task>) -> Self {
        self.tasks_response = Some(Ok(tasks));
        self
    }

    /// Set the task response
    pub fn with_task(mut self, task: Task) -> Self {
        self.task_response = Some(Ok(task));
        self
    }

    /// Set the create task response
    pub fn with_create_task_response(mut self, task: Task) -> Self {
        self.create_task_response = Some(Ok(task));
        self
    }

    /// Set the update task response
    pub fn with_update_task_response(mut self, task: Task) -> Self {
        self.update_task_response = Some(Ok(task));
        self
    }

    /// Set the delete task response
    pub fn with_delete_task_success(mut self) -> Self {
        self.delete_task_response = Some(Ok(()));
        self
    }

    /// Set the documents response
    pub fn with_documents(mut self, documents: Vec<Document>) -> Self {
        self.search_docs_response = Some(Ok(documents));
        self
    }

    /// Set the pages response
    pub fn with_pages(mut self, pages: Vec<Page>) -> Self {
        self.doc_pages_response = Some(Ok(pages));
        self
    }
}

#[async_trait::async_trait]
impl ClickUpApi for MockClickUpClient {
    async fn authenticate_with_credentials(
        &self,
        _username: &str,
        _password: &str,
    ) -> Result<AuthToken> {
        match &self.auth_credentials_response {
            Some(Ok(token)) => Ok(token.clone()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Err(anyhow!("Credential authentication not configured")),
        }
    }

    async fn get_workspaces(&self) -> Result<Vec<Workspace>> {
        match &self.workspaces_response {
            Some(Ok(workspaces)) => Ok(workspaces.clone()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Ok(vec![]), // Default to empty list
        }
    }

    async fn get_spaces(&self, _team_id: &str) -> Result<Vec<ClickUpSpace>> {
        match &self.spaces_response {
            Some(Ok(spaces)) => Ok(spaces.clone()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Ok(vec![]),
        }
    }

    async fn get_space(&self, _space_id: &str) -> Result<ClickUpSpace> {
        match &self.spaces_response {
            Some(Ok(spaces)) => spaces.first().cloned().ok_or_else(|| anyhow!("Space not found")),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Err(anyhow!("No spaces configured")),
        }
    }

    async fn get_folders(&self, _space_id: &str) -> Result<Vec<Folder>> {
        match &self.folders_response {
            Some(Ok(folders)) => Ok(folders.clone()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Ok(vec![]),
        }
    }

    async fn get_lists_in_folder(
        &self,
        _folder_id: &str,
        _archived: Option<bool>,
    ) -> Result<Vec<List>> {
        match &self.lists_in_folder_response {
            Some(Ok(lists)) => Ok(lists.clone()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Ok(vec![]),
        }
    }

    async fn get_lists_in_space(
        &self,
        _space_id: &str,
        _archived: Option<bool>,
    ) -> Result<Vec<List>> {
        match &self.lists_in_space_response {
            Some(Ok(lists)) => Ok(lists.clone()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Ok(vec![]),
        }
    }

    async fn get_tasks(&self, _list_id: &str, _filters: &TaskFilters) -> Result<Vec<Task>> {
        match &self.tasks_response {
            Some(Ok(tasks)) => Ok(tasks.clone()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Ok(vec![]),
        }
    }

    async fn get_task(&self, _task_id: &str) -> Result<Task> {
        match &self.task_response {
            Some(Ok(task)) => Ok(task.clone()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Err(anyhow!("Task not found")),
        }
    }

    async fn create_task(&self, _list_id: &str, _task: &CreateTaskRequest) -> Result<Task> {
        match &self.create_task_response {
            Some(Ok(task)) => Ok(task.clone()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Err(anyhow!("Create task not configured")),
        }
    }

    async fn update_task(&self, _task_id: &str, _task: &UpdateTaskRequest) -> Result<Task> {
        match &self.update_task_response {
            Some(Ok(task)) => Ok(task.clone()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Err(anyhow!("Update task not configured")),
        }
    }

    async fn delete_task(&self, _task_id: &str) -> Result<()> {
        match &self.delete_task_response {
            Some(Ok(())) => Ok(()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Err(anyhow!("Delete task not configured")),
        }
    }

    async fn search_docs(&self, _filters: &DocumentFilters) -> Result<Vec<Document>> {
        match &self.search_docs_response {
            Some(Ok(docs)) => Ok(docs.clone()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Ok(vec![]),
        }
    }

    async fn get_doc_pages(&self, _doc_id: &str) -> Result<Vec<Page>> {
        match &self.doc_pages_response {
            Some(Ok(pages)) => Ok(pages.clone()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Ok(vec![]),
        }
    }

    async fn get_page(&self, _page_id: &str) -> Result<Page> {
        match &self.page_response {
            Some(Ok(page)) => Ok(page.clone()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Err(anyhow!("Page not found")),
        }
    }
}
