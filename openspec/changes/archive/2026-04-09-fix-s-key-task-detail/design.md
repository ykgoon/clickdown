## Context

The TUI app has two task-related screens:
- **Task List view** (`Screen::Tasks`): `s` key opens the status picker via `open_status_picker(task)`, which uses the selected task from the list.
- **Task Detail view** (`Screen::TaskDetail`): Only `Ctrl+S` is handled (save task). Plain `s` falls through to `_ => {}` silently.

The status picker infrastructure (`open_status_picker()`, `handle_status_picker_input()`, `save_status()`, `render_status_picker()`) already exists and is fully functional. The only gap is that `update_task_detail()` doesn't wire plain `s` to it.

The app has a `task_detail.task: Option<Task>` field that holds the current task being viewed — the same data needed to open the status picker.

## Goals / Non-Goals

**Goals:**
- Make `s` in Task Detail view open the status picker for the currently viewed task
- Provide consistent UX between Task List and Task Detail views
- Add test coverage to prevent regression

**Non-Goals:**
- No changes to status picker UI/behavior itself (reuse existing implementation)
- No new API endpoints or data model changes
- No changes to Task List view behavior

## Decisions

### Decision 1: Reuse `open_status_picker()` from Task Detail

**Approach**: Call the same `open_status_picker(task)` method used by Task List view, passing `self.task_detail.task.clone()`.

**Alternative considered**: Create a separate `open_status_picker_from_detail()` method.
**Rejected because**: The existing method already takes a `Task` parameter and sets all required state (`status_picker_task_id`, `status_picker_original_status`, `status_picker_statuses`, `status_picker_cursor`, `status_picker_open`). No duplication needed.

**Guard condition**: Only open status picker if `self.task_detail.task` is `Some(task)`. If no task is loaded, show a status message: "No task selected".

### Decision 2: Placement in `update_task_detail()`

Insert the `s` key handler alongside the existing `Ctrl+S` handler (around line 1575), before the comment editing and navigation match block. This keeps all status-related shortcuts grouped together.

The handler goes in the existing `match key.code` block inside the comment editing section (after the `Backspace` arm), in the same match that handles `Ctrl+S` for saving the task.

### Decision 3: Key behavior in comment editing mode

When a user is actively typing a comment (comment_editing_index is set or comment_new_text is non-empty), `s` should be treated as a character to type, not a shortcut. The existing code already handles this by returning early in the comment editing block — the `s` key handler for status picker will only trigger in the normal navigation match, which is unreachable during comment editing because of the early return.

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| User accidentally opens status picker while typing comment | Comment editing mode intercepts all key events before they reach the status picker handler |
| Status picker overlaps with comment form | Status picker is a centered overlay with dimmed background — existing rendering handles z-order correctly |
| No task loaded in detail view (edge case) | Guard with `if let Some(task) = &self.task_detail.task` — show "No task selected" status message |

## Migration Plan

Not applicable — this is a client-side TUI bug fix with no deployment or migration concerns.

## Open Questions

None.
