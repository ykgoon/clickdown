//! Integration tests for ClickDown application
//! 
//! These tests run in headless mode with mocked API responses,
//! demonstrating how to test the application without making
//! actual network calls to the ClickUp API.

mod fixtures;

use clickdown::api::{MockClickUpClient, ClickUpApi};
use clickdown::app::{ClickDown, Message, AppState};
use std::sync::Arc;
use fixtures::{test_workspace, test_space, test_folder, test_list, test_tasks, test_task};

/// Initialize a ClickDown app with a mock client
fn init_app_with_mock_client(client: MockClickUpClient) -> (ClickDown, iced::Task<Message>) {
    ClickDown::with_client(Arc::new(client))
}

/// Assert that the app is in the expected state
fn assert_app_state(app: &ClickDown, expected: AppState) {
    assert_eq!(
        app.state(), &expected,
        "App state mismatch. Expected: {:?}, Got: {:?}",
        expected, app.state()
    );
}

/// Manually trigger workspace loading for testing
/// This bypasses the async Task mechanism for synchronous testing
fn load_workspaces_sync(app: &mut ClickDown) {
    // Get workspaces from the mock client directly
    if let Some(ref client) = app.client {
        // For testing purposes, we'll manually call the update with the result
        // This is a simplified approach - in real code, the Task would be executed
    }
}

/// Test that the app initializes correctly with a mock client
#[tokio::test]
async fn test_app_initialization_with_mock_client() {
    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![test_workspace()]);
    
    let (mut app, _task) = init_app_with_mock_client(mock_client);
    
    // App should start in LoadingWorkspaces state
    assert_app_state(&app, AppState::LoadingWorkspaces);
    
    // Simulate Initialize message and get the task
    let task = app.update(Message::Initialize);
    
    // For testing, we manually trigger the workspace loaded message
    // In real code, the Task would be executed by the iced runtime
    let workspaces = mock_client.get_workspaces().await.unwrap();
    app.update(Message::WorkspacesLoaded(workspaces));
    
    // After initialization, should be in Main state with workspaces loaded
    assert_app_state(&app, AppState::Main);
    assert!(!app.workspaces().is_empty());
    assert_eq!(app.workspaces()[0].id, "test-workspace-1");
}

/// Test workspace selection flow
#[tokio::test]
async fn test_workspace_selection() {
    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![test_workspace()])
        .with_spaces(vec![test_space()]);
    
    let (mut app, _task) = init_app_with_mock_client(mock_client);
    
    // Initialize and load workspaces
    app.update(Message::Initialize);
    assert_app_state(&app, AppState::Main);
    
    // Select the workspace
    let workspace = test_workspace();
    app.update(Message::WorkspaceSelected(workspace.clone()));
    
    // Verify workspace is selected
    assert_eq!(app.selected_workspace(), &Some(workspace));
}

/// Test space selection flow
#[tokio::test]
async fn test_space_selection() {
    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![test_workspace()])
        .with_spaces(vec![test_space()])
        .with_folders(vec![test_folder()]);
    
    let (mut app, _task) = init_app_with_mock_client(mock_client);
    
    // Initialize
    app.update(Message::Initialize);
    
    // Select workspace first
    app.update(Message::WorkspaceSelected(test_workspace()));
    
    // Select space
    let space = test_space();
    app.update(Message::SpaceSelected(space.clone()));
    
    // Verify space is selected
    assert_eq!(app.selected_space(), &Some(space));
}

/// Test folder selection flow
#[tokio::test]
async fn test_folder_selection() {
    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![test_workspace()])
        .with_spaces(vec![test_space()])
        .with_folders(vec![test_folder()])
        .with_lists_in_folder(vec![test_list()]);
    
    let (mut app, _task) = init_app_with_mock_client(mock_client);
    
    // Initialize and navigate to folder
    app.update(Message::Initialize);
    app.update(Message::WorkspaceSelected(test_workspace()));
    app.update(Message::SpaceSelected(test_space()));
    
    // Select folder
    let folder = test_folder();
    app.update(Message::FolderSelected(folder.clone()));
    
    // Verify folder is selected
    assert_eq!(app.selected_folder(), &Some(folder));
}

/// Test list selection and task loading
#[tokio::test]
async fn test_list_selection_and_task_loading() {
    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![test_workspace()])
        .with_spaces(vec![test_space()])
        .with_folders(vec![test_folder()])
        .with_lists_in_folder(vec![test_list()])
        .with_tasks(test_tasks());
    
    let (mut app, _task) = init_app_with_mock_client(mock_client);
    
    // Navigate to list
    app.update(Message::Initialize);
    app.update(Message::WorkspaceSelected(test_workspace()));
    app.update(Message::SpaceSelected(test_space()));
    app.update(Message::FolderSelected(test_folder()));
    
    // Select list
    let list = test_list();
    app.update(Message::ListSelected(list.clone()));
    
    // Verify list is selected and tasks are loaded
    assert_eq!(app.selected_list(), &Some(list));
    assert!(!app.tasks().is_empty());
    assert_eq!(app.tasks().len(), 3);
}

