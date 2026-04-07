## Context

The "Assigned to Me" feature aggregates tasks and comments assigned to the current user into a unified view. It requires `current_user_id` to filter items from the ClickUp API. Currently, user ID detection is **reactive**:

1. User must navigate to a task list
2. User must open a task detail
3. `detect_user_id_from_task()` extracts user ID from `task.creator` or `task.assignees[0]`
4. Only then can "Assigned to Me" work

This creates a broken first-time experience where users who click "Assigned to Me" immediately see an error: "User identity not detected. Navigate to a task list and open a task..."

The ClickUp API provides a `GET /user` endpoint that returns the current authenticated user's profile, including their ID. This endpoint is already used in `fetch_current_user_and_load_tasks()` but only as a fallback.

**Constraints:**
- Must not block app startup (async fetch only)
- Must not show errors during initialization (graceful degradation)
- Must preserve existing task-based detection as fallback
- Session restore behavior unchanged

## Goals / Non-Goals

**Goals:**
- Proactively fetch user profile during app initialization (after workspaces load)
- Enable "Assigned to Me" to work immediately on first launch
- Maintain backwards compatibility with existing detection mechanisms
- Add on-demand user fetch fallback if user_id still unavailable

**Non-Goals:**
- Changing session restore behavior (out of scope)
- Modifying the ClickUp API client (existing `get_current_user()` is sufficient)
- Adding user profile caching beyond current session (future enhancement)
- Changing how assigned items are fetched or displayed

## Decisions

### Decision 1: Fetch User Profile After Workspace Load

**Chosen:** Call `fetch_current_user_profile()` in `AppMessage::WorkspacesLoaded` handler, after workspaces are successfully loaded.

**Rationale:**
- Workspaces load early in app initialization
- API client is guaranteed to exist at this point
- Fetch is async - doesn't block UI
- User ID available before user navigates anywhere

**Alternatives considered:**
- **Fetch in `TuiApp::new()`**: Too early - client might not exist yet
- **Fetch on demand when "Assigned to Me" clicked**: Adds latency to user action
- **Fetch in parallel with workspaces**: Unnecessary complexity, workspace load is sufficient trigger

### Decision 2: Silent Initialization with Graceful Degradation

**Chosen:** User profile fetch during init logs success/failure but doesn't show status messages. If fetch fails, fall back to task-based detection silently.

**Rationale:**
- User profile is infrastructure, not user-facing feature
- Error messages during startup are alarming and often transient
- Task-based detection provides reliable fallback
- User only needs to know if "Assigned to Me" itself fails

**Alternatives considered:**
- **Show "Loading user profile..." status**: Adds noise to startup
- **Show error if fetch fails**: Alarming for non-critical infrastructure
- **Retry on failure**: Adds complexity, marginal benefit

### Decision 3: On-Demand Fetch Fallback in `load_assigned_items()`

**Chosen:** If `current_user_id` is `None` when loading assigned items, call `fetch_current_user_and_load_tasks()` instead of showing error immediately.

**Rationale:**
- Provides second chance if init fetch failed
- User sees loading state instead of error
- Consistent with existing fallback pattern

**Alternatives considered:**
- **Show error immediately**: Current behavior - poor UX
- **Always require init fetch**: Breaks backwards compatibility
- **Multiple retries**: Over-engineering for edge case

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| **API rate limiting**: Extra `get_current_user()` call on every startup | Call is once per session, ClickUp rate limits are generous. Fallback exists if rate limited. |
| **Slower startup**: Additional API call during init | Fetch is async, doesn't block UI. Workspaces already loading in parallel. |
| **User profile fetch fails silently**: Debugging harder if issues occur | Logging at `info` and `debug` levels provides visibility. Status shown if "Assigned to Me" explicitly requested. |
| **Race condition**: User clicks "Assigned to Me" before fetch completes | Handled by on-demand fallback - triggers fetch if user_id still unavailable. |
| **Breaking change to initialization flow**: Unintended side effects | Existing task-based detection preserved as fallback. Session restore unchanged. |

## Migration Plan

**No migration required** - this is a client-side behavior change with no data migration or deployment dependencies.

**Rollback strategy:**
1. Revert the three code changes in `src/tui/app.rs`
2. User experience returns to current state (task-based detection only)
3. No data or state to clean up

## Open Questions

None - implementation approach is clear and well-scoped.
