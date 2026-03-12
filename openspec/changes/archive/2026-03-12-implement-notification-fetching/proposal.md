## Why

The inbox navigation entry point exists and is now accessible (fixed in `2026-03-04-fix-inbox-navigation`), but the inbox always appears empty even when the user has unread notifications in ClickUp. The root cause is that notification fetching from the ClickUp API was never implemented in the TUI application flow.

The infrastructure exists (API client, cache methods, database schema) but the integration code that fetches notifications and populates the cache is missing. Users cannot see their notifications, making the inbox feature non-functional.

## What Changes

- **Added**: `AppMessage::NotificationsLoaded` variant for async notification loading
- **Added**: `load_notifications()` method to fetch from API and cache results
- **Added**: `pre_load_notifications()` method to check cache and fetch if needed
- **Added**: `pre_load_notifications_background()` for async fetching at startup
- **Modified**: `navigate_into()` to call `load_notifications()` when entering Inbox
- **Modified**: Manual refresh ('r' key) now fetches from API, not just reads cache
- **Behavior**: Inbox now displays actual notifications from ClickUp API
- **Behavior**: Notifications are cached for instant reloads

## Capabilities

### New Capabilities
- `notification-fetching`: Fetch notifications from ClickUp API and cache them locally

### Modified Capabilities
- `inbox-navigation`: Now loads and displays actual notification data
- `offline-cache`: Notification caching integrated with fetch flow

## Impact

- **Code**: `src/tui/app.rs` - Add notification loading methods and message handling
- **Code**: `src/tui/app.rs` - Update `navigate_into()` and `update_inbox()` to fetch notifications
- **API**: Uses existing `ClickUpClient::get_notifications()` method
- **Cache**: Uses existing `cache_notifications()` method (now actually called)
- **User experience**: Inbox becomes functional - shows real notifications from ClickUp

## Dependencies

This change depends on:
- Existing API client infrastructure (`ClickUpClient::get_notifications()`)
- Existing cache infrastructure (`cache_notifications()`)
- Existing inbox navigation (`2026-03-04-fix-inbox-navigation`)

No external dependencies or API changes required.
