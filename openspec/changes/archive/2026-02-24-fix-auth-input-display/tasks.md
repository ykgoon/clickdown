## 1. Dependencies and Setup

- [x] 1.1 Add `arboard` crate to `Cargo.toml` for clipboard access
- [x] 1.2 Verify dependency compiles on Linux target platform

## 2. Fix Quit Shortcut Detection

- [x] 2.1 Update `is_quit()` in `src/tui/input.rs` to use exact modifier match (`key.modifiers == KeyModifiers::CONTROL`)
- [x] 2.2 Test that Ctrl+Q triggers quit dialog
- [x] 2.3 Test that Ctrl+Shift+Q does NOT trigger quit dialog
- [x] 2.4 Test that Ctrl+Shift+V does NOT trigger quit dialog

## 3. Implement Paste Support

- [x] 3.1 Add clipboard import to `src/tui/app.rs`
- [x] 3.2 Add paste handling logic in `update_auth()` for Ctrl+V
- [x] 3.3 Add paste handling logic in `update_auth()` for Ctrl+Shift+V
- [x] 3.4 Handle clipboard errors gracefully (show "Paste failed" message)
- [x] 3.5 Test paste functionality with various token strings

## 4. Update Token Display

- [x] 4.1 Modify `render_auth()` in `src/tui/widgets/auth.rs` to show first 4 characters unmasked
- [x] 4.2 Show remaining characters as bullets (•)
- [x] 4.3 Handle edge case: token shorter than 4 characters shows all unmasked
- [x] 4.4 Add cursor position indicator in token input
- [x] 4.5 Test display with tokens of various lengths

## 5. Testing and Verification

- [x] 5.1 Test complete auth flow: type token, verify display, press Enter
- [x] 5.2 Test paste flow: copy token, paste with Ctrl+V, verify display
- [x] 5.3 Test paste with Ctrl+Shift+V
- [x] 5.4 Test quit dialog only appears with Ctrl+Q
- [x] 5.5 Test backspace deletes characters correctly
- [x] 5.6 Run `cargo build --release` and verify no compilation errors
- [x] 5.7 Run existing test suite to ensure no regressions
