## 1. Data model: Grouped task list state

- [x] 1.1 Add `ListRow` enum to `src/tui/widgets/task_list.rs` with variants `Header { label: String, count: usize }` and `Task(Task)`
- [x] 1.2 Add `GroupedTaskList` struct replacing `TaskListState`, containing `rows: Vec<ListRow>` and wrapping a `SelectableList<ListRow>`
- [x] 1.3 Implement `GroupedTaskList::from_tasks(tasks: Vec<Task>)` that groups tasks by `status_group`, sorts within groups, creates `ListRow` sequences with headers, and skips empty groups
- [x] 1.4 Override `select_next()` and `select_previous()` on `GroupedTaskList` to skip `Header` rows during navigation
- [x] 1.5 Implement `selected_task(&self) -> Option<&Task>` that extracts the task from the selected `ListRow::Task` variant

## 2. Rendering: Grouped task list widget

- [x] 2.1 Update `render_task_list()` to accept `&GroupedTaskList` and render `ListRow` items — headers with distinct style (dimmed bold text, separator), tasks with existing style
- [x] 2.2 Add header label formatting: map `StatusGroupPriority` to display string (e.g., `InProgress` → "IN PROGRESS (N)")
- [x] 2.3 Style header rows differently from task rows (use dimmed color, bold, no highlight on selection attempt)
- [x] 2.4 Ensure the `highlight_symbol` ("▸ ") only applies to task rows, not headers

## 3. Integration: Update app.rs callers

- [x] 3.1 Replace `TaskListState` with `GroupedTaskList` in `TuiApp` state
- [x] 3.2 Update `TasksLoaded` message handler to call `GroupedTaskList::from_tasks()` instead of setting flat task vec
- [x] 3.3 Update `load_tasks()` and `load_tasks_with_assigned_filter()` to populate the grouped list
- [x] 3.4 Update `selected_task()` callers in task detail, edit, and delete handlers to use the new method

## 4. Navigation and edge cases

- [x] 4.1 Implement `select_first()` that skips to the first task row (not a header)
- [x] 4.2 Handle empty task list: no rows rendered, no selection
- [x] 4.3 Handle single-group list: one header + all tasks under it
- [x] 4.4 Verify wrapping behavior: navigating past last task wraps to first task, skipping headers

## 5. Tests

- [x] 5.1 Add unit test: `from_tasks` groups tasks correctly by status_group with proper order
- [x] 5.2 Add unit test: empty groups are not rendered
- [x] 5.3 Add unit test: `select_next` skips header rows
- [x] 5.4 Add unit test: `select_previous` skips header rows and wraps correctly
- [x] 5.5 Add unit test: `selected_task` returns None when a header would be selected
- [x] 5.6 Run `cargo test` and verify all existing tests still pass

## 6. Build and verify

- [x] 6.1 Run `cargo build` with no errors
- [x] 6.2 Run `cargo clippy` and fix any warnings
- [x] 6.3 Manually verify grouped rendering in both unfiltered and "Assigned to Me" views
