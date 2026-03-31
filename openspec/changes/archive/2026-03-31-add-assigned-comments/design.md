## Context

The "Assigned to Me" navigation feature (spec: `assigned-tasks-nav`) currently displays only tasks where the user is in the `assignees` field. However, ClickUp also supports comment assignments via the `assigned_commenters` field on comments. Users miss discussions they're expected to participate in because there's no unified view of assigned work items.

The existing comment system (spec: `task-comments`) already supports fetching and displaying comments with their metadata including `assigned_commenters`. The CLI debug mode (`debug_ops.rs`) already has support for comment operations including assigned commenter filtering.

## Goals / Non-Goals

**Goals:**
- Display assigned comments alongside assigned tasks in a unified "Assigned to Me" view
- Visually distinguish comments from tasks in the list
- Enable navigation to parent task when selecting an assigned comment
- Show combined count badge (tasks + comments)
- Support filtering by type (all, tasks only, comments only)
- Cache assigned comments with task association for offline access

**Non-Goals:**
- Modifying the ClickUp API comment assignment functionality
- Changing how comments are displayed in the task detail view
- Adding comment assignment creation UI (this is done in ClickUp web app)
- Modifying the existing task-only assigned view behavior when accessed from list context

## Decisions

### Decision 1: Unified vs. Separate View
**Chosen**: Unified view with type indicators

**Alternatives considered:**
1. **Separate "Assigned Comments" navigation item**: Would fragment the assigned work view
2. **Tabs within Assigned view**: Adds complexity to navigation

**Rationale**: Users want to see all work assigned to them in one place. Comments and tasks are both action items requiring attention. Visual distinction (icons) provides clarity without separation.

### Decision 2: Data Model - AssignedItem Enum
**Chosen**: Create `AssignedItem` enum wrapping Task or Comment

```rust
pub enum AssignedItem {
    Task(Task),
    AssignedComment(AssignedComment),
}

pub struct AssignedComment {
    pub comment: Comment,
    pub task: TaskReference,  // Parent task info
    pub assigned_at: Option<i64>,
}
```

**Rationale**: 
- Type-safe handling of heterogeneous list
- Preserves full task/comment data structures
- Easy to extend with other assigned work types (e.g., subtasks)

### Decision 3: Fetch Strategy
**Chosen**: Parallel fetch with timeout

```rust
// Fetch tasks and comments in parallel
let (tasks, comments) = tokio::join!(
    fetch_assigned_tasks(user_id),
    fetch_assigned_comments(user_id)
);
```

**Rationale**:
- Minimizes load time
- Graceful degradation if one source fails
- Tasks are more critical - show even if comments fail

### Decision 4: Sorting Strategy
**Chosen**: Sort by updated_at descending across both types

**Rationale**:
- Most recent activity first regardless of type
- Consistent with existing task sorting
- Comments and tasks updated at similar times appear together

### Decision 5: Navigation Behavior
**Chosen**: Selecting comment navigates to parent task with comment thread opened

**Rationale**:
- Context is essential for comments
- Consistent with existing comment interaction model
- Leverages existing thread view navigation

## Risks / Trade-offs

**[Performance] Multiple API calls** → Mitigation: Parallel fetch, caching with 5-minute TTL, pagination for large lists

**[API Rate Limits] Fetching comments across all lists** → Mitigation: Use existing list cache, batch requests, respect rate limits

**[Complexity] Heterogeneous list rendering** → Mitigation: Clear visual distinction, enum pattern for type safety

**[Cache Invalidation] Comments may be added/updated independently** → Mitigation: Short cache TTL, manual refresh with 'r'

**[API Compatibility] assigned_commenters field may vary** → Mitigation: Flexible deserializers, graceful fallback if field missing

## Migration Plan

Not applicable - this is a new feature with no data migration required.

**Deployment strategy:**
1. Add `AssignedItem` model and fetch logic
2. Implement unified view rendering
3. Add filtering capability
4. Update cache layer
5. Test with real ClickUp workspace

**Rollback**: Feature flag not required - can disable by not showing comments in assigned view (tasks-only fallback)

## Open Questions

1. Should the filter state (all/tasks/comments) persist across sessions?
2. Should assigned comments count separately in the badge (e.g., "5📝2")?
3. Should we prefetch parent task data when fetching assigned comments, or fetch on-demand?
