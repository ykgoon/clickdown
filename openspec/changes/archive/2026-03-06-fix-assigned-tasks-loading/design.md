## Context

The "Assigned to Me" feature was implemented with the infrastructure for loading tasks assigned to the current user, but critical UI feedback mechanisms were never connected:

1. **Loading state exists but isn't rendered**: The `assigned_tasks_loading` boolean flag is set correctly but never checked during rendering
2. **Error state exists but isn't displayed**: The `assigned_tasks_error` field captures errors but they're never shown to users
3. **User ID detection is fragile**: The `try_detect_user_id()` function only checks `task_list.tasks()`, which is empty if the user hasn't navigated to any list yet
4. **Status bar uses wrong flag**: The status bar checks `self.loading` (generic) instead of `self.assigned_tasks_loading` (specific)

The result: users see an empty list with "Loading assigned tasks..." in the status bar momentarily, but it never completes and never shows an error.

## Goals / Non-Goals

**Goals:**
- Display a visible loading indicator in the main content area while fetching assigned tasks
- Show error messages when loading fails (auth error, user detection failure, API error)
- Improve user ID detection to work even when no local tasks exist
- Connect the loading state to the status bar rendering logic
- Maintain backward compatibility with existing navigation flow

**Non-Goals:**
- Optimize the `get_all_accessible_lists()` API call performance (separate concern)
- Add pagination for assigned tasks (future enhancement)
- Change the underlying data model or caching strategy
- Modify how tasks are filtered or sorted

## Decisions

### Decision 1: Add Loading Indicator to `render_task_list`

**Approach**: Modify `render_task_list()` to accept an optional `loading` parameter. When loading is true, display a centered "Loading..." message with a spinner or ellipsis animation instead of an empty list.

**Rationale**: 
- Minimal change to existing rendering logic
- Consistent with how other loading states work in the app (e.g., inbox loading)
- Keeps loading state visualization close to the data it represents

**Alternatives Considered**:
- Create a separate `render_assigned_tasks()` function → Rejected: would duplicate rendering logic
- Use a generic overlay/placeholder → Rejected: less clear about what's loading

### Decision 2: Display Errors in Status Bar with Higher Priority

**Approach**: Update the status bar rendering logic in `render()` to check `assigned_tasks_error` when on the AssignedTasks screen, giving it priority over the generic `self.status`.

**Rationale**:
- Status bar is already used for error display elsewhere in the app
- Consistent with existing error handling patterns
- Non-intrusive to the main content rendering

**Alternatives Considered**:
- Show error in main content area → Rejected: status bar is the established pattern for transient errors
- Use a dialog/popup → Rejected: too disruptive for recoverable errors

### Decision 3: Fallback User ID Detection via API

**Approach**: When `try_detect_user_id()` fails to find a local task, make a lightweight API call to fetch the current user's identity (e.g., from `/user` endpoint or from the first workspace's member list).

**Rationale**:
- Ensures user ID is available even on first use
- More reliable than depending on navigation history
- One-time cost vs. repeated failures

**Alternatives Considered**:
- Prompt user to enter their user ID → Rejected: poor UX, adds friction
- Store user ID in config after first detection → Rejected: adds complexity, user ID rarely changes
- Skip user ID requirement and fetch all tasks then filter → Rejected: inefficient, defeats the purpose

### Decision 4: Connect `assigned_tasks_loading` to Status Bar

**Approach**: Update the status bar priority logic to check `assigned_tasks_loading` when on the AssignedTasks screen, similar to how `self.loading` is checked generically.

**Rationale**:
- Provides immediate feedback that something is happening
- Uses existing status bar infrastructure
- Minimal code change

**Alternatives Considered**:
- Use a separate loading indicator widget → Rejected: redundant with status bar
- Show loading only in main content → Rejected: status bar is more visible

## Risks / Trade-offs

### Risk 1: API Call for User ID May Fail

**Risk**: The fallback API call to get user identity could fail (network error, API change, permission issue).

**Mitigation**: 
- Handle the error gracefully and show a clear error message
- Cache the user ID after successful detection to avoid repeated calls
- Log the error for debugging

### Risk 2: Loading Indicator May Flicker

**Risk**: If the API call is very fast, the loading indicator may appear and disappear quickly, causing visual flicker.

**Mitigation**: 
- Consider adding a minimum display duration (e.g., 100ms) before showing the loading indicator
- Or accept the flicker as acceptable for a rare edge case (first-time users only)

### Risk 3: Status Bar Logic Becomes Complex

**Risk**: Adding more conditions to the status bar priority logic could make it harder to maintain.

**Mitigation**: 
- Extract status bar logic into a dedicated method
- Add comments explaining the priority order
- Keep the logic simple and focused

### Trade-off: Simplicity vs. Completeness

**Trade-off**: This design focuses on fixing the immediate bug (loading never completes visually) rather than optimizing the underlying API calls.

**Rationale**: The API performance issue is separate and can be addressed in a future change. The priority is to provide user feedback so they understand what's happening.

## Migration Plan

No migration required. This is a UI fix that:
- Does not change data structures
- Does not modify the database schema
- Does not change API contracts
- Is backward compatible with existing cached data

**Rollback Strategy**: Simply revert the code changes. No data migration needed.

## Open Questions

None. The implementation approach is clear and all technical decisions have been made.
