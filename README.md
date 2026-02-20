# ClickDown

A fast and responsive ClickUp desktop client built with Rust.

## Features

- **Fast & Native**: Built with Rust and iced GUI framework for native performance
- **Workspace Navigation**: Browse workspaces, spaces, folders, and lists
- **Task Management**: View, create, edit, and delete tasks
- **Document Viewing**: Read ClickUp documents with Markdown rendering
- **Offline Cache**: SQLite-based caching for instant reloads
- **Dark Theme**: Easy on the eyes for extended use
- **Responsive UI**: Clean, modern interface with sidebar navigation

## Requirements

- Rust 1.70+ (edition 2021)
- ClickUp API token (get from ClickUp Settings → Apps → ClickUp API)

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

## Project Structure

```
src/
├── main.rs              # Application entry point
├── app.rs               # Main application state (Elm architecture)
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
├── ui/
│   ├── mod.rs           # UI module
│   ├── sidebar.rs       # Navigation sidebar
│   ├── task_list.rs     # Task list view
│   ├── task_detail.rs   # Task detail panel
│   ├── auth_view.rs     # Authentication screen
│   └── components/      # Reusable UI components
├── cache/
│   ├── mod.rs           # SQLite cache module
│   └── schema.rs        # Database schema
└── config/
    ├── mod.rs           # Configuration management
    └── storage.rs       # Config file locations
```

## Architecture

ClickDown uses the **Elm Architecture** via the iced framework:

- **Model**: Application state (`ClickDown` struct)
- **Update**: Message handling (`Message` enum)
- **View**: UI rendering (`view` methods)

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
- [x] Dark theme UI

### In Progress / Planned 🚧
- [ ] Keyboard shortcuts (Ctrl+N for new task, Ctrl+S to save, etc.)
- [ ] Task filtering and sorting (by status, priority, due date, assignee)
- [ ] Background sync mechanism (periodic refresh)
- [ ] Task comments viewing and creation
- [ ] Custom fields support
- [ ] Subtasks and checklists
- [ ] Task attachments
- [ ] Rich text editor for task descriptions
- [ ] Document editing
- [ ] System tray integration
- [ ] Desktop notifications
- [ ] Multiple window support
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
