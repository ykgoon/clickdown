## Context

ClickDown is a terminal-based ClickUp client built with Rust and ratatui. The application already supports task CRUD operations (create, update, delete) via the `UpdateTaskRequest` struct, which includes an `assignees: Option<Vec<i64>>` field. The `Task` model already contains `assignees: Vec<User>` from the API response. The task detail view (`task_detail.rs`) renders name, status, priority, and description but does not currently display or allow editing assignees.

The app uses a trait-based API abstraction (`ClickUpApi`) with a real implementation (`ClickUpClient`) and a mock (`MockClickUpClient`). Async operations flow through `AppMessage` variants via a tokio channel. The TUI already uses overlay dialogs for confirmations and a help dialog.

## Goals / Non-Goals

**Goals:**
- Enable multi-select task assignment from the TUI without leaving the terminal
- Reuse existing `UpdateTaskRequest` infrastructure — no new API contract with ClickUp
- Cache list members in-memory to avoid redundant API calls
- Follow existing patterns: trait-based API, overlay dialogs, async message flow
- Maintain testability via `MockClickUpClient`

**Non-Goals:**
- `group_assignees` are out of scope (ClickUp-specific feature for assigning to groups)
- Persistent caching of members (no new SQLite tables)
- Pre-fetching members on app startup (lazy fetch on first picker open)
- Searching/filtering the member list (the list is ~25 people, scrollable is sufficient)
- Reordering assignees (ClickUp doesn't support ordered assignees)

## Decisions

### 1. Reuse `User` model instead of creating new `Member` type

**Decision:** Use the existing `User` struct for member deserialization. The API response from `GET /list/{list_id}/member` returns objects with `id`, `username`, `email`, `color`, `initials`, `profilePicture` — all fields that map directly to `User`'s existing fields. The extra `profileInfo` field in the response is silently ignored by serde.

**Alternatives considered:**
- **New `Member` struct:** More semantically accurate but adds duplicate code since 6+ fields are identical. Would require conversion functions between `Member` and `User`.
- **`Member` as a newtype wrapping `User`:** Unnecessary indirection.

**Rationale:** Reusing `User` reduces boilerplate and keeps the picker's data type consistent with what's already stored in `task.assignees`. The only extra field (`profileInfo`) is not needed for display.

### 2. In-memory cache in `TuiApp` state, not SQLite

**Decision:** Store cached members in `TuiApp` as `HashMap<String, Vec<User>>` keyed by list ID.

**Alternatives considered:**
- **SQLite table:** Persistent across sessions, more accurate membership over time. Adds schema migration, cache invalidation complexity, and I/O overhead for data that changes rarely (team membership changes are infrequent).
- **Pre-fetch all lists on startup:** Users may never open the picker. Unnecessary API calls at startup add latency.

**Rationale:** List membership is stable and rarely changes. An in-memory cache per session avoids the complexity of persistence and invalidation. The memory cost is negligible (~2KB per list × ~25 lists max).

### 3. Overlay dialog pattern (not inline edit in task detail)

**Decision:** The picker opens as a centered overlay/modal on top of the task detail view, using the existing confirmation dialog rendering pattern.

**Alternatives considered:**
- **Inline expansion in task detail:** Would require restructuring the task detail layout to accommodate a variable-height member list. Confusing when the task detail already has scrollable description and comments areas.
- **Replace task detail content temporarily:** Loses context — user can't see the task they're assigning while choosing assignees.

**Rationale:** Overlay is the clearest UX — the background dims, focus is on the picker, keyboard shortcuts are explicit. Matches the existing confirmation dialog pattern already in use.

### 4. Picker state stored in `TuiApp`, not in widget

**Decision:** Picker state (members list, selected IDs, cursor position) lives in `TuiApp` as fields, not encapsulated in a self-contained widget state struct. This matches the existing pattern used by `comment_editing_index`, `comment_new_text`, etc.

**Rationale:** Consistency with existing codebase. The app already uses the "flat state on TuiApp" pattern for transient UI state like form inputs and editing modes.

### 5. Fetch list members from the task's parent list

**Decision:** When the user opens the picker, use the `current_list_id` from app state to determine which list's members to show. This is the list the user navigated to before opening the task detail.

**Alternatives considered:**
- **Fetch from the task's list via `task.list.id`:** The task object includes a `list: Option<ListReference>` field with just `id` and `name`. This is more accurate but `current_list_id` is simpler and already tracked.
- **Fetch from workspace level (`GET /team/{team_id}/member`):** This endpoint returned "Route not found" when tested. List-level members is the correct scope anyway, as different lists may have different access.

**Rationale:** `current_list_id` is already maintained as navigation state and represents the list context the user is working within.

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| Stale cache if team membership changes during session | User may not see new team member until app restart | Low-impact: membership changes are rare; user can restart app. Could add cache TTL in future. |
| Large teams (>100 members) make picker unwieldy | Scrolling through long list is tedious | Current workspace has ~25 members. If this grows, add search/filter. Not needed now. |
| `current_list_id` may not match task's actual list if task was moved | Picker shows wrong member set | Edge case: task moved between lists. Using `task.list.id` would be more accurate but adds complexity. Can fix later if reported. |
| Overlay blocks interaction with task detail underneath | User can't reference task info while picking | The overlay can be semi-transparent (dim background). Task info is visible behind. |
| `Ctrl+S` in picker conflicts with existing `Ctrl+S` save in task edit | Ambiguous which save action triggers | Picker only opens when not in task edit mode. When picker is open, `Ctrl+S` saves picker, not task. Picker takes input focus. |

## Migration Plan

This is an additive feature with no database changes or breaking API changes. Deployment is straightforward:

1. Build and release new binary
2. No data migration required
3. No rollback strategy needed — feature is self-contained
4. Existing functionality is untouched

## Open Questions

None identified. All design decisions are resolved.
