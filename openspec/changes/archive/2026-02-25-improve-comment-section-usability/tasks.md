## 1. Layout Updates

- [x] 1.1 Update `src/tui/layout.rs` to add a new layout splitting function that supports percentage-based constraints for task detail view with 3:7 ratio
- [x] 1.2 Add scroll state tracking structure to support independent scroll offsets for multiple panels
- [x] 1.3 Add scroll indicator rendering function to display visual scroll bars on scrollable panels

## 2. Task Detail View Refactoring

- [x] 2.1 Refactor `src/tui/widgets/task_detail.rs` to use the new 3:7 layout ratio between description and comment panels
- [x] 2.2 Add scroll state management for the description panel
- [x] 2.3 Implement independent scrolling for the description panel when content overflows
- [x] 2.4 Add scroll indicator to the description panel

## 3. Comment Panel Scrolling

- [x] 3.1 Update `src/tui/widgets/comments.rs` to add scroll offset state tracking
- [x] 3.2 Implement auto-scroll logic that adjusts scroll position when selected comment moves outside visible area
- [x] 3.3 Add scroll indicator rendering to the comment panel
- [x] 3.4 Update `render_comment_list` to render only visible comments based on scroll offset
- [x] 3.5 Implement smooth scroll behavior (or predictable scroll jumps if smooth scrolling not supported)

## 4. Input Handling Integration

- [x] 4.1 Update keyboard navigation handler in `src/tui/input.rs` to trigger auto-scroll when navigating comments with j/k
- [x] 4.2 Ensure scroll position updates when comment selection changes
- [x] 4.3 Add scroll wheel support for comment panel (if not already implemented)
- [x] 4.4 Test focus switching between description and comment panels preserves scroll state

## 5. Edge Cases and Error Handling

- [x] 5.1 Handle terminal resize events to recalculate panel heights and scroll positions
- [x] 5.2 Implement graceful degradation for terminals below minimum height (24 rows)
- [x] 5.3 Test with empty comments, single comment, and many comments scenarios
- [x] 5.4 Test with very long comments that require significant vertical space

## 6. Testing and Verification

- [x] 6.1 Add unit tests for scroll offset calculations in `src/tui/widgets/comments.rs`
- [x] 6.2 Add unit tests for auto-scroll boundary conditions
- [x] 6.3 Manually test layout on various terminal sizes (80x24, 120x40, 160x60)
- [x] 6.4 Verify 3:7 ratio is maintained across different terminal sizes
- [x] 6.5 Test auto-scroll behavior with rapid j/k navigation
- [x] 6.6 Run existing tests to ensure no regressions in task detail or comment functionality
