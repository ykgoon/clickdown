## Context

ClickDown currently provides task viewing and editing capabilities but lacks comment management. Users must switch to the ClickUp web interface to view, create, or update comments on tasks. The task detail widget displays task properties (name, description, status, priority, assignees, due date) but has no comments section.

The application follows the Elm architecture pattern with:
- **Models** in `src/models/` for data structures
- **API client** in `src/api/` with trait-based dependency injection
- **TUI widgets** in `src/tui/widgets/` for rendering
- **Messages** in `src/app.rs` for state transitions

## Goals / Non-Goals

**Goals:**
- Display task comments in the task detail view with proper text wrapping
- Enable comment creation via a text input form
- Enable comment editing for user's own comments
- Support keyboard navigation (j/k) through comments list
- Cache comments locally in SQLite for offline access
- Follow existing code patterns for API calls, models, and TUI rendering

**Non-Goals:**
- Comment deletion (not supported by ClickUp API for regular users)
- Comment reactions or mentions
- Rich text editing (markdown input only)
- Real-time comment synchronization (manual refresh only)
- Nested/reply comments (flat list structure)

## Decisions

### 1. Comment Model Structure
**Decision:** Create a `Comment` model with fields: `id`, `text`, `text_preview`, `commenter` (User), `created_at`, `updated_at`, `assigned_commenter`, `assigned_by`.

**Rationale:** Matches ClickUp API response structure. The `text` field contains full comment content, `text_preview` is a truncated version for list views. `commenter` is the author, `assigned_commenter` is for assigned comments feature.

**Alternatives considered:**
- Minimal model with only `id`, `text`, `author`, `created_at`: Rejected because ClickUp API provides richer data that may be useful.
- Separate models for comment list vs. detail: Rejected as unnecessary complexity; same model works for both.

### 2. API Integration Pattern
**Decision:** Add `get_task_comments(task_id)`, `create_comment(task_id, text)`, and `update_comment(comment_id, text)` methods to the `ClickUpApi` trait, implemented by `ClickUpClient` and `MockClickUpClient`.

**Rationale:** Follows existing pattern for task and document API methods. Trait-based approach enables testing with mock client.

**Alternatives considered:**
- Single generic `comment_api()` method returning a comment service: Rejected as it adds indirection without benefit.
- Inline HTTP calls in app.rs: Rejected as it violates separation of concerns.

### 3. UI Layout for Comments
**Decision:** Add a comments panel below the task description in the task detail view. The panel shows a scrollable list of comments (most recent first) with a "Add comment" button at the bottom. Selected comment can be edited inline.

**Rationale:** Maintains consistency with existing task detail layout. Vertical stacking works well with terminal constraints. Scrollable list handles arbitrary number of comments.

**Alternatives considered:**
- Separate comments screen (toggle with 'c'): Rejected as it adds navigation overhead for a related feature.
- Side-by-side layout (comments in right panel): Rejected as it reduces width for task description, causing more text wrapping issues.
- Collapsible comments section: Rejected as it adds complexity; users can scroll past if not interested.

### 4. Text Wrapping Strategy
**Decision:** Use ratatui's `Paragraph` widget with `Wrap { trim: true }` for all comment text. Maximum line width is the widget width (no horizontal overflow).

**Rationale:** Addresses the requirement that "content shall be wrapped rather than over-extend beyond view width". Ratatui's built-in wrapping is reliable and handles unicode correctly.

**Alternatives considered:**
- Manual line breaking at word boundaries: Rejected as ratatui provides this out of the box.
- Horizontal scrolling: Rejected as it conflicts with the requirement; users shouldn't need to scroll horizontally to read content.

### 5. Comment Editing Flow
**Decision:** Pressing 'e' on a selected comment opens an inline edit form (multi-line text input). Pressing Ctrl+s saves, Esc cancels. Only the comment author can edit (checked via user ID match).

**Rationale:** Consistent with existing task editing pattern (press 'e' to edit, Ctrl+s to save). Inline editing maintains context.

**Alternatives considered:**
- Separate edit screen: Rejected as it loses context and adds navigation.
- Popup dialog: Rejected as terminal space is limited; inline form uses available space better.

### 6. Caching Strategy
**Decision:** Comments are cached in SQLite with a `task_comments` table: `task_id TEXT`, `comment_id TEXT PRIMARY KEY`, `text TEXT`, `commenter_id INTEGER`, `created_at INTEGER`, `updated_at INTEGER`, `fetched_at INTEGER`. Cache is invalidated after 5 minutes.

**Rationale:** Follows existing caching pattern for tasks and documents. 5-minute TTL balances freshness with API rate limits.

**Alternatives considered:**
- No caching (always fetch from API): Rejected as it causes slow loading and API rate limit issues.
- Cache forever with manual refresh: Rejected as comments are more dynamic than task metadata.

### 7. Keyboard Shortcuts
**Decision:** In task detail view with comments loaded:
- `j/k`: Navigate comments list (when focus is on comments)
- `Enter`: Open comment detail (future: show full thread)
- `e`: Edit selected comment (if author)
- `n`: New comment (opens form at bottom)
- `Tab`: Toggle focus between task form and comments panel

**Rationale:** Consistent with existing navigation patterns. `Tab` for panel switching is standard terminal behavior.

**Alternatives considered:**
- Dedicated comments mode (toggle with 'c'): Rejected as it adds mode complexity.
- Mouse click to focus: Rejected as ClickDown is keyboard-first (mouse is optional enhancement).

## Risks / Trade-offs

**[API Rate Limits]** → Comments are fetched per-task, which could lead to many API calls if users open many tasks. **Mitigation:** Caching with 5-minute TTL reduces repeated fetches. Future: batch comment fetching.

**[Long Comments]** → Very long comments may dominate the view. **Mitigation:** Show preview (first 3 lines) in list, full text on selection. Future: collapsible comments.

**[Edit Conflicts]** → If comment is edited via web while viewing in ClickDown, local copy becomes stale. **Mitigation:** Refresh action ('r') to refetch. Future: optimistic UI with conflict detection.

**[Terminal Width]** → Narrow terminals may cause excessive wrapping, making comments hard to read. **Mitigation:** Minimum terminal width warning (already exists). Comments use full available width.

**[SQLite Schema Migration]** → Adding comments table requires schema migration. **Mitigation:** Use `CREATE TABLE IF NOT EXISTS` for initial version. Future: proper migration system.
