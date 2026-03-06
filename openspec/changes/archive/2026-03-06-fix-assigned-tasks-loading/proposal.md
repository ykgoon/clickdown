## Why

The "Assigned to Me" section displays "Loading assigned tasks..." indefinitely but never completes or shows tasks. Users see an empty list with no feedback about what's happening, making the feature unusable. This breaks a core workflow for users who want to quickly access their assigned tasks.

## What Changes

- **Loading state is now visible**: Users see a loading indicator in the main content area while tasks are being fetched
- **Error messages are displayed**: When authentication fails or user ID detection fails, users see a clear error message instead of an empty list
- **User ID detection improved**: Falls back to fetching user identity from API when no local tasks exist, instead of silently failing
- **Status bar reflects assigned tasks loading**: The `assigned_tasks_loading` flag is now checked when displaying status messages
- **Better feedback**: Status bar shows progress messages like "Loading assigned tasks..." and "Loaded X assigned task(s)"

## Capabilities

### New Capabilities

- `assigned-tasks-error-display`: Error state rendering for the assigned tasks view, including authentication errors and user detection failures
- `assigned-tasks-loading-indicator`: Visual loading state in the main content area when fetching assigned tasks from API

### Modified Capabilities

- `tui-navigation`: Enhanced to include proper loading and error state handling for the AssignedTasks screen

## Impact

- **Files modified**:
  - `src/tui/app.rs`: Loading state rendering, error display, improved user ID detection
  - `src/tui/widgets/task_list.rs`: Added loading indicator support
- **API calls**: `get_all_accessible_lists()` may be slow for large workspaces; users now see loading feedback
- **User experience**: Users can now understand when loading is in progress vs. when an error occurred
- **No breaking changes**: Existing functionality preserved, only adds missing feedback mechanisms
