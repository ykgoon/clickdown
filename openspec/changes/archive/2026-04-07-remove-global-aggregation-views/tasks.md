## 1. Add per-list assigned filter (new feature)

- [x] 1.1 Add `assigned_filter_active: bool` field to `TuiApp` struct, initialized to `false` in all constructors (`new()`, `with_client()`, etc.)
- [x] 1.2 Add `load_tasks_with_assigned_filter(&mut self, list_id: String)` method that calls `get_tasks_with_assignee(list_id, user_id, Some(100))` instead of `get_tasks(list_id, TaskFilters::default())`, sending `TasksLoaded` message
- [x] 1.3 Modify `load_tasks()` to check `assigned_filter_active` — if true, call `load_tasks_with_assigned_filter` instead of the standard fetch
- [x] 1.4 Add keyboard handler for filter toggle (e.g., 'a' key) in `Screen::Tasks` input handling that toggles `assigned_filter_active` and re-fetches tasks
- [x] 1.5 Update screen title generation for `Screen::Tasks` to include "(Assigned to Me)" suffix when filter is active
- [x] 1.6 Update help dialog (`src/tui/widgets/help.rs`) to show the assigned filter shortcut when in task list view

## 2. Remove Assigned to Me screen and state

- [x] 2.1 Remove `Screen::AssignedTasks` variant from the `Screen` enum and all match arms referencing it
- [x] 2.2 Remove `SidebarItem::AssignedTasks` variant from `SidebarItem` enum in `src/tui/widgets/sidebar.rs`
- [x] 2.3 Remove all sidebar population code that adds `SidebarItem::AssignedTasks` to the sidebar item list (search for `vec![SidebarItem::AssignedTasks, SidebarItem::Inbox]` patterns)
- [x] 2.4 Remove assigned tasks state fields from `TuiApp`: `assigned_tasks`, `assigned_tasks_count`, `assigned_tasks_loading`, `assigned_tasks_error`
- [x] 2.5 Remove assigned items state fields from `TuiApp`: `assigned_items`, `assigned_items_count`, `assigned_items_loading`, `assigned_items_error`, `assigned_items_filter`, `assigned_items_selected_index`
- [x] 2.6 Remove message variants: `AssignedTasksLoaded`, `AssignedTasksPreloaded`, `AssignedItemsLoaded` from `AppMessage` enum and their handlers
- [x] 2.7 Remove `load_assigned_tasks()`, `load_assigned_items()`, `pre_load_assigned_tasks()`, `pre_load_assigned_tasks_background()` methods from `TuiApp`
- [x] 2.8 Remove `update_assigned_tasks()` input handler method from `TuiApp`
- [x] 2.9 Remove `handle_assigned_view_input` function and `render_assigned_view` function from `src/tui/widgets/assigned_view.rs`
- [x] 2.10 Delete `src/tui/widgets/assigned_view.rs` and remove from `src/tui/widgets/mod.rs`
- [x] 2.11 Delete `src/models/assigned_item.rs` and remove from `src/models/mod.rs` (remove both `mod` declaration and `pub use` exports)
- [x] 2.12 Remove `pre_load_assigned_tasks()` call from application initialization flow

## 3. Remove Inbox screen and state

- [x] 3.1 Remove `Screen::Inbox` variant from the `Screen` enum and all match arms referencing it
- [x] 3.2 Remove `SidebarItem::Inbox` variant from `SidebarItem` enum
- [x] 3.3 Remove all sidebar population code that adds `SidebarItem::Inbox` to the sidebar item list
- [x] 3.4 Remove inbox state fields from `TuiApp`: `inbox_showing_detail`, `inbox_loading`, `inbox_list`, `inbox_activity`, `inbox_activity_loading`, `inbox_activity_error`
- [x] 3.5 Remove `InboxActivityLoaded` and `NotificationsLoaded` message variants from `AppMessage` enum and their handlers
- [x] 3.6 Remove `load_inbox_activity()` method from `TuiApp`
- [x] 3.7 Remove `update_inbox()` input handler method from `TuiApp`
- [x] 3.8 Remove `InboxListState` import and usage from `src/tui/app.rs`
- [x] 3.9 Delete `src/tui/widgets/inbox_view.rs` and remove from `src/tui/widgets/mod.rs`
- [x] 3.10 Delete `src/models/inbox_activity.rs` and remove from `src/models/mod.rs`
- [x] 3.11 Update help dialog to remove inbox-related keyboard shortcuts
- [x] 3.12 Remove `Screen::Inbox` rendering branch and `render_inbox_list`, `render_notification_detail` imports

