## 1. Data Model & Database Schema

- [x] 1.1 Create Notification model in `src/models/notification.rs` with fields: id, workspace_id, title, description, created_at, read_at
- [x] 1.2 Add custom deserializers for flexible timestamp handling (milliseconds, ISO 8601)
- [x] 1.3 Create notifications table migration in `src/cache/schema.rs`
- [x] 1.4 Implement NotificationCache in `src/cache/mod.rs` with CRUD operations
- [x] 1.5 Add cache cleanup method for old read notifications

## 2. API Integration

- [x] 2.1 Add `get_notifications(workspace_id)` method to `ClickUpApi` trait in `src/api/client_trait.rs`
- [x] 2.2 Implement notifications endpoint in `src/api/endpoints.rs`
- [x] 2.3 Implement notifications fetch in `ClickUpClient` in `src/api/client.rs`
- [x] 2.4 Add mock notifications support in `MockClickUpClient` in `src/api/mock_client.rs`
- [x] 2.5 Add CLI debug command for notifications in `src/main.rs` (parallel to existing debug commands)

## 3. Navigation Integration

- [x] 3.1 Add Inbox variant to navigation state enum in `src/app.rs`
- [x] 3.2 Add inbox entry to sidebar navigation in `src/tui/widgets/sidebar.rs`
- [x] 3.3 Add inbox icon/label rendering (📬 or "IN")
- [x] 3.4 Handle inbox selection in navigation message handler
- [x] 3.5 Update help dialog to show inbox navigation shortcut

## 4. Inbox UI Components

- [x] 4.1 Create `src/tui/widgets/inbox_view.rs` module
- [x] 4.2 Implement notification list widget with oldest-first ordering
- [x] 4.3 Implement empty inbox state display
- [x] 4.4 Implement notification detail panel
- [x] 4.5 Add inbox view rendering to `src/tui/app.rs`
- [x] 4.6 Add status bar context help for inbox view

## 5. Inbox Actions & Event Handling

- [x] 5.1 Add `InboxRefresh`, `InboxClear`, `InboxClearAll` variants to `Message` enum
- [x] 5.2 Implement 'c' key handler for clearing single notification
- [x] 5.3 Implement 'C' key handler for clearing all notifications
- [x] 5.4 Implement 'r' key handler for manual refresh
- [x] 5.5 Implement Enter key handler for opening notification detail
- [x] 5.6 Implement Esc key handler for returning from detail view
- [x] 5.7 Update help dialog (`?`) with inbox-specific shortcuts

## 6. State Management & Integration

- [x] 6.1 Add inbox state to `TuiApp` struct (current notifications, selection, detail view)
- [x] 6.2 Implement inbox refresh on view entry
- [x] 6.3 Integrate cache fetch in inbox update handler
- [x] 6.4 Handle API errors gracefully with user-facing messages
- [x] 6.5 Add loading indicator during refresh

## 7. Testing & Verification

- [x] 7.1 Add unit tests for Notification model deserialization
- [ ] 7.2 Add mock client tests for notifications API
- [x] 7.3 Add cache integration tests
- [x] 7.4 Add inbox UI component tests
- [ ] 7.5 Manual testing with real ClickUp API
- [ ] 7.6 Verify keyboard shortcuts work correctly
- [ ] 7.7 Test empty inbox state
- [ ] 7.8 Test with large number of notifications
