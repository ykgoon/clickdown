## 1. Data Model

- [x] 1.1 Create `InboxActivity` struct in `src/models/inbox_activity.rs`
- [x] 1.2 Create `ActivityType` enum with variants: Assignment, Comment, StatusChange, DueDate
- [x] 1.3 Add `InboxActivity` to module exports in `src/models/mod.rs`
- [x] 1.4 Add serde Serialize/Deserialize derives for `InboxActivity`

## 2. API Layer

- [x] 2.1 Add `get_tasks_assigned_to_user()` method to `ClickUpApi` trait in `src/api/client_trait.rs`
- [x] 2.2 Implement `get_tasks_assigned_to_user()` in `ClickUpClient` in `src/api/client.rs`
- [x] 2.3 Add `get_comments_for_tasks()` method to `ClickUpApi` trait
- [x] 2.4 Implement `get_comments_for_tasks()` in `ClickUpClient`
- [x] 2.5 Add `get_tasks_with_due_dates()` method to `ClickUpApi` trait
- [x] 2.6 Implement `get_tasks_with_due_dates()` in `ClickUpClient`
- [x] 2.7 Add endpoint builders in `src/api/endpoints.rs` for new query patterns
- [x] 2.8 Add mock implementations in `MockClickUpClient` for all new methods
- [x] 2.9 Add `get_inbox_activity()` convenience method that orchestrates all fetches

## 3. Cache Layer

- [x] 3.1 Create `inbox_activity` table in SQLite schema
- [x] 3.2 Add `cache_inbox_activity()` method to `CacheManager`
- [x] 3.3 Add `get_cached_inbox_activity()` method to `CacheManager`
- [x] 3.4 Add `store_last_inbox_check()` method to `CacheManager`
- [x] 3.5 Add `get_last_inbox_check()` method to `CacheManager`
- [x] 3.6 Add `cleanup_old_inbox_activity()` method to remove activities older than 30 days
- [x] 3.7 Add database migration for new table and key-value store

## 4. TUI Application Logic

- [x] 4.1 Add `AppMessage::InboxActivityLoaded` variant to message enum
- [x] 4.2 Add `load_inbox_activity()` method to `TuiApp`
- [x] 4.3 Add `pre_load_inbox_activity()` method for cache-first loading
- [x] 4.4 Add `pre_load_inbox_activity_background()` for async background fetch
- [x] 4.5 Add message handler for `AppMessage::InboxActivityLoaded` in `process_async_messages()`
- [x] 4.6 Update `navigate_into()` to call `load_inbox_activity()` when entering Inbox
- [x] 4.7 Update `update_inbox()` 'r' key handler to fetch from API
- [x] 4.8 Add `inbox_activity` field to `TuiApp` state
- [x] 4.9 Add `inbox_activity_loading` field to track loading state
- [x] 4.10 Add `inbox_activity_error` field for error display

## 5. UI Updates

- [x] 5.1 Update `InboxListState` to work with `InboxActivity` instead of `Notification`
- [x] 5.2 Add activity type icon rendering in `render_inbox_list()`
- [x] 5.3 Update list item format to show activity type, title, and description
- [x] 5.4 Add activity detail view showing full activity information
- [x] 5.5 Update empty state message to reflect activity feed
- [x] 5.6 Update loading indicator to show "Loading activity..."
- [x] 5.7 Update error display to show partial success messages
- [x] 5.8 Update help dialog with inbox keyboard shortcuts

## 6. Testing

- [x] 6.1 Add unit tests for `InboxActivity` model
- [x] 6.2 Add unit tests for activity deduplication logic
- [x] 6.3 Add integration tests for smart inbox with mock client
- [x] 6.4 Add snapshot tests for inbox activity display
- [x] 6.5 Add snapshot tests for different activity types
- [x] 6.6 Add tests for incremental polling behavior
- [x] 6.7 Add tests for cache-first loading strategy
- [x] 6.8 Run existing inbox tests and update for new behavior

## 7. Documentation

- [x] 7.1 Update README.md with smart inbox description
- [x] 7.2 Update CLI debug help with inbox activity command
- [x] 7.3 Add inline documentation for new API methods
- [x] 7.4 Document activity types and their sources
