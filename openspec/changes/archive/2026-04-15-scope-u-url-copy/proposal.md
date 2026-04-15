## Why

The single-key `u` shortcut for copying element URLs fires globally, blocking the letter `u` from text input in comments, task creation forms, URL input dialogs, and any other text entry fields. Unlike other single-key shortcuts (`n`, `e`, `d`, `a`, `s`) which are scoped to specific screens or contexts, `u` has no guard — it intercepts the key before any text input handler can receive it.

## What Changes

- Scope the single-key `u` URL copy shortcut to only fire when **no text input is active**
- Define a clear "text input active" state that covers all current and future text entry modes
- Keep `g u` chord (URL navigation dialog) working as-is — it already has proper modal guards

## Capabilities

### New Capabilities

- `text-input-guards`: Centralized "is typing?" check that global shortcuts respect. Provides a single method (`is_text_input_active()`) answering whether any text input field is focused, so shortcuts below it are safe.

### Modified Capabilities

- `keyboard-shortcuts`: The `u` shortcut requirement needs a guard clause. Add a scenario: "S does not interfere with typing" equivalent for `u` — when text input is active, `u` types the letter, doesn't copy URL.
- `element-url-copying`: The keyboard shortcut requirement for `u` needs scoping. Add requirement that `u` only fires outside text input contexts (comments, task creation, URL dialog, etc.).

## Impact

- **`src/tui/app.rs`**: `handle_input()` method — restructure to check text input state before global shortcuts
- **`src/tui/app.rs`**: New `is_text_input_active()` method on `TuiApp`
- **`openspec/specs/keyboard-shortcuts/spec.md`**: Add scenario for `u` not interfering with typing
- **`openspec/specs/element-url-copying/spec.md`**: Add text-input-guard requirement
- **`src/tui/widgets/help.rs`**: No changes needed (help text already documents `u` as global, behavior becomes more precise)
