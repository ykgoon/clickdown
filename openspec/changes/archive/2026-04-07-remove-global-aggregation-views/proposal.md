## Why

The "Assigned to Me" and "Inbox" sidebar items are global aggregation views that require expensive cross-workspace API calls. "Assigned to Me" fetches every workspace → space → folder → list, then tasks per list (capped at 100) and comments per task (capped at 50 tasks/list), with client-side filtering for comments. "Inbox" similarly aggregates activity across all workspaces. Both produce incomplete results due to API pagination limits and degrade in performance as workspaces grow. The solution is to replace these global views with scoped filtering within the list context where data is already loaded.

## What Changes

- **Remove "Assigned to Me" sidebar item** and its dedicated view, including all related state, message handlers, and pre-loading logic
- **Remove "Inbox" sidebar item** and its dedicated activity feed view, including all related state and message handlers
- **Add per-list "Assigned to Me" task filter** — when viewing a list, toggle a filter that shows only tasks assigned to the current user (uses existing efficient `assignees[]` API filter)
- **Add per-list "Assigned to Me" comment filter** — when viewing a list's tasks, toggle a filter that highlights or filters tasks with comments where the user is the assigned commenter
- **Remove assigned tasks/comments cache tables** (`assigned_tasks`, `assigned_comments`) and their pre-loading/fetching logic
- **Remove inbox activity cache tables** (`inbox_activity`, `inbox_metadata`) and related fetching logic
- **BREAKING**: Users who relied on the global "Assigned to Me" view will need to navigate to individual lists and apply the filter instead
- **BREAKING**: Users who relied on the global "Inbox" activity feed will no longer have a cross-workspace notification summary

## Capabilities

### New Capabilities
- `list-assigned-filter`: Per-list filtering for tasks and comments assigned to the current user. When viewing a list, users can toggle an "Assigned to Me" filter that narrows the task list to only items assigned to them, including tasks with assigned comments.
- `list-assigned-ui`: UI controls for toggling assigned filter within a list view. Visual indicator (keyboard shortcut, filter bar, or toggle) that shows filter state and filtered result count.

### Modified Capabilities
- `assigned-tasks-nav`: **REMOVED** — The global "Assigned to Me" navigation entry and its entire view hierarchy is replaced by `list-assigned-filter`.
- `assigned-comments-aggregation`: **REMOVED** — Cross-list comment aggregation is replaced by scoped per-list filtering in `list-assigned-filter`.
- `inbox-navigation`: **REMOVED** — The global "Inbox" sidebar item and navigation is eliminated.
- `inbox-api-integration`: **REMOVED** — The `get_inbox_activity` API method and its aggregation logic is no longer used.
- `inbox-list-ui`: **REMOVED** — The inbox rendering widget and its detail view are eliminated.
- `inbox-caching`: **REMOVED** — The `inbox_activity` and `inbox_metadata` cache tables are no longer populated or queried.
- `smart-inbox`: **REMOVED** — The smart inbox feature (activity aggregation across assignments, comments, status changes, due dates) is eliminated.
- `inbox-message-actions`: **REMOVED** — Inbox-specific message actions (dismiss, clear all) are eliminated.
- `notification-fetching`: **REMOVED** — Background notification/inbox activity polling is eliminated.

## Impact

**Removed modules/code:**
- `src/models/inbox_activity.rs` — InboxActivity model and aggregation utilities
- `src/tui/widgets/inbox_view.rs` — Inbox rendering widgets
- `src/tui/widgets/assigned_view.rs` — Assigned items unified view rendering
- `src/models/assigned_item.rs` — AssignedItem enum and AssignedComment model
- Assigned tasks/cache logic in `src/cache/mod.rs` (assigned_tasks, assigned_comments tables)
- Inbox cache logic in `src/cache/mod.rs` (inbox_activity, inbox_metadata tables)
- `get_inbox_activity`, `get_assigned_comments`, `get_all_accessible_lists`, `get_tasks_with_assignee` API methods (may still be useful for list filtering — see design)
- Message handlers: `AssignedTasksLoaded`, `AssignedTasksPreloaded`, `AssignedItemsLoaded`, `InboxActivityLoaded`
- Screen variants: `Screen::AssignedTasks`, `Screen::Inbox`
- Sidebar items: `SidebarItem::AssignedTasks`, `SidebarItem::Inbox`
- CLI debug commands: `debug assigned-tasks`, `debug assigned-comments`

**Modified:**
- `src/tui/app.rs` — Sidebar population, screen routing, message handling, state management
- `src/tui/widgets/sidebar.rs` — Sidebar item list
- `src/cache/schema.rs` — Database schema (assigned_tasks, assigned_comments, inbox_activity, inbox_metadata tables removed)
- `src/api/mock_client.rs` — Mock client assigned/inbox fixtures
- `src/tui/widgets/help.rs` — Help dialog (remove inbox/assigned shortcuts)
- Tests in `tests/` referencing removed features

**Preserved:**
- `get_tasks_with_assignee(list_id, user_id)` — Still needed for per-list filtering
- `get_comments_with_assigned_commenter(task_id, user_id)` — Still needed for per-list comment filtering
- `get_all_accessible_lists()` — May still be needed if list-level filtering uses it, or could be simplified since we only need the current list's data
