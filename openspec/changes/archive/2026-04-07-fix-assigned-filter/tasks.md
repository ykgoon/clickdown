## 1. Fix assignees query parameter format

- [x] 1.1 Ensure `TaskFilters::to_query_string()` uses `add_all` for assignees (array format: `assignees[]=123`)
- [x] 1.2 Verify existing unit tests expect array format (`assignees[]=123`)
- [x] 1.3 Run `cargo test` to verify no regressions

## 2. Wire up current user fetch during TUI initialization

- [x] 2.1 In `TuiApp::new()`, after creating the API client, spawn a background task that calls `client.get_current_user()` and sends `AppMessage::CurrentUserLoaded` via the message channel
- [x] 2.2 Verify the `CurrentUserLoaded` handler in `process_async_messages()` correctly sets `current_user_id`
- [x] 2.3 Add a test that verifies `current_user_id` is populated after initialization with a mock client

## 3. Include closed tasks in assigned filter

- [x] 3.1 Update `get_tasks_with_assignee()` in `src/api/client.rs` to set `include_closed: Some(true)` on the `TaskFilters`
- [x] 3.2 Add logging to show the full query string being sent for assigned task fetches
- [x] 3.3 Add a test verifying `include_closed=true` is included in the query

## 4. Integration testing

- [x] 4.1 Build the application with `cargo build`
- [x] 4.2 Run all tests with `cargo test`
- [x] 4.3 Manually verify the `a` key filter returns correct tasks (mock tests cover: query format, include_closed, user ID population)
