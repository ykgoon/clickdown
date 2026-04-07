## Why

The "Assigned to Me" filter (pressing `a` in a task list) returns zero tasks even when the user has assigned tasks. Investigation reveals two root causes:

1. **Missing user ID initialization**: `current_user_id` is never populated during normal TUI operation — the `CurrentUserLoaded` message handler exists but nothing ever sends it. The background fetch of `get_current_user()` was removed during refactoring but the `a` key filter depends on this value. When `current_user_id` is restored from session cache, it may contain a stale or incorrect value.

2. **Closed tasks excluded by default**: `include_closed` is not set when fetching assigned tasks, so the ClickUp API excludes completed tasks from results.

**Note**: The existing `task-filtering` spec incorrectly states the assignees parameter should use comma-separated format. ClickUp API v2 actually requires array format (`assignees[]=123&assignees[]=456`). Using comma-separated format results in `400 Bad Request: {"err":"assignees must be an array","ECODE":"PUBAPITASK_017"}`. This change confirms array format is correct.

## What Changes

- Wire up `get_current_user()` call during TUI initialization so `current_user_id` is populated with the correct user ID on every launch
- Set `include_closed=true` when fetching assigned tasks to avoid hiding completed tasks
- Confirm array format for assignees query parameter (not comma-separated)
- Add `include_closed` as a parameter to `get_tasks_with_assignee()`

## Capabilities

### New Capabilities
- `user-id-fetching`: Background fetch of current user profile during TUI initialization to populate `current_user_id` for assignee filtering

### Modified Capabilities
- `task-filtering`: Clarify that `assignees` parameter uses array format (`assignees[]=123`), not comma-separated; add `include_closed` requirement for assigned task fetches

## Impact

- `src/tui/app.rs`: Add `get_current_user()` spawn during initialization
- `src/api/client.rs`: `get_tasks_with_assignee()` — add `include_closed=true` parameter
- `src/models/task.rs`: Add test for `include_closed` in query string; confirm array format for assignees
- `openspec/specs/task-filtering/spec.md`: Existing main spec has wrong assignees format — this change adds a delta spec with the correct format
