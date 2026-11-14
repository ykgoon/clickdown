//! Mock ClickUp API client for testing

use crate::api::client_trait::ClickUpApi;
use crate::models::{
    ClickUpSpace, Comment, CreateCommentRequest, CreateTaskRequest, Document, DocumentFilters,
    Folder, List, Page, Task, TaskFilters, UpdateCommentRequest, UpdateTaskRequest, User,
    Workspace,
};
use anyhow::{anyhow, Result};

/// Helper function to return configured response or default empty vec
#[allow(dead_code)]
fn return_vec_response<T: Clone>(configured: &Option<Result<Vec<T>>>) -> Result<Vec<T>> {
    match configured {
        Some(Ok(items)) => Ok(items.clone()),
        Some(Err(e)) => Err(anyhow!(e.to_string())),
        None => Ok(vec![]),
    }
}

/// Helper function to return configured response or default single item
#[allow(dead_code)]
fn return_response<T: Clone>(configured: &Option<Result<T>>, not_found_msg: &str) -> Result<T> {
    match configured {
        Some(Ok(item)) => Ok(item.clone()),
        Some(Err(e)) => Err(anyhow!(e.to_string())),
        None => Err(anyhow!("{}", not_found_msg)),
    }
}

/// Helper function to return configured response for unit result
#[allow(dead_code)]
fn return_unit_response(configured: &Option<Result<()>>, not_configured_msg: &str) -> Result<()> {
    match configured {
        Some(Ok(())) => Ok(()),
        Some(Err(e)) => Err(anyhow!(e.to_string())),
        None => Err(anyhow!("{}", not_configured_msg)),
    }
}

/// Mock ClickUp API client for headless testing
///
/// This client implements the ClickUpApi trait and can be configured
/// to return predefined responses for testing without making actual
/// network calls.
#[derive(Default)]
#[allow(dead_code)]
pub struct MockClickUpClient {
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
    /// Override for get_list response
    pub list_response: Option<Result<List>>,
    /// Override for get_tasks response
    pub tasks_response: Option<Result<Vec<Task>>>,
    /// Override for get_task response
    pub task_response: Option<Result<Task>>,
    /// Override for create_task response
    pub create_task_response: Option<Result<Task>>,
    /// Override for update_task response
    pub update_task_response: Option<Result<Task>>,
    /// Override for delete_task response (raw JSON body string)
    pub delete_task_response: Option<Result<String>>,
    /// Override for search_docs response
    pub search_docs_response: Option<Result<Vec<Document>>>,
    /// Override for get_doc_pages response
    pub doc_pages_response: Option<Result<Vec<Page>>>,
    /// Override for get_page response
    pub page_response: Option<Result<Page>>,
    /// Override for get_task_comments response
    pub task_comments_response: Option<Result<Vec<Comment>>>,
    /// Override for get_comment_replies response (maps comment_id -> replies)
    pub comment_replies_response: Option<std::collections::HashMap<String, Result<Vec<Comment>>>>,
    /// Override for create_comment response
    pub create_comment_response: Option<Result<Comment>>,
    /// Override for create_comment_reply response
    pub create_comment_reply_response: Option<Result<Comment>>,
    /// Override for update_comment response
    pub update_comment_response: Option<Result<Comment>>,
    /// Override for get_tasks_with_assignee response
    pub tasks_with_assignee_response: Option<Result<Vec<Task>>>,
    /// Override for get_team_tasks_with_assignee, keyed by page so tests can
    /// exercise multi-page fetching.
    pub team_tasks_response: Option<Result<Vec<Task>>>,
    /// Page-aware override: (page, tasks). Takes precedence when present.
    pub team_tasks_pages: std::collections::HashMap<u32, Vec<Task>>,
    /// Pages requested from the workspace-wide assigned endpoint, in order
    pub assigned_page_log: AssignedPageLog,
    /// Override for get_current_user response
    pub current_user_response: Option<Result<User>>,
    /// Override for get_list_members response
    pub list_members_response: Option<Result<Vec<User>>>,
    /// Override for get_inbox response
    pub inbox_response: Option<Result<crate::models::Inbox>>,
    /// Override for add/remove comment reaction (shared: both return no body)
    pub comment_reaction_response: Option<Result<()>>,
    /// Reaction calls recorded as (comment_id, reaction, added).
    /// Tests keep a clone of the handle to inspect calls after handing the
    /// client to the app.
    pub reaction_calls: ReactionCalls,
}

