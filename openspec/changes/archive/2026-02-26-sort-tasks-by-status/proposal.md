## Why

Tasks in the task list are currently displayed in the order returned by the ClickUp API, making it difficult to find active work. Users need to manually scan through all tasks to identify what needs attention. Sorting tasks by status (in-progress first, then to-do, then done) with recent activity prioritized within each group will improve task discoverability and workflow efficiency.

## What Changes

- Task list will be sorted by status: in-progress → to-do → done
- Within each status group, tasks will be sorted by most recently active (updated_at descending)
- The sorting will be applied automatically when tasks are loaded
- No user configuration required - this is the default behavior
- Visual status indicators (colors/badges) remain unchanged

## Capabilities

### New Capabilities
- `task-sorting`: Defines the task sorting logic and status priority ordering

### Modified Capabilities
- `task-list-ui`: Updated to display sorted tasks instead of API order

## Impact

- **Modified files**: `src/models/task.rs` (add sorting logic), `src/tui/widgets/task_list.rs` (apply sorting)
- **Dependencies**: None - uses existing `updated_at` and `status` fields
- **Breaking changes**: None - this is a UI behavior improvement
- **API changes**: None - sorting is client-side only
