## Context

The `handle_input()` method in `src/tui/app.rs` processes all key events through a linear gauntlet of checks. Currently, the single-key `u` shortcut for URL copy (line ~1582) fires unconditionally — it has no guard for text input state. Other modal inputs (`url_input_open`, `status_picker_open`, `assignee_picker_open`, `task_creating`, `comment_editing_index`) each have their own early-return block, but `u` sits between them and the screen-specific handlers, catching all `u` keystrokes that slip past the explicit modal guards.

Text input in this app is handled in three places:
1. **Auth screen** (`update_auth`) — `auth_state.add_char(c)`
2. **Task creation** (`update_task_detail`) — `task_name_input.push(c)`, `task_description_input.push(c)` with `task_creating` flag
3. **Comment editing** (`update_task_detail`) — `comment_new_text.push(c)` with `comment_editing_index.is_some() || !comment_new_text.is_empty()`

Only `url_input_open` and `status_picker_open` have guards before the `u` handler. Comment editing and task creation do not.

## Goals / Non-Goals

**Goals:**
- Letter `u` types into text fields (comments, task creation, auth, URL dialog)
- Letter `u` still copies URL when not typing
- Single method `is_text_input_active()` centralizes the check
- Future text input modes automatically covered

**Non-Goals:**
- Not changing other shortcuts (`n`, `e`, `d`, `a`, `s`) — they already have screen-level guards
- Not changing the `g u` chord — it works correctly
- Not adding new UI/UX features — this is a bug fix

## Decisions

### Decision 1: Inverted guard — `is_text_input_active()` before global shortcuts

Place a single check at the top of the key handling gauntlet:

```
if self.is_text_input_active() {
    self.handle_text_input(key);
    return;
}
// Now safe for 'u' and other global shortcuts
```

**Why not add guards to `u` itself?** Adding `&& !self.comment_editing_index.is_some() && self.comment_new_text.is_empty() && !self.task_creating && ...` creates a fragile expression that must be updated every time a new text input mode is added. One centralized method is easier to maintain and test.

**Why not move `u` lower?** Moving `u` after screen handlers would still miss text input because comment editing lives inside `update_task_detail()` which is called after `u` already returned.

### Decision 2: `is_text_input_active()` covers all input states

```rust
fn is_text_input_active(&self) -> bool {
    self.url_input_open
        || self.status_picker_open
        || self.assignee_picker_open
        || self.task_creating
        || self.comment_editing_index.is_some()
        || !self.comment_new_text.is_empty()
}
```

This is conservative — it also guards modal pickers that don't do text input. That's fine: those modals already have their own handlers that return early, so this just adds an extra safety layer.

### Decision 3: `handle_text_input()` delegates to existing handlers

```rust
fn handle_text_input(&mut self, key: KeyEvent) {
    if self.url_input_open {
        self.handle_url_input(key);
    } else if self.status_picker_open {
        self.handle_status_picker_input(key);
    } else if self.assignee_picker_open {
        self.handle_assignee_picker_input(key);
    } else if self.task_creating {
        self.handle_task_creation_input(key);
    } else if self.comment_editing_index.is_some() || !self.comment_new_text.is_empty() {
        self.handle_comment_input(key);
    }
}
```

**Refactoring note:** The existing code has these handlers inline inside `update_task_detail()`. Extracting them into dedicated methods (`handle_task_creation_input`, `handle_comment_input`) improves testability and keeps `handle_input()` clean. This is a small refactor, not a behavioral change.

### Decision 4: Move dialog guard before text input guard

The dialog confirmation handler (Enter/Esc for confirm-quit, confirm-delete) should stay before text input, because dialogs are truly modal and should intercept all keys. Current order is already correct for this.

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| Refactoring inline handlers into methods could introduce subtle behavior changes | Extract without modifying logic — pure method extraction, same code paths |
| `is_text_input_active()` might miss a future text input mode | Document the method clearly; code review checklist should include "did you update this?" |
| Slightly more indirection in key handling | Negligible performance impact; clarity gain outweighs one extra method call |
