## Context

The current Inbox feature attempts to call a non-existent ClickUp API endpoint (`/team/{workspace_id}/notifications`), resulting in 404 errors. ClickUp API v2 has no notifications endpoint - notifications are only available via webhooks (push model) or in the ClickUp web/mobile apps.

This design implements a **polling-based smart inbox** that simulates notifications by aggregating recent activity from existing ClickUp API endpoints. The approach fits the TUI app paradigm (pull, not push), requires no additional infrastructure, and provides users with a functional activity feed.

**Current State:**
- Inbox navigation exists and is accessible
- `Screen::Inbox` state exists in TUI
- `InboxListState` widget renders notifications
- Notification model and cache tables exist
- All inbox infrastructure is in place but non-functional due to missing API endpoint

**Constraints:**
- Must work with existing ClickUp API v2 endpoints only
- Must fit TUI app model (no always-on server for webhooks)
- Should reuse existing inbox UI/UX patterns where possible
- Must maintain "Inbox" name in navigation (user-facing terminology)

## Goals / Non-Goals

**Goals:**
- Implement functional inbox using activity feed from existing API endpoints
- Fetch task assignments, comments, status changes, and due dates
- Normalize activities into unified `InboxActivity` model
- Implement incremental polling (fetch only new activity since last check)
- Cache activities locally for instant reloads
- Display activity type indicators (icons) in UI
- Follow existing patterns from assigned tasks implementation

**Non-Goals:**
- Real-time push notifications (would require webhooks + server)
- Cross-workspace activity aggregation (workspace-scoped only)
- Activity filtering or categorization beyond type (future enhancement)
- Changing inbox navigation name (remains "Inbox")
- Modifying existing Task or Comment models

## Decisions

### Decision 1: Activity Feed Architecture

**Approach:**
Implement a multi-endpoint polling strategy that fetches:
1. Tasks assigned to user: `GET /team/{id}/task?assignees={user_id}&date_updated_gt={timestamp}`
2. Comments on user's tasks: `GET /task/{id}/comment` for each assigned task
3. Tasks with status changes: Query tasks with `status` filter
4. Tasks with approaching due dates: Query tasks with `due_date` in next 7 days

**Rationale:**
- Uses only existing, documented ClickUp API endpoints
- Fits TUI pull-based model (no server required)
- Provides comprehensive activity coverage
- Incremental polling minimizes API calls

**Alternatives Considered:**
- *Webhook-based push*: Requires always-on server, doesn't fit TUI model
- *Single endpoint polling*: No such endpoint exists (404 error)
- *Task-only activity feed*: Would miss comments and other important activity

### Decision 2: Activity Data Model

**Approach:**
Create new `InboxActivity` struct that normalizes different activity types:

```rust
pub enum ActivityType {
    Assignment,    // Task assigned to user
    Comment,       // New comment on user's task
    StatusChange,  // Task status changed
    DueDate,       // Task due date approaching
}

pub struct InboxActivity {
    pub id: String,              // Composite: "{type}_{source_id}"
    pub activity_type: ActivityType,
    pub title: String,           // Human-readable title
    pub description: String,     // Context/description
    pub timestamp: i64,          // Unix timestamp (milliseconds)
    pub task_id: Option<String>, // Source task ID
    pub comment_id: Option<String>, // Source comment ID (if applicable)
    pub workspace_id: String,    // Workspace context
    pub task_name: String,       // Source task name (for display)
}
```

**Rationale:**
- Unified model simplifies UI rendering
- Type enum enables icon display
- Optional fields handle different activity sources
- Composite ID ensures uniqueness across types

**Alternatives Considered:**
- *Separate models per type*: More type-safe but complicates UI
- *Use Notification model*: Designed for non-existent API, wrong semantics
- *Enum with nested data*: More Rust-idiomatic but harder to cache in SQLite

### Decision 3: Incremental Polling Strategy

**Approach:**
Store `last_inbox_check` timestamp per workspace in SQLite. On refresh:
1. If `last_inbox_check` exists: fetch activity with `date_updated_gt={timestamp}`
2. If not exists (first time): fetch activity from last 7 days
3. After successful fetch: update `last_inbox_check` to current timestamp
4. Retain activities for 30 days, cleanup older entries

**Rationale:**
- Minimizes API calls (only fetch new activity)
- Respects ClickUp rate limits (100 req/min)
- Provides "fresh" inbox experience
- 30-day retention balances history vs. database size

**Alternatives Considered:**
- *Always fetch all*: Wasteful, hits rate limits, slow
- *Fixed time window (always 7 days)*: Misses older unread activity
- *No timestamp tracking*: Loses "new" vs. "seen" distinction

