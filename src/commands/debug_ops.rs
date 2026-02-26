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

    /// List spaces in a workspace
    pub async fn list_spaces(&self, workspace_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        let spaces = api.get_spaces(workspace_id).await?;

        if spaces.is_empty() {
            println!("No spaces found in workspace {}.", workspace_id);
            return Ok(());
        }

        for space in &spaces {
            let private = if space.private { " (private)" } else { "" };
            println!("{} - {}{}", space.id, space.name, private);
        }

        Ok(())
    }

    /// List spaces as JSON
    pub async fn list_spaces_json(&self, workspace_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        let spaces = api.get_spaces(workspace_id).await?;

        let json = serde_json::to_string_pretty(&spaces)?;
        println!("{}", json);

        Ok(())
    }

    /// List folders in a space
    pub async fn list_folders(&self, space_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        let folders = api.get_folders(space_id).await?;

        if folders.is_empty() {
            println!("No folders found in space {}.", space_id);
            return Ok(());
        }

        for folder in &folders {
            let private = if folder.private { " (private)" } else { "" };
            println!("{} - {}{}", folder.id, folder.name, private);
        }

        Ok(())
    }

    /// List folders as JSON
    pub async fn list_folders_json(&self, space_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        let folders = api.get_folders(space_id).await?;

        let json = serde_json::to_string_pretty(&folders)?;
        println!("{}", json);

        Ok(())
    }

    /// List lists in a folder
    pub async fn list_lists_in_folder(&self, folder_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        let lists = api.get_lists_in_folder(folder_id, None).await?;

        if lists.is_empty() {
            println!("No lists found in folder {}.", folder_id);
            return Ok(());
        }

        for list in &lists {
            let archived = if list.archived { " (archived)" } else { "" };
            println!("{} - {}{}", list.id, list.name, archived);
        }

        Ok(())
    }

    /// List lists in a space (folderless)
    pub async fn list_lists_in_space(&self, space_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        let lists = api.get_lists_in_space(space_id, None).await?;

        if lists.is_empty() {
            println!("No folderless lists found in space {}.", space_id);
            return Ok(());
        }

        for list in &lists {
            let archived = if list.archived { " (archived)" } else { "" };
            println!("{} - {}{}", list.id, list.name, archived);
        }

        Ok(())
    }

    /// List lists as JSON (from folder)
    pub async fn list_lists_json(&self, folder_id: &str, in_space: bool) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        let lists = if in_space {
            api.get_lists_in_space(folder_id, None).await?
        } else {
            api.get_lists_in_folder(folder_id, None).await?
        };

        let json = serde_json::to_string_pretty(&lists)?;
        println!("{}", json);

        Ok(())
    }

    /// Get a single task as JSON
    pub async fn get_task_json(&self, task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        let task = api.get_task(task_id).await?;

        let json = serde_json::to_string_pretty(&task)?;
        println!("{}", json);

        Ok(())
    }

    /// Explore full hierarchy: workspace -> spaces -> folders -> lists -> tasks
    pub async fn explore_hierarchy(&self, workspace_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();

        println!("=== Exploring Workspace: {} ===\n", workspace_id);

        // Get spaces
        let spaces = api.get_spaces(workspace_id).await?;
        println!("Found {} space(s)\n", spaces.len());

        for space in &spaces {
            println!("--- Space: {} ({}) ---", space.name, space.id);

            // Get folders in space
            let folders = api.get_folders(&space.id).await?;
            println!("  Folders: {}", folders.len());

            for folder in &folders {
                println!("    Folder: {} ({})", folder.name, folder.id);

                // Get lists in folder
                let lists = api.get_lists_in_folder(&folder.id, None).await?;
                println!("      Lists: {}", lists.len());

                for list in &lists {
                    println!("        List: {} ({})", list.name, list.id);

                    // Get sample tasks from first list
                    let filters = TaskFilters::default();
                    let tasks = api.get_tasks(&list.id, &filters).await?;
                    println!("          Tasks: {}", tasks.len());

                    if !tasks.is_empty() {
                        println!("          Sample task ID: {}", tasks[0].id);
                    }
                }
            }

            // Also check folderless lists in space
            let space_lists = api.get_lists_in_space(&space.id, None).await?;
            if !space_lists.is_empty() {
                println!("  Folderless Lists: {}", space_lists.len());
                for list in &space_lists {
                    println!("    List: {} ({})", list.name, list.id);
                }
            }

            println!();
        }

        Ok(())
    }

    /// Get comments for a task (human-readable format)
    pub async fn get_comments(&self, task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        let comments = api.get_task_comments(task_id).await?;

        if comments.is_empty() {
            println!("No comments found for task {}.", task_id);
            return Ok(());
        }

        println!("Comments for task {}:\n", task_id);
        for (i, comment) in comments.iter().enumerate() {
            let author = comment.commenter.as_ref()
                .map(|c| c.username.as_str())
                .unwrap_or("Anonymous");
            
            let date_str = comment.created_at
                .map(|ts| format_timestamp(ts))
                .unwrap_or_else(|| "Unknown date".to_string());

            let edited = if comment.updated_at.is_some() && comment.updated_at != comment.created_at {
                " (edited)"
            } else {
                ""
            };

            println!("[{}] {} - {}{}", i + 1, author, date_str, edited);
            println!("    {}", comment.text);
            println!();
        }

        Ok(())
    }

    /// Get comments for a task as JSON
    pub async fn get_comments_json(&self, task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        let comments = api.get_task_comments(task_id).await?;

        let json = serde_json::to_string_pretty(&comments)?;
        println!("{}", json);

        Ok(())
    }

    /// Create a new comment on a task (human-readable format)
    pub async fn create_comment(
        &self,
        task_id: &str,
        text: &str,
        parent_id: Option<&str>,
        assignee: Option<i64>,
        assigned_commenter: Option<i64>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        use crate::models::CreateCommentRequest;

        let request = CreateCommentRequest {
            comment_text: text.to_string(),
            assignee,
            assigned_commenter,
            parent_id: parent_id.map(String::from),
        };

        let comment = if parent_id.is_some() {
            api.create_comment_reply(parent_id.unwrap(), &request).await?
        } else {
            api.create_comment(task_id, &request).await?
        };

        println!("Comment created: {}", comment.id);
        println!("Text: {}", comment.text);

        if let Some(user) = &comment.commenter {
            println!("Author: {}", user.username);
        }

        Ok(())
    }

    /// Create a new comment on a task (JSON format)
    pub async fn create_comment_json(
        &self,
        task_id: &str,
        text: &str,
        parent_id: Option<&str>,
        assignee: Option<i64>,
        assigned_commenter: Option<i64>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        use crate::models::CreateCommentRequest;

        let request = CreateCommentRequest {
            comment_text: text.to_string(),
            assignee,
            assigned_commenter,
            parent_id: parent_id.map(String::from),
        };

        let comment = if parent_id.is_some() {
            api.create_comment_reply(parent_id.unwrap(), &request).await?
        } else {
            api.create_comment(task_id, &request).await?
        };

        let json = serde_json::to_string_pretty(&comment)?;
        println!("{}", json);

        Ok(())
    }

    /// Create a reply to an existing comment (human-readable format)
    pub async fn create_reply(
        &self,
        comment_id: &str,
        text: &str,
        assignee: Option<i64>,
        assigned_commenter: Option<i64>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // create_reply is a convenience wrapper that calls create_comment with parent_id set
        self.create_comment("", text, Some(comment_id), assignee, assigned_commenter).await
    }

    /// Create a reply to an existing comment (JSON format)
    pub async fn create_reply_json(
        &self,
        comment_id: &str,
        text: &str,
        assignee: Option<i64>,
        assigned_commenter: Option<i64>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // create_reply_json is a convenience wrapper
        self.create_comment_json("", text, Some(comment_id), assignee, assigned_commenter).await
    }

    /// Update an existing comment (human-readable format)
    pub async fn update_comment(
        &self,
        comment_id: &str,
        text: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        use crate::models::UpdateCommentRequest;

        let request = UpdateCommentRequest {
            comment_text: Some(text.to_string()),
            assigned: None,
            assignee: None,
            assigned_commenter: None,
        };

        let comment = api.update_comment(comment_id, &request).await?;

        println!("Comment updated: {}", comment.id);
        println!("Text: {}", comment.text);

        Ok(())
    }

    /// Update an existing comment (JSON format)
    pub async fn update_comment_json(
        &self,
        comment_id: &str,
        text: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.get_api();
        use crate::models::UpdateCommentRequest;

        let request = UpdateCommentRequest {
            comment_text: Some(text.to_string()),
            assigned: None,
            assignee: None,
            assigned_commenter: None,
        };

        let comment = api.update_comment(comment_id, &request).await?;

        let json = serde_json::to_string_pretty(&comment)?;
        println!("{}", json);

        Ok(())
    }
}

/// Format a Unix timestamp (milliseconds) to a readable date string
fn format_timestamp(ts: i64) -> String {
    use chrono::{DateTime, Local};
    
    let secs = ts / 1000;
    match DateTime::from_timestamp(secs, 0) {
        Some(dt) => {
            let local_dt: DateTime<Local> = dt.into();
            local_dt.format("%b %d, %Y %H:%M").to_string()
        }
        None => "Unknown date".to_string(),
    }
}
