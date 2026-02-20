# ClickDown - Agent Context

## Project Overview

**ClickDown** is a fast and responsive desktop client for ClickUp, built with Rust and the [iced](https://iced.rs) GUI framework. It provides native performance for managing ClickUp workspaces, tasks, and documents.

### Key Features
- Workspace navigation (Workspaces → Spaces → Folders → Lists)
- Task management (view, create, edit, delete)
- Document viewing with Markdown rendering
- SQLite-based offline caching
- Dark theme UI with sidebar navigation

### Technology Stack
| Component | Technology |
|-----------|------------|
| Language | Rust (edition 2021) |
| GUI Framework | iced 0.13 |
| Async Runtime | tokio |
| HTTP Client | reqwest |
| Serialization | serde, serde_json |
| Database | rusqlite |
| Error Handling | thiserror, anyhow |
| Logging | tracing, tracing-subscriber |

## Architecture

### Elm Architecture
The application follows the **Elm Architecture** pattern via iced:
- **Model**: `ClickDown` struct holds application state
- **Update**: `Message` enum handles all state transitions
- **View**: `view()` methods render UI elements

### Dependency Injection
The API layer uses a trait-based dependency injection pattern for testability:
- `ClickUpApi` trait defines the API interface
- `ClickUpClient` implements real HTTP calls
- `MockClickUpClient` provides mock responses for testing

### Project Structure
```
src/
├── main.rs              # Entry point, logging initialization
├── app.rs               # Main application state (Elm architecture)
├── api/
│   ├── mod.rs           # Module exports
│   ├── client.rs        # Real HTTP client implementation
│   ├── client_trait.rs  # ClickUpApi trait definition
│   ├── auth.rs          # Token management
│   ├── endpoints.rs     # API endpoint URLs
│   └── mock_client.rs   # Mock client for testing
├── models/
│   ├── mod.rs           # Model exports
│   ├── workspace.rs     # Workspace, Space, Folder, List
│   ├── task.rs          # Task, TaskStatus, Priority, TaskFilters
│   └── document.rs      # Document, Page, DocumentFilters
├── ui/
│   ├── mod.rs           # UI module exports
│   ├── sidebar.rs       # Navigation sidebar
│   ├── task_list.rs     # Task list view
│   ├── task_detail.rs   # Task create/edit panel
│   ├── auth_view.rs     # Authentication screen
│   └── document_view.rs # Document/Markdown viewer
├── cache/
│   ├── mod.rs           # SQLite cache module
│   └── schema.rs        # Database schema
└── config/
    ├── mod.rs           # Configuration management
    └── storage.rs       # Config file locations

tests/
├── app_test.rs          # Integration tests
└── fixtures.rs          # Test data fixtures
```

## Building and Running

### Prerequisites
- Rust 1.70+ (edition 2021)
- ClickUp API token (from ClickUp Settings → Apps → ClickUp API)

### Commands
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run the application
cargo run

# Run all tests
cargo test

# Run specific test
cargo test --test app_test test_app_initialization_with_mock_client

# Run tests with output
cargo test -- --nocapture
```

### Configuration Storage
- **Linux**: `~/.config/clickdown/`
- **macOS**: `~/Library/Application Support/clickdown/`
- **Windows**: `%APPDATA%\clickdown\`

Files:
- `config.toml` - Application settings
- `token` - API token (keep out of version control)
- `cache/cache.db` - SQLite cache database

## Development Conventions

### Code Style
- Follows Rust edition 2021 idioms
- Uses `anyhow::Result` for application-level errors
- Uses `thiserror` for library-level error types
- Async functions use `async_trait` for trait implementations

### Testing Practices
1. **Headless Testing**: All tests use `MockClickUpClient` - no network calls
2. **Fixtures**: Reusable test data in `tests/fixtures.rs`
3. **Integration Tests**: Full application flow tests in `tests/app_test.rs`

Example test pattern:
```rust
use clickdown::api::MockClickUpClient;
use clickdown::app::{ClickDown, Message};

#[tokio::test]
async fn test_example() {
    let mock_client = MockClickUpClient::new()
        .with_workspaces(vec![test_workspace()]);
    
    let (mut app, _task) = ClickDown::with_client(Arc::new(mock_client));
    app.update(Message::Initialize);
    // ... assertions
}
```

### Message Pattern
All state changes flow through the `Message` enum in `app.rs`:
- Authentication: `TokenEntered`, `AuthSuccess`, `AuthError`, `Logout`
- Navigation: `WorkspaceSelected`, `SpaceSelected`, `FolderSelected`, `ListSelected`
- Tasks: `TasksLoaded`, `TaskSelected`, `TaskCreated`, `TaskUpdated`, `TaskDeleted`
- UI: `ToggleSidebar`, `WindowResized`, `WindowCloseRequested`

### Error Handling
- API errors propagate through `Message::AuthError`
- Errors are displayed in the UI and logged via `tracing`
- Use `?` operator for error propagation, `anyhow` for wrapping

## API Reference

### ClickUpApi Trait Methods
| Method | Description |
|--------|-------------|
| `get_workspaces()` | Get all authorized workspaces |
| `get_spaces(team_id)` | Get spaces in a team/workspace |
| `get_folders(space_id)` | Get folders in a space |
| `get_lists_in_folder(folder_id, archived)` | Get lists in a folder |
| `get_tasks(list_id, filters)` | Get tasks in a list |
| `get_task(task_id)` | Get a single task |
| `create_task(list_id, request)` | Create a new task |
| `update_task(task_id, request)` | Update a task |
| `delete_task(task_id)` | Delete a task |
| `search_docs(filters)` | Search documents |
| `get_doc_pages(doc_id)` | Get pages in a document |

### Mock Client Configuration
```rust
MockClickUpClient::new()
    .with_workspaces(vec![...])
    .with_spaces(vec![...])
    .with_folders(vec![...])
    .with_lists_in_folder(vec![...])
    .with_tasks(vec![...])
    .with_create_task_response(task)
    .with_update_task_response(task)
    .with_delete_task_success()
    .with_documents(vec![...])
    .with_pages(vec![...])
```

## Current Status

### Completed ✅
- Workspace navigation hierarchy
- Task list viewing with status/priority indicators
- Task CRUD operations
- Document viewing with Markdown rendering
- SQLite caching layer
- Configuration and token management
- Dark theme UI

### Roadmap 🚧
- Keyboard shortcuts (Ctrl+N, Ctrl+S, etc.)
- Task filtering and sorting
- Background sync mechanism
- Task comments
- Custom fields support
- Subtasks and checklists
- Rich text editor for descriptions
- Document editing
- Search functionality

## Related Documentation
- `README.md` - User-facing documentation and feature overview
- `TESTING.md` - Detailed headless testing guide
- `Cargo.toml` - Dependency versions and build configuration
