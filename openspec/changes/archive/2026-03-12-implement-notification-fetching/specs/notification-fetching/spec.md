## ADDED Requirements

### Requirement: Fetch notifications from ClickUp API
The system SHALL fetch notifications from the ClickUp API `/team/{workspace_id}/notifications` endpoint when the user enters the inbox view. The fetched notifications SHALL be cached locally in the SQLite database.

#### Scenario: Fetch notifications on inbox navigation
- **WHEN** user presses Enter on the "Inbox" sidebar item
- **THEN** the system calls `ClickUpClient::get_notifications(workspace_id)`
- **AND** stores results in the cache via `cache_notifications()`
- **AND** displays notifications in the inbox view

#### Scenario: Cache notifications locally
- **WHEN** notifications are fetched from the API
- **THEN** results are stored in the `notifications` SQLite table
- **AND** include `fetched_at` timestamp for cache invalidation

#### Scenario: Handle empty notifications
- **WHEN** the API returns an empty notification list
- **THEN** the inbox displays "📬 Inbox is empty - All caught up!"
- **AND** the cache is updated with empty list

#### Scenario: Handle API errors
- **WHEN** the notification API call fails
- **THEN** an error message is displayed in the status bar
- **AND** the inbox view shows the last cached data (if available)

### Requirement: Display unread notifications
The system SHALL display unread notifications from the cache, ordered by creation date (oldest first). Each notification SHALL show title, description preview, and timestamp.

#### Scenario: Display notification list
- **WHEN** the inbox view is rendered with notifications
- **THEN** notifications are displayed in a scrollable list
- **AND** each item shows: index, title, description preview, timestamp

#### Scenario: Show unread notifications only
- **WHEN** notifications are displayed
- **THEN** only notifications with `read_at IS NULL` are shown
- **AND** notifications are ordered by `created_at ASC`

#### Scenario: Highlight selected notification
- **WHEN** user navigates with j/k keys
- **THEN** the selected notification is highlighted with blue background
- **AND** a "▶ " symbol precedes the selected item

### Requirement: Manual refresh fetches from API
The system SHALL fetch fresh notifications from the API when the user presses the 'r' key in the inbox view, not just re-read from cache.

#### Scenario: Manual refresh with 'r' key
- **WHEN** user presses 'r' in the inbox view
- **THEN** the system calls `get_notifications()` from the API
- **AND** updates the cache with fresh data
- **AND** refreshes the displayed notification list

#### Scenario: Refresh status message
- **WHEN** refresh completes
- **THEN** the status bar shows "Refreshed X notification(s)"
- **OR** shows error message if refresh failed

### Requirement: Pre-load notifications at startup (Optional)
The system MAY pre-load notifications in the background at startup, similar to the assigned tasks pre-loading behavior. This provides instant inbox access without waiting for API fetch.

#### Scenario: Background pre-fetch at startup
- **WHEN** workspaces are loaded at startup
- **THEN** notifications are pre-fetched in the background for the current workspace
- **AND** cached for instant access when user navigates to inbox

#### Scenario: Cache-first display
- **WHEN** user navigates to inbox
- **THEN** cached notifications are displayed immediately (if available)
- **AND** background refresh fetches latest data

### Requirement: Mark notification as read
The system SHALL allow users to mark individual notifications as read using the 'c' key, which removes the notification from the unread list and updates the cache.

#### Scenario: Clear selected notification
- **WHEN** user presses 'c' on a selected notification
- **THEN** `cache.mark_notification_read(notification_id)` is called
- **AND** the notification is removed from the displayed list
- **AND** selection moves to the next notification

#### Scenario: Clear all notifications
- **WHEN** user presses 'C' (shift+c)
- **THEN** `cache.mark_all_notifications_read(workspace_id)` is called
- **AND** all notifications are cleared from the list
- **AND** empty inbox state is displayed

### Requirement: Notification detail view
The system SHALL allow users to view full notification details in a detail panel by pressing Enter on a selected notification.

#### Scenario: Open notification detail
- **WHEN** user presses Enter on a selected notification
- **THEN** a detail panel overlay is displayed
- **AND** shows full title, complete description, and timestamp

#### Scenario: Close detail view
- **WHEN** user presses Esc in detail view
- **THEN** the detail panel closes
- **AND** returns to the notification list view
