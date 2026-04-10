## 1. Remove status prefix from task list rendering

- [x] 1.1 Remove `status_str` computation and its `Span` from `render_task_list` in `src/tui/widgets/task_list.rs`
- [x] 1.2 Add `#[allow(dead_code)]` annotation to `get_status_color` function
- [x] 1.3 Run `cargo build` to verify no compilation errors

## 2. Update tests

- [x] 2.1 Run `cargo test` to ensure all existing tests pass
- [x] 2.2 Verify no test references the removed status prefix rendering
