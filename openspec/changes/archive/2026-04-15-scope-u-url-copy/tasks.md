## 1. Add `is_text_input_active()` method

- [x] 1.1 Add `is_text_input_active(&self) -> bool` method to `TuiApp` in `src/tui/app.rs` that returns true when any of: `url_input_open`, `status_picker_open`, `assignee_picker_open`, `task_creating`, `comment_editing_index.is_some()`, `!comment_new_text.is_empty()`
- [x] 1.2 Add unit test for `is_text_input_active()` covering all active/inactive states

## 2. Add `handle_text_input()` method

- [x] 2.1 Add `handle_text_input(&mut self, key: KeyEvent)` method that delegates to the active input handler (`handle_url_input`, `handle_status_picker_input`, assignee picker handler, task creation input, comment input)
- [x] 2.2 Refactor inline comment editing input handling in `update_task_detail()` into a dedicated `handle_comment_input(&mut self, key: KeyEvent)` method
- [x] 2.3 Refactor inline task creation input handling in `update_task_detail()` into a dedicated `handle_task_creation_input(&mut self, key: KeyEvent)` method
- [x] 2.4 Add `handle_assignee_picker_input(&mut self, key: KeyEvent)` method (extract from inline code in `update_task_detail()`)

## 3. Restructure `handle_input()` to use text input guard

- [x] 3.1 Move `u` key handler below the new text input guard block
- [x] 3.2 Insert `is_text_input_active()` check after dialog guards and before the `u` key handler, routing to `handle_text_input()` when active
- [x] 3.3 Verify existing modal guards (`url_input_open`, `status_picker_open`) still function (they are now covered by the unified guard too)
- [x] 3.4 Remove redundant individual modal return blocks if they are now covered by `handle_text_input()` (kept for redundancy/safety)

## 4. Add snapshot tests for text input `u` key handling

- [x] 4.1 Add public getter methods on `TuiApp` for test assertions: `is_comment_editing_active()`, `is_task_creating()`, `comment_new_text()`, `task_name_input()`, `task_description_input()`, `task_creation_focus()`
- [x] 4.2 Create snapshot test `test_u_key_in_comment_editing`: navigate to task detail, start comment editing, type `u` via `app.update()`, assert letter `u` appears in `comment_new_text` (not URL copy). Snapshot the comment text content
- [x] 4.3 Create snapshot test `test_u_key_in_task_name_creation`: start task creation with name field focused, type `u` via `app.update()`, assert letter `u` appears in `task_name_input`. Snapshot the task name input content
- [x] 4.4 Create snapshot test `test_u_key_in_task_description_creation`: start task creation with description field focused, type `u` via `app.update()`, assert letter `u` appears in `task_description_input`. Snapshot the task description input content
- [x] 4.5 Create snapshot test `test_g_u_chord_still_opens_url_dialog`: press `g` then `u`, assert URL dialog opens. Snapshot the dialog state. This guards against regression when modifying the `u` key handler

## 5. Verify no regression

- [x] 5.1 Run `cargo test` to ensure all existing tests pass including new snapshot tests
- [x] 5.2 Run `cargo test --test snapshot_test` to verify all snapshots match expected output