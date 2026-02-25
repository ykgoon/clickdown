## Why

Users need to view, create, and update comments on tasks directly within the terminal interface. Currently, the task detail view displays task properties but lacks comment functionality, forcing users to switch to the ClickUp web interface for comment management.

## What Changes

- **Task detail view** will display a comments section showing all task comments
- **Comment creation** capability added to task detail view with a text input form
- **Comment editing** capability for existing comments (user's own comments)
- **Text wrapping** for all comment content to prevent horizontal overflow
- **Comment metadata** displayed (author, creation date, updated date)
- **Keyboard navigation** extended to comments list with vim-style j/k keys

## Capabilities

### New Capabilities
- `task-comments`: Comment viewing, creation, and updating functionality within task detail view

### Modified Capabilities
- `tui-widgets`: Task detail widget requirements extended to include comments section and comment form

## Impact

- **UI**: Task detail layout modified to include comments panel with scrollable comment list
- **API**: New API calls for fetching, creating, and updating task comments via ClickUp API
- **Models**: New `Comment` model to represent comment data structure
- **Cache**: SQLite schema extended to cache comments locally
- **Keyboard shortcuts**: New shortcuts for comment actions (create, edit, save)
- **Text rendering**: Markdown rendering for comment content with proper text wrapping
