# ClickDown

A fast and responsive terminal-based ClickUp client built with Rust.

## Features

- **Fast & Native**: Built with Rust and ratatui TUI framework for native terminal performance
- **Workspace Navigation**: Browse workspaces, spaces, folders, and lists
- **Task Management**: View, create, edit, and delete tasks
- **Document Viewing**: Read ClickUp documents with Markdown rendering
- **Offline Cache**: SQLite-based caching for instant reloads
- **Dark Theme**: Easy on the eyes for extended use
- **Keyboard-Driven**: Vim-style navigation (j/k to navigate, Enter to select, Esc to go back)
- **Terminal Native**: Runs directly in your terminal with no GUI dependencies

## Requirements

- Rust 1.70+ (edition 2021)
- ClickUp Personal API Token (from Settings → Apps → ClickUp API)

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

## Running

```bash
cargo run
```

## Authentication

ClickDown uses Personal API Token authentication via a terminal-based form:

1. Obtain your Personal API Token from ClickUp:
   - Go to ClickUp web app
   - Navigate to Settings → Apps → ClickUp API
   - Generate a new token or copy an existing one
2. Launch ClickDown
3. Enter your Personal API Token using the keyboard on the authentication screen (characters are masked)
4. Press Enter to connect and authenticate
5. Your token is stored securely for future sessions

**Note:** The token is stored in `~/.config/clickdown/token` (Linux) with restrictive file permissions.

## Project Structure

```
src/
├── main.rs              # Application entry point
├── app.rs               # Main application state (Elm architecture pattern)
├── api/
│   ├── mod.rs           # API module
│   ├── client.rs        # ClickUp HTTP client
│   ├── auth.rs          # Token management
│   └── endpoints.rs     # API endpoint definitions
├── models/
│   ├── mod.rs           # Models module
│   ├── workspace.rs     # Workspace, Space, Folder, List types
│   ├── task.rs          # Task types
│   └── document.rs      # Document types
├── tui/
│   ├── mod.rs           # TUI module
│   ├── app.rs           # TUI application state and rendering
│   ├── terminal.rs      # Terminal initialization and cleanup
│   ├── layout.rs        # Screen layout definitions
│   ├── input.rs         # Keyboard input handling
│   ├── widgets/         # Reusable TUI widgets
│   │   ├── sidebar.rs   # Navigation sidebar
│   │   ├── task_list.rs # Task list view
│   │   ├── task_detail.rs # Task detail panel
│   │   ├── auth_view.rs # Authentication screen
│   │   └── components/  # Reusable widget components
├── cache/
│   ├── mod.rs           # SQLite cache module
│   └── schema.rs        # Database schema
└── config/
    ├── mod.rs           # Configuration management
    └── storage.rs       # Config file locations
```

## Architecture

ClickDown uses the **Elm Architecture** pattern adapted for TUI with ratatui:

- **Model**: Application state (`TuiApp` struct)
- **Update**: Message handling (`Message` enum)
- **View**: Terminal rendering (`render` methods)

The application uses a continuous rendering loop that:
1. Processes keyboard events via crossterm
2. Updates application state based on messages
3. Renders the terminal buffer with ratatui widgets
4. Runs at ~30 FPS for responsive interaction

## Configuration

Configuration is stored in:
- **Linux**: `~/.config/clickdown/`
- **macOS**: `~/Library/Application Support/clickdown/`
- **Windows**: `%APPDATA%\clickdown\`

Files:
- `config.toml` - Application settings
- `token` - API token (restricted permissions)
- `cache/cache.db` - SQLite cache database

## API Usage

The application uses the ClickUp API v2:
- Base URL: `https://api.clickup.com/api/v2`
- Authentication: Personal Token or OAuth

### Supported Endpoints

| Resource | Operations |
|----------|------------|
| Workspaces | List |
| Spaces | List |
| Folders | List |
| Lists | List |
| Tasks | List, Create, Update, Delete |
| Documents | List, View |

## Roadmap

### Completed ✅
- [x] Workspace navigation (Workspaces, Spaces, Folders, Lists)
- [x] Task list viewing with status and priority indicators
- [x] Task create/update/delete operations
- [x] Document viewing with Markdown rendering
- [x] SQLite caching layer
- [x] Configuration and token management
- [x] Dark theme TUI with vim-style navigation

### In Progress / Planned 🚧
- [ ] Task filtering and sorting (by status, priority, due date, assignee)
- [ ] Background sync mechanism (periodic refresh)
- [ ] Task comments viewing and creation
- [ ] Custom fields support
- [ ] Subtasks and checklists
- [ ] Task attachments
- [ ] Rich text editor for task descriptions
- [ ] Document editing
- [ ] Bulk operations
- [ ] Search functionality
- [ ] Activity log / history
- [ ] Time tracking
- [ ] Goal tracking
- [ ] Dashboard widgets
- [ ] Plugin/extensions system

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
