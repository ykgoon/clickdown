## 1. Implement s key handler in Task Detail

- [x] 1.1 Add `s` key case in `update_task_detail()` that calls `open_status_picker()` with the current task from `self.task_detail.task`
- [x] 1.2 Add guard: if no task is loaded, show "No task selected" status message instead of opening picker

## 2. Verify and update tests

- [x] 2.1 Run `test_s_key_in_task_detail_should_respond` — should now pass (currently failing)
- [x] 2.2 Run `test_s_key_in_task_detail_no_response_snapshot` — update snapshot to show status picker overlay
- [x] 2.3 Run all existing `s` key tests to ensure no regression
- [x] 2.4 Run full test suite to confirm no breakage

## 3. Update help dialog

- [x] 3.1 Add `s` key shortcut documentation to the Task Detail section of the help overlay (`?` dialog)
