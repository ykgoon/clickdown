## Context

The comment threading UI infrastructure is in place with `CommentViewMode` enum supporting both `TopLevel` and `InThread` views. The `create_comment` method already accepts a `parent_id` parameter and routes to the appropriate API endpoint. However, pressing 'r' in thread view only displays a status message without actually opening the reply input form, making it impossible for users to create replies.

**Current State:**
- `KeyCode::Char('r')` handler exists but only sets `self.status` message
- Input form activation logic exists for 'n' (new comment) but not triggered for 'r'
- `create_comment` method correctly handles `parent_id` for replies
- API endpoints exist for both top-level comments and replies

**Constraints:**
- Must maintain compatibility with existing comment threading navigation
- Should reuse existing comment input form UI
- Must work within Elm architecture pattern (Message-based state updates)

## Goals / Non-Goals

**Goals:**
- Enable 'r' key to open reply input form in thread view
- Properly set `parent_id` when creating replies
- Provide clear visual feedback during reply creation
- Maintain existing navigation and view mode behavior
- Add input validation for empty replies

**Non-Goals:**
- Do not change the comment data model or API structure
- Do not modify top-level comment creation ('n' key)
- Do not change thread navigation behavior (Enter/Esc)
- Do not add rich text editing (markdown only)

## Decisions

### Decision 1: Reply Form Activation Pattern

**Choice:** Reuse existing comment input form state variables (`comment_new_text`, `comment_editing_index`) with view mode context

**Rationale:**
- The existing form infrastructure already supports multi-line input with Ctrl+S save
- Adding separate reply form state would duplicate logic unnecessarily
- The `comment_view_mode` already provides context for determining `parent_id`

**Alternatives Considered:**
- Create separate `reply_new_text` state variable → Adds complexity without benefit
- Use a modal dialog for replies → Breaks existing UX pattern, more complex UI changes

### Decision 2: Parent ID Determination

**Choice:** Determine `parent_id` at save time based on current `comment_view_mode`

**Rationale:**
- Consistent with existing implementation in 'n' key handler
- Allows same form logic to work for both top-level and reply creation
- View mode is already tracked reliably in state

**Implementation:**
```rust
let parent_id = match &self.comment_view_mode {
    CommentViewMode::InThread { parent_comment_id, .. } => {
        Some(parent_comment_id.clone())
    }
    CommentViewMode::TopLevel => None,
};
```

### Decision 3: Status Message and Feedback

**Choice:** Show contextual status message with author name when opening reply form

**Rationale:**
- Provides clear confirmation that reply is for the correct thread
- Consistent with existing UX patterns in task editing
- Helps avoid confusion between 'n' (new comment) and 'r' (reply)

**Implementation:**
```rust
self.status = format!("Replying to {} (Ctrl+S save, Esc cancel)", author);
```

## Risks / Trade-offs

**[State Management Complexity]** → The shared form state (`comment_new_text`) is used for both new comments and replies. If a user switches view modes while editing, the context could become confused.

**Mitigation:** Clear `comment_new_text` when changing view modes or exiting thread view.

**[API Compatibility]** → ClickUp API may have specific requirements for `parent_id` field that differ from our current implementation.

**Mitigation:** Test with real API calls, add error handling for API-specific validation errors.

**[Form Focus Management]** → Need to ensure focus is properly set to the input form when 'r' is pressed, similar to 'n' key behavior.

**Mitigation:** Follow exact same pattern as 'n' key handler for form activation.

**[Validation Edge Cases]** → Empty reply validation must work correctly for both top-level comments and replies.

**Mitigation:** Reuse existing validation logic that checks `!self.comment_new_text.is_empty()`.

## Migration Plan

Not applicable - this is a bug fix/enhancement to existing functionality with no data migration or deployment changes required.

## Open Questions

None - the implementation path is clear based on existing patterns.
