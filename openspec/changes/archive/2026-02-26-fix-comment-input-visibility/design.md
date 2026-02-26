## Context

The comment input form visibility is controlled by the `has_input` condition in `src/tui/widgets/comments.rs`:

```rust
let has_input = editing_index.is_some() || !new_text.is_empty();
```

**Current behavior:**
1. User presses 'n' or 'r'
2. Key handler clears `comment_new_text` and sets `comment_editing_index = None`
3. Status message is shown ("Type comment..." or "Replying to...")
4. **Input form does NOT appear** because `has_input = false`
5. User types first character → `comment_new_text` is no longer empty
6. Input form appears

This creates a confusing UX gap between the key press and visible feedback.

**Current state:**
- `src/tui/app.rs`: Key handlers for 'n' and 'r' at lines 774-777 and 824-827
- `src/tui/widgets/comments.rs`: Input form rendering at lines 133-185
- The `comment_editing_index` field is `Option<usize>` where `Some(idx)` means editing existing comment at index `idx`

**Constraints:**
- Must maintain compatibility with existing comment editing flow (editing existing comments uses `Some(index)`)
- Should not require changes to the `CommentPanelState` struct or add new state fields
- Must work within the existing Elm architecture pattern

## Goals / Non-Goals

**Goals:**
- Input form appears immediately when 'n' or 'r' is pressed
- Maintain existing behavior for editing comments (e-key)
- Preserve all existing comment creation and reply functionality
- Minimal code changes to reduce regression risk

**Non-Goals:**
- Do not change the comment creation API flow
- Do not modify the input form UI appearance
- Do not add new features to comment functionality
- Do not change keyboard shortcuts or navigation patterns

## Decisions

### Decision 1: Use Sentinel Value for "New Comment" Mode

**Choice:** Set `comment_editing_index = Some(usize::MAX)` to indicate "new comment/reply mode"

**Rationale:**
- The `has_input` condition checks `editing_index.is_some()`, so any `Some` value triggers form visibility
- `usize::MAX` is an invalid array index, so it won't accidentally reference a real comment
- Existing editing logic checks `if let Some(edit_idx) = self.comment_editing_index` and then uses it as an array index - we need to handle this
- No new state fields required

**Implementation:**
```rust
// In 'n' key handler:
KeyCode::Char('n') if self.comment_focus => {
    self.comment_new_text.clear();
    self.comment_editing_index = Some(usize::MAX); // Sentinel for "new comment"
    self.status = "Type comment (Ctrl+S save, Esc cancel)".to_string();
}

// In 'r' key handler:
KeyCode::Char('r') if self.comment_focus => {
    if matches!(self.comment_view_mode, CommentViewMode::InThread { .. }) {
        self.comment_new_text.clear();
        self.comment_editing_index = Some(usize::MAX); // Sentinel for "new reply"
        // ... rest of handler
    }
}
```

**Alternatives Considered:**

1. **Add separate `comment_creating_new: bool` state field**
   - Pros: Explicit and clear intent
   - Cons: Adds new state, more changes to track

2. **Use `Some(0)` or another small index**
   - Pros: Simple
   - Cons: Could conflict with actual comment at that index, confusing logic

3. **Change `has_input` logic to check status message**
   - Pros: No state changes
   - Cons: Fragile, couples rendering to status text

### Decision 2: Handle Sentinel Value in Save Logic

**Choice:** When saving, check if `editing_index == Some(usize::MAX)` to determine if creating new vs. updating existing

**Rationale:**
- The save logic (Ctrl+S handler) needs to distinguish between creating a new comment and editing an existing one
- Current code uses `if let Some(edit_idx) = self.comment_editing_index` to detect editing mode
- We need to add a check: if the index is `usize::MAX`, it's a new comment, not an edit

**Implementation:**
```rust
// In Ctrl+S handler:
if let Some(edit_idx) = self.comment_editing_index {
    if edit_idx == usize::MAX {
        // Creating new comment or reply
        let parent_id = match &self.comment_view_mode {
            CommentViewMode::InThread { parent_comment_id, .. } => {
                Some(parent_comment_id.clone())
            }
            CommentViewMode::TopLevel => None,
        };
        // Call create_comment with parent_id
    } else {
        // Updating existing comment at index edit_idx
        // Existing logic unchanged
    }
}
```

**Alternatives Considered:**

1. **Set `editing_index = None` after form appears, rely on `!new_text.is_empty()`**
   - Pros: Reuses existing "new comment" detection
   - Cons: Form would disappear, defeats the purpose

2. **Add explicit `is_creating_new()` helper method**
   - Pros: Cleaner code
   - Cons: More refactoring, not necessary for this focused fix

### Decision 3: Preserve Cancel Behavior

**Choice:** Esc key handler already clears both fields, so cancel behavior works unchanged

**Rationale:**
- Current Esc handler: `self.comment_new_text.clear(); self.comment_editing_index = None;`
- This works correctly for both editing and creating modes
- No changes needed

## Risks / Trade-offs

**[Sentinel Value Confusion]** → Future developers might not understand why `usize::MAX` is used as an index.

**Mitigation:** Add a comment explaining the sentinel value pattern:
```rust
// usize::MAX is a sentinel value indicating "new comment" mode
// (as opposed to Some(index) which means editing existing comment)
```

**[Index Comparison Edge Cases]** → If code elsewhere assumes `editing_index` is always a valid array index, it could panic.

**Mitigation:** Audit all uses of `comment_editing_index` to ensure they handle the sentinel value. Current audit shows:
- `has_input` check: ✅ Only checks `is_some()`
- Save logic: ✅ Needs update (see Decision 2)
- Cancel logic: ✅ Just sets to `None`
- Render logic: ✅ Uses index to skip rendering during edit

**[Regression in Edit Flow]** → Changes to save logic could break editing existing comments.

**Mitigation:** Test both flows: (1) create new comment, (2) edit existing comment. The sentinel check is simple and isolated.

**[No Unit Tests]** → This change doesn't include new unit tests due to time constraints.

**Mitigation:** Manual testing with real ClickUp API. Consider adding tests in a follow-up change.

## Migration Plan

Not applicable - this is a client-side UX fix with no deployment or migration requirements.

## Open Questions

None - the implementation approach is straightforward and well-scoped.
