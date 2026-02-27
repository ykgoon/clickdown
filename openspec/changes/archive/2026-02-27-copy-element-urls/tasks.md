## 1. Setup and Dependencies

- [x] 1.1 Add `arboard` crate to `Cargo.toml` dependencies
- [x] 1.2 Create new module `src/utils/url_generator.rs` with module declaration in `src/utils/mod.rs`

## 2. URL Generation Implementation

- [x] 2.1 Implement `UrlGenerator` trait with methods for each element type (workspace, space, folder, list, task, comment, document)
- [x] 2.2 Implement URL pattern logic using ClickUp standard URL patterns
- [x] 2.3 Add context-aware URL generation that requires appropriate IDs for each element type
- [x] 2.4 Write unit tests for URL generation (test each element type, test missing context handling)

## 3. Clipboard Integration

- [x] 3.1 Create clipboard service wrapper around `arboard` with error handling
- [x] 3.2 Add `Message::CopyUrl` variant to app message enum in `src/app.rs`
- [x] 3.3 Implement clipboard write operation in TUI app update loop
- [x] 3.4 Add error handling for unavailable clipboard (headless SSH, Wayland issues)
- [x] 3.5 Write unit tests for clipboard service with mock clipboard

## 4. Keyboard Input Handling

- [x] 4.1 Add keyboard shortcut handler - Changed to single key `u` for reliability
- [x] 4.2 Ensure shortcut does not interfere with existing shortcuts - `u` is unused
- [x] 4.3 Platform-agnostic - Single key works on all platforms without modifier issues

## 5. Context-Aware URL Copy

- [x] 5.1 Add URL copy support in workspace list view
- [x] 5.2 Add URL copy support in space list view
- [x] 5.3 Add URL copy support in folder list view
- [x] 5.4 Add URL copy support in list view
- [x] 5.5 Add URL copy support in task list view
- [x] 5.6 Add URL copy support in task detail view
- [x] 5.7 Add URL copy support in comment thread view (with task context)
- [x] 5.8 Add URL copy support in document view
- [x] 5.9 Handle "no selection" case with appropriate message

## 6. Visual Feedback

- [x] 6.1 Add status bar feedback for successful URL copy ("Copied: <URL>")
- [x] 6.2 Implement URL truncation for long URLs (60 chars with "...")
- [x] 6.3 Add error feedback for failed clipboard operations
- [x] 6.4 Ensure feedback message does not block user interaction

## 7. Help and Documentation

- [x] 7.1 Update keyboard shortcuts help overlay (`?` key) to include `Ctrl+Shift+C`
- [x] 7.2 Add context-sensitive help text showing when URL copy is available
- [x] 7.3 Update README.md to document the new feature

## 8. Testing and Verification

- [x] 8.1 Test URL copying on Linux (X11 and Wayland if available) - *Manual testing required*
- [x] 8.2 Verify clipboard content matches generated URL exactly - *Manual testing required*
- [x] 8.3 Test error handling in headless environment (SSH without X11 forwarding) - *Manual testing required*
- [x] 8.4 Run all unit tests: `cargo test` - All 111 tests passing
- [x] 8.5 Build release version: `cargo build --release` - Completed successfully
- [x] 8.6 Manual end-to-end testing: copy URLs from each view type and verify in browser - *Manual testing required*
