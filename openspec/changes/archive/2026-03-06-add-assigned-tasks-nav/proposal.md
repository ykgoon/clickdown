## Why

Users need a quick way to view tasks assigned to them without navigating through the workspace hierarchy. This adds an "Inbox" or "Assigned to Me" navigation item that provides direct access to personally assigned tasks, improving workflow efficiency.

## What Changes

- Add a new "Assigned to Me" navigation item in the sidebar
- Display tasks where the current user is listed as an assignee
- Fetch assigned tasks from all accessible lists across workspaces
- Show task count badge on the navigation item
- Support keyboard navigation (j/k/Enter) for assigned tasks list
- Integrate with existing task detail view when selecting a task

## Capabilities

### New Capabilities
- `assigned-tasks-nav`: Navigation item showing tasks assigned to the current user

### Modified Capabilities
- `navigation-layout`: Add assigned tasks item to sidebar layout
- `tui-navigation`: Add keyboard navigation support for assigned tasks view

## Impact

- **API**: Need to fetch tasks with assignee filter across multiple lists
- **UI**: Sidebar widget needs new navigation item type
- **State**: App state needs to track assigned tasks view
- **Navigation**: New view state for assigned tasks list
- **Performance**: May need caching strategy for cross-list task aggregation
