## Why

On first launch with no session state, the "Assigned to Me" view fails with an error when users navigate to it immediately, because `current_user_id` is `None`. The current design requires users to first navigate into a task list and open a task to auto-detect their user ID. This creates a poor first-time user experience where a core feature appears broken until the user performs seemingly unrelated actions.

## What Changes

- **Proactive user profile fetch**: Fetch the current user's profile during app initialization (after workspaces load) instead of waiting for task-based detection
- **Silent initialization**: User profile fetch happens in background during startup - no status messages shown unless it fails
- **Improved fallback**: If user_id is still unavailable when loading "Assigned to Me", trigger on-demand user fetch instead of showing error immediately
- **Better error handling**: Distinguish between "fetching user..." (loading state) vs "failed to fetch user" (error state)

## Capabilities

### New Capabilities

- `user-profile-fetch`: Proactive fetching of current user profile at app initialization to enable assignee filtering across all features

### Modified Capabilities

- `assigned-items-view`: Requirements updated to handle proactive user_id availability and on-demand user fetch fallback

## Impact

- **Affected code**: `src/tui/app.rs` - initialization flow, user detection logic, AssignedItemsLoaded handler
- **API calls**: Adds one `get_current_user()` call during startup (async, non-blocking)
- **Dependencies**: None - uses existing `get_current_user()` API method
- **Behavior**: First-time users can immediately access "Assigned to Me" without navigating to tasks first
- **Backwards compatible**: Existing session restore and task-based detection remain as fallbacks
