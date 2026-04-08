## 1. API Layer — List Members Endpoint

- [x] 1.1 Add `MembersResponse { members: Vec<User> }` struct to `src/models/user.rs`
- [x] 1.2 Add `list_members(list_id: &str) -> String` endpoint helper to `src/api/endpoints.rs`
- [x] 1.3 Add `get_list_members(list_id: &str) -> Result<Vec<User>>` method to `ClickUpApi` trait in `src/api/client_trait.rs`
- [x] 1.4 Implement `get_list_members` in `ClickUpClient` (`src/api/client.rs`) using the new endpoint and endpoint helper
- [x] 1.5 Add `get_list_members_response` field and `with_list_members_response` builder to `MockClickUpClient` (`src/api/mock_client.rs`)
- [x] 1.6 Implement `get_list_members` on `MockClickUpClient` to return the configured response or error
- [x] 1.7 Compile and verify `cargo build` passes

## 2. App State — Cache and Messages

- [x] 2.1 Add `cached_list_members: HashMap<String, Vec<User>>` field to `TuiApp` in `src/tui/app.rs`
- [x] 2.2 Initialize the HashMap in `TuiApp::new()`
- [x] 2.3 Add `MembersLoaded(Result<Vec<User>, String>)` variant to `AppMessage` enum
- [x] 2.4 Add `AssigneesUpdated(Result<Task, String>)` variant to `AppMessage` enum
- [x] 2.5 Add message handler for `MembersLoaded` — store in cache, open picker if success
- [x] 2.6 Add message handler for `AssigneesUpdated` — update task detail, close picker, show status
- [x] 2.7 Compile and verify `cargo build` passes

## 3. Task Detail — Display Assignees

- [x] 3.1 Update `render_task_detail` in `src/tui/widgets/task_detail.rs` to display assignees line (comma-separated usernames, "None" if empty)
- [x] 3.2 Add assignee line to the layout constraints (new `Constraint::Length(1)`)
- [ ] 3.3 Manually test with `cargo run` to verify assignees display correctly

## 4. Assignee Picker — Widget and State

- [x] 4.1 Create `src/tui/widgets/assignee_picker.rs` with `render_assignee_picker()` function
- [x] 4.2 Implement checkbox list rendering: `[x]` for selected, `[ ]` for unselected, cursor highlight
- [x] 4.3 Add keyboard shortcut hint line at bottom: "Space: toggle | j/k: navigate | Ctrl+S: save | Esc: cancel"
- [x] 4.4 Add picker state fields to `TuiApp`: `assignee_picker_open: bool`, `assignee_picker_members: Vec<User>`, `assignee_picker_selected: HashSet<i64>`, `assignee_picker_cursor: usize`
- [x] 4.5 Export `assignee_picker` module in `src/tui/widgets/mod.rs`
- [x] 4.6 Compile and verify `cargo build` passes

## 5. App Integration — Key Handling and Flow

- [x] 5.1 Add `A` key handler in task detail input processing to open picker
- [x] 5.2 Implement guard: only open if `current_list_id` is Some
- [x] 5.3 Implement cache check: if list members in cache, populate picker state directly
- [x] 5.4 Implement cache miss: spawn tokio task to call `get_list_members`, send `MembersLoaded`
- [x] 5.5 Add j/k cursor navigation in picker (with bounds checking)
- [x] 5.6 Add Space key handler to toggle member selection in picker
- [x] 5.7 Add Esc handler in picker to cancel (clear picker state, set `assignee_picker_open: false`)
- [x] 5.8 Add Ctrl+S handler in picker to save: build `UpdateTaskRequest`, call `update_task` API
- [x] 5.9 Add rendering logic: when `assignee_picker_open` is true, render picker overlay on top of task detail
- [ ] 5.10 Manually test full flow with `cargo run`: open task, press A, toggle, save, verify

## 6. Tests

- [x] 6.1 Add unit test for `MembersResponse` deserialization in `src/models/user.rs`
- [x] 6.2 Add integration test: mock list members, open picker, verify members displayed
- [x] 6.3 Add integration test: toggle assignees and save, verify `UpdateTaskRequest` sent with correct IDs
- [x] 6.4 Add integration test: cancel picker with Esc, verify no API call made
- [x] 6.5 Add integration test: empty assignee list saves successfully (assignees: [])
- [x] 6.6 Run `cargo test` and verify all tests pass

## 7. Cleanup and Verification

- [x] 7.1 Run `cargo clippy` and fix all warnings (no new warnings from our code)
- [x] 7.2 Run `cargo fmt` to ensure consistent formatting
- [ ] 7.3 Run `cargo build --release` and verify no errors (still running, started at task execution time)
- [x] 7.4 Run full `cargo test` suite one final time (405 tests, all passing)
