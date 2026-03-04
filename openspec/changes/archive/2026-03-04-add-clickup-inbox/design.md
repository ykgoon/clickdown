## Context

ClickDown currently provides workspace navigation (Workspaces → Spaces → Folders → Lists) and task management, but lacks access to ClickUp's notification/inbox system. Users must switch to the web app to view notifications, messages, and updates. The ClickUp API provides a notifications endpoint that can be integrated into the existing TUI architecture.

**Current Architecture:**
- Elm architecture pattern (Model-Update-View) with ratatui
- Sidebar navigation with workspace hierarchy
- SQLite caching for offline access
- ClickUpApi trait for dependency injection

**Stakeholders:**
- Terminal-focused developers who want to stay in their workflow
- Power users managing multiple workspaces who need notification visibility

## Goals / Non-Goals

**Goals:**
- Add inbox entry point in navigation sidebar
- Display unread notifications with oldest-first ordering
- Provide keyboard shortcuts to clear/mark messages as read
- Cache notifications locally for offline viewing
- Integrate with ClickUp notifications API
- Maintain consistent vim-style navigation patterns

**Non-Goals:**
- Real-time push notifications (polling-based only)
- Rich media rendering in notifications (text-only)
- Notification categorization or filtering (all unread shown)
- Cross-workspace notification aggregation (single workspace context)
- Notification settings management (view/clear only)

## Decisions

### Decision 1: Navigation Integration Pattern
**Choice:** Add inbox as a top-level navigation item in sidebar, separate from workspace hierarchy

**Rationale:**
- Inbox is workspace-agnostic (notifications span across workspaces)
- Consistent with email/inbox mental models
- Doesn't interfere with existing workspace → space → folder → list flow
- Easy to discover and access

**Alternatives Considered:**
- *Nested under workspace*: Would require selecting workspace first, adds friction
- *Separate screen with dedicated key*: Less discoverable, breaks navigation paradigm
- *Status bar indicator only*: Too limited, doesn't support message management

### Decision 2: Notification Ordering
**Choice:** Display oldest-first (chronological ascending)

**Rationale:**
- User request specified "oldest first on top"
- Allows users to process notifications in order received
- Matches traditional inbox mental models
- Easier to track which notifications have been seen

**Alternatives Considered:**
- *Newest-first*: Common in modern apps, but contradicts user requirement
- *Priority-based*: ClickUp notifications don't have priority metadata
- *User-configurable*: Adds complexity, can be added later if needed

### Decision 3: Message Clearing Strategy
**Choice:** Single-key shortcut (`c` for clear) to mark as read/remove from inbox

**Rationale:**
- Consistent with vim-style navigation (single-key actions)
- Fast workflow for power users
- Matches existing patterns (`d` for delete, `e` for edit)
- "Clear" semantics match inbox mental model

**Alternatives Considered:**
- *Confirmation dialog*: Adds friction for routine operation
- *Batch selection mode*: Adds complexity, can be added later
- *Auto-clear on view*: Too aggressive, users may want to review multiple times

### Decision 4: Data Model Design
**Choice:** Store notifications in SQLite with fields: id, workspace_id, title, description, created_at, read_at

**Rationale:**
- Matches ClickUp API response structure
- Supports offline caching pattern used elsewhere in app
- `read_at` timestamp allows tracking read status without deletion
- Enables future features like "show read notifications"

**Schema:**
```sql
CREATE TABLE notifications (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    created_at INTEGER NOT NULL,
    read_at INTEGER,
    fetched_at INTEGER NOT NULL
);
```

### Decision 5: API Integration Pattern
**Choice:** Extend ClickUpApi trait with `get_notifications(workspace_id)` method

**Rationale:**
- Consistent with existing API trait pattern
- Supports mock client for testing
- Workspace-scoped notifications align with ClickUp API
- Enables dependency injection for testing

**Alternatives Considered:**
- *Global notifications endpoint*: ClickUp API is workspace-scoped
- *Separate notification service*: Over-engineering for single endpoint
- *Direct HTTP calls in TUI*: Breaks dependency injection pattern

### Decision 6: Polling Strategy
**Choice:** Manual refresh only (no automatic polling)

**Rationale:**
- Terminal app paradigm (user-controlled actions)
- Avoids background network churn
- Consistent with existing refresh patterns
- Can add auto-refresh later if needed

**Alternatives Considered:**
- *Background polling every N minutes*: Adds complexity, may annoy users
- *Refresh on app startup*: Good, but doesn't help long-running sessions
- *WebSocket/push*: Not supported by ClickUp API

## Risks / Trade-offs

**[Risk] Notification API rate limits** → Mitigation: Cache aggressively, manual refresh only, respect API rate limiting

**[Risk] Large notification volume impacts performance** → Mitigation: Pagination support in API, limit display to recent N notifications, archive old entries

**[Risk] Notification schema changes in ClickUp API** → Mitigation: Flexible deserialization with custom handlers, robust error handling, version tolerance

**[Risk] TUI layout complexity with new view** → Mitigation: Reuse existing list widget patterns, consistent layout structure, test on small terminals

**[Trade-off] No real-time updates** → Acceptable: Terminal app paradigm, users expect manual refresh, can be added later

**[Trade-off] Text-only notifications** → Acceptable: Matches terminal capabilities, emojis/icons can convey meaning

**[Trade-off] No notification categorization** → Acceptable: MVP approach, can add filtering/sorting later based on user feedback
