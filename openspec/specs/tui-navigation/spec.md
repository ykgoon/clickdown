## MODIFIED Requirements

### Requirement: Assigned tasks view navigation
The system SHALL implement keyboard navigation for the assigned tasks view, consistent with existing task list navigation patterns.

#### Scenario: Navigate assigned tasks with j/k
- **WHEN** user is in the assigned tasks view
- **AND** user presses `j` or `↓` (down arrow)
- **THEN** selection moves to the next task in the list
- **AND** view scrolls if selection moves out of visible area

#### Scenario: Navigate assigned tasks with up arrow
- **WHEN** user is in the assigned tasks view
- **AND** user presses `k` or `↑` (up arrow)
- **THEN** selection moves to the previous task in the list
- **AND** view scrolls if selection moves out of visible area

#### Scenario: Select assigned task
- **WHEN** user has a task selected in assigned tasks view
- **AND** user presses `Enter`
- **THEN** the task detail view opens showing the selected task's information

#### Scenario: Return from assigned tasks
- **WHEN** user is in the assigned tasks view
- **AND** user presses `Esc`
- **THEN** the assigned tasks view closes
- **AND** the previous view (e.g., workspace list) is displayed

### Requirement: Assigned tasks view state
The system SHALL maintain an `AssignedTasksView` state in the navigation state machine, allowing users to navigate to and from the assigned tasks view.

#### Scenario: Enter assigned tasks view from sidebar
- **WHEN** user selects the "Assigned to Me" item in the sidebar
- **AND** presses `Enter`
- **THEN** the navigation state changes to `AssignedTasksView`
- **AND** assigned tasks are fetched and displayed

#### Scenario: Assigned tasks view is distinct from list view
- **WHEN** user is in the assigned tasks view
- **THEN** the view state is `AssignedTasksView`, not `ListView`
- **AND** the header displays "Assigned to Me" instead of list name

#### Scenario: Return to assigned tasks from task detail
- **WHEN** user opens a task detail from assigned tasks view
- **AND** user presses `Esc` to return
- **THEN** the view returns to assigned tasks list, not workspace navigation

### Requirement: Refresh assigned tasks
The system SHALL support manual refresh of assigned tasks using the `r` key.

#### Scenario: Refresh assigned tasks
- **WHEN** user is in the assigned tasks view
- **AND** user presses `r`
- **THEN** the cache is invalidated
- **AND** fresh tasks are fetched from the API
- **AND** a loading indicator is displayed during refresh

#### Scenario: Refresh preserves view state
- **WHEN** refresh completes
- **THEN** the view remains in `AssignedTasksView` state
- **AND** the task list is updated with fresh data

### Requirement: Keyboard navigation bindings
The system SHALL implement vim-style keyboard bindings for navigation. The bindings SHALL be consistent across all screens including the inbox view.

#### Scenario: Move selection down
- **WHEN** user presses `j` or `↓` (down arrow)
- **THEN** selection moves to the next item in the list
- **AND** view scrolls if selection moves out of visible area

#### Scenario: Move selection up
- **WHEN** user presses `k` or `↑` (up arrow)
- **THEN** selection moves to the previous item in the list
- **AND** view scrolls if selection moves out of visible area

#### Scenario: Select item
- **WHEN** user presses `Enter` on a selected item
- **THEN** the item is opened or activated
- **AND** appropriate detail view is displayed

#### Scenario: Go back
- **WHEN** user presses `Esc`
- **THEN** the current view is closed
- **AND** previous view is displayed

#### Scenario: Inbox navigation with j/k
- **WHEN** user is in inbox view and presses j or k
- **THEN** selection moves through the notification list

### Requirement: Global navigation keys
The system SHALL implement global navigation keys that work from any screen. These keys SHALL provide quick access to common actions including inbox-specific actions.

#### Scenario: Quit application
- **WHEN** user presses `q`
- **THEN** confirmation dialog is shown
- **AND** application exits on confirmation

#### Scenario: Toggle sidebar
- **WHEN** user presses `Tab` or `Ctrl+b`
- **THEN** sidebar visibility is toggled
- **AND** main content area is resized accordingly

#### Scenario: Show help
- **WHEN** user presses `?`
- **THEN** help overlay is displayed
- **AND** all keyboard shortcuts are listed

#### Scenario: Refresh inbox
- **WHEN** user presses `r` in inbox view
- **THEN** notifications are refreshed from the API

### Requirement: Context-aware actions
The system SHALL implement context-aware action keys that change behavior based on the current screen. Inbox view SHALL have its own set of context-aware actions.

#### Scenario: Create new item
- **WHEN** user presses `n` in task list view
- **THEN** task creation form is opened

#### Scenario: Edit item
- **WHEN** user presses `e` on a selected task
- **THEN** task edit form is opened with current values

#### Scenario: Delete item
- **WHEN** user presses `d` on a selected task
- **THEN** confirmation dialog is shown
- **AND** task is deleted on confirmation

#### Scenario: Clear notification
- **WHEN** user presses `c` on a selected notification in inbox view
- **THEN** the notification is marked as read and removed from the list

#### Scenario: Clear all notifications
- **WHEN** user presses `C` (shift+c) in inbox view
- **THEN** all unread notifications are marked as read

### Requirement: Focus management
The system SHALL manage focus between different panels (sidebar, main content, detail panel). Focus SHALL be visually indicated. Inbox view SHALL support focus management for its detail panel.

#### Scenario: Switch focus to sidebar
- **WHEN** sidebar is visible and user presses `Ctrl+h`
- **THEN** focus moves to sidebar
- **AND** sidebar is highlighted to indicate focus

#### Scenario: Switch focus to main content
- **WHEN** user presses `Ctrl+l`
- **THEN** focus moves to main content area
- **AND** main content is highlighted to indicate focus

#### Scenario: Focus indicator visible
- **WHEN** a panel has focus
- **THEN** the panel border or background is highlighted
- **AND** keyboard input is directed to that panel

#### Scenario: Inbox detail panel focus
- **WHEN** notification detail view is open
- **THEN** focus is on the detail panel
- **AND** Esc returns to notification list
