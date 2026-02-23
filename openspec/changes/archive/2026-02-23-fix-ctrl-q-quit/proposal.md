## Why

Users expect `ctrl-q` to quit the application (standard desktop convention), but the current implementation only captures `ctrl+q` when it produces a character event, missing cases where the keyboard layout or system intercepts the character event.

## What Changes

- **Keyboard event handling**: Expand the subscription to capture both `Character` and `Named` keyboard events for `ctrl-q`
- **Quit confirmation**: Add optional confirmation dialog for `ctrl-q` (configurable)
- **Graceful shutdown**: Ensure proper cleanup before application exit

## Capabilities

### New Capabilities

- `keyboard-shortcuts`: Centralized keyboard shortcut handling system for all global shortcuts including `ctrl-q`

### Modified Capabilities

- `application-lifecycle`: Enhanced shutdown handling to support graceful exit from keyboard shortcuts

## Impact

- **Code**: `src/app.rs` subscription method, potentially refactored to new keyboard shortcut module
- **UI**: May add confirmation dialog for quit action
- **Dependencies**: No new dependencies required
- **Breaking**: None - this is a bug fix that makes existing functionality work as expected
