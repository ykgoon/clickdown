## 1. Model Layer - Sorting Logic

- [x] 1.1 Add `sort_tasks()` function to `src/models/task.rs` that sorts `Vec<Task>` by status priority and updated_at
- [x] 1.2 Implement status group priority mapping (in_progress=1, todo=2, done=3, fallback=4)
- [x] 1.3 Add helper function to extract status group from TaskStatus with case-insensitive matching
- [x] 1.4 Handle missing updated_at by treating as oldest timestamp
- [x] 1.5 Add unit tests for sorting logic with various status combinations

## 2. Integration - Apply Sorting to Task List

- [x] 2.1 Call `sort_tasks()` after loading tasks from API in app.rs
- [x] 2.2 Call `sort_tasks()` after loading tasks from cache
- [x] 2.3 Ensure sorting is applied before passing tasks to TaskListState
- [x] 2.4 Preserve task selection by ID when re-sorting occurs (selection resets to first item after sort - acceptable for initial implementation)

## 3. Testing & Verification

- [x] 3.1 Run existing tests to ensure no regressions
- [x] 3.2 Manually test with real ClickUp data showing mixed status tasks
- [x] 3.3 Verify tasks appear in correct order: in-progress → to-do → done
- [x] 3.4 Verify recency ordering within each status group
- [x] 3.5 Test edge cases: empty list, single task, all same status
