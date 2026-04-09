## Why

In the Task Detail view, pressing `s` is silently ignored. Users expect `s` to open the status picker (consistent with the Task List view), but `update_task_detail()` only handles `Ctrl+S` (save) and has no handler for plain `s`. This results in zero visual feedback — no status message, no overlay, no action — which is a UX bug.

## What Changes

- Add `s` key handler in `update_task_detail()` to open the status picker for the current task, matching the behavior in `update_tasks()`
- Add a test verifying `s` opens the status picker in Task Detail view
- Add snapshot tests confirming the status picker renders correctly from Task Detail

## Capabilities

### New Capabilities
- `task-status-from-detail`: Ability to change task status via `s` key from the Task Detail view, using the same status picker overlay as the Task List view

### Modified Capabilities
- (none — existing `tui-framework` and `task-management` specs already cover status picker behavior)

## Impact

- `src/tui/app.rs`: Add `s` key case in `update_task_detail()` method
- `tests/tui_test.rs`: New test `test_s_key_in_task_detail_should_respond` (already written, currently failing)
- `tests/snapshot_test.rs`: New snapshot tests (already written, passing)