#[allow(dead_code)]
impl MockClickUpClient {
    /// Create a new mock client with default (empty) responses
    pub fn new() -> Self {
        Self {
            workspaces_response: None,
            spaces_response: None,
            folders_response: None,
            lists_in_folder_response: None,
            lists_in_space_response: None,
            list_response: None,
            tasks_response: None,
            task_response: None,
            create_task_response: None,
            update_task_response: None,
            delete_task_response: None,
            search_docs_response: None,
            doc_pages_response: None,
            page_response: None,
            task_comments_response: None,
            comment_replies_response: None,
            create_comment_response: None,
            create_comment_reply_response: None,
            update_comment_response: None,
            tasks_with_assignee_response: None,
            team_tasks_response: None,
            team_tasks_pages: std::collections::HashMap::new(),
            assigned_page_log: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            current_user_response: None,
            list_members_response: None,
            inbox_response: None,
            comment_reaction_response: None,
            reaction_calls: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Configure the reaction response
    pub fn with_comment_reaction(mut self) -> Self {
        self.comment_reaction_response = Some(Ok(()));
        self
    }

    /// Configure a reaction failure
    pub fn with_comment_reaction_error(mut self, msg: &str) -> Self {
        self.comment_reaction_response = Some(Err(anyhow!("{}", msg)));
        self
    }

    /// Recorded reaction calls, as (comment_id, reaction, added)
    pub fn reaction_calls(&self) -> Vec<(String, String, bool)> {
        self.reaction_calls
            .lock()
            .map(|calls| calls.clone())
            .unwrap_or_default()
    }

    /// A handle to this client's reaction log, valid after the client moves
    pub fn reaction_calls_handle(&self) -> ReactionCalls {
        self.reaction_calls.clone()
    }

    /// A handle to this client's assigned-page log, valid after it moves
    pub fn assigned_page_log_handle(&self) -> AssignedPageLog {
        self.assigned_page_log.clone()
    }

    /// Record one reaction call and return the configured outcome
    fn record_reaction(&self, comment_id: &str, reaction: &str, added: bool) -> Result<()> {
        if let Ok(mut calls) = self.reaction_calls.lock() {
            calls.push((comment_id.to_string(), reaction.to_string(), added));
        }
        match &self.comment_reaction_response {
            Some(Ok(())) => Ok(()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Ok(()),
        }
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

    /// Set the lists in space response (folderless lists)
    pub fn with_lists_in_space(mut self, lists: Vec<List>) -> Self {
        self.lists_in_space_response = Some(Ok(lists));
        self
    }

    /// Override the get_list response (single list with statuses)
    pub fn with_list(mut self, list: List) -> Self {
        self.list_response = Some(Ok(list));
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

    /// Set the delete task response with raw JSON body (simulates API response parsing)
    pub fn with_delete_task_json(mut self, json: &str) -> Self {
        self.delete_task_response = Some(Ok(json.to_string()));
        self
    }

    /// Set the delete task error
    pub fn with_delete_task_error(mut self, msg: &str) -> Self {
        self.delete_task_response = Some(Err(anyhow!("{}", msg)));
        self
    }

    /// Convenience: set delete task success with minimal valid JSON
    pub fn with_delete_task_success(mut self) -> Self {
        self.delete_task_response = Some(Ok("{}".to_string()));
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

    /// Set the task comments response
    pub fn with_task_comments(mut self, comments: Vec<Comment>) -> Self {
        self.task_comments_response = Some(Ok(comments));
        self
    }

    /// Set the create comment response
    pub fn with_create_comment_response(mut self, comment: Comment) -> Self {
        self.create_comment_response = Some(Ok(comment));
        self
    }

    /// Set the update comment response
    pub fn with_update_comment_response(mut self, comment: Comment) -> Self {
        self.update_comment_response = Some(Ok(comment));
        self
    }

    /// Set the comment replies response for a specific comment
    pub fn with_comment_replies(mut self, comment_id: &str, replies: Vec<Comment>) -> Self {
        if self.comment_replies_response.is_none() {
            self.comment_replies_response = Some(std::collections::HashMap::new());
        }
        if let Some(ref mut map) = self.comment_replies_response {
            map.insert(comment_id.to_string(), Ok(replies));
        }
        self
    }

    /// Set the create comment reply response
    pub fn with_create_comment_reply_response(mut self, comment: Comment) -> Self {
        self.create_comment_reply_response = Some(Ok(comment));
        self
    }

    /// Set the workspace-wide assigned tasks response (all pages)
    pub fn with_team_tasks(mut self, tasks: Vec<Task>) -> Self {
        self.team_tasks_response = Some(Ok(tasks));
        self
    }

    /// Set one page of workspace-wide assigned tasks
    pub fn with_team_tasks_page(mut self, page: u32, tasks: Vec<Task>) -> Self {
        self.team_tasks_pages.insert(page, tasks);
        self
    }

    /// Set the inbox response
    pub fn with_inbox(mut self, inbox: crate::models::Inbox) -> Self {
        self.inbox_response = Some(Ok(inbox));
        self
    }

    /// Set an inbox error (e.g. the endpoint being unavailable)
    pub fn with_inbox_error(mut self, msg: &str) -> Self {
        self.inbox_response = Some(Err(anyhow!("{}", msg)));
        self
    }

    /// Set the list members response
    pub fn with_list_members(mut self, members: Vec<User>) -> Self {
        self.list_members_response = Some(Ok(members));
        self
    }

    /// Set the list members error
    pub fn with_list_members_error(mut self, error: String) -> Self {
        self.list_members_response = Some(Err(anyhow!(error)));
        self
    }
}

/// Shared handle to a mock's recorded reaction calls, so tests can inspect
/// them after the client has been moved into the app.
pub type ReactionCalls = std::sync::Arc<std::sync::Mutex<Vec<(String, String, bool)>>>;

/// Shared handle to the pages requested from the assigned-tasks endpoint.
pub type AssignedPageLog = std::sync::Arc<std::sync::Mutex<Vec<u32>>>;

#[async_trait::async_trait]
impl ClickUpApi for MockClickUpClient {
    async fn get_workspaces(&self) -> Result<Vec<Workspace>> {
        return_vec_response(&self.workspaces_response)
    }

    async fn get_current_user(&self) -> Result<User> {
        match &self.current_user_response {
            Some(Ok(user)) => Ok(user.clone()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Ok(User {
                id: 1,
                username: "test_user".to_string(),
                color: None,
                email: None,
                profile_picture: None,
                initials: None,
            }),
        }
    }

    async fn get_spaces(&self, _team_id: &str) -> Result<Vec<ClickUpSpace>> {
        return_vec_response(&self.spaces_response)
    }

    async fn get_space(&self, _space_id: &str) -> Result<ClickUpSpace> {
        match &self.spaces_response {
            Some(Ok(spaces)) => spaces
                .first()
                .cloned()
                .ok_or_else(|| anyhow!("Space not found")),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Err(anyhow!("No spaces configured")),
        }
    }

    async fn get_folders(&self, _space_id: &str) -> Result<Vec<Folder>> {
        return_vec_response(&self.folders_response)
    }

    async fn get_lists_in_folder(
        &self,
        _folder_id: &str,
        _archived: Option<bool>,
    ) -> Result<Vec<List>> {
        return_vec_response(&self.lists_in_folder_response)
    }

    async fn get_lists_in_space(
        &self,
        _space_id: &str,
        _archived: Option<bool>,
    ) -> Result<Vec<List>> {
        return_vec_response(&self.lists_in_space_response)
    }

    async fn get_list(&self, _list_id: &str) -> Result<List> {
        return_response(&self.list_response, "List not found")
    }

    async fn get_tasks(&self, _list_id: &str, _filters: &TaskFilters) -> Result<Vec<Task>> {
        return_vec_response(&self.tasks_response)
    }

    async fn get_task(&self, _task_id: &str) -> Result<Task> {
        return_response(&self.task_response, "Task not found")
    }

    async fn create_task(&self, _list_id: &str, _task: &CreateTaskRequest) -> Result<Task> {
        return_response(&self.create_task_response, "Create task not configured")
    }

    async fn update_task(&self, _task_id: &str, update: &UpdateTaskRequest) -> Result<Task> {
        let mut task = return_response(&self.update_task_response, "Update task not configured")?;

        // Merge update request fields into the task
        if let Some(ref name) = update.name {
            task.name = name.clone();
        }
        if let Some(ref desc) = update.description {
            task.description = Some(crate::models::task::TaskDescription::Plain(desc.clone()));
        }
        if let Some(ref status) = update.status {
            task.status = Some(crate::models::task::TaskStatus {
                id: None,
                status: status.clone(),
                color: None,
                type_field: None,
                orderindex: None,
                status_group: None,
            });
        }
        if let Some(ref assignees_update) = update.assignees {
            // Create User stubs for the assigned IDs from the add array
            let ids = assignees_update.add.as_deref().unwrap_or(&[]);
            task.assignees = ids
                .iter()
                .map(|id| User {
                    id: *id,
                    username: format!("user_{}", id),
                    color: None,
                    email: None,
                    profile_picture: None,
                    initials: None,
                })
                .collect();
        }
        if let Some(due) = update.due_date {
            task.due_date = Some(due);
        }

        Ok(task)
    }

    async fn delete_task(&self, _task_id: &str) -> Result<()> {
        match &self.delete_task_response {
            Some(Ok(json)) => {
                // Parse the JSON body like the real client does
                if json.trim().is_empty() {
                    anyhow::bail!("Failed to parse response: {}", json);
                }
                serde_json::from_str::<serde_json::Value>(json)
                    .map(|_| ())
                    .map_err(|e| anyhow!("Failed to parse response: {}", e))
            }
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Err(anyhow!("Delete task not configured")),
        }
    }

    async fn search_docs(&self, _filters: &DocumentFilters) -> Result<Vec<Document>> {
        return_vec_response(&self.search_docs_response)
    }

    async fn get_doc_pages(&self, _doc_id: &str) -> Result<Vec<Page>> {
        return_vec_response(&self.doc_pages_response)
    }

    async fn get_page(&self, _page_id: &str) -> Result<Page> {
        return_response(&self.page_response, "Page not found")
    }

    async fn get_task_comments(&self, _task_id: &str) -> Result<Vec<Comment>> {
        return_vec_response(&self.task_comments_response)
    }

    async fn get_comment_replies(&self, comment_id: &str) -> Result<Vec<Comment>> {
        match &self.comment_replies_response {
            Some(map) => match map.get(comment_id) {
                Some(Ok(replies)) => Ok(replies.clone()),
                Some(Err(e)) => Err(anyhow!(e.to_string())),
                None => Ok(vec![]),
            },
            None => Ok(vec![]),
        }
    }

    async fn create_comment(
        &self,
        _task_id: &str,
        _comment: &CreateCommentRequest,
    ) -> Result<Comment> {
        return_response(
            &self.create_comment_response,
            "Create comment not configured",
        )
    }

    async fn create_comment_reply(
        &self,
        _parent_comment_id: &str,
        _comment: &CreateCommentRequest,
    ) -> Result<Comment> {
        return_response(
            &self.create_comment_reply_response,
            "Create comment reply not configured",
        )
    }

    async fn update_comment(
        &self,
        _comment_id: &str,
        _comment: &UpdateCommentRequest,
    ) -> Result<Comment> {
        return_response(
            &self.update_comment_response,
            "Update comment not configured",
        )
    }

    async fn get_tasks_with_assignee(
        &self,
        _list_id: &str,
        _user_id: i32,
        _limit: Option<i32>,
    ) -> Result<Vec<Task>> {
        return_vec_response(&self.tasks_with_assignee_response)
    }

    async fn get_team_tasks_with_assignee(
        &self,
        _team_id: &str,
        _user_id: i32,
        page: u32,
    ) -> Result<Vec<Task>> {
        if let Ok(mut log) = self.assigned_page_log.lock() {
            log.push(page);
        }
        if !self.team_tasks_pages.is_empty() {
            return Ok(self
                .team_tasks_pages
                .get(&page)
                .cloned()
                .unwrap_or_default());
        }
        return_vec_response(&self.team_tasks_response)
    }

    async fn get_inbox(&self, _team_id: &str) -> Result<crate::models::Inbox> {
        match &self.inbox_response {
            Some(Ok(inbox)) => Ok(inbox.clone()),
            Some(Err(e)) => Err(anyhow!(e.to_string())),
            None => Ok(crate::models::Inbox::default()),
        }
    }

    async fn get_list_members(&self, _list_id: &str) -> Result<Vec<User>> {
        return_vec_response(&self.list_members_response)
    }

    async fn add_comment_reaction(&self, comment_id: &str, reaction: &str) -> Result<()> {
        self.record_reaction(comment_id, reaction, true)
    }

    async fn remove_comment_reaction(&self, comment_id: &str, reaction: &str) -> Result<()> {
        self.record_reaction(comment_id, reaction, false)
    }
}
