## Context

ClickDown currently has two global aggregation views in the sidebar: "Assigned to Me" and "Inbox". Both require expensive cross-workspace API calls that produce incomplete results due to pagination limits.

**Current state:**
- `TuiApp` holds ~30 fields of state for assigned tasks/items and inbox (~15 fields each)
- `Screen::AssignedTasks` and `Screen::Inbox` are top-level screen variants
- `SidebarItem::AssignedTasks` and `SidebarItem::Inbox` are always-present sidebar entries
- `get_all_accessible_lists()` traverses all workspaces → spaces → folders → lists
- `get_tasks_with_assignee()` fetches up to 100 tasks per list across all lists
- `get_assigned_comments()` fetches all tasks (capped at 50/list) then all comments per task
- `get_inbox_activity()` polls multiple endpoints per workspace for activity aggregation
- Separate cache tables: `assigned_tasks`, `assigned_comments`, `inbox_activity`, `inbox_metadata`
- Background pre-loading of assigned tasks at startup
- Multiple message handlers: `AssignedTasksLoaded`, `AssignedTasksPreloaded`, `AssignedItemsLoaded`, `InboxActivityLoaded`

**Constraints:**
- The ClickUp API has no single endpoint for "all things assigned to me"
- `get_tasks_with_assignee(list_id, user_id)` uses the efficient `assignees[]` API filter — this is preserved
- `get_comments_with_assigned_commenter(task_id, user_id)` filters client-side — this is preserved for per-list use
- The existing list view (`Screen::Tasks`, `load_tasks()`) already loads tasks for a specific list

## Goals / Non-Goals

**Goals:**
- Eliminate all cross-workspace aggregation API calls (assigned tasks, assigned comments, inbox activity)
- Remove the "Assigned to Me" and "Inbox" sidebar items and their dedicated screens
- Add a per-list "Assigned to Me" filter that reuses the existing efficient `assignees[]` API parameter
- Reduce application state complexity by removing ~30 state fields
- Remove assigned tasks/comments cache tables and inbox cache tables

**Non-Goals:**
- No new API endpoints are introduced
- No changes to the ClickUp API client's core task/comment fetching methods (they're reused for per-list filtering)
- No changes to task detail, comment threading, or document viewing features
- No changes to workspace/space/folder/list navigation

## Decisions

### Decision 1: Filter toggle vs. dedicated screen

**Chosen: Toggle within existing list view.**

When the user is viewing a list's tasks (`Screen::Tasks`), a keyboard shortcut toggles the "Assigned to Me" filter. When active, the task list is re-fetched using `get_tasks_with_assignee(list_id, user_id)` instead of `get_tasks(list_id, TaskFilters::default())`.

**Alternatives considered:**
- **Dedicated "Assigned" screen per list**: Would require new screen variant and complex navigation. Overkill for what is essentially a filter parameter change.
- **Filter bar UI element**: Would require new TUI widget. Keyboard shortcut is simpler and consistent with existing patterns ('f' for filter cycling in assigned view already exists).

**Rationale:** The list view already has the data loading pipeline. The only change is which API method is called and whether the `assignees[]` parameter is included.

### Decision 2: Filter state storage

**Chosen: Per-session boolean on `TuiApp`.**

A single `assigned_filter_active: bool` field replaces the current `assigned_items_filter: AssignedItemsFilter` (All/Tasks Only/Comments Only). The filter is either on or off — no multi-state cycling needed since tasks and comments are no longer mixed in a separate view.

**Alternatives considered:**
- **Per-list filter persistence**: Store filter state per list ID in a HashMap. Adds complexity for minimal user benefit — users are unlikely to want different filter states for different lists.
- **Saved in session state**: Persist across restarts. Over-engineered for a simple toggle.

**Rationale:** A single boolean is the simplest state representation. If the user navigates away and back, the filter resets to off — consistent with how other transient filters work in the app.

### Decision 3: Assigned comment indicator approach

**Chosen: Fetch comments for visible tasks only, show inline indicator.**

When the assigned filter is active, after fetching assigned tasks, also check each visible task for comments where the user is the assigned commenter. A 💬 indicator is appended to the task line if such comments exist.

**Alternatives considered:**
- **Pre-fetch all comments for all tasks in the list**: Would be expensive for lists with many tasks.
- **Show comment count**: Would require fetching all comments first, then filtering. Same cost, less useful.
- **No comment indicator, filter on tasks only**: Simpler but loses the "assigned comments" capability entirely.

**Rationale:** The inline indicator is a lightweight addition to the existing task list rendering. It only fires when the assigned filter is active, so normal list viewing is unaffected.

### Decision 4: API methods to preserve vs. remove

**Preserved:**
- `get_tasks_with_assignee(list_id, user_id, limit)` — Used directly for per-list filtering
- `get_comments_with_assigned_commenter(task_id, user_id)` — Used for per-task comment indicator

**Removed:**
- `get_all_accessible_lists()` — Was only used for cross-workspace aggregation
- `get_assigned_comments(user_id)` — The expensive cross-list comment aggregation method
- `get_inbox_activity(team_id, user_id, since)` — The inbox activity aggregation method

