## Context

The current comment panel in ClickDown displays all comments in a flat, chronological list. While this works for simple conversations, it becomes difficult to follow threaded discussions where multiple conversation branches exist. Users cannot easily distinguish between top-level comments and their replies, making it hard to track specific conversation threads.

The existing implementation (see `comment-panel-scrolling` spec) provides independent scrolling and navigation through comments, but lacks thread awareness. The comment model currently has no parent-child relationship tracking.

## Goals / Non-Goals

**Goals:**
- Display top-level comments initially in the comment panel
- Enable users to "enter" any top-level comment to view its thread (nested replies)
- Provide clear visual indication of current navigation context (top-level vs. inside thread)
- Support keyboard navigation for entering/exiting threads
- Maintain existing scrolling and navigation behavior within each view
- Add breadcrumb navigation showing current position

**Non-Goals:**
- Creating nested replies beyond 2 levels (only top-level and first-level replies)
- Real-time comment synchronization or webhooks
- Comment reactions or emoji support
- Resolving/unresolving comment threads
- Rich text editor for comment composition (markdown input remains)

## Decisions

### Decision: Two-level threading model
**Chosen approach:** Support exactly 2 levels - top-level comments and their replies (second-level).

**Rationale:**
- Matches common UX patterns (GitHub, GitLab, Linear)
- Keeps UI complexity manageable in terminal environment
- Avoids deeply nested indentation that would be hard to render in TUI
- Sufficient for most conversation structures

**Alternatives considered:**
- **Unlimited nesting:** Too complex for TUI, would require horizontal scrolling or severe text wrapping
- **Flat list with @mentions:** Loses thread structure, harder to follow conversations
- **Separate thread view:** Would require modal or full-screen view, breaking flow

### Decision: Parent ID field for thread relationships
**Chosen approach:** Add optional `parent_id: Option<String>` field to Comment model to track thread relationships.

**Rationale:**
- Minimal change to existing model
- Aligns with ClickUp API's likely structure (most APIs use parent_id for comments)
- Enables efficient filtering of top-level vs. threaded comments
- Database schema can index parent_id for fast lookups

**Implementation:**
```rust
pub struct Comment {
    pub id: String,
    pub parent_id: Option<String>,  // NEW: None = top-level, Some(parent_id) = reply
    // ... existing fields
}
```

**Alternatives considered:**
- **Thread ID grouping:** Each thread has unique ID, but requires additional field and logic
- **Separate Reply model:** Overcomplicates, comments and replies are semantically similar

### Decision: Navigation pattern - Enter to view, Esc to go back
**Chosen approach:** Use Enter key to "enter" a selected top-level comment thread, Esc to go back to top-level view.

**Rationale:**
- Consistent with existing navigation patterns (Enter to select, Esc to go back)
- Intuitive for users familiar with file explorers or email clients
- No new keyboard shortcuts to memorize

**Implementation:**
- When viewing top-level comments: Enter on selected comment → switch to thread view
- When viewing thread: Esc → return to top-level comments view
- Tab continues to switch focus between task form and comments

**Alternatives considered:**
- **Dedicated 't' key for thread:** Less discoverable, conflicts with potential 't' shortcuts
- **Double Enter:** Unconventional, may cause accidental thread entries

### Decision: Breadcrumb navigation indicator
**Chosen approach:** Show breadcrumb in comment panel title: "Comments > Comment by {username}"

**Rationale:**
- Clear indication of current position in navigation hierarchy
- Minimal screen space (single line in panel title)
- Follows established breadcrumb pattern from workspace navigation

**Implementation:**
```rust
// In comment panel state
enum CommentViewMode {
    TopLevel,
    InThread { parent_comment_id: String, parent_author: String },
}

// Panel title dynamically rendered:
let title = match view_mode {
    CommentViewMode::TopLevel => " Comments ",
    CommentViewMode::InThread { parent_author } => format!(" Comments > {} ", parent_author),
};
```

**Alternatives considered:**
- **Status bar message:** Less visible, status bar already crowded
- **Full header row:** Wastes vertical space in TUI
- **No indicator:** Users would get lost

### Decision: Action context switching
**Chosen approach:** "New comment" (n) always creates top-level comment. "Reply" action (r) creates nested reply in current thread view.

**Rationale:**
- Clear semantic distinction between starting new conversation vs. replying to thread
- Prevents accidental off-topic replies in active threads
- Status bar can show context-appropriate hints

**Implementation:**
- Top-level view: Status shows "'n' new comment, Enter view thread"
- Thread view: Status shows "'r' reply, Esc go back"
- Reply form shows "Replying to {author}..." context

**Alternatives considered:**
- **Single 'n' for both:** Ambiguous, user might accidentally reply to wrong thread
- **Modal choice:** Interrupts flow, adds extra keystroke

## Risks / Trade-offs

**[Risk] ClickUp API may not support parent_id** → Mitigation: Research ClickUp comment API; if no parent_id support, implement client-side threading using conversation_id or timestamp grouping

**[Risk] Two-level limit feels restrictive** → Mitigation: Can extend to flat list with @mentions as fallback; most conversations fit 2 levels

**[Risk] Screen space constraints in TUI** → Mitigation: Breadcrumb in title bar uses minimal space; thread view shows only relevant comments

**[Risk] Confusion about where new comments go** → Mitigation: Clear status bar messages and form labels ("New comment" vs "Reply to...")

**[Trade-off] No real-time updates** → Acceptable; users can manually refresh task to see new comments from others

**[Trade-off] No comment reactions** → Out of scope; can be added in future enhancement

## Migration Plan

1. **Phase 1: Model changes**
   - Add `parent_id` field to Comment model
   - Update API client to fetch/handle parent_id if available
   - Update cache schema to store parent_id

2. **Phase 2: State management**
   - Add `CommentViewMode` enum to track current view
   - Add thread navigation state to TUI app
   - Implement Enter/Esc handlers for thread navigation

3. **Phase 3: UI updates**
   - Update comment panel title to show breadcrumb
   - Modify comment list rendering to filter by parent_id
   - Add visual indicators for thread context

4. **Phase 4: Actions and forms**
   - Implement "Reply" action (r key) for threaded replies
   - Update "New comment" (n key) to always create top-level
   - Update status bar hints based on context

5. **Phase 5: Testing**
   - Manual testing with real ClickUp tasks with threaded comments
   - Verify navigation flow and breadcrumb display
   - Test edge cases (empty threads, deleted parent comments)

**Rollback strategy:** Feature flag via config option `comment_threading: bool` (default false) to disable if issues arise.

## Open Questions

1. Does ClickUp API provide parent-child relationships for comments? If not, how should we infer threads?
2. Should deleted parent comments hide their replies, or show replies as orphaned?
3. Should thread entry show all replies, or paginate if many replies exist?
