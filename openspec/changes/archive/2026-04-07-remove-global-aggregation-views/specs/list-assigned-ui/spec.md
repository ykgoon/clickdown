## ADDED Requirements

### Requirement: Assigned filter toggle control
The system SHALL provide a keyboard shortcut to toggle the "Assigned to Me" filter on and off while viewing a task list. The shortcut SHALL be discoverable via the help dialog.

#### Scenario: Toggle filter with keyboard shortcut
- **WHEN** user is viewing a task list
- **AND** presses the assigned filter shortcut key
- **THEN** the filter toggles between active and inactive states
- **AND** the task list refreshes to reflect the filter state

#### Scenario: Help dialog shows assigned filter shortcut
- **WHEN** user presses `?` to open the help dialog
- **AND** the current view is a task list
- **THEN** the help dialog displays the keyboard shortcut for toggling the "Assigned to Me" filter

### Requirement: Filter state visual indicator
The system SHALL display a visual indicator showing whether the "Assigned to Me" filter is currently active. The indicator SHALL include the filter label and the count of filtered results.

#### Scenario: Active filter shows label and count
- **WHEN** the "Assigned to Me" filter is active
- **THEN** a label such as "Assigned to Me (N tasks)" is displayed in the view header or status bar
- **AND** N reflects the number of tasks matching the filter

#### Scenario: Inactive filter shows no label
- **WHEN** the "Assigned to Me" filter is not active
- **THEN** no filter label is displayed
- **AND** the view header shows the standard list title and task count

#### Scenario: Loading state during filter fetch
- **WHEN** the filter is toggled and tasks are being fetched
- **THEN** a loading indicator is displayed
- **AND** the previous task list remains visible until the new data arrives

### Requirement: Status bar context for assigned filter
The system SHALL display assigned filter-related information in the status bar when the filter is active.

#### Scenario: Status bar shows filter context
- **WHEN** the "Assigned to Me" filter is active
- **THEN** the status bar shows the filter name and result count
- **AND** the shortcut to toggle the filter is shown in the status bar hints

#### Scenario: Status bar shows error when no assigned tasks
- **WHEN** the filter is active
- **AND** no tasks are assigned to the user in the current list
- **THEN** the status bar shows "No tasks assigned to you in this list"