**Rationale:** `get_tasks_with_assignee` and `get_comments_with_assigned_commenter` are efficient single-context methods that work perfectly for per-list filtering. `get_all_accessible_lists` and `get_assigned_comments` are the expensive aggregation methods that are the root cause of the performance problem.

### Decision 5: Cache tables to remove

**Removed tables:**
- `assigned_tasks` — No longer needed; tasks are fetched per-list with standard caching
- `assigned_comments` — No longer needed; comments are accessed through task threads
- `inbox_activity` — Inbox feature removed entirely
- `inbox_metadata` — Inbox feature removed entirely

**Preserved tables:**
- All existing task/cache tables for standard list viewing
- Comment cache tables for task comment threads

**Migration:** On next app startup, the schema migration drops the removed tables. No data migration is needed — cached assigned tasks and inbox activity were transient, derivable data.

### Decision 6: Message enum simplification

**Removed variants:**
- `AssignedTasksLoaded` — Replaced by reusing `TasksLoaded` with the assigned filter parameter
- `AssignedTasksPreloaded` — Pre-loading is eliminated
- `AssignedItemsLoaded` — Unified assigned items view is removed
- `InboxActivityLoaded` — Inbox feature is removed
- `NotificationsLoaded` — Notification loading is removed

**Rationale:** When the assigned filter is active, the same `load_tasks()` method is called but with `get_tasks_with_assignee` instead of `get_tasks`. The result still comes through `TasksLoaded`. This eliminates the need for dedicated assigned message variants entirely.

### Decision 7: Sidebar population

**Current:** Sidebar always includes `[AssignedTasks, Inbox, ...workspace hierarchy...]`.

**New:** Sidebar only contains the workspace hierarchy. No global aggregation items.

```
Before:  [✓ Assigned to Me (5)] [📬 Inbox (3)] [Workspace A] [Workspace B] ...
After:   [Workspace A] [Workspace B] [Workspace C] ...
```

When a list is selected and the assigned filter is toggled on, the filter is indicated in the main content area's header/status bar, not in the sidebar.

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| **Users lose quick overview of all assigned work** — No longer possible to see everything assigned across all lists in one view. | The trade-off is intentional. The global view was incomplete (100-task cap per list, 50-task cap for comments). Per-list filtering is accurate within its scope. Users can navigate to each list of interest. |
| **Comment indicator requires N API calls** — One `get_comments` call per visible task to check for assigned comments. | Only triggered when the assigned filter is active. For a typical list view with 10-20 tasks, this is 10-20 calls — far fewer than the current cross-workspace approach. Can be optimized later with batch fetching. |
| **Schema migration drops tables** — If users had cached data in `assigned_tasks`/`inbox_activity`, it's lost. | Cached data was transient (5-minute TTL). No user-visible data is lost — it would have expired anyway. |
| **`get_all_accessible_lists()` removal** — If any other code path depends on this method, it breaks. | Verified via grep: only used by assigned tasks, assigned comments, and inbox activity loading. All three are being removed. |
| **No replacement for inbox notifications** — Users who used inbox for tracking due dates, status changes lose that feature. | The inbox was a simulated activity feed, not real notifications. The due date / status change data was incomplete. This is an acceptable loss given the feature's poor reliability. |

## Migration Plan

**Deployment (single PR):**
1. Add `assigned_filter_active: bool` to `TuiApp` and the keyboard handler in `Screen::Tasks`
2. Modify `load_tasks()` to use `get_tasks_with_assignee` when filter is active
3. Remove `Screen::AssignedTasks`, `Screen::Inbox` variants
4. Remove `SidebarItem::AssignedTasks`, `SidebarItem::Inbox` variants
5. Remove assigned tasks/items state fields (~15 fields) and inbox state fields (~6 fields)
6. Remove message variants: `AssignedTasksLoaded`, `AssignedTasksPreloaded`, `AssignedItemsLoaded`, `InboxActivityLoaded`, `NotificationsLoaded`
7. Remove cache methods for `assigned_tasks`, `assigned_comments`, `inbox_activity`, `inbox_metadata`
8. Update cache schema to drop removed tables
9. Remove `get_all_accessible_lists`, `get_assigned_comments`, `get_inbox_activity` from API trait and implementations
10. Remove deleted module files: `inbox_activity.rs`, `inbox_view.rs`, `assigned_view.rs`, `assigned_item.rs`
11. Update mock client, tests, help dialog, CLI debug commands
12. Update sidebar population to exclude AssignedTasks/Inbox items

**Rollback:** Revert the PR. All removed code is deletion-only — no data migration to reverse.

## Open Questions

- **Comment indicator performance**: For lists with many tasks, should the comment check be lazy (only check tasks visible on screen) or batched? Current design checks all tasks in the list.
- **Filter discoverability**: What keyboard shortcut for the assigned filter? The existing 'f' key cycles through All/Tasks/Comments in the current assigned view. Reusing 'f' in the list view could work, or a new shortcut like 'a' for "assigned."
- **Visual design**: Should the assigned filter change the task list title (e.g., "Tasks: Sprint (Assigned to Me)") or use a separate status bar indicator?
