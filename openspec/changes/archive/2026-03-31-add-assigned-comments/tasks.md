## 1. Data Models

- [x] 1.1 Create `AssignedComment` struct with comment, parent task reference, and assigned_at fields
- [x] 1.2 Create `AssignedItem` enum with Task and AssignedComment variants
- [x] 1.3 Add `AssignedComment` deserializer with flexible handling for assigned_commenters field
- [x] 1.4 Update comment model to include assigned_commenters field if not present

## 2. API Layer

- [x] 2.1 Add `get_assigned_comments(user_id)` method to ClickUpApi trait
- [x] 2.2 Implement assigned comments fetch in ClickUpClient (fetch from all lists, filter by assigned_commenters)
- [x] 2.3 Add mock implementation for assigned comments in MockClickUpClient
- [x] 2.4 Add CLI debug command `debug assigned-comments` for testing (parallel to `debug assigned-tasks`)

## 3. Cache Layer

- [x] 3.1 Add SQLite table/schema for assigned_comments cache
- [x] 3.2 Implement cache write for assigned comments with timestamp
- [x] 3.3 Implement cache read with TTL check (5-minute expiry)
- [x] 3.4 Add cache invalidation method for assigned comments

## 4. Application Logic

- [x] 4.1 Add `fetch_assigned_items()` method combining tasks and comments in parallel
- [x] 4.2 Implement sorting logic for AssignedItem list (by updated_at descending)
- [x] 4.3 Add filter state management (All, Tasks Only, Comments Only)
- [x] 4.4 Update Message enum with `AssignedItemsLoaded`, `AssignedItemSelected`, `AssignedItemsFilterChanged`

## 5. UI - Assigned View Widget

- [x] 5.1 Create `AssignedView` widget in `src/tui/widgets/assigned_view.rs`
- [x] 5.2 Implement rendering for task items (existing task row style)
- [x] 5.3 Implement rendering for comment items (comment icon, preview, parent task name)
- [x] 5.4 Add visual distinction between task and comment items (icons, colors)
- [x] 5.5 Implement filter toggle UI (tabs or segmented control at top)
- [x] 5.6 Implement empty state message for no assigned items
- [x] 5.7 Implement loading state indicator

## 6. Navigation Integration

- [x] 6.1 Update sidebar to show combined count badge for "Assigned to Me" item
- [x] 6.2 Add keyboard shortcut for filter toggle (e.g., 'f' or 't' for type filter)
- [x] 6.3 Handle comment selection - navigate to parent task and open comment thread
- [x] 6.4 Handle task selection - existing task detail behavior
- [x] 6.5 Add refresh handler ('r' key) for assigned items view

## 7. Testing

- [x] 7.1 Add unit tests for AssignedComment deserializer (edge cases: null, array, missing field)
- [x] 7.2 Add unit tests for AssignedItem sorting logic
- [x] 7.3 Add integration test for assigned items fetch with mock client
- [x] 7.4 Add snapshot test for assigned view rendering (tasks + comments)
- [x] 7.5 Manual testing with real ClickUp workspace (verify assigned comments appear)

## 8. Documentation

- [x] 8.1 Update README.md with assigned comments feature description
- [x] 8.2 Update keyboard shortcuts help dialog with filter toggle shortcut
- [x] 8.3 Add AGENTS.md documentation for assigned items architecture
