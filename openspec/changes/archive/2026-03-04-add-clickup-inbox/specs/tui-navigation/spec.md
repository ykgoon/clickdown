## MODIFIED Requirements

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
