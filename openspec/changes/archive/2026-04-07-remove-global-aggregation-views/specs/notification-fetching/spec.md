## REMOVED Requirements

### Requirement: Fetch notifications from ClickUp API
**Reason**: The inbox notification fetching mechanism is removed along with the entire inbox feature. The ClickUp API `/team/{workspace_id}/notifications` endpoint was already known to return 404.

**Migration**: No direct replacement. Use the per-list "Assigned to Me" filter to find relevant tasks within a list context.

### Requirement: Display unread notifications
**Reason**: The unread notification display was specific to the inbox view, which is being removed.

**Migration**: No replacement. Task lists continue to display tasks with their standard metadata (status, priority, due date).

### Requirement: Manual refresh fetches from API
**Reason**: Inbox-specific refresh via 'r' key in inbox view is removed.

**Migration**: Use the standard list refresh mechanism ('r' key in list view).

### Requirement: Pre-load notifications at startup (Optional)
**Reason**: Background notification pre-fetching for the inbox is no longer needed.

**Migration**: No replacement. Removing this eliminates expensive cross-workspace API calls at startup.

### Requirement: Mark notification as read
**Reason**: The notification read-state tracking was specific to the inbox feature.

**Migration**: No replacement needed.

### Requirement: Notification detail view
**Reason**: The notification detail overlay was part of the inbox UI, which is being removed.

**Migration**: Task detail view remains accessible from the list view.
