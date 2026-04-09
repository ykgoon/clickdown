## Context

The TUI app follows an Elm architecture pattern where `handle_input()` reads raw crossterm events and `update()` processes `InputEvent` to mutate state. Currently, dialog confirmation (Enter on `ConfirmDelete`) is handled inside `handle_input()` — a private method that reads directly from crossterm, making it untestable in headless tests.

The stub at line 1436-1438 of `src/tui/app.rs` sets a status message instead of calling `delete_task()`. The `ClickUpApi` trait already defines `delete_task(&self, task_id: &str) -> Result<()>`, and `MockClickUpClient` already supports `with_delete_task_success()`.

## Goals / Non-Goals

**Goals:**
- Task deletion actually calls the API and removes the task from the local list
- Dialog confirmation logic moves into `update()` for Elm architecture compliance and testability
- A failing test case exists first, then passes after implementation

**Non-Goals:**
- No undo functionality (out of scope)
- No bulk delete
- No changes to the API client itself (already has `delete_task`)

## Decisions

### Decision 1: Move dialog confirmation from `handle_input()` to `update()`

The `handle_input()` method currently intercepts Enter/Esc for the dialog and executes side effects directly. This violates the Elm pattern where all state changes flow through `update()`.

**Approach**: Add a new `InputEvent::ConfirmDialog` and `InputEvent::CancelDialog` variant (or handle dialog state in `update()` by checking `dialog.is_visible()` and routing Enter/Esc accordingly). The `update()` method will check if a dialog is visible first, and if Enter is received with `dialog.confirmed() == true`, execute the appropriate action.

**Alternative**: Add a `Message::DeleteTaskConfirmed` async message variant. This was considered but adds unnecessary complexity for a simple synchronous action.

**Chosen**: Handle dialog confirmation directly in `update()` by checking `dialog.is_visible()` before screen-specific handlers.

### Decision 2: Error handling on delete failure

If `delete_task()` fails, show the error in the status bar and do NOT remove the task from the local list. The user can retry.

### Decision 3: Dialog message refinement

Change "Are you sure you want to delete this item?" to "Are you sure you want to delete this task?" for clarity, since this dialog is only used for task deletion.

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| `handle_input()` and `update()` both process dialog keys during TUI run | Ensure `handle_input()` returns `InputEvent::None` for dialog keys so `update()` doesn't double-process |
| Task list desync if API succeeds but local removal fails | Local removal is just `Vec::retain` — cannot fail independently |
| Dialog confirmation in `update()` could conflict with screen-specific Enter handlers | Check `dialog.is_visible()` first, return early before screen handlers |
