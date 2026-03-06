## Context

ClickDown currently requires users to navigate through the workspace hierarchy (Workspaces → Spaces → Folders → Lists) to find tasks assigned to them. The Task model already includes an `assignees` field containing user information. The navigation sidebar displays workspace items but has no dedicated view for personally assigned tasks.

**Current State:**
- Navigation sidebar shows workspace hierarchy only
- Tasks are fetched per-list via `get_tasks(list_id, filters)`
- Task model has `assignees: Vec<User>` field
- No cross-list aggregation of assigned tasks
- No user identity tracking for filtering "my tasks"

**Constraints:**
- Must work with existing ClickUp API (no backend changes)
- Must integrate with existing TUI navigation pattern (j/k/Enter)
- Should not break existing workspace navigation
- Performance: fetching across many lists could be slow

## Goals / Non-Goals

**Goals:**
- Add "Assigned to Me" navigation item in sidebar
- Fetch and display all tasks where current user is an assignee
- Support keyboard navigation (j/k/Enter/Esc) for assigned tasks view
- Show task count badge on navigation item
- Integrate with existing task detail view
- Cache assigned tasks for performance

**Non-Goals:**
- Task filtering by status/priority within assigned view (future enhancement)
- Editing assignees from assigned view (use existing task edit)
- Real-time sync of assigned tasks (manual refresh only)
- Group assignee filtering (only direct assignees)

## Decisions

### 1. User Identity Detection
**Decision:** Use the `creator` field from any fetched task to identify the current user, then match against `assignees` field.

**Rationale:** ClickUp API doesn't provide a "get current user" endpoint in the task API scope. The authenticated user's identity can be inferred from tasks they created or from the workspace membership.

**Alternatives Considered:**
- Store user ID in config during auth: Requires additional API call during auth
- Pass user ID with token: More complex token management
- Filter client-side after fetching all tasks: Less efficient but simpler

### 2. Task Aggregation Strategy
**Decision:** Fetch tasks from all accessible lists in parallel, then filter client-side for assignee match.

**Rationale:** ClickUp API doesn't support cross-list assignee filtering in a single call. Parallel fetching minimizes latency.

**Alternatives Considered:**
- Sequential fetching: Simpler but slower
- Cache-first with background refresh: More complex, better UX
- API-level filtering per list: Requires knowing all list IDs upfront

### 3. Navigation State
**Decision:** Add new `AssignedTasksView` state to the TUI navigation enum, parallel to `ListView`, `FolderView`, etc.

**Rationale:** Consistent with existing navigation pattern. Allows reusing task list rendering logic.

**Alternatives Considered:**
- Special filter on existing list view: Confusing state management
- Separate screen type: More code duplication

### 4. Caching Strategy
**Decision:** Store assigned tasks in a dedicated cache table with timestamp, invalidate after 5 minutes or on manual refresh.

**Rationale:** Cross-list aggregation is expensive. Cache prevents repeated API calls during navigation.

**Alternatives Considered:**
- No caching: Poor performance
- Cache per-list only: Doesn't help aggregation cost
- Real-time sync: Too complex for initial implementation

### 5. Performance Optimization
**Decision:** Limit initial fetch to 100 tasks, provide "load more" capability.

**Rationale:** Users typically have <100 active assigned tasks. Prevents UI freeze on large workspaces.

**Alternatives Considered:**
- Fetch all tasks: Risk of slow performance
- Pagination with offset: ClickUp API supports but adds complexity
- Virtual scrolling: Overkill for initial implementation

## Risks / Trade-offs

**[Performance]** Fetching across many lists could be slow → Mitigation: Parallel fetching, caching, 100-task limit

**[Stale Data]** Cached assigned tasks may be outdated → Mitigation: 5-minute TTL, manual refresh key (`r`)

**[User Identity]** Inferring user ID from tasks may fail for new users → Mitigation: Fallback to fetching all tasks if identity unknown

**[API Rate Limits]** Multiple parallel requests could hit rate limits → Mitigation: Batch requests, respect rate limit headers

**[Memory]** Holding all assigned tasks in memory → Mitigation: 100-task limit, future: pagination

## Migration Plan

1. Add user identity tracking to app state
2. Create cache table for assigned tasks
3. Add API method to fetch assigned tasks across lists
4. Add `AssignedTasksView` to navigation state
5. Add "Assigned to Me" item to sidebar widget
6. Implement keyboard navigation for assigned view
7. Add refresh capability
8. Test with large workspace (many lists)

**Rollback:** Feature is additive only - no breaking changes. Can be disabled by removing sidebar item.

## Open Questions

- Should assigned tasks include completed/closed tasks by default?
- Should group assignees be included or only direct assignees?
- What refresh interval is appropriate for cache invalidation?