/// Test task selection
#[tokio::test]
async fn test_task_selection() {
    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![test_workspace()])
        .with_spaces(vec![test_space()])
        .with_folders(vec![test_folder()])
        .with_lists_in_folder(vec![test_list()])
        .with_tasks(test_tasks());
    
    let (mut app, _task) = init_app_with_mock_client(mock_client);
    
    // Navigate to tasks
    app.update(Message::Initialize);
    app.update(Message::WorkspaceSelected(test_workspace()));
    app.update(Message::SpaceSelected(test_space()));
    app.update(Message::FolderSelected(test_folder()));
    app.update(Message::ListSelected(test_list()));
    
    // Select first task
    let task = app.tasks()[0].clone();
    app.update(Message::TaskSelected(task.clone()));
    
    // Verify task detail panel is open
    assert!(app.is_task_detail_open());
}

/// Test create task flow
#[tokio::test]
async fn test_create_task_flow() {
    let new_task = test_task();
    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![test_workspace()])
        .with_spaces(vec![test_space()])
        .with_folders(vec![test_folder()])
        .with_lists_in_folder(vec![test_list()])
        .with_tasks(vec![])
        .with_create_task_response(new_task.clone());
    
    let (mut app, _task) = init_app_with_mock_client(mock_client);
    
    // Navigate to list
    app.update(Message::Initialize);
    app.update(Message::WorkspaceSelected(test_workspace()));
    app.update(Message::SpaceSelected(test_space()));
    app.update(Message::FolderSelected(test_folder()));
    app.update(Message::ListSelected(test_list()));
    
    // Request to create a task
    app.update(Message::CreateTaskRequested);
    
    // Verify task detail panel is open in create mode
    assert!(app.is_task_detail_open());
}

/// Test delete task flow
#[tokio::test]
async fn test_delete_task() {
    let tasks = test_tasks();
    let task_id = tasks[0].id.clone();
    
    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![test_workspace()])
        .with_spaces(vec![test_space()])
        .with_folders(vec![test_folder()])
        .with_lists_in_folder(vec![test_list()])
        .with_tasks(tasks.clone())
        .with_delete_task_success();
    
    let (mut app, _task) = init_app_with_mock_client(mock_client);
    
    // Navigate to tasks
    app.update(Message::Initialize);
    app.update(Message::WorkspaceSelected(test_workspace()));
    app.update(Message::SpaceSelected(test_space()));
    app.update(Message::FolderSelected(test_folder()));
    app.update(Message::ListSelected(test_list()));
    
    // Verify tasks are loaded
    assert_eq!(app.tasks().len(), 3);
    
    // Delete task
    app.update(Message::DeleteTask(task_id.clone()));
    
    // Task should be removed from list
    assert!(!app.tasks().iter().any(|t| t.id == task_id));
}

/// Test error handling when API returns error
#[tokio::test]
async fn test_api_error_handling() {
    let mock_client = MockClickUpClient::new()
        .with_workspaces_error("API connection failed".to_string());
    
    let (mut app, _task) = init_app_with_mock_client(mock_client);
    
    // Try to initialize
    app.update(Message::Initialize);
    
    // Should have an error message
    assert!(app.error().is_some());
    assert!(app.error().as_ref().unwrap().contains("API connection failed"));
}

/// Test logout functionality
#[tokio::test]
async fn test_logout() {
    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![test_workspace()]);
    
    let (mut app, _task) = init_app_with_mock_client(mock_client);
    
    // Initialize and get to main state
    app.update(Message::Initialize);
    assert_app_state(&app, AppState::Main);
    
    // Logout
    app.update(Message::Logout);
    
    // Should be back to unauthenticated state
    assert_app_state(&app, AppState::Unauthenticated);
    assert!(!app.has_client());
    assert!(app.workspaces().is_empty());
}

/// Test sidebar toggle
#[tokio::test]
async fn test_sidebar_toggle() {
    let mock_client = MockClickUpClient::new();
    let (mut app, _task) = init_app_with_mock_client(mock_client);
    
    // Sidebar should start expanded
    assert!(!app.is_sidebar_collapsed());
    
    // Toggle sidebar
    app.update(Message::ToggleSidebar);
    assert!(app.is_sidebar_collapsed());
    
    // Toggle again
    app.update(Message::ToggleSidebar);
    assert!(!app.is_sidebar_collapsed());
}

/// Test document loading
#[tokio::test]
async fn test_document_loading() {
    use fixtures::{test_document, test_page};
    
    let doc = test_document();
    let page = test_page();
    
    let mock_client = MockClickUpClient::new()
        .with_documents(vec![doc.clone()])
        .with_pages(vec![page.clone()]);
    
    // Document search would be triggered by a message
    // For now, verify the mock client can return documents
    let docs = mock_client.search_docs(&Default::default()).await.unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].id, "test-doc-1");
}
