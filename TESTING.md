# Headless Testing Guide for ClickDown

This document explains how to run headless tests for the ClickDown application with mocked ClickUp API responses.

## Overview

ClickDown uses a **dependency injection** pattern to enable headless testing without making actual network calls. The key components are:

1. **`ClickUpApi` trait** (`src/api/client_trait.rs`) - Defines the API interface
2. **`ClickUpClient`** (`src/api/client.rs`) - Real implementation that makes HTTP calls
3. **`MockClickUpClient`** (`src/api/mock_client.rs`) - Mock implementation for testing
4. **Test fixtures** (`tests/fixtures.rs`) - Predefined test data

## Running Tests

### Run All Tests

```bash
cargo test
```

### Run Integration Tests Only

```bash
cargo test --test app_test
```

### Run Specific Test

```bash
cargo test --test app_test test_app_initialization_with_mock_client
```

### Run Tests with Output

```bash
cargo test --test app_test -- --nocapture
```

## Architecture

### Dependency Injection

The application uses trait objects to enable swapping between real and mock clients:

```rust
// In app.rs
pub struct ClickDown {
    client: Option<Arc<dyn ClickUpApi>>,
    // ...
}
```

### Mock Client

The `MockClickUpClient` can be configured to return predefined responses:

```rust
use clickdown::api::MockClickUpClient;

let mock_client = MockClickUpClient::new()
    .with_workspaces(vec![test_workspace()])
    .with_spaces(vec![test_space()])
    .with_tasks(test_tasks());
```

### Test Fixtures

Predefined test data is available in `tests/fixtures.rs`:

- `test_workspace()` - Sample workspace
- `test_space()` - Sample space
- `test_folder()` - Sample folder
- `test_list()` - Sample list
- `test_task()` - Sample task
- `test_tasks()` - Multiple sample tasks
- `test_document()` - Sample document
- `test_page()` - Sample page

## Writing Tests

### Basic Test Structure

```rust
use clickdown::api::MockClickUpClient;
use clickdown::app::{ClickDown, Message, AppState};
use std::sync::Arc;

#[tokio::test]
async fn test_example() {
    // 1. Create mock client with predefined responses
    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![test_workspace()]);
    
    // 2. Initialize app with mock client
    let (mut app, _task) = ClickDown::with_client(Arc::new(mock_client));
    
    // 3. Simulate user actions
    app.update(Message::Initialize);
    
    // 4. Assert expected state
    assert_eq!(app.state(), &AppState::Main);
}
```

### Testing API Error Handling

```rust
#[tokio::test]
async fn test_api_error() {
    let mock_client = MockClickUpClient::new()
        .with_workspaces_error("API connection failed".to_string());
    
    let (mut app, _task) = ClickDown::with_client(Arc::new(mock_client));
    
    // Trigger the error
    let workspaces = mock_client.get_workspaces().await;
    assert!(workspaces.is_err());
}
```

### Testing Task Operations

```rust
#[tokio::test]
async fn test_create_task() {
    let new_task = test_task();
    let mock_client = MockClickUpClient::new()
        .with_create_task_response(new_task.clone());
    
    // Test task creation logic
    let created = mock_client.create_task("list-123", &CreateTaskRequest {
        name: "New Task".to_string(),
        ..Default::default()
    }).await.unwrap();
    
    assert_eq!(created.name, "New Task");
}
```

## Headless Testing Benefits

1. **No Network Required** - Tests run without internet connection
2. **Fast Execution** - No API rate limits or network latency
3. **Deterministic** - Tests always return the same results
4. **No API Credentials** - Tests don't need real ClickUp tokens
5. **CI/CD Friendly** - Can run in headless CI environments

## Testing the UI (Advanced)

For UI testing without displaying windows, you can use iced's advanced testing features:

```rust
use iced::advanced::testing::harness;

#[test]
fn test_view_rendering() {
    harness::run(|_| {
        let (app, _) = ClickDown::with_client(Arc::new(mock_client));
        let view = app.view();
        // Assert view properties
    });
}
```

## Mock Client Configuration Methods

| Method | Description |
|--------|-------------|
| `with_workspaces(vec)` | Set workspaces response |
| `with_workspaces_error(str)` | Set workspaces error |
| `with_spaces(vec)` | Set spaces response |
| `with_folders(vec)` | Set folders response |
| `with_lists_in_folder(vec)` | Set lists in folder response |
| `with_tasks(vec)` | Set tasks response |
| `with_task(task)` | Set single task response |
| `with_create_task_response(task)` | Set create task response |
| `with_update_task_response(task)` | Set update task response |
| `with_delete_task_success()` | Set delete task success |
| `with_documents(vec)` | Set documents response |
| `with_pages(vec)` | Set pages response |

## Troubleshooting

### Tests Hang

If tests hang, ensure you're properly awaiting async operations:

```rust
// Good
let result = mock_client.get_workspaces().await;

// Bad - missing .await
let result = mock_client.get_workspaces();
```

### State Mismatch

If state assertions fail, check that you're simulating the complete flow:

```rust
// Complete flow
app.update(Message::Initialize);
let workspaces = mock_client.get_workspaces().await.unwrap();
app.update(Message::WorkspacesLoaded(workspaces));
// Now state should be Main
```

## Future Enhancements

Potential improvements to the testing framework:

1. **Snapshot Testing** - Compare UI views against saved snapshots
2. **Property-based Testing** - Generate random test data
3. **Integration with CI** - Automated test runs on pull requests
4. **Coverage Reports** - Track test coverage with `cargo tarpaulin`
5. **Performance Tests** - Benchmark UI rendering and state updates
