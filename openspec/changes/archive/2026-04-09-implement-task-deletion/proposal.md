## Why

The `d` key shows a delete confirmation dialog, but confirming only displays "Task deletion - coming soon" and dismisses the dialog without actually deleting the task. The `tui-navigation` spec already requires task deletion on confirmation — this change completes the implementation.

## What Changes

- Wire up the `ConfirmDelete` dialog confirmation to call `delete_task` API
- Remove the selected task from the local task list after successful deletion
- Add a failing integration test that reproduces the bug, then make it pass
- Move dialog confirmation logic from `handle_input()` into `update()` for testability (Elm architecture compliance)

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `tui-navigation`: The "Delete item" context-aware action requirement already specifies that `d` → confirmation → delete. This change implements the missing delete call and removes the stub placeholder.

## Impact

- `src/tui/app.rs`: Dialog confirmation logic refactored from `handle_input()` into `update()`; `ConfirmDelete` handler calls `delete_task`
- `src/tui/widgets/dialog.rs`: Dialog message text may need refinement ("this item" → "this task")
- `tests/tui_test.rs` or new test file: Failing integration test for task deletion
- `src/api/mock_client.rs`: Already supports `with_delete_task_success()` — no changes needed
