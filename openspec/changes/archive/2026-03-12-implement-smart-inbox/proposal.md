## Why

The current Inbox feature attempts to fetch notifications from a ClickUp API endpoint (`/team/{workspace_id}/notifications`) that doesn't exist. ClickUp API v2 has no notification endpoint - notifications are only available via webhooks (push model) or in the ClickUp web/mobile apps. This causes a 404 "Route not found" error when users open the Inbox, making the feature non-functional.

This change implements a **polling-based smart inbox** that simulates notifications by aggregating recent activity from existing ClickUp API endpoints. This approach fits the TUI app paradigm (pull, not push), requires no additional infrastructure, and provides users with a functional activity feed that captures the spirit of an inbox.

## What Changes

- **Modified**: Inbox now displays an activity feed instead of true notifications
- **Added**: Smart inbox fetches recent activity from multiple API endpoints:
  - Tasks recently assigned to the user
  - Comments on user's tasks
  - Tasks with recent status changes
  - Tasks with approaching due dates
- **Added**: Activity items are normalized into a unified `InboxActivity` model
- **Added**: Last-check timestamp tracking for incremental polling
- **Modified**: Inbox UI displays activity type icons (assignment, comment, status, due date)
- **Added**: Manual refresh ('r' key) fetches new activity since last check
- **Kept**: "Inbox" name in navigation (user-facing terminology unchanged)
- **Kept**: Existing inbox UI/UX patterns (j/k navigation, Enter for detail, c to mark read)

## Capabilities

### New Capabilities
- `smart-inbox`: Polling-based activity feed that aggregates assignments, comments, status changes, and due dates into a unified inbox view
- `activity-tracking`: Tracks last-check timestamp and fetches only new/updated activity for efficient polling

### Modified Capabilities
- `inbox-navigation`: Now loads activity feed from multiple API endpoints instead of non-existent notifications endpoint
- `inbox-list-ui`: Display now includes activity type indicators (assignment, comment, status, due date icons)

## Impact

- **Code**: `src/models/inbox_activity.rs` - New activity model (replaces notification model for inbox display)
- **Code**: `src/api/client.rs` - Add methods to fetch assignments, comments, and updated tasks
- **Code**: `src/api/endpoints.rs` - Add endpoint builders for activity-related queries
- **Code**: `src/cache/mod.rs` - Add activity caching and last-check timestamp storage
- **Code**: `src/tui/app.rs` - Update inbox loading logic to poll multiple endpoints
- **Code**: `src/tui/widgets/inbox_view.rs` - Update rendering to show activity type icons
- **Database**: Add `inbox_activity` table and `last_inbox_check` key-value store
- **User experience**: Inbox becomes functional with activity feed instead of empty/404 error

## Dependencies

This change depends on:
- Existing API client infrastructure (`ClickUpApi` trait, `ClickUpClient` implementation)
- Existing cache infrastructure (SQLite caching layer)
- Existing inbox navigation and UI components
- User profile endpoint (to get current user ID for filtering)

No external dependencies or new infrastructure required.
