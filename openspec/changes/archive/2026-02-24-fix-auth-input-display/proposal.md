## Why

Users cannot see their API token input on the authentication screen - typing produces no visible feedback, and paste operations (Ctrl+V, Ctrl+Shift+V) incorrectly trigger the quit confirmation dialog instead of pasting the token. This creates a confusing user experience where users cannot verify their token was received before connecting.

## What Changes

- **Token input display**: Show first few characters of the token (unmasked or partially masked) so users can verify input is being received
- **Paste support**: Properly handle Ctrl+V and Ctrl+Shift+V paste events to insert clipboard content into the token field
- **Quit shortcut**: Restrict quit confirmation to only Ctrl+Q (not Ctrl+Shift+V or other modifier combinations)
- **Visual feedback**: Add cursor position indicator and clearer input state display

## Capabilities

### New Capabilities

- `auth-input`: Token input handling with visible feedback, paste support, and proper keyboard shortcut handling

### Modified Capabilities

- (none)

## Impact

- **Code**: `src/tui/widgets/auth.rs` (input display, paste handling), `src/tui/input.rs` (keyboard event handling), `src/tui/app.rs` (quit shortcut logic)
- **Dependencies**: crossterm event handling for paste detection
- **Systems**: Authentication flow, keyboard input processing
