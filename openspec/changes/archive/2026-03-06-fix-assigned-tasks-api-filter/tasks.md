## 1. Core Implementation

- [x] 1.1 Add `add_comma_separated_ints()` method to `QueryParams` in `src/utils/query.rs`
- [x] 1.2 Update `TaskFilters::to_query_string()` to use comma-separated format for assignees
- [x] 1.3 Verify `get_tasks_with_assignee()` in `src/api/client.rs` uses the updated filters correctly

## 2. Testing

- [x] 2.1 Add unit test for `add_comma_separated_ints()` method
- [x] 2.2 Add unit test for `TaskFilters` with assignees producing correct format
- [x] 2.3 Run existing tests to ensure no regressions
- [x] 2.4 Add snapshot tests for "Assigned to Me" view

## 3. Verification

- [x] 3.1 Build the project and verify no compilation errors
- [x] 3.2 Test with snapshot tests verifying assigned tasks view rendering
