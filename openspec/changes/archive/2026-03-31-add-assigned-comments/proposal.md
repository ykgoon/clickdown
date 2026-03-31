## Why

Users currently see only tasks assigned to them in the "Assigned to Me" section, but comments assigned to them (via the `assigned_commenters` field in ClickUp) are not visible. This creates a gap in awareness - users may miss important discussions they're expected to participate in.

## What Changes

- The "Assigned to Me" section will now display both assigned tasks AND assigned comments in a unified view
- Comments will be visually distinguished from tasks with a comment icon
- Clicking on an assigned comment will navigate to the parent task and open the comment thread
- The count badge will show combined count (tasks + comments)
- Users can filter to show only tasks, only comments, or both

## Capabilities

### New Capabilities
- `assigned-comments-aggregation`: Unified view showing assigned tasks and assigned comments together in the "Assigned to Me" section

### Modified Capabilities
- `assigned-tasks-nav`: Extend the assigned tasks navigation to include assigned comments in the aggregated view

## Impact

- **Code**: `src/tui/widgets/assigned_view.rs` - new widget for combined view
- **API**: Fetch assigned comments via ClickUp API (new endpoint integration)
- **Cache**: Store assigned comments in SQLite with task association
- **Navigation**: Update navigation logic to handle comment selection
- **UI**: Visual distinction between tasks and comments in the list
