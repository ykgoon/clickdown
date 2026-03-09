# Assigned Tasks "Zero Tasks" Bug Fix

## Problem

Users reported that the "Assigned to Me" view was showing zero tasks even when tasks were assigned to them in ClickUp.

## Root Causes Identified

### 1. Missing `limit` Parameter in TaskFilters

The `TaskFilters` struct was missing a `limit` field. The ClickUp API uses:
- `limit` - Controls how many tasks to return (default: 100)
- `page` - Pagination page number (not for limiting results)

Previously, the code was incorrectly using `page` when it should use `limit`.

**Fix:** Added `limit: Option<u32>` field to `TaskFilters` and updated `to_query_string()` to include it.

### 2. Insufficient Logging

There wasn't enough visibility into what URLs were being generated and what the API was returning.

**Fix:** Added detailed logging in `get_tasks_with_assignee()`:
- Logs the generated query string
- Logs the filters being used
- Logs the number of tasks returned by the API

## Changes Made

### `src/models/task.rs`
- Added `limit: Option<u32>` field to `TaskFilters` struct
- Updated `to_query_string()` to include `limit` parameter
- Added test `test_task_filters_with_limit()` to verify the format

### `src/api/client.rs`
- Updated `get_tasks_with_assignee()` to use `filters.limit` instead of `filters.page`
- Added detailed logging at `info!` level showing:
  - The generated query string (e.g., `?assignees[]=123&limit=100`)
  - The number of tasks returned by the API
- Added debug logging showing the filter values

## Expected Query Format

The ClickUp API expects the assignees parameter in **array format**:
```
GET https://api.clickup.com/api/v2/list/{list_id}/task?assignees[]=123&limit=100
```

**NOT** comma-separated:
```
❌ ?assignees=123,456
✓ ?assignees[]=123&assignees[]=456
```

## Testing

### 1. Test with Debug Command (Recommended)

Use the debug command with verbose logging to see the actual API calls:

```bash
# See the query strings being generated
clickdown debug assigned-tasks --verbose 2>&1 | grep "query:"

# Expected output:
# Fetching tasks for user {id} from list {list_id} with query: ?assignees[]=123&limit=100
# API returned {N} tasks for user {id} in list {list_id}
```

### 2. Test with JSON Output

```bash
# Get assigned tasks as JSON
clickdown debug assigned-tasks --json | jq 'length'

# Should return the count of assigned tasks
```

### 3. Test in TUI

1. Launch the TUI: `cargo run`
2. Navigate to "Assigned to Me" in the sidebar (press `j`/`k` to navigate, `Enter` to select)
3. The view should now show tasks assigned to you

### 4. Check Logs

If tasks still show zero, check the logs:

```bash
# Run with debug logging
RUST_LOG=debug cargo run 2>&1 | grep -E "(assignees|query:|API returned)"
```

Look for:
- `assignees[]=` in the query string (not `assignees=`)
- `API returned N tasks` where N > 0

## Troubleshooting

### Still showing zero tasks?

1. **Verify API token permissions:**
   ```bash
   clickdown debug auth-status --verbose
   ```

2. **Check if you have accessible lists:**
   ```bash
   clickdown debug lists-all
   ```

3. **Verify tasks exist in ClickUp:**
   - Open ClickUp web UI
   - Filter by "Assigned to Me"
   - Confirm tasks exist

4. **Check the query format:**
   ```bash
   clickdown debug assigned-tasks --verbose 2>&1 | grep "query:"
   ```
   Should show: `?assignees[]={your_user_id}&limit=100`

5. **Check API response:**
   ```bash
   clickdown debug assigned-tasks --verbose 2>&1 | grep "API returned"
   ```
   Should show: `API returned N tasks for user {id}`

### Common Issues

| Issue | Symptom | Solution |
|-------|---------|----------|
| Wrong parameter format | `assignees=123` instead of `assignees[]=123` | Already fixed in this PR |
| Missing limit parameter | Using default API limit | Already fixed - now sends `limit=100` |
| No accessible lists | "No accessible lists found" error | Check workspace/space permissions |
| Invalid API token | Auth error | Run `clickdown debug auth-status` |
| User ID not detected | "User identity not detected" error | Open a task to auto-detect user ID |

## Test Results

All tests pass:
- ✅ 137 lib tests (including new `test_task_filters_with_limit`)
- ✅ 7 integration tests for assigned tasks
- ✅ 8 CLI tests

## Next Steps for User

1. Build the updated code:
   ```bash
   cargo build --release
   ```

2. Test with verbose logging:
   ```bash
   ./target/release/clickdown debug assigned-tasks --verbose
   ```

3. Launch TUI and verify "Assigned to Me" shows tasks:
   ```bash
   ./target/release/clickdown
   ```

4. If issues persist, share the verbose output for further debugging.
