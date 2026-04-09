## 1. State and Message Setup

- [x] 1.1 Add `task_name_input: String`, `task_description_input: String`, `task_creating: bool`, and `task_creation_focus: TaskCreationField` (enum with `Name` and `Description`) fields to `TuiApp` struct in `src/tui/app.rs`
- [x] 1.2 Add `AppMessage::TaskCreated(Result<Task, String>)` variant to the `AppMessage` enum in `src/tui/app.rs`
- [x] 1.3 Add `creating: bool` field to `TaskDetailState` struct in `src/tui/widgets/task_detail.rs`

## 2. Task Creation Form Input Handling

- [x] 2.1 Replace the 'n' key handler in `update_tasks()` (app.rs ~line 1582) to set `task_creating = true`, `task_name_input = ""`, `task_description_input = ""`, `task_creation_focus = Name`, `task_detail.creating = true`, and navigate to `Screen::TaskDetail`
- [x] 2.2 Add input capture in `update_task_detail()` for when `task_creating` is true: handle `Char(c)`, `Backspace` to append/remove characters in the focused field
- [x] 2.3 Add `Tab` key handler in `update_task_detail()` to toggle `task_creation_focus` between `Name` and `Description` when `task_creating` is true
- [x] 2.4 Add `Esc` key handler in `update_task_detail()` for creation mode: clear inputs, set `task_creating = false`, `task_detail.creating = false`, navigate back to `Screen::Tasks`
- [x] 2.5 Add Ctrl+S handler in `update_task_detail()` for creation mode: validate name is non-empty, call async `create_task()` via spawn with the list_id, name, and description

## 3. Async Task Creation Completion

- [x] 3.1 Implement the async spawn block for task creation: call `client.create_task(list_id, CreateTaskRequest)`, send `AppMessage::TaskCreated(result)` through the channel
- [x] 3.2 Add `AppMessage::TaskCreated(Ok(task))` handler in `process_async_messages()`: reload the current task list, switch to `Screen::Tasks`, show success status
- [x] 3.3 Add `AppMessage::TaskCreated(Err(e))` handler in `process_async_messages()`: set `self.error`, keep `task_creating = true` so the form stays open

## 4. Task Creation Form Rendering

- [x] 4.1 Update `render_task_detail()` to check `state.creating` — if true, render a creation form instead of read-only task data
- [x] 4.2 Render the creation form with: "New Task" title, name input field (with visual focus indicator), description input field, and hint text ("Ctrl+S to create, Esc to cancel")
- [x] 4.3 When `state.creating` is false, render the existing read-only task detail view (unchanged)

## 5. Help Text and Polish

- [x] 5.1 Update `get_task_list_hints()` in `src/tui/widgets/task_list.rs` to keep `n: New` (already present, no change needed)
- [x] 5.2 Update help dialog in `src/tui/widgets/help.rs` to document task creation shortcuts if not already covered
- [x] 5.3 Add test in `tests/app_test.rs` or unit test: verify 'n' key navigates to TaskDetail with creating mode
