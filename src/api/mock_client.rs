//! Mock ClickUp API client for testing

use crate::api::client_trait::ClickUpApi;
use crate::models::{
    ClickUpSpace, Comment, CreateCommentRequest, CreateTaskRequest, Document, DocumentFilters,
    Folder, List, Notification, Page, Task, TaskFilters, UpdateCommentRequest, UpdateTaskRequest,
    User, Workspace,
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
    /// Override for get_notifications response
    pub notifications_response: Option<Result<Vec<Notification>>>,
    /// Override for get_tasks_assigned_to_user response
    pub tasks_assigned_to_user_response: Option<Result<Vec<Task>>>,
    /// Override for get_comments_for_tasks response
    pub comments_for_tasks_response: Option<Result<Vec<Comment>>>,
    /// Override for get_tasks_with_due_dates response
    pub tasks_with_due_dates_response: Option<Result<Vec<Task>>>,
    /// Override for get_inbox_activity response
    pub inbox_activity_response: Option<Result<crate::models::InboxActivityResponse>>,
    /// Override for get_all_accessible_lists response
    pub accessible_lists_response: Option<Result<Vec<List>>>,
    /// Override for get_tasks_with_assignee response
    pub tasks_with_assignee_response: Option<Result<Vec<Task>>>,
    /// Override for get_current_user response
    pub current_user_response: Option<Result<User>>,
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
            notifications_response: None,
            tasks_assigned_to_user_response: None,
            comments_for_tasks_response: None,
            tasks_with_due_dates_response: None,
            inbox_activity_response: None,
            accessible_lists_response: None,
            tasks_with_assignee_response: None,
            current_user_response: None,
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

    /// Set the notifications response
    pub fn with_notifications(mut self, notifications: Vec<Notification>) -> Self {
        self.notifications_response = Some(Ok(notifications));
        self
    }

    /// Set the notifications error
    pub fn with_notifications_error(mut self, error: String) -> Self {
        self.notifications_response = Some(Err(anyhow!(error)));
        self
    }

    /// Set the inbox activity response
    pub fn with_inbox_activities(mut self, activities: Vec<crate::models::InboxActivity>) -> Self {
        use crate::models::InboxActivityResponse;
        self.inbox_activity_response = Some(Ok(InboxActivityResponse { activities }));
        self
    }

    /// Set the inbox activity error
    pub fn with_inbox_activity_error(mut self, error: String) -> Self {
        self.inbox_activity_response = Some(Err(anyhow!(error)));
        self
    }

    /// Set the tasks assigned to user response
    pub fn with_tasks_assigned_to_user(mut self, tasks: Vec<Task>) -> Self {
        self.tasks_assigned_to_user_response = Some(Ok(tasks));
        self
    }

    /// Set the comments for tasks response
    pub fn with_comments_for_tasks(mut self, comments: Vec<Comment>) -> Self {
        self.comments_for_tasks_response = Some(Ok(comments));
        self
    }

    /// Set the tasks with due dates response
    pub fn with_tasks_with_due_dates(mut self, tasks: Vec<Task>) -> Self {
        self.tasks_with_due_dates_response = Some(Ok(tasks));
        self
    }

    /// Set the inbox activity response
    pub fn with_inbox_activity(mut self, activities: crate::models::InboxActivityResponse) -> Self {
        self.inbox_activity_response = Some(Ok(activities));
        self
    }

    /// Set the accessible lists response
    pub fn with_accessible_lists(mut self, lists: Vec<List>) -> Self {
        self.accessible_lists_response = Some(Ok(lists));
        self
    }

    /// Set the tasks with assignee response
    pub fn with_tasks_with_assignee_response(mut self, tasks: Vec<Task>) -> Self {
        self.tasks_with_assignee_response = Some(Ok(tasks));
        self
    }

    /// Set the current user response
    pub fn with_current_user(mut self, user: User) -> Self {
        self.current_user_response = Some(Ok(user));
        self
    }

    /// Set the current user error
    pub fn with_current_user_error(mut self, error: String) -> Self {
        self.current_user_response = Some(Err(anyhow!(error)));
        self
    }
}

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

    async fn get_tasks(&self, _list_id: &str, _filters: &TaskFilters) -> Result<Vec<Task>> {
        return_vec_response(&self.tasks_response)
    }

    async fn get_task(&self, _task_id: &str) -> Result<Task> {
        return_response(&self.task_response, "Task not found")
    }

    async fn create_task(&self, _list_id: &str, _task: &CreateTaskRequest) -> Result<Task> {
        return_response(&self.create_task_response, "Create task not configured")
    }

    async fn update_task(&self, _task_id: &str, _task: &UpdateTaskRequest) -> Result<Task> {
        return_response(&self.update_task_response, "Update task not configured")
    }

    async fn delete_task(&self, _task_id: &str) -> Result<()> {
        return_unit_response(&self.delete_task_response, "Delete task not configured")
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

    async fn get_notifications(&self, _workspace_id: &str) -> Result<Vec<Notification>> {
        return_vec_response(&self.notifications_response)
    }

    async fn get_tasks_assigned_to_user(
        &self,
        _team_id: &str,
        _user_id: i32,
        _date_updated_gt: Option<i64>,
    ) -> Result<Vec<Task>> {
        return_vec_response(&self.tasks_assigned_to_user_response)
    }

    async fn get_comments_for_tasks(
        &self,
        _task_ids: &[String],
        _date_created_gt: Option<i64>,
    ) -> Result<Vec<Comment>> {
        return_vec_response(&self.comments_for_tasks_response)
    }

    async fn get_tasks_with_due_dates(
        &self,
        _team_id: &str,
        _due_date_before: i64,
        _date_updated_gt: Option<i64>,
    ) -> Result<Vec<Task>> {
        return_vec_response(&self.tasks_with_due_dates_response)
    }

    async fn get_inbox_activity(
        &self,
        _team_id: &str,
        _user_id: i32,
        _since_timestamp: Option<i64>,
    ) -> Result<crate::models::InboxActivityResponse> {
        return_response(
            &self.inbox_activity_response,
            "Inbox activity not configured",
        )
    }

    async fn get_all_accessible_lists(&self) -> Result<Vec<List>> {
        // If accessible_lists_response is explicitly set, use it (for direct mocking)
        if let Some(ref result) = self.accessible_lists_response {
            match result {
                Ok(lists) => return Ok(lists.clone()),
                Err(e) => return Err(anyhow!(e.to_string())),
            }
        }

        // Otherwise, traverse the hierarchy like the real client does
        let mut all_lists = Vec::new();

        // Get all workspaces
        let workspaces = self.get_workspaces().await?;

        // For each workspace, get spaces
        for workspace in &workspaces {
            let spaces = self.get_spaces(&workspace.id).await?;

            // For each space, get folders and folderless lists
            for space in &spaces {
                // Get folderless lists in space
                let space_lists = self.get_lists_in_space(&space.id, None).await?;
                all_lists.extend(space_lists);

                // Get folders in space
                let folders = self.get_folders(&space.id).await?;

                // For each folder, get lists
                for folder in &folders {
                    let folder_lists = self.get_lists_in_folder(&folder.id, None).await?;
                    all_lists.extend(folder_lists);
                }
            }
        }

        Ok(all_lists)
    }

    async fn get_tasks_with_assignee(
        &self,
        _list_id: &str,
        _user_id: i32,
        _limit: Option<i32>,
    ) -> Result<Vec<Task>> {
        return_vec_response(&self.tasks_with_assignee_response)
    }
}
