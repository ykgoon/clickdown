## 1. UI Event Handler Fix

- [x] 1.1 Modify 'r' key handler in `src/tui/app.rs` to activate reply input form (set `comment_editing_index = None`, clear `comment_new_text`)
- [x] 1.2 Add status message showing "Replying to {author} (Ctrl+S save, Esc cancel)" when form opens
- [x] 1.3 Ensure focus moves to input form when 'r' is pressed in thread view
- [x] 1.4 Add guard to prevent 'r' from opening form in top-level view (show hint message instead)

## 2. Reply Input Validation

- [x] 2.1 Add validation to check reply text is not empty or whitespace-only before submission
- [x] 2.2 Show error message "Reply cannot be empty" when validation fails
- [x] 2.3 Keep input form open when validation fails to allow editing

## 3. Reply Creation Logic

- [x] 3.1 Verify `create_comment` method correctly determines `parent_id` from `comment_view_mode`
- [x] 3.2 Ensure `CreateCommentRequest` includes `parent_id` field for replies
- [x] 3.3 Route reply creation to `create_comment_reply` API method when `parent_id` is Some
- [x] 3.4 Route top-level comment creation to `create_comment` API method when `parent_id` is None

## 4. State Management and Feedback

- [x] 4.1 Set `loading = true` when reply creation starts
- [x] 4.2 Show "Saving reply..." status during API call
- [x] 4.3 Insert new reply into `self.comments` vector on success
- [x] 4.4 Clear `comment_new_text` and reset `comment_editing_index` after successful save
- [x] 4.5 Show "Reply added" status message on success
- [x] 4.6 Set `loading = false` after reply creation completes (success or error)

## 5. Error Handling

- [x] 5.1 Handle network errors during reply creation with descriptive error message
- [x] 5.2 Handle authentication errors and prompt re-authentication if needed
- [x] 5.3 Handle rate limit errors with appropriate messaging
- [x] 5.4 Log all reply creation errors via `tracing::error!`
- [x] 5.5 Keep input form open on error to allow retry

## 6. Form Cancellation

- [x] 6.1 Handle Esc key to cancel reply creation
- [x] 6.2 Clear `comment_new_text` when reply is cancelled
- [x] 6.3 Show "Reply cancelled" status message
- [x] 6.4 Keep thread view open after cancellation (don't exit to top-level)

## 7. Testing

- [x] 7.1 Add unit test for reply form activation with 'r' key
- [x] 7.2 Add unit test for empty reply validation
- [x] 7.3 Add unit test for parent_id assignment in thread view
- [x] 7.4 Add integration test for reply creation using MockClickUpClient
- [x] 7.5 Add test for error handling in reply creation
- [x] 7.6 Manually test reply creation with real ClickUp API

## 8. Documentation

- [x] 8.1 Update help text to mention 'r' key for replying in thread view
- [x] 8.2 Verify status bar hints show correct key bindings in thread view
