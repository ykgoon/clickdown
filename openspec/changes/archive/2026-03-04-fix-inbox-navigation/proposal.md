## Why

The Inbox item in the sidebar navigation is unresponsive when selected from any workspace hierarchy screen (Workspaces, Spaces, Folders, Lists). Pressing Enter on "Inbox" produces no reaction, preventing users from accessing their notifications from these screens. This breaks the expected navigation flow and makes the Inbox feature inaccessible except through unknown means.

## What Changes

- **Fixed**: Pressing Enter on "Inbox" in the sidebar now navigates to the Inbox view from any workspace hierarchy screen (Workspaces, Spaces, Folders, Lists)
- **Modified**: The `navigate_into()` function in `src/tui/app.rs` now handles `SidebarItem::Inbox` selection in all navigation screen match arms
- **Behavior**: Users can now access Inbox from any level of the workspace hierarchy without first navigating back to a specific screen

## Capabilities

### New Capabilities
<!-- No new capabilities - this is a bug fix for existing navigation -->

### Modified Capabilities
<!-- No spec-level requirement changes - the navigation behavior was always intended to work this way -->

## Impact

- **Code**: `src/tui/app.rs` - `navigate_into()` function needs modification
- **Navigation flow**: All workspace hierarchy screens (Workspaces, Spaces, Folders, Lists) will now properly handle Inbox selection
- **User experience**: Restores expected keyboard navigation pattern where Enter on any sidebar item navigates to that destination