## 4. Remove cache tables and methods

- [x] 4.1 Update `src/cache/schema.rs` — remove `CREATE TABLE` statements for `assigned_tasks`, `assigned_comments`, `inbox_activity`, and `inbox_metadata` tables, plus their indexes
- [x] 4.2 Add `DROP TABLE IF EXISTS` migration statements in schema init to clean up existing tables for users upgrading
- [x] 4.3 Remove `cache_assigned_tasks()`, `get_assigned_tasks()`, `is_assigned_tasks_cache_valid()`, `clear_assigned_tasks()` methods from `src/cache/mod.rs`
- [x] 4.4 Remove `cache_assigned_comments()`, `get_assigned_comments()`, `is_assigned_comments_cache_valid()`, `clear_assigned_comments()` methods from `src/cache/mod.rs`
- [x] 4.5 Remove `cache_inbox_activity()`, `get_cached_inbox_activity()`, `store_last_inbox_check()`, `get_last_inbox_check()`, `cleanup_old_inbox_activity()`, `is_inbox_activity_cache_valid()` methods from `src/cache/mod.rs`
- [x] 4.6 Remove `cache_notifications()`, `get_notifications()`, `mark_notification_read()`, `mark_all_notifications_read()`, `cleanup_old_notifications()` methods from `src/cache/mod.rs` (if present — check for notification-related methods)

## 5. Remove API methods

- [x] 5.1 Remove `get_all_accessible_lists()` from `ClickUpApi` trait (`client_trait.rs`) and `ClickUpClient` implementation (`client.rs`)
- [x] 5.2 Remove `get_assigned_comments()` from `ClickUpApi` trait and `ClickUpClient` implementation
- [x] 5.3 Remove `get_inbox_activity()` from `ClickUpApi` trait and `ClickUpClient` implementation
- [x] 5.4 Update `MockClickUpClient` — remove `all_lists_response`, `assigned_comments_response`, `inbox_activity_response` fields and their builder methods (`with_assigned_comments`, `with_inbox_activities`, etc.)
- [x] 5.5 Remove trait macro delegations for the removed methods in `client.rs` (the `impl_clickup_api!` macro block)

## 6. Remove CLI debug commands

- [x] 6.1 Remove `DebugOperation::AssignedTasks` and `DebugOperation::AssignedComments` from `src/cli/args.rs`
- [x] 6.2 Remove CLI argument parsing for `assigned-tasks` and `assigned-comments` commands in `src/cli/args.rs`
- [x] 6.3 Remove `get_assigned_tasks()`, `get_assigned_tasks_json()`, `get_assigned_comments()`, `get_assigned_comments_json()` methods from `src/commands/debug_ops.rs`
- [x] 6.4 Remove match arms for `DebugOperation::AssignedTasks` and `DebugOperation::AssignedComments` in `src/cli/run.rs`
- [x] 6.5 Update help text in `src/cli/args.rs` to remove references to `assigned-tasks` and `assigned-comments` commands

## 7. Update tests and fix compilation

- [x] 7.1 Run `cargo build` to identify all compilation errors from the removals
- [x] 7.2 Fix all import errors, unused import warnings, and missing match arm errors
- [ ] 7.3 Remove or update tests in `tests/app_test.rs` that reference `Screen::AssignedTasks`, `Screen::Inbox`, `SidebarItem::AssignedTasks`, `SidebarItem::Inbox`, or assigned task/inbox loading
- [ ] 7.4 Remove or update tests in `tests/fixtures.rs` that create `InboxActivity`, `AssignedItem`, or `AssignedComment` test data
- [x] 7.5 Remove unit tests in `src/models/assigned_item.rs` (file deleted) and `src/models/inbox_activity.rs` (file deleted) — already handled by file deletion
- [x] 7.6 Remove unit tests in `src/tui/widgets/assigned_view.rs` and `src/tui/widgets/inbox_view.rs` — already handled by file deletion
- [ ] 7.7 Run `cargo test` to verify all remaining tests pass
- [ ] 7.8 Run `cargo clippy` to check for remaining warnings or issues