### Decision 4: Activity Deduplication

**Approach:**
When merging activities from multiple endpoints:
1. Use composite key: `{activity_type}_{task_id}_{timestamp}`
2. If same task appears with different activity types, show both (e.g., assignment + comment)
3. If same task appears from same query twice, deduplicate by ID
4. Sort by timestamp descending (newest first)

**Rationale:**
- Preserves meaningful activity (assignment and comment are separate events)
- Removes technical duplicates from overlapping queries
- Newest-first matches modern activity feed conventions

**Alternatives Considered:**
- *Show only most recent per task*: Loses important context
- *Group by task*: More complex UI, harder to scan
- *Oldest-first*: Original spec, but less intuitive for activity feeds

### Decision 5: Cache-First Loading Strategy

**Approach:**
When user enters inbox:
1. Check cache for activities (valid if < 5 minutes old)
2. If cache valid: display immediately, then background refresh
3. If cache invalid/empty: show loading indicator, fetch from API
4. Manual refresh ('r' key): always fetch from API, update cache

**Rationale:**
- Instant UI response when cache is valid
- Reduces API calls during navigation
- Consistent with assigned tasks behavior
- Better UX than waiting for API every time

**Alternatives Considered:**
- *Always fetch from API*: Slower, more API calls
- *Cache-only with no refresh*: Stale data, no way to update
- *Background-only fetch*: No loading indicator, confusing UX

### Decision 6: Activity Type Icons

**Approach:**
Display Unicode emoji icons for activity types:
- 📋 (`U+1F4CB`) for Assignment
- 💬 (`U+1F4AC`) for Comment
- 🔄 (`U+1F504`) for Status Change
- ⏰ (`U+23F0`) for Due Date

**Rationale:**
- Unicode emojis render in most terminals
- No external icon dependencies
- Instantly recognizable semantics
- Consistent with terminal-native aesthetic

**Alternatives Considered:**
- *ASCII art icons*: Less clear, more screen space
- *Ratatui symbols*: Limited selection, less semantic
- *No icons, text labels only*: Harder to scan quickly

### Decision 7: API Client Methods

**Approach:**
Add methods to `ClickUpApi` trait and `ClickUpClient`:

```rust
// Get tasks assigned to user
async fn get_tasks_assigned_to_user(
    &self,
    team_id: &str,
    user_id: i32,
    date_updated_gt: Option<i64>
) -> Result<Vec<Task>>;

// Get comments for multiple tasks (batched)
async fn get_comments_for_tasks(
    &self,
    task_ids: &[String],
    date_created_gt: Option<i64>
) -> Result<Vec<Comment>>;

// Get tasks with approaching due dates
async fn get_tasks_with_due_dates(
    &self,
    team_id: &str,
    due_date_before: i64
) -> Result<Vec<Task>>;
```

**Rationale:**
- High-level methods encapsulate query logic
- Consistent with existing API trait pattern
- Supports mock client for testing
- Optional timestamp params enable incremental polling

**Alternatives Considered:**
- *Single `get_inbox_activity` method*: Too monolithic, hard to test
- *Raw query params in existing methods*: Less discoverable
- *Separate service layer*: Over-engineering for this scope

## Risks / Trade-offs

**[API Rate Limits]** Fetching from multiple endpoints could hit 100 req/min limit
- **Mitigation**: Incremental polling reduces calls, batch comments fetch, cache aggressively

**[Incomplete Activity]** Activity feed may not capture all notification types users expect
- **Mitigation**: Document what's included, gather feedback for additional activity types

**[Stale Data]** Cached activities may be outdated between refreshes
- **Mitigation**: 5-minute cache TTL, manual refresh ('r' key), background refresh

**[Performance]** Multiple API calls could slow inbox loading
- **Mitigation**: Parallel fetches where possible, cache-first strategy, loading indicator

**[Deduplication Complexity]** Merging activities from multiple sources is error-prone
- **Mitigation**: Clear deduplication rules, comprehensive tests, logging for debugging

**[Terminal Emoji Support]** Some terminals may not render Unicode emojis correctly
- **Mitigation**: Fallback to ASCII symbols, test on common terminals

**[Workspace Context]** User might expect cross-workspace activity aggregation
- **Mitigation**: Clear UI indication of which workspace's activity is shown, future enhancement

## Open Questions

- Should activity types be configurable (user selects which to show)?
- Should there be a "mark all as read" confirmation dialog?
- Should activity count badge show on Inbox sidebar item?
- Should we support filtering by activity type (e.g., show only comments)?
