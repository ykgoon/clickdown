## Why

When users press 'n' (new comment) or 'r' (reply), the input form does not appear immediately—it only shows after typing the first character. This creates a confusing UX where users press a key, see a status message, but no visible input area appears, making them uncertain whether the action worked.

## What Changes

- **Input form visibility**: The comment input form will appear immediately when 'n' or 'r' is pressed, rather than waiting for the first character to be typed
- **State management**: The `comment_editing_index` will be set to a sentinel value to indicate "new comment/reply mode" and trigger form rendering
- **Visual feedback**: Users will see the input form with focus immediately after pressing 'n' or 'r', confirming the action was recognized

## Capabilities

### New Capabilities
<!-- No new capabilities - this is a bug fix to existing comment functionality -->

### Modified Capabilities
<!-- No spec-level behavior changes - the requirements for comments remain the same, only the implementation timing changes -->

## Impact

- **Modified files**: `src/tui/app.rs` (key handlers for 'n' and 'r'), potentially `src/tui/widgets/comments.rs` (if visibility logic needs adjustment)
- **No API changes**: The comment creation API flow remains unchanged
- **No breaking changes**: Existing functionality is preserved, only the timing of form visibility improves
- **Minimal scope**: This is a focused UX bug fix with no ripple effects to other systems
