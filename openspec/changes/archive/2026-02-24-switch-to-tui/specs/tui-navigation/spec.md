## ADDED Requirements

### Requirement: Keyboard navigation bindings
The system SHALL implement vim-style keyboard bindings for navigation. The bindings SHALL be consistent across all screens.

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

### Requirement: Global navigation keys
The system SHALL implement global navigation keys that work from any screen. These keys SHALL provide quick access to common actions.

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

### Requirement: Context-aware actions
The system SHALL implement context-aware action keys that change behavior based on the current screen.

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

### Requirement: Workspace hierarchy navigation
The system SHALL support navigation through the workspace hierarchy (Workspaces → Spaces → Folders → Lists) using keyboard.

#### Scenario: Navigate into workspace
- **WHEN** user selects a workspace and presses `Enter`
- **THEN** spaces within that workspace are displayed

#### Scenario: Navigate into space
- **WHEN** user selects a space and presses `Enter`
- **THEN** folders within that space are displayed

#### Scenario: Navigate into folder
- **WHEN** user selects a folder and presses `Enter`
- **THEN** lists within that folder are displayed

#### Scenario: Navigate into list
- **WHEN** user selects a list and presses `Enter`
- **THEN** tasks within that list are displayed

#### Scenario: Go back in hierarchy
- **WHEN** user presses `Esc` or `Backspace`
- **THEN** parent level in hierarchy is displayed

### Requirement: Focus management
The system SHALL manage focus between different panels (sidebar, main content, detail panel). Focus SHALL be visually indicated.

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
