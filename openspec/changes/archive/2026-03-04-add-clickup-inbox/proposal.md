## Why

ClickUp users need a centralized place to view notifications and messages from their workspace. Currently, ClickDown lacks visibility into ClickUp's notification system, forcing users to switch between the web app and terminal. Adding an inbox feature keeps users in the terminal workflow while staying informed of important updates.

## What Changes

- Add inbox entry point in the navigation sidebar
- Display unread messages with oldest-first ordering
- Provide keyboard shortcut to mark messages as read/clear them
- Integrate with ClickUp's notifications API
- Cache notifications locally for offline viewing

## Capabilities

### New Capabilities
- `inbox-navigation`: Entry point in sidebar navigation to access inbox
- `inbox-list-ui`: Display unread messages list with oldest-first ordering
- `inbox-message-actions`: Keyboard shortcuts to clear/mark messages as read
- `inbox-api-integration`: Fetch notifications from ClickUp API
- `inbox-caching`: Local SQLite caching for notifications

### Modified Capabilities
- `navigation-layout`: Add inbox item to sidebar navigation structure
- `tui-navigation`: Extend navigation state machine to include inbox view

## Impact

- **New API endpoints**: ClickUp notifications API integration
- **Database schema**: New `notifications` table in SQLite cache
- **TUI components**: New inbox list widget and message detail view
- **Navigation state**: Extend app state machine to handle inbox view
- **Keyboard shortcuts**: Add inbox-specific key bindings
