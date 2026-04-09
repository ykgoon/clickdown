## Why

The task list view advertises `n: New` in its hint text, but pressing `n` only displays a "coming soon" status message. This is a dead-end interaction that frustrates users expecting to create tasks. The backend API layer (`create_task()`) is fully implemented and tested — only the TUI input handling and form rendering are missing.

## What Changes

- Pressing `n` in the task list view opens a task creation form (instead of showing "coming soon")
- Task creation form with name input (required) and optional description
- Ctrl+S saves the new task via the existing `create_task()` API
- Esc cancels and returns to the task list
- On success, the task list reloads to show the newly created task
- On error, displays an error message and keeps the form open

## Capabilities

### New Capabilities
- `task-creation-ui`: Task creation form with name/description input, save/cancel handling, and task list reload on success

### Modified Capabilities
<!-- No existing specs are being modified — this is a net-new capability -->

## Impact

- `src/tui/app.rs`: Add `task_name_input`, `task_description_input`, `task_creating` state fields; implement 'n' key handler, input capture, Ctrl+S save handler
- `src/tui/widgets/task_detail.rs`: Add `creating` mode to `TaskDetailState`; render creation form vs. read-only view
- `src/tui/app.rs` Message enum: Add `TaskCreated(Result<Task, String>)` variant
- `src/tui/widgets/help.rs`: Update help text to document task creation shortcuts
- Existing `CreateTaskRequest` model, `ClickUpApi::create_task()`, and `MockClickUpClient` are reused (no changes needed)
