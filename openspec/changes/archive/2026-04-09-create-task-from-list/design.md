## Context

ClickDown is a terminal-based ClickUp client using ratatui + crossterm. The TUI follows the Elm Architecture pattern — state lives in `TuiApp`, events produce `Message` variants, and the `update()` method processes messages to mutate state.

Currently, the task list view shows `n: New` as a hint but the handler just displays "Create task - coming soon". The backend is fully ready: `CreateTaskRequest` model, `ClickUpApi::create_task()` trait method, `ClickUpClient::create_task()` HTTP implementation, and `MockClickUpClient` for testing all exist.

The comment system already implements the exact pattern we need: text input with Ctrl+S save, Esc cancel, and a sentinel value (`usize::MAX`) for "new" vs "editing existing".

## Goals / Non-Goals

**Goals:**
- Pressing `n` in the task list opens a creation form in the TaskDetail screen
- User can type a task name (required) and optional description
- Ctrl+S creates the task via the API, reloads the task list, and returns to the list view
- Esc cancels and returns to the task list
- Error handling: show error message, keep form open for retry

**Non-Goals:**
- No priority, status, due date, or assignee selection in this iteration (the `CreateTaskRequest` model supports these, but the UI will not expose them yet)
- No rich text editing for description (plain text only)
- No modal overlay — reuse the existing TaskDetail screen in a new "creating" mode
- No offline queueing — if the API fails, the user retries

## Decisions

### 1. Reuse TaskDetail screen for creation (not a modal overlay)

**Decision:** Navigate to `Screen::TaskDetail` with a new `creating: bool` flag set to `true`.

**Rationale:** The TaskDetail screen already exists and has the right layout (name, status, priority, assignees, description). Using it avoids creating a new screen type. A modal overlay would require new rendering logic and input routing. The comment system uses inline editing within the existing view — we'll follow the same pattern but at the screen level.

**Alternatives considered:**
- **Modal overlay on task list:** More complex input routing, new rendering code, inconsistent with existing patterns
- **Separate screen type:** Duplicates too much of TaskDetail's layout logic

### 2. Task name is required, description is optional

**Decision:** The creation form requires a non-empty name field. Description is optional. Ctrl+S with an empty name shows an error and keeps the form open.

**Rationale:** ClickUp's API requires `name` on `CreateTaskRequest`. The description field is `Option<String>`. This matches the comment system's validation pattern (empty text shows error).

### 3. Use dedicated input fields on TuiApp, not TaskDetailState

**Decision:** Add `task_name_input: String` and `task_description_input: String` directly to `TuiApp`, not to `TaskDetailState`.

**Rationale:** This follows the existing pattern — `comment_new_text` lives on `TuiApp`, not on the comment widget state. Input fields are application-level state because they're mutated by the `update()` method during input capture, not by the rendering code.

### 4. Add `AppMessage::TaskCreated(Result<Task, String>)`

**Decision:** New message variant for async task creation completion. On success, reload the task list and switch back to `Screen::Tasks`. On error, set `self.error` and stay on the creation form.

**Rationale:** All API calls in the app follow the async spawn → channel → message pattern. This is consistent with `TaskStatusUpdated`, `CommentsLoaded`, etc.

### 5. Input capture: check `task_creating` flag before general input handling

**Decision:** In `update_task_detail()`, check `self.task_creating` first. If true, capture all character input into `task_name_input` (or `task_description_input` if a focus flag is set). Esc cancels. Ctrl+S saves.

**Rationale:** The comment system uses `comment_editing_index.is_some() || !comment_new_text.is_empty()` as the guard. We'll use `task_creating` as the simpler boolean equivalent.

### 6. Description input focus with Tab

**Decision:** Tab toggles focus between name and description fields during task creation. Visual indication of which field is focused (cursor or highlight).

**Rationale:** The comment system uses `comment_focus` boolean. We'll use `task_creation_focus: TaskCreationField` enum with `Name` and `Description` variants for clearer intent.

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| Input capture conflicts with existing TaskDetail key handlers | Gate task creation input behind `self.task_creating` flag — checked before existing handlers |
| Task name with special characters may not serialize correctly | `CreateTaskRequest` already uses serde Serialize — tested and working |
| User creates duplicate tasks by pressing Ctrl+S multiple times | Disable save after first press by setting `task_creating = false` immediately; API deduplication is ClickUp's responsibility |
| Task list reload may lose scroll position | Acceptable trade-off for v1; the list will show the new task |
| Description field is plain text (no markdown preview during creation) | Acceptable for v1; description is optional; users can edit later |

## Migration Plan

Not applicable — this is a client-side feature with no deployment or migration concerns.

## Open Questions

None — the existing backend and UI patterns provide sufficient guidance for implementation.
