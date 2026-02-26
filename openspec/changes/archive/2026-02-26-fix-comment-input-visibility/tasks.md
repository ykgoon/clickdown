## 1. Update Key Handlers for Immediate Form Visibility

- [x] 1.1 Modify 'n' key handler in `src/tui/app.rs` to set `comment_editing_index = Some(usize::MAX)` instead of `None`
- [x] 1.2 Modify 'r' key handler in `src/tui/app.rs` to set `comment_editing_index = Some(usize::MAX)` instead of `None`
- [x] 1.3 Add comment explaining sentinel value pattern: "usize::MAX indicates new comment/reply mode"

## 2. Update Save Logic to Handle Sentinel Value

- [x] 2.2 Modify Ctrl+S handler to check if `editing_index == Some(usize::MAX)` for new comment creation
- [x] 2.3 Ensure new comment flow determines `parent_id` from `comment_view_mode` (None for TopLevel, Some for InThread)
- [x] 2.4 Verify existing comment edit flow (when `editing_index != Some(usize::MAX)`) remains unchanged

## 3. Verify Cancel and Edge Case Handling

- [x] 3.1 Verify Esc key handler correctly clears state for both new and edit modes
- [x] 3.2 Audit all uses of `comment_editing_index` to ensure sentinel value is handled safely
- [ ] 3.3 Test that input form appears immediately after pressing 'n' in top-level view
- [ ] 3.4 Test that input form appears immediately after pressing 'r' in thread view

## 4. Manual Testing

- [ ] 4.1 Test new comment creation with real ClickUp API
- [ ] 4.2 Test reply creation with real ClickUp API
- [ ] 4.3 Test editing existing comments still works correctly
- [ ] 4.4 Test cancel behavior (Esc key) for both new and edit modes
- [ ] 4.5 Verify status messages display correctly throughout the flow
