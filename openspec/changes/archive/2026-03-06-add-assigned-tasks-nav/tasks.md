## 1. Data Model & State

- [x] 1.1 Add `AssignedTasksView` variant to navigation view state enum
- [x] 1.2 Add `current_user_id` field to application state for identity tracking
- [x] 1.3 Add `assigned_tasks` cache field to store fetched assigned tasks
- [x] 1.4 Add `assigned_tasks_timestamp` for cache invalidation

## 2. API Layer

- [x] 2.1 Add `get_current_user_id()` method to infer user identity from workspace/tasks
- [x] 2.2 Add `get_assigned_tasks()` method to fetch tasks across all lists where user is assignee
- [x] 2.3 Implement parallel fetching of tasks from multiple lists
- [x] 2.4 Add client-side filtering for assignee matching

## 3. Cache Layer

- [x] 3.1 Create `assigned_tasks` table in SQLite cache schema
- [x] 3.2 Add cache insert/query methods for assigned tasks
- [x] 3.3 Implement cache invalidation after 5-minute TTL
- [x] 3.4 Add cache hit/miss logging for debugging

## 4. Sidebar UI

- [x] 4.1 Add "Assigned to Me" item to sidebar navigation list
- [x] 4.2 Add user icon (👤) prefix to assigned tasks item
- [x] 4.3 Add count badge showing number of assigned tasks
- [x] 4.4 Add visual separator between assigned item and workspace hierarchy
- [x] 4.5 Implement selection highlighting for assigned tasks item

## 5. Assigned Tasks View

- [x] 5.1 Create assigned tasks view rendering function
- [x] 5.2 Display task list with name, status, priority, due date columns
- [x] 5.3 Implement empty state message when no tasks assigned
- [x] 5.4 Add header displaying "Assigned to Me" title
- [x] 5.5 Integrate with existing task detail view on selection

## 6. Keyboard Navigation

- [x] 6.1 Add j/k navigation bindings for assigned tasks list
- [x] 6.2 Add Enter key handler to open task detail from assigned view
- [x] 6.3 Add Esc key handler to return from assigned view
- [x] 6.4 Add `r` key handler for manual refresh
- [x] 6.5 Implement scroll behavior when selection moves out of view

## 7. Refresh & Loading

- [x] 7.1 Add loading indicator for assigned tasks fetch
- [x] 7.2 Implement cache refresh logic on `r` key press
- [x] 7.3 Update count badge after refresh completes
- [x] 7.4 Preserve task selection after refresh if still in list

## 8. Error Handling

- [x] 8.1 Handle unknown user identity with helpful error message
- [x] 8.2 Handle API errors during assigned tasks fetch
- [x] 8.3 Handle empty result set gracefully
- [x] 8.4 Add error messages to status bar

## 9. Testing & Verification

- [x] 9.1 Test with workspace containing no assigned tasks
- [x] 9.2 Test with workspace containing 100+ assigned tasks
- [x] 9.3 Test cache invalidation and refresh
- [x] 9.4 Test keyboard navigation edge cases (first/last item)
- [x] 9.5 Verify task detail opens correctly from assigned view
