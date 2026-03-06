## Why

The "Assigned to Me" navigation feature exists but may not correctly filter tasks by assignee due to incorrect API query parameter format. The ClickUp API expects `assignees` as a comma-separated list (e.g., `assignees=123,456`) but the current implementation sends it as repeated array parameters (e.g., `assignees[]=123&assignees[]=456`).

## What Changes

- Fix the query parameter format for the `assignees` filter in task API requests
- Update `TaskFilters::to_query_string()` to send assignees as comma-separated values
- Ensure `get_tasks_with_assignee()` correctly filters tasks by the specified user ID
- Add test coverage for the assignees filter format

## Capabilities

### New Capabilities

### Modified Capabilities

- `task-filtering`: Fix the assignees filter parameter format to match ClickUp API expectations

## Impact

- Modified code: `src/models/task.rs` (TaskFilters struct and to_query_string method)
- Modified code: `src/api/client.rs` (get_tasks_with_assignee method)
- Modified code: `src/utils/query.rs` (may need add_comma_separated helper)
- API integration: Task filtering by assignee
- Testing: Add integration test for assignees filter format
