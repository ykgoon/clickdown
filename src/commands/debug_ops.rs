//! Debug operations for CLI mode
//!
//! Implements the actual data-fetching operations for debug commands.

use std::sync::Arc;
use crate::api::{ClickUpApi, AuthManager};
use crate::models::task::TaskFilters;
use crate::models::document::DocumentFilters;

/// Exit codes for CLI operations
pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const INVALID_ARGS: i32 = 2;
    pub const AUTH_ERROR: i32 = 3;
    pub const NETWORK_ERROR: i32 = 4;
}

/// Debug operations handler
pub struct DebugOperations {
    api: Arc<dyn ClickUpApi>,
    #[allow(dead_code)]
    auth: AuthManager,
    #[allow(dead_code)]
    token_override: Option<String>,
}

impl DebugOperations {
    /// Create a new DebugOperations instance
    pub fn new(api: Arc<dyn ClickUpApi>, auth: AuthManager, token_override: Option<String>) -> Self {
        Self { api, auth, token_override }
    }

    /// Get the API client, using override token if provided
    fn get_api(&self) -> Arc<dyn ClickUpApi> {
        Arc::clone(&self.api)
    }

    /// List all workspaces
    pub async fn list_workspaces(&self) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        let workspaces = api.get_workspaces().await?;

        if workspaces.is_empty() {
            println!("No workspaces found.");
            return Ok(());
        }

        for ws in &workspaces {
            let color = ws.color.as_deref().unwrap_or("none");
            println!("{} - {} (color: {})", ws.id, ws.name, color);
        }

        Ok(())
    }

    /// List workspaces as JSON
    pub async fn list_workspaces_json(&self) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        let workspaces = api.get_workspaces().await?;

        let json = serde_json::to_string_pretty(&workspaces)?;
        println!("{}", json);

        Ok(())
    }

    /// List tasks from a list
    pub async fn list_tasks(&self, list_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        let filters = TaskFilters::default();
        let tasks = api.get_tasks(list_id, &filters).await?;

        if tasks.is_empty() {
            println!("No tasks found in list {}.", list_id);
            return Ok(());
        }

        for task in &tasks {
            let status = task.status.as_ref().map(|s| s.status.as_str()).unwrap_or("unknown");
            let priority = match &task.priority {
                Some(p) => format!("{:?}", p),
                None => "none".to_string(),
            };
            println!("{} - {} [status: {}, priority: {}]", task.id, task.name, status, priority);
        }

        Ok(())
    }

    /// List tasks as JSON
    pub async fn list_tasks_json(&self, list_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        let filters = TaskFilters::default();
        let tasks = api.get_tasks(list_id, &filters).await?;

        let json = serde_json::to_string_pretty(&tasks)?;
        println!("{}", json);

        Ok(())
    }

    /// Search documents
    pub async fn search_docs(&self, query: &str) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        let filters = DocumentFilters {
            query: Some(query.to_string()),
            ..Default::default()
        };
        let docs = api.search_docs(&filters).await?;

        if docs.is_empty() {
            println!("No documents found matching '{}'.", query);
            return Ok(());
        }

        for doc in &docs {
            println!("{} - {}", doc.id, doc.name);
        }

        Ok(())
    }

    /// Search documents as JSON
    pub async fn search_docs_json(&self, query: &str) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        let filters = DocumentFilters {
            query: Some(query.to_string()),
            ..Default::default()
        };
        let docs = api.search_docs(&filters).await?;

        let json = serde_json::to_string_pretty(&docs)?;
        println!("{}", json);

        Ok(())
    }

    /// Check authentication status
    pub async fn check_auth_status(&self) -> Result<bool, Box<dyn std::error::Error>> {
        // Try to get workspaces to verify auth
        match self.get_api().get_workspaces().await {
            Ok(workspaces) => {
                println!("Authenticated: yes");
                println!("Workspace count: {}", workspaces.len());
                Ok(true)
            }
            Err(e) => {
                println!("Authenticated: no");
                eprintln!("Error: {}", e);
                Ok(false)
            }
        }
    }
}
