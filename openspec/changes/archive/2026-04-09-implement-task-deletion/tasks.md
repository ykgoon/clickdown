## 1. Failing Test

- [x] 1.1 Write an integration test that selects a task, opens the delete dialog, confirms, and asserts the task is deleted (expect failure: status message shown but task remains)

## 2. Refactor Dialog Confirmation into `update()`

- [x] 2.1 Move dialog Enter/Esc handling from `handle_input()` into `update()` — check `dialog.is_visible()` before screen-specific handlers, process Enter for `ConfirmDelete` and `ConfirmQuit`, process Esc to hide dialog
- [x] 2.2 Ensure `handle_input()` returns `InputEvent::None` for dialog keys so `update()` is the sole handler

## 3. Implement Task Deletion

- [x] 3.1 In `update()`, when `ConfirmDelete` is confirmed, get the selected task ID and call `client.delete_task()` asynchronously (spawn async, send result via message channel)
- [x] 3.2 Add `AppMessage::TaskDeleted(Result<String, String>)` variant for the async result
- [x] 3.3 Handle `TaskDeleted(Ok(_))` — remove task from local list, clear selection, show success status
- [x] 3.4 Handle `TaskDeleted(Err(_))` — show error status, task remains in list

## 4. Polish

- [x] 4.1 Update dialog message from "this item" to "this task" for clarity
- [x] 4.2 Run `cargo test` to verify the failing test now passes
- [x] 4.3 Run `cargo clippy` and `cargo build` to verify no warnings
