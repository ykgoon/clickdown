## Context

The ClickDown TUI application has a fully functional notification infrastructure that is never used:

**Existing Infrastructure:**
- `ClickUpClient::get_notifications(workspace_id)` - API client method (src/api/client.rs:309)
- `cache_notifications()` - Cache storage method (src/cache/mod.rs:248)
- `get_unread_notifications()` - Cache read method (src/cache/mod.rs:285)
- `notifications` table in SQLite schema with proper indexes
- `InboxListState` - UI state for displaying notifications
- `Screen::Inbox` - Navigation state for inbox view
- CLI debug mode successfully fetches notifications (src/commands/debug_ops.rs:590)

**Current State:**
- TUI navigates to `Screen::Inbox` when user presses Enter on Inbox sidebar item
- `navigate_into()` calls `cache.get_unread_notifications()` which returns empty Vec
- Cache is never populated because `cache_notifications()` is never called
- Manual refresh ('r' key) also only reads from empty cache
- Inbox always shows "📬 Inbox is empty - All caught up!"

**Root Cause:**
Unlike assigned tasks (which has `pre_load_assigned_tasks()`, `pre_load_assigned_tasks_background()`, and `AppMessage::AssignedTasksLoaded`), notifications have no fetching mechanism in the TUI flow.

## Goals / Non-Goals

**Goals:**
- Fetch notifications from ClickUp API when entering Inbox view
- Cache notifications locally for instant reloads
- Add background pre-fetch at startup (optional, like assigned tasks)
- Fix manual refresh to fetch from API, not just read cache
- Follow existing patterns from assigned tasks implementation
- Minimal code changes, reuse existing infrastructure

**Non-Goals:**
- No changes to notification data model
- No changes to inbox UI rendering
- No changes to cache schema
- No real-time notification sync (future enhancement)
- No notification actions beyond mark-as-read (future enhancement)

## Decisions

### Decision 1: Follow Assigned Tasks Pattern

**Approach:**
Mirror the assigned tasks implementation pattern:
1. Add `AppMessage::NotificationsLoaded` variant
2. Add `load_notifications()` - synchronous fetch + cache + state update
3. Add `pre_load_notifications()` - check cache, fetch if stale/empty
4. Add `pre_load_notifications_background()` - async fetch at startup
5. Integrate with `navigate_into()` for Inbox screen

**Rationale:**
- Proven pattern already working for assigned tasks
- Consistent code structure across the codebase
- Easy to understand and maintain
- Reuses existing error handling and state management

**Alternatives Considered:**
- Direct API call in `navigate_into()`: Would block UI, no caching
- Separate notification service: Over-engineering for this scope
- Polling for real-time updates: Future enhancement, not MVP

### Decision 2: Cache-First Strategy

**Approach:**
1. Check cache first when entering Inbox
2. If cache has data (< 5 minutes old), display immediately
3. Background refresh from API to get latest
4. If cache is empty/old, show loading indicator, fetch from API

**Rationale:**
- Instant UI response when cache is valid
- Reduces API calls during navigation
- Consistent with assigned tasks behavior
- Better user experience than waiting for API every time

**Alternatives Considered:**
- Always fetch from API: Slower, more API calls
- Cache-only with manual refresh: Stale data
- Real-time WebSocket: Not supported by ClickUp API

### Decision 3: Workspace-Based Notification Loading

**Approach:**
Load notifications for the currently selected workspace only:
- Use `self.current_workspace_id` to determine which notifications to fetch
- Show message if no workspace selected
- Notifications are workspace-scoped in ClickUp API

**Rationale:**
- Consistent with how tasks are loaded per-list
- ClickUp API returns notifications per workspace
- Prevents overwhelming users with all notifications
- Matches user mental model (navigate to workspace, see its notifications)

**Alternatives Considered:**
- Aggregate notifications across all workspaces: More complex, potentially overwhelming
- Use a "global inbox": Requires additional API calls, not standard ClickUp pattern

### Decision 4: Manual Refresh Fetches from API

**Approach:**
Update the 'r' key handler in `update_inbox()` to:
1. Call `load_notifications()` which fetches from API
2. Not just re-read from cache

**Rationale:**
- Manual refresh should actually refresh data
- Consistent with user expectations
- Matches behavior of other refresh operations in the app

**Current Bug:**
The 'r' key currently only calls `cache.get_unread_notifications()` - it re-reads the same (empty) cache.

## Risks / Trade-offs

**[API Latency]** Fetching notifications could be slow on large workspaces
- **Mitigation:** Background loading, cache-first strategy, loading indicator

**[Stale Data]** Cached notifications may be outdated
- **Mitigation:** 5-minute TTL, manual refresh ('r' key), background refresh

**[Workspace Context]** User might expect cross-workspace notifications
- **Mitigation:** Clear UI indication of which workspace's notifications are shown

**[Code Duplication]** Similar pattern to assigned tasks, some duplication
- **Mitigation:** Future refactoring could extract common loading logic

**[Memory]** Holding all notifications in memory
- **Mitigation:** ClickUp API typically returns manageable notification counts

## Implementation Plan

1. Add `AppMessage::NotificationsLoaded` variant to `AppMessage` enum
2. Add `load_notifications()` method to `TuiApp`
3. Add `pre_load_notifications()` method to `TuiApp`
4. Add `pre_load_notifications_background()` async method
5. Update `navigate_into()` Screen::Inbox case to call `load_notifications()`
6. Update `update_inbox()` 'r' key handler to fetch from API
7. Add message handler for `AppMessage::NotificationsLoaded` in `process_async_messages()`
8. Optional: Call `pre_load_notifications()` at startup after workspaces load

## Open Questions

- Should notifications auto-refresh periodically? (future enhancement)
- Should there be a "mark all as read" confirmation dialog? (currently no confirmation)
- Should notification count badge show on Inbox sidebar item? (future enhancement)
