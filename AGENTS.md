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

# Get comments for a task
clickdown debug comments <task_id>
clickdown debug comments <task_id> --json

# Create a new comment
clickdown debug create-comment <task_id> --text "Comment text"
clickdown debug create-comment <task_id> --text "Text" --json

# Create a reply to a comment
clickdown debug create-reply <comment_id> --text "Reply text"
clickdown debug create-reply <comment_id> --text "Text" --json

# Update an existing comment
clickdown debug update-comment <comment_id> --text "Updated text"
clickdown debug update-comment <comment_id> --text "Text" --json

# Comment options (for create-comment)
clickdown debug create-comment <task_id> --text "Text" --parent-id <comment_id>
clickdown debug create-comment <task_id> --text "Text" --assignee <user_id>
clickdown debug create-comment <task_id> --text "Text" --assigned-commenter <user_id>
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

**Debug comment parsing issues:**

When you encounter "failed to parse" errors with comment operations:

1. **Reproduce with verbose logging**:
   ```bash
   # Create a comment with full logging
   clickdown debug create-comment <task_id> --text "Test" --verbose 2>&1 | tee debug.log
   
   # Create a reply with JSON output
   clickdown debug create-reply <comment_id> --text "Reply" --verbose --json
   ```

2. **Identify the failing field**:
   - Error messages now include field paths via `serde_path_to_error`
   - Look for errors like: `date: invalid type: floating point...`
   - The error format is: `path.to.field: error message`

3. **Common parsing failures**:
   - **Float timestamps**: API returns `1234567890.123` instead of `1234567890` or `"1234567890"`
   - **ISO 8601 dates**: API returns `"2024-01-15T10:30:00Z"` instead of milliseconds
   - **Type mismatches**: User ID as string instead of integer
   - **Null for required fields**: Handled by custom deserializers

4. **Check the Comment model**:
   - See `src/models/comment.rs` for deserializer documentation
   - The module doc explains known API variations and debugging steps
   - Custom deserializers handle: null strings, null bools, flexible timestamps

5. **Add test coverage**:
   - Add a test with the problematic JSON to `src/models/comment.rs`
   - Verify the deserializer handles the edge case
   - Run `cargo test --lib comment` to verify

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
4. **Snapshot Testing for Visual Bugs**: When bugs are reported about incorrect user experience, especially visual issues (layout, rendering, colors, text alignment, etc.), the correct workflow is to **create a snapshot test to reproduce the behavior first** before attempting to fix it. This ensures:
   - The bug is accurately captured and reproducible
   - The fix can be verified against the exact same conditions
   - Regression testing is available for future changes

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

### ClickUp API v2 Reference

**Base URL:** `https://api.clickup.com/api/v2`

**Authentication:** All endpoints require authentication via header:
```
Authorization: pk_{your_api_token}
```

