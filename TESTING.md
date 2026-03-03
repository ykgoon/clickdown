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

## CLI Debug Mode Testing

ClickDown includes a CLI debug mode that can be used for manual testing and debugging:

### Running CLI Tests

```bash
# Run CLI integration tests
cargo test --test cli_test

# Run specific CLI test
cargo test --test cli_test test_debug_list_workspaces

# Run CLI tests with output
cargo test --test cli_test -- --nocapture
```

### CLI Test Structure

CLI tests use the same `MockClickUpClient` pattern as integration tests:

```rust
use clickdown::api::MockClickUpClient;
use clickdown::commands::DebugOperations;
use clickdown::api::AuthManager;

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
    
    let result = debug_ops.list_workspaces().await;
    assert!(result.is_ok());
}
```

### Manual CLI Testing

For manual testing without the TUI:

```bash
# Build the CLI
cargo build

# Test authentication
./target/debug/clickdown debug auth-status

# Test workspace listing
./target/debug/clickdown debug workspaces

# Test with JSON output
./target/debug/clickdown debug workspaces --json | jq

# Test verbose logging
./target/debug/clickdown debug workspaces --verbose

# Test token override
./target/debug/clickdown debug auth-status --token test_token
```

### CLI Exit Codes

When testing CLI behavior, check exit codes:

```bash
# Success (authenticated)
./target/debug/clickdown debug auth-status
echo $?  # Should be 0

# Auth error
./target/debug/clickdown debug auth-status --token invalid
echo $?  # Should be 3

# Invalid arguments
./target/debug/clickdown debug invalid_command
echo $?  # Should be 2
```

### CLI vs. Unit Tests

| CLI Tests | Unit Tests |
|-----------|------------|
| Test CLI argument parsing | Test business logic |
| Test output formatting | Test state transitions |
| Manual verification | Automated regression |
| Real terminal behavior | Headless execution |

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

## Snapshot Testing

ClickDown uses snapshot testing with the [`insta`](https://insta.rs/) crate to capture and compare rendered UI output. Snapshot tests provide visual regression testing for TUI widgets and layouts.

### Running Snapshot Tests

```bash
# Run all snapshot tests
cargo test --test snapshot_test

# Run specific snapshot test
cargo test --test snapshot_test test_sidebar_empty

# Run snapshot tests with output
cargo test --test snapshot_test -- --nocapture
```

### Reviewing and Accepting Snapshots

When snapshot tests fail (because snapshots changed), you need to review and accept the new snapshots:

```bash
# Install cargo-insta (one-time)
cargo install cargo-insta

# Review snapshots interactively
cargo insta review

# Accept all pending snapshots
cargo insta accept

# Auto-update snapshots during test run
INSTA_UPDATE=always cargo test --test snapshot_test
```

### Snapshot Organization

Snapshots are stored in `tests/snapshots/` organized by category:

- `tests/snapshots/snapshot_test__*.snap` - Widget and layout snapshots

Snapshot files contain the expected rendered output. When UI changes, tests will fail and show the difference between old and new output.

### Adding New Snapshot Tests

1. **Add a new test function** in `tests/snapshot_test.rs`:

```rust
#[test]
fn test_my_new_widget() {
    let widget = MyWidgetState::new();
    
    assert_widget_snapshot("my_widget", 80, 24, |frame| {
        let area = Rect::new(0, 0, 80, 24);
        render_my_widget(frame, &widget, area);
    });
}
```

2. **Run the test** - it will fail and create a new snapshot file

3. **Review the snapshot** with `cargo insta review` or accept with `INSTA_UPDATE=always`

4. **Commit the snapshot file** along with your test

### Snapshot Test Best Practices

- Use fixed terminal sizes (80x24, 120x30, 160x40) for consistency
- Use deterministic test data from `fixtures.rs`
- Name snapshots descriptively (e.g., `sidebar_empty`, `task_list_with_selection`)
- Review snapshots carefully before accepting - they become the source of truth
- Update snapshots when intentionally changing UI behavior

### CI/CD Integration

In CI, snapshot tests run automatically. If snapshots don't match, the build fails. To update snapshots in CI:

1. Run tests locally with `INSTA_UPDATE=always`
2. Commit the updated snapshot files
3. Push changes

### Troubleshooting Snapshot Tests

**Test fails with "no snapshot found":**
- Run with `INSTA_UPDATE=always` to create initial snapshot
- Check that snapshot file exists in `tests/snapshots/`

**Test fails with "snapshot assertion failed":**
- Review differences with `cargo insta review`
- Accept changes if UI update was intentional
- Fix code if regression was unintentional

**Snapshots differ across platforms:**
- Ensure tests use fixed terminal sizes
- Avoid platform-specific rendering paths in tests
- Use `insta`'s platform-agnostic settings

## Future Enhancements

Potential improvements to the testing framework:

1. **Property-based Testing** - Generate random test data
2. **Integration with CI** - Automated snapshot review on pull requests
3. **Coverage Reports** - Track test coverage with `cargo tarpaulin`
5. **Performance Tests** - Benchmark UI rendering and state updates
