## Why

Currently, tasks in the unfiltered view are sorted by status priority but lack visual grouping headers, making it hard to scan tasks by their status at a glance. The filtered "Assigned to Me" view also has no grouping. Users expect to see clear visual sections for "In Progress", "Todo", "Done", etc., similar to how Kanban boards or ClickUp's own UI organizes tasks.

## What Changes

- Add visual status group headers (e.g., "▸ IN PROGRESS (3)", "▸ TODO (5)", "▸ DONE (2)") in the task list view
- Group tasks under their respective status headers in both unfiltered and filtered views
- Collapse empty status groups (don't show headers for groups with zero tasks)
- Maintain the existing sort order within each group (by `updated_at` descending)
- Adjust list navigation to skip over group headers (they are not selectable)

## Capabilities

### New Capabilities
- `task-list-grouping`: Visual status group headers in the task list widget, rendering tasks grouped by `status_group` with collapsible section headers showing count

### Modified Capabilities
<!-- No existing capabilities have changing requirements -->

## Impact

- `src/tui/widgets/task_list.rs`: `TaskListState` and `render_task_list()` will need to support grouped rendering with headers
- `src/models/task.rs`: May need a `GroupedTasks` struct or helper to partition tasks by status group
- `src/tui/app.rs`: Task loading and rendering integration may need minor adjustments for the new grouped structure
- Navigation logic: keyboard navigation (j/k) must skip non-selectable header rows