**Rate Limits:** Apply to all API requests. See [Rate Limits](https://developer.clickup.com/docs/rate-limits).

#### Workspace Hierarchy Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/team` | `GET` | Get all authorized workspaces |
| `/team/{team_id}/space` | `GET` | Get spaces in a workspace |
| `/team/{team_id}/space` | `POST` | Create a new space |
| `/space/{space_id}` | `GET` | Get a specific space |
| `/space/{space_id}` | `PUT` | Update a space |
| `/space/{space_id}` | `DELETE` | Delete a space |
| `/space/{space_id}/folder` | `GET` | Get folders in a space |
| `/space/{space_id}/folder` | `POST` | Create a folder |
| `/folder/{folder_id}` | `GET` | Get a specific folder |
| `/folder/{folder_id}` | `PUT` | Update a folder |
| `/folder/{folder_id}` | `DELETE` | Delete a folder |
| `/folder/{folder_id}/list` | `GET` | Get lists in a folder |
| `/space/{space_id}/list` | `GET` | Get folderless lists in a space |
| `/folder/{folder_id}/list` | `POST` | Create a list in a folder |
| `/space/{space_id}/list` | `POST` | Create a folderless list |
| `/list/{list_id}` | `GET` | Get a specific list |
| `/list/{list_id}` | `PUT` | Update a list |
| `/list/{list_id}` | `DELETE` | Delete a list |

**Query Parameters:**
- `archived` (boolean) - Include archived items (for spaces, folders, lists)

#### Tasks API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/list/{list_id}/task` | `GET` | Get tasks in a list |
| `/list/{list_id}/task` | `POST` | Create a new task |
| `/task/{task_id}` | `GET` | Get a single task |
| `/task/{task_id}` | `PUT` | Update a task |
| `/task/{task_id}` | `DELETE` | Delete a task |
| `/task/{task_id}/field/{field_id}` | `PUT` | Set custom field value |
| `/task/{task_id}/relationship` | `GET/POST/DELETE` | Manage task relationships |

**Task Creation/Update Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Task name |
| `description` | string | No | Plain text description |
| `markdown_description` | string | No | Markdown-formatted description |
| `assignees` | array | No | Array of user IDs |
| `status` | string | No | Task status |
| `priority` | integer | No | Priority: 1=Urgent, 2=High, 3=Normal, 4=Low |
| `start_date` | string | No | Start date (with optional time) |
| `due_date` | string | No | Due date (with optional time) |
| `tags` | array | No | Tags for categorization |
| `custom_fields` | array | No | Custom field values (creation only) |
| `time_estimate` | integer | No | Time estimate in **milliseconds** |
| `points` | integer | No | Story points |
| `parent` | string | No | Parent task ID for subtasks |

**Important Notes:**
- Dates follow ClickUp's date formatting standards
- `time_estimate` must be in **milliseconds**
- Custom fields can only be set during creation; use `/task/{task_id}/field/{field_id}` to update
- Escape double quotes in descriptions with `\"`
- See [Tasks API](https://developer.clickup.com/docs/tasks) for full documentation

#### Comments API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/task/{task_id}/comment` | `GET` | Get comments for a task |
| `/task/{task_id}/comment` | `POST` | Create a comment |
| `/comment/{comment_id}` | `PUT` | Update a comment |
| `/comment/{comment_id}` | `DELETE` | Delete a comment |

**Comment Fields:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `comment_text` | string | Comment content (supports Markdown) |
| `assignee` | integer | User ID to assign comment to |
| `parent_id` | string | Parent comment ID for replies |

**Notes:**
- Comments support Markdown formatting
- Pagination available for task comments
- See [Comments API](https://developer.clickup.com/docs/comments) for details

#### Documents API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/doc` | `GET` | Search documents |
| `/doc/{doc_id}` | `GET` | Get a document |
| `/doc/{doc_id}/page` | `GET` | Get pages in a document |

**Notes:**
- Documents support Markdown rendering
- See [ClickUp Docs](https://developer.clickup.com/docs) for details

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
| `get_task_comments(task_id)` | Get comments for a task |
| `get_comment_replies(comment_id)` | Get replies to a comment |
| `create_comment(task_id, request)` | Create a new comment |
| `create_comment_reply(parent_id, request)` | Create a reply to a comment |
| `update_comment(comment_id, request)` | Update a comment |

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
    .with_task_comments(vec![...])
    .with_comment_replies("comment_id", vec![...])
    .with_create_comment_response(comment)
    .with_create_comment_reply_response(comment)
    .with_update_comment_response(comment)
```

### Related API Documentation

| Topic | URL |
|-------|-----|
| Authentication | https://developer.clickup.com/docs/authentication |
| Rate Limits | https://developer.clickup.com/docs/rate-limits |
| Date Formatting | https://developer.clickup.com/docs/date-formatting |
| Tasks API | https://developer.clickup.com/docs/tasks |
| Spaces API | https://developer.clickup.com/docs/spaces |
| Custom Fields | https://developer.clickup.com/docs/custom-fields |
| Comments API | https://developer.clickup.com/docs/comments |
| ClickUp Docs | https://developer.clickup.com/docs/clickup-docs |
| API v2 vs v3 | https://developer.clickup.com/docs/clickup-api-v2-and-v3-terminology |
| FAQ | https://developer.clickup.com/docs/faq |
| Common Errors | https://developer.clickup.com/docs/common-errors |

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
- Comment operations (view, create, reply, update)
- CLI debug commands for comment debugging
- Help dialog with keyboard shortcuts reference (`?` to toggle)

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
