## REMOVED Requirements

### Requirement: Assigned tasks navigation item
**Reason**: The global "Assigned to Me" sidebar navigation item is being removed. Per-list filtering replaces this feature — users now apply the "Assigned to Me" filter within individual list views instead of navigating to a dedicated global view.

**Migration**: Use the per-list "Assigned to Me" filter defined in `list-assigned-filter/spec.md`. Navigate to any list and toggle the filter to see assigned tasks.

### Requirement: Fetch assigned tasks from all accessible lists
**Reason**: Cross-list task fetching is eliminated as part of removing the global "Assigned to Me" view. Per-list filtering fetches tasks only from the current list using the same `assignees[]` API parameter.

**Migration**: The `get_tasks_with_assignee(list_id, user_id)` API method is preserved for per-list filtering. It now operates on a single list context rather than all accessible lists.

### Requirement: Assigned tasks view display
**Reason**: The dedicated assigned tasks list view is removed. Task display within a list view already exists; the new filter narrows the existing list.

**Migration**: Task display behavior is covered by existing list view specs. The per-list filter reuses the same task list rendering with an additional API filter parameter.

### Requirement: Refresh assigned tasks
**Reason**: The dedicated refresh mechanism for the global "Assigned to Me" view is removed. List-level refresh already handles re-fetching tasks.

**Migration**: Use the existing list refresh mechanism ('r' key in list view) with the assigned filter active.
