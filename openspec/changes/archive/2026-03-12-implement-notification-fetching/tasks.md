## 1. App Message & State

- [x] 1.1 Add `NotificationsLoaded(Result<Vec<Notification>, String>)` variant to `AppMessage` enum in `src/tui/app.rs`
- [x] 1.2 Add `notifications_loading` boolean field to `TuiApp` state struct (already existed as `inbox_loading`)
- [x] 1.3 Add `notifications_error` optional string field to `TuiApp` state struct (optional, can use `status` instead)

## 2. TUI App - Load Methods

- [x] 2.1 Add `load_notifications(&mut self)` method to fetch from API and update state
  - Check if `current_workspace_id` is available
  - Call `client.get_notifications(workspace_id)`
  - Call `cache.cache_notifications(workspace_id, &notifications)`
  - Update `self.notifications` and `self.inbox_list`
  - Set `inbox_loading` to false after completion
- [x] 2.2 Add `pre_load_notifications(&mut self)` method to check cache and fetch if needed
  - Check if `current_workspace_id` is available
  - Try to load from cache: `cache.get_unread_notifications()`
  - If cache is empty or stale, call `pre_load_notifications_background()`
  - Update state with cached data if available
- [x] 2.3 Add `pre_load_notifications_background(&mut self, workspace_id: String)` async method
  - Spawn tokio task to fetch from API
  - Cache results via `cache.cache_notifications()`
  - Send `AppMessage::NotificationsLoaded` via channel

## 3. TUI App - Navigation Integration

- [x] 3.1 Update `navigate_into()` method - `Screen::Inbox` case
  - Call `load_notifications()` after setting screen state
  - Show loading indicator during fetch
- [x] 3.2 Update `update_inbox()` method - 'r' key handler
  - Change from cache-only read to API fetch via `load_notifications()`
  - Update status message to indicate refresh from API

## 4. TUI App - Async Message Handler

- [x] 4.1 Add `AppMessage::NotificationsLoaded` handler in `process_async_messages()`
  - On success: update `self.notifications`, update `self.inbox_list`, clear loading state
  - On error: set `self.status` with error message, clear loading state
  - Match the pattern used for `AssignedTasksLoaded`

## 5. Cache Integration

- [x] 5.1 Verify `cache_notifications()` is called after API fetch (in `load_notifications()` and background method)
- [x] 5.2 Add cache timestamp check for stale data (optional - can skip for MVP, always fetch)
  - Added `is_notifications_cache_valid()` method to CacheManager

## 6. Startup Pre-fetch (Optional)

- [x] 6.1 Add call to `pre_load_notifications()` after workspaces are loaded (in `AppMessage::WorkspacesLoaded` handler)
- [x] 6.2 Ensure notifications are pre-fetched in background without blocking UI

## 7. Testing & Verification

- [x] 7.1 Test inbox navigation from Workspaces screen
- [x] 7.2 Test inbox navigation from Spaces/Folders/Lists screens
- [x] 7.3 Test manual refresh ('r' key) fetches from API
- [x] 7.4 Test with empty notifications (API returns empty list)
- [x] 7.5 Test with multiple notifications
- [x] 7.6 Test error handling (API error, network error)
- [x] 7.7 Test cache is populated (verify SQLite database has notifications)
- [x] 7.8 Run `cargo test` to ensure no regressions
- [x] 7.9 Test with CLI debug mode to compare: `clickdown debug notifications <workspace_id>`

## 8. Documentation

- [x] 8.1 Update help dialog (?) to show inbox shortcuts if not already present (already present)
- [x] 8.2 Update status bar context help for inbox view (already present)
