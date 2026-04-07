## 1. Add Proactive User Profile Fetch

- [x] 1.1 Add `fetch_current_user_profile()` method to `TuiApp` in `src/tui/app.rs`
- [x] 1.2 Call `fetch_current_user_profile()` in `AppMessage::WorkspacesLoaded` handler after workspaces are loaded
- [x] 1.3 Update `AppMessage::CurrentUserLoaded` handler to store user_id silently during initialization (no status message unless explicitly requested)

## 2. Improve Fallback in load_assigned_items()

- [x] 2.1 Modify `load_assigned_items()` to call `fetch_current_user_and_load_tasks()` when `current_user_id` is `None` instead of showing error immediately
- [x] 2.2 Update error message to distinguish between "fetching user..." (loading) and "failed to fetch user" (error state)

## 3. Update Status Messages and Logging

- [x] 3.1 Add logging for proactive user profile fetch success/failure at appropriate levels (info for success, debug for failure)
- [x] 3.2 Ensure status messages are shown only when user explicitly triggers "Assigned to Me" load, not during background initialization

## 4. Testing and Verification

- [x] 4.1 Manual test: Fresh install → Launch app → Click "Assigned to Me" immediately → Verify items load without error
- [x] 4.2 Manual test: Verify session restore with user_id still works (no redundant fetch)
- [x] 4.3 Manual test: Verify session restore without user_id triggers proactive fetch
- [x] 4.4 Run existing tests to verify no regression in `test_pre_load_assigned_tasks_*` tests
- [x] 4.5 Add integration test for first-launch scenario with mock client
