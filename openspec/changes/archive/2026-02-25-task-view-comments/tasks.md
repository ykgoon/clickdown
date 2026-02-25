## 1. Data Models and API

- [x] 1.1 Create `Comment` model in `src/models/comment.rs` with fields: id, text, text_preview, commenter (User), created_at, updated_at, assigned_commenter, assigned_by
- [x] 1.2 Add `Comment` model exports to `src/models/mod.rs`
- [x] 1.3 Add `get_task_comments(task_id)`, `create_comment(task_id, text)`, `update_comment(comment_id, text)` methods to `ClickUpApi` trait in `src/api/client_trait.rs`
- [x] 1.4 Implement comment API methods in `ClickUpClient` in `src/api/client.rs` with proper endpoint URLs
- [x] 1.5 Implement mock comment API methods in `MockClickUpClient` in `src/api/mock_client.rs` for testing
- [x] 1.6 Add comment request/response types if needed (similar to CreateTaskRequest pattern)

## 2. Database Caching

- [x] 2.1 Create SQLite migration for `task_comments` table with columns: task_id TEXT, comment_id TEXT PRIMARY KEY, text TEXT, commenter_id INTEGER, created_at INTEGER, updated_at INTEGER, fetched_at INTEGER
- [x] 2.2 Add cache functions: `cache_comments`, `get_cached_comments`, `is_cache_valid` in `src/cache/mod.rs`
- [x] 2.3 Update cache module exports in `src/cache/mod.rs`

## 3. Application State and Messages

- [x] 3.1 Add comment-related messages to `Message` enum in `src/app.rs`: `CommentsLoaded`, `CommentCreated`, `CommentUpdated`, `CommentCreateError`, `CommentUpdateError`
- [x] 3.2 Add comment state fields to `TuiApp` struct: comments list, selected comment index, comment input form state, edit mode flag
- [x] 3.3 Implement message handlers for comment operations in `TuiApp::update()` method

## 4. TUI Widgets

- [x] 4.1 Create `CommentList` widget in `src/tui/widgets/` with scrollable comment rendering and text wrapping
- [x] 4.2 Create `CommentForm` widget for creating/editing comments with multi-line text input
- [x] 4.3 Add widget exports to `src/tui/widgets/mod.rs`
- [x] 4.4 Implement text wrapping using ratatui's `Paragraph` widget with `Wrap { trim: true }`

## 5. Task Detail Integration

- [x] 5.1 Modify task detail layout in `src/tui/widgets/task_detail.rs` to include comments section below description
- [x] 5.2 Add comments panel rendering with proper height allocation
- [x] 5.3 Implement focus switching between task form and comments panel with Tab key
- [x] 5.4 Add keyboard navigation (j/k) for comments list when focus is on comments section
- [x] 5.5 Add status bar hints for comment actions based on current focus and selection

## 6. Comment Actions

- [x] 6.1 Implement 'n' key handler for new comment form in task detail view
- [x] 6.2 Implement 'e' key handler for editing selected comment (with authorship check)
- [x] 6.3 Implement Ctrl+s save handler for comment form (create and update)
- [x] 6.4 Implement Esc cancel handler with confirmation dialog for discarding changes
- [x] 6.5 Add visual selection highlight for focused comment

## 7. Loading and Error States

- [x] 7.1 Add "Loading comments..." indicator when fetching comments
- [x] 7.2 Add "No comments yet" empty state with helpful hint
- [x] 7.3 Add error display for failed comment operations with retry option
- [x] 7.4 Add "Saving..." indicator during comment create/update API calls

## 8. Testing

- [x] 8.1 Add unit tests for Comment model serialization in `src/models/comment.rs`
- [x] 8.2 Add integration tests for comment API methods in `tests/app_test.rs` using MockClickUpClient
- [x] 8.3 Add tests for comment caching logic in `tests/app_test.rs`
- [x] 8.4 Add fixture helpers for creating test comments in `tests/fixtures.rs`

## 9. Documentation and Polish

- [x] 9.1 Update keyboard shortcuts help overlay (?) to include comment shortcuts
- [x] 9.2 Add inline code comments for complex comment logic
- [x] 9.3 Test text wrapping with various comment lengths and terminal widths
- [x] 9.4 Verify markdown rendering in comments matches document rendering
