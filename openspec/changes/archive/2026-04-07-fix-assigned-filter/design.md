## Context

The "Assigned to Me" filter (`a` key in task list view) currently returns zero tasks. Investigation reveals two root causes:

1. **Missing user ID initialization**: The `current_user_id` field is never populated during normal TUI operation. The `CurrentUserLoaded` message handler exists but no code path sends it. The previous `fetch_current_user_and_load_tasks()` function was removed during refactoring. When `current_user_id` is restored from session cache, it may contain a stale or incorrect value.

2. **Closed tasks excluded by default**: `include_closed` is not set when fetching assigned tasks, so the ClickUp API excludes completed tasks from results.

**Important note on assignees parameter format**: During initial investigation, it was hypothesized that the assignees query parameter format was wrong (array vs comma-separated). Testing against the live ClickUp API confirmed that **array format (`assignees[]=123`) is correct** — the API returns `400 Bad Request: {"err":"assignees must be an array","ECODE":"PUBAPITASK_017"}` for comma-separated format. The existing code was already using the correct format. The existing `task-filtering` main spec is incorrect on this point.

The codebase already has infrastructure to handle the real issues:
- `get_current_user()` API method exists and works
- `CurrentUserLoaded` message handler exists in `process_async_messages()`
- `add_all` method produces the correct array format for assignees

## Goals / Non-Goals

**Goals:**
- Fix assignees query parameter to use comma-separated format
- Wire up `get_current_user()` call during TUI initialization
- Include closed tasks in assigned filter results
- Maintain backward compatibility with existing code paths

**Non-Goals:**
- Do not re-introduce global "Assigned to Me" aggregation view (that was intentionally removed)
- Do not change the ClickUp API client trait interface
- Do not modify caching behavior

## Decisions

### Decision 1: Confirm array format for assignees (no code change needed)

**Choice**: Keep `params.add_all("assignees", &self.assignees)` which produces `assignees[]=123&assignees[]=456`.

**Rationale**: The ClickUp API v2 explicitly requires array format. Testing confirmed comma-separated format produces `400 Bad Request: {"err":"assignees must be an array","ECODE":"PUBAPITASK_017"}`. The existing implementation was already correct — the hypothesis that the format was wrong was disproven by live API testing.

**Note**: The existing `task-filtering` main spec incorrectly states comma-separated format is required. This delta spec corrects that record.

**Alternatives considered**:
- Switch to `add_comma_separated_ints` — produces wrong format, causes 400 error (tested)
- Create a new method — unnecessary, `add_all` already works

### Decision 2: Spawn `get_current_user()` in `TuiApp::new()`

**Choice**: After creating the API client in `TuiApp::new()`, spawn a background task that calls `get_current_user()` and sends `AppMessage::CurrentUserLoaded`.

**Rationale**: This is the minimal change to wire up the existing handler. The handler already sets `current_user_id` and logs the result.

**Alternatives considered**:
- Store user ID in config file — adds complexity, token already proves identity
- Call `get_current_user()` synchronously during init — blocks startup, poor UX
- Use a different message type — unnecessary, `CurrentUserLoaded` already exists

### Decision 3: Set `include_closed=true` in `get_tasks_with_assignee()`

**Choice**: Add `include_closed: Some(true)` to the `TaskFilters` in `get_tasks_with_assignee()`.

**Rationale**: Users expect "Assigned to Me" to show ALL their tasks, including completed ones. The ClickUp API excludes closed tasks by default.

**Alternatives considered**:
- Make it a configurable option — over-engineering for now
- Don't include closed — users won't see their completed work

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| `get_current_user()` API call fails on slow networks | Non-fatal — `current_user_id` stays `None`, filter shows error message |
| User ID from `get_current_user()` differs from task assignee IDs | ClickUp uses consistent user IDs across endpoints; logging helps diagnose mismatches |
| Including closed tasks may return large result sets | `limit=100` is already set on `get_tasks_with_assignee()` |
| Race condition: user presses `a` before `get_current_user()` completes | The filter checks `current_user_id` — if `None`, shows "User ID not available" message. User can retry after a moment. |
