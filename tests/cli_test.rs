//! CLI integration tests

use std::sync::Arc;
use clickdown::api::MockClickUpClient;
use clickdown::commands::DebugOperations;
use clickdown::api::AuthManager;

mod fixtures;
use fixtures::{test_workspace, test_task, test_document};

#[tokio::test]
async fn test_debug_list_workspaces() {
    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![test_workspace()]);
    
    let auth = AuthManager::default();
    let debug_ops = DebugOperations::new(
        Arc::new(mock_client),
        auth,
        None,
    );
    
    // Should not panic - actual output goes to stdout
    let result = debug_ops.list_workspaces().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_debug_list_workspaces_json() {
    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![test_workspace()]);
    
    let auth = AuthManager::default();
    let debug_ops = DebugOperations::new(
        Arc::new(mock_client),
        auth,
        None,
    );
    
    let result = debug_ops.list_workspaces_json().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_debug_list_tasks() {
    let mock_client = MockClickUpClient::new()
        .with_tasks(vec![test_task()]);
    
    let auth = AuthManager::default();
    let debug_ops = DebugOperations::new(
        Arc::new(mock_client),
        auth,
        None,
    );
    
    let result = debug_ops.list_tasks("list123").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_debug_list_tasks_json() {
    let mock_client = MockClickUpClient::new()
        .with_tasks(vec![test_task()]);
    
    let auth = AuthManager::default();
    let debug_ops = DebugOperations::new(
        Arc::new(mock_client),
        auth,
        None,
    );
    
    let result = debug_ops.list_tasks_json("list123").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_debug_search_docs() {
    let mock_client = MockClickUpClient::new()
        .with_documents(vec![test_document()]);
    
    let auth = AuthManager::default();
    let debug_ops = DebugOperations::new(
        Arc::new(mock_client),
        auth,
        None,
    );
    
    let result = debug_ops.search_docs("test").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_debug_search_docs_json() {
    let mock_client = MockClickUpClient::new()
        .with_documents(vec![test_document()]);
    
    let auth = AuthManager::default();
    let debug_ops = DebugOperations::new(
        Arc::new(mock_client),
        auth,
        None,
    );
    
    let result = debug_ops.search_docs_json("test").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_debug_auth_status_authenticated() {
    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![test_workspace()]);
    
    let auth = AuthManager::default();
    let debug_ops = DebugOperations::new(
        Arc::new(mock_client),
        auth,
        None,
    );
    
    let result = debug_ops.check_auth_status().await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_debug_auth_status_not_authenticated() {
    let mock_client = MockClickUpClient::new()
        .with_workspaces_error("Authentication failed".to_string());
    
    let auth = AuthManager::default();
    let debug_ops = DebugOperations::new(
        Arc::new(mock_client),
        auth,
        None,
    );
    
    let result = debug_ops.check_auth_status().await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}
