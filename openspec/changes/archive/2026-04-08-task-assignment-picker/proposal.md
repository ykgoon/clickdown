## Why

Task assignees are currently visible only as a read-only field in the task model (`assignees: Vec<User>`). Users cannot change who a task is assigned to from within the TUI — this requires opening ClickUp in a browser. Adding an assignee picker brings the full task management workflow into the terminal, eliminating context-switching for one of the most common task operations.

## What Changes

- Add API support for fetching list members via `GET /list/{list_id}/member`
- Display current assignees in the task detail view (read-only line showing assignee names)
- Introduce an overlay assignee picker dialog (triggered by pressing `A` in task detail)
- Support multi-select assignee assignment (toggle multiple users, save all at once)
- Cache list members in-memory per session to avoid repeated API calls
- Ignore `group_assignees` for this change (scope limited to direct assignees)

## Capabilities

### New Capabilities

- `list-members-api`: API endpoint, models, and trait method for fetching members who can access a list via `GET /list/{list_id}/member`. Includes mock client support for testing.
- `task-assignment-ui`: Assignee picker widget with multi-select, keyboard navigation (j/k to navigate, Space to toggle, Ctrl+S to save, Esc to cancel), and integration into the task detail screen. Includes updating task assignees via the existing `UpdateTaskRequest` API.
- `member-cache`: In-memory caching strategy for list members, stored in `TuiApp` state as a `HashMap<String, Vec<Member>>`, fetched lazily on first picker open per list per session.

### Modified Capabilities

- `tui-widgets`: Add assignee picker as a new widget type alongside existing sidebar, task list, task detail, auth view, document view, and comment widgets.
- `keyboard-shortcuts`: Add `A` key binding for opening assignee picker from task detail screen.

## Impact

- **API layer**: New `get_list_members` method on `ClickUpApi` trait, implemented in `ClickUpClient` and `MockClickUpClient`
- **Models**: New `MembersResponse` struct (reuses existing `User` type for individual members)
- **Endpoints**: New `list_members(list_id)` endpoint in `ApiEndpoints`
- **TUI**: New `assignee_picker.rs` widget, enhanced `task_detail.rs` to render assignees, new key handling in `app.rs`
- **State**: New `cached_list_members` HashMap in `TuiApp`, new `AppMessage` variants (`MembersLoaded`, `AssigneesUpdated`)
- **No breaking changes**: All additions are additive — existing functionality is untouched
- **No database changes**: Cache is in-memory only, cleared on app restart
