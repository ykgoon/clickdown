## Why

Users currently lose their navigation context every time they close and reopen ClickDown, forcing them to re-navigate through the workspace hierarchy from scratch. This change adds session persistence to restore users to their last viewed location, improving productivity and reducing friction in daily workflows.

## What Changes

- **New**: Application state is saved on exit (current screen, selected items in navigation hierarchy)
- **New**: On startup, application restores the user to their last viewed location
- **New**: Session data is persisted to SQLite cache database
- **New**: Graceful handling of stale session data (e.g., restored list no longer exists)
- **Modified**: Application shutdown process to include state serialization
- **Modified**: Application initialization to include state restoration

## Capabilities

### New Capabilities

- `session-state-persistence`: Persistence and restoration of application state including current screen type, navigation hierarchy selections (workspace, space, folder, list IDs), and optionally scroll positions or filters

### Modified Capabilities

- `application-lifecycle`: Extending shutdown and initialization requirements to include session state serialization and restoration

## Impact

- **Database**: SQLite cache schema needs new table for session state
- **TUI App**: `TuiApp::run()` shutdown flow needs state serialization; `TuiApp::new()` initialization needs state restoration logic
- **Cache Module**: New methods to save/load session state
- **Error Handling**: Need graceful degradation when restored IDs reference deleted resources
- **User Experience**: Silent restore expected; may need status message showing what was restored
