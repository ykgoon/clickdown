# ClickDown - Agent Context

## Project Overview

**ClickDown** is a fast and responsive terminal-based client for ClickUp, built with Rust and the [ratatui](https://ratatui.rs) TUI framework with [crossterm](https://github.com/crossterm-rs/crossterm) backend. It provides native terminal performance for managing ClickUp workspaces, tasks, and documents with keyboard-driven navigation.

### Key Features
- Workspace navigation (Workspaces → Spaces → Folders → Lists)
- Task management (view, create, edit, delete)
- Document viewing with Markdown rendering
- SQLite-based offline caching
- Dark theme TUI with vim-style keyboard navigation
- Terminal-native: runs in any terminal with no GUI dependencies

### Technology Stack
| Component | Technology |
|-----------|------------|
| Language | Rust (edition 2021) |
| TUI Framework | ratatui 0.29 + crossterm 0.28 |
| Async Runtime | tokio |
| HTTP Client | reqwest |
| Serialization | serde, serde_json |
| Database | rusqlite |
| Error Handling | thiserror, anyhow |
| Logging | tracing, tracing-subscriber |

## Architecture

### Elm Architecture Pattern for TUI
The application follows the **Elm Architecture** pattern adapted for terminal user interfaces:
- **Model**: `TuiApp` struct holds application state
- **Update**: `Message` enum handles all state transitions
- **View**: `render()` methods draw UI elements to terminal buffer

### Rendering Loop
The application uses a continuous rendering loop powered by ratatui and crossterm:
1. Terminal is initialized in raw mode for direct keyboard capture
2. Events (keyboard input, terminal resize) are captured via crossterm
3. Events are converted to `Message` variants and processed
4. State is updated and terminal buffer is re-rendered
5. Loop runs at ~30 FPS for responsive interaction
6. On exit, terminal state is restored to normal

### Dependency Injection
The API layer uses a trait-based dependency injection pattern for testability:
- `ClickUpApi` trait defines the API interface
- `ClickUpClient` implements real HTTP calls
- `MockClickUpClient` provides mock responses for testing

### Project Structure
```
src/
├── main.rs              # Entry point, logging initialization
├── lib.rs               # Library root
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
├── tui/
│   ├── mod.rs           # TUI module exports
│   ├── app.rs           # TUI application state and rendering loop
│   ├── terminal.rs      # Terminal initialization and cleanup
│   ├── layout.rs        # Screen layout definitions
│   ├── input.rs         # Keyboard input handling
│   └── widgets/         # TUI widgets
│       ├── sidebar.rs   # Navigation sidebar
│       ├── task_list.rs # Task list view
│       ├── task_detail.rs # Task create/edit panel
│       ├── auth_view.rs # Authentication screen
│       └── document_view.rs # Document/Markdown viewer
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

## Debugging with CLI Mode

ClickDown includes a CLI debug mode for headless debugging and bug reproduction. This is useful for:
- Reproducing bugs reported by users
- Testing API connectivity without launching the TUI
- Inspecting raw API responses
- Debugging authentication issues

### Quick Start

```bash
# Show all available commands
clickdown --help
clickdown debug --help

# Check authentication status
clickdown debug auth-status
echo $?  # 0 = authenticated, 3 = not authenticated

# List workspaces (human-readable)
clickdown debug workspaces

# List workspaces (JSON for inspection)
clickdown debug workspaces --json | jq '.[] | {id, name}'

# Fetch tasks from a list
clickdown debug tasks <list_id>
clickdown debug tasks <list_id> --json

# Search documents
clickdown debug docs "Sprint Planning"
clickdown debug docs "Sprint Planning" --json
```

### Verbose Logging

Use `--verbose` to see HTTP requests and responses. Logs go to stderr, data goes to stdout:

```bash
# Verbose output with JSON data
clickdown debug workspaces --json --verbose

# Pipe data while seeing logs
clickdown debug tasks list123 --json --verbose 2>debug.log | jq '.[].name'

# Full trace logging
RUST_LOG=trace clickdown debug auth-status --verbose
```

### Token Override for Testing

Test with different tokens without modifying the stored token:

```bash
# Use alternate token (not saved to disk)
clickdown debug workspaces --token pk_test_abc123

# Combine with verbose to see auth behavior
clickdown debug auth-status --token pk_test_abc123 --verbose
```

**Warning:** The override token is NOT saved. Logs never include the token value.

### Exit Codes

| Code | Meaning | When |
|------|---------|------|
| 0 | Success | Operation completed |
| 1 | General error | Unexpected error |
| 2 | Invalid arguments | Bad CLI syntax |
| 3 | Auth error | Invalid/missing token |
| 4 | Network error | Connection failed |

### Common Debugging Workflows

**Reproduce a bug:**
1. Check auth: `clickdown debug auth-status`
2. List workspaces: `clickdown debug workspaces --json`
3. Fetch specific data: `clickdown debug tasks <list_id> --json`
4. Inspect with verbose: `clickdown debug tasks <list_id> --verbose`

**Test before TUI launch:**
```bash
# Quick connectivity check
clickdown debug auth-status && cargo run
```

**Compare API vs. cache:**
```bash
# Fetch fresh data
clickdown debug workspaces --json > fresh.json

# Launch TUI and compare
cargo run
```

### CLI Mode vs. Unit Tests

| Use CLI Mode | Use Unit Tests |
|--------------|----------------|
| Real API calls | Mock data |
| Headless, quick iteration | Automated, reproducible |
| Inspect live data | Test edge cases |
| Reproduce user bugs | Regression testing |

See [TESTING.md](TESTING.md) for mock client testing patterns.

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
- TUI: `ToggleSidebar`, `TerminalResized`, `QuitRequested`, `KeyPressed`

### Error Handling
- API errors propagate through `Message::AuthError`
- Errors are displayed in the TUI status bar and logged via `tracing`
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
- Dark theme TUI with vim-style keyboard navigation (j/k to navigate, Enter to select, Esc to go back)
- Terminal initialization and cleanup with crossterm
- Keyboard input handling for forms and navigation
- Screen titles and status bar with contextual help

### Roadmap 🚧
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
