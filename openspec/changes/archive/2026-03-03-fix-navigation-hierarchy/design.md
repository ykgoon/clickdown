## Context

The ClickDown TUI uses a hierarchical navigation model: Workspaces → Spaces → Folders → Lists → Tasks. Each level is displayed in the sidebar with items populated from cached API responses. The application tracks context IDs (`current_workspace_id`, `current_space_id`, `current_folder_id`, `current_list_id`) for URL generation and session restore.

**Current State:**
- Navigation down the hierarchy works correctly via `navigate_into()`
- Navigation up via `navigate_back()` only changes the `screen` enum and clears child context IDs
- Sidebar items are NOT repopulated when going back, leaving stale data displayed
- Selection state is lost when async loaders populate sidebar items
- Users become stuck because they cannot select different items at parent levels

**Constraints:**
- Must work with existing data caching (workspaces, spaces, folders, lists already cached in memory)
- Must preserve existing session restore functionality
- No API changes required - all data is already fetched
- Must maintain vim-style keyboard navigation paradigm

## Goals / Non-Goals

**Goals:**
- Users can navigate up and down the hierarchy seamlessly using Enter (drill down) and Esc (go back)
- Selection is preserved at each level - returning to a parent level shows the previously selected item highlighted
- Sidebar always displays items matching the current screen
- Code changes are minimal and focused on navigation logic

**Non-Goals:**
- Adding breadcrumb navigation UI (could be future enhancement)
- Changing the sidebar to show a tree view instead of flat lists
- Adding refresh/reload functionality for stale data
- Modifying session restore behavior (though it may benefit indirectly)

## Decisions

### Decision 1: Repopulate Sidebar in `navigate_back()`

**Approach:** When navigating back, repopulate the sidebar from cached data rather than reloading from API.

**Rationale:**
- Data is already cached in memory (`self.workspaces`, `self.spaces`, etc.)
- Provides instant navigation without network latency
- Consistent with existing architecture where data is fetched on drill-down only
- Simpler than implementing a cache invalidation strategy

**Alternatives Considered:**
- Reload from API on back navigation: Would be slow and unreliable offline
- Keep separate "back stack" of navigation state: Over-engineered for this use case

### Decision 2: Use Context IDs for Selection Restoration

**Approach:** Use existing `current_*_id` fields to restore selection via `sidebar.select_by_id()`.

**Rationale:**
- Context IDs are already tracked and maintained correctly during drill-down
- `select_by_id()` is more robust than storing indices (indices change when lists are reordered)
- Minimal code changes - leverages existing infrastructure
- Works seamlessly with session restore which already uses these IDs

**Alternatives Considered:**
- Store selected index at each level: Fragile if list order changes
- Store full selected item objects: Risk of data staleness, more memory

### Decision 3: Fix Async Handlers to Restore Selection

**Approach:** After populating sidebar items in async message handlers, check if a context ID exists and restore selection to that item. If no context ID exists, select the first item as fallback.

**Rationale:**
- Handles both navigation scenarios (drill-down and session restore)
- Maintains backward compatibility with existing behavior
- Ensures selection is always valid after data load

**Implementation Pattern:**
```rust
*self.sidebar.items_mut() = self.spaces.iter()...;
if let Some(ref space_id) = self.current_space_id {
    if self.sidebar.select_by_id(space_id) {
        // Selection restored
    } else {
        self.sidebar.select_first();
    }
} else {
    self.sidebar.select_first();
}
```

## Risks / Trade-offs

**[Stale Data]** Cached data may become stale if user makes changes in ClickUp web UI.
→ Mitigation: Future enhancement could add manual refresh or periodic background sync. Not in scope for this fix.

**[Memory Usage]** Keeping all hierarchy levels in memory could be significant for users with many workspaces/spaces.
→ Mitigation: Current users haven't reported memory issues. Can add LRU eviction later if needed.

**[Selection Restoration Fails]** If an item was deleted on the server, `select_by_id()` will fail and we fall back to `select_first()`.
→ Mitigation: This is acceptable behavior - user sees the list with first item selected, can manually navigate.

**[Complexity in Async Handlers]** Adding selection logic to each handler increases code duplication.
→ Mitigation: Could extract to helper method in future refactor. For now, clarity over abstraction.

## Migration Plan

This is a client-side bug fix with no deployment or migration requirements:

1. Build and run: `cargo run`
2. Test navigation flow:
   - Select workspace → Enter
   - Select space → Enter  
   - Select folder → Enter
   - Select list → Enter
   - Press Esc × 4 to return to workspaces
   - Verify sidebar shows correct items at each level
   - Verify selection is restored at each level
3. Test session restore still works after quitting and restarting

**Rollback:** Simple git revert - no database migrations, no API changes.

## Open Questions

None - this is a straightforward bug fix with clear implementation path.
