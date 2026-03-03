## Why

The session restore feature implemented in `restore-last-session` has a critical bug: after restoring a saved session on app restart, the navigation hierarchy is not properly loaded. The app sets the restored screen type (e.g., Tasks) but doesn't replay the navigation chain to load data at each level, leaving users stuck at the root level with an empty view and unable to navigate.

## What Changes

- **Fix `restore_session_state()`** to trigger the navigation chain after restoring IDs, rather than just setting the screen type
- **Add progressive loading logic** that waits for each async data load to complete before proceeding to the next level
- **Restore sidebar selection** at each level using the saved IDs instead of always calling `select_first()`
- **Update status messages** to indicate when session restore is in progress vs complete
- **Add tests** covering the navigation restore workflow

## Capabilities

### New Capabilities

(none - this is a bug fix, not a new feature)

### Modified Capabilities

- `session-management`: The session restore behavior is changing from "set screen type and IDs" to "replay full navigation chain with selection restore at each level"

## Impact

- Modified files: `src/tui/app.rs` (main fix in `restore_session_state()` and async message handlers)
- Potential changes to `src/models/session.rs` if additional state tracking is needed
- Test updates in `tests/app_test.rs` to cover navigation restore scenarios
- No breaking changes to user-facing behavior - this fixes broken functionality
- No API or database schema changes required
