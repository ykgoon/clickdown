## REMOVED Requirements

### Requirement: Unified assigned work items view
**Reason**: The global "Assigned to Me" unified view combining tasks and comments is being removed. Per-list filtering replaces this feature.

**Migration**: Use the per-list "Assigned to Me" filter (`list-assigned-filter/spec.md`) to see assigned tasks within a list. Assigned comments are indicated per-task within the list context.

### Requirement: Visual distinction between tasks and comments
**Reason**: No longer needed since tasks and comments are no longer mixed in a single list. Comments are indicated within the existing task list view.

**Migration**: Tasks with assigned comments show a 💬 indicator in the list view. Comment details are visible when viewing the task's comment thread.

### Requirement: Combined count badge
**Reason**: The sidebar badge next to "Assigned to Me" is removed along with the navigation item itself.

**Migration**: No direct replacement. Filter results show count within the list view header when the filter is active.

### Requirement: Filter by item type
**Reason**: The All/Tasks Only/Comments Only filter was specific to the unified view. Per-list filtering shows tasks only; comments are indicated inline.

**Migration**: The per-list filter shows assigned tasks. Tasks with assigned comments are marked with an indicator. View the task's comment thread to see assigned comments.

### Requirement: Navigate to parent task from assigned comment
**Reason**: Comments are no longer standalone items in a unified list. They are always accessed from within their parent task's comment thread.

**Migration**: Navigate to the task normally, then open the comment thread. Assigned comments are visible within the thread.

### Requirement: Fetch assigned comments from all workspaces
**Reason**: Cross-workspace comment fetching is eliminated. Per-list comment fetching operates only on tasks visible in the current list.

**Migration**: The `get_comments_with_assigned_commenter(task_id, user_id)` API method is preserved. It is now called per-task within the current list context rather than across all workspaces.

### Requirement: Cache assigned comments
**Reason**: The dedicated `assigned_comments` cache table is removed. Comment data is accessed through the task's comment thread, which has its own caching.

**Migration**: Comment caching is handled by the existing task-comments cache. No separate assigned comments cache is needed.

### Requirement: Refresh assigned items
**Reason**: The unified refresh for tasks+comments in the global view is removed. List-level refresh handles both tasks and their comments.

**Migration**: Use the existing list refresh mechanism. If the assigned filter is active, it re-fetches tasks with the assignees parameter.
