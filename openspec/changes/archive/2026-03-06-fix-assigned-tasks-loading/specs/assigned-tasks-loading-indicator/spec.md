## ADDED Requirements

### Requirement: Show loading indicator in main content area

The system SHALL display a visible loading indicator in the main content area while fetching assigned tasks from the API.

#### Scenario: Loading indicator displayed during fetch
- **WHEN** user navigates to "Assigned to Me" section
- **AND** the cache is invalid or empty
- **AND** the system begins fetching assigned tasks from the API
- **THEN** the main content area SHALL display a loading indicator with the text "Loading assigned tasks..."
- **AND** the loading indicator SHALL remain visible until the API call completes or fails

#### Scenario: Loading indicator uses appropriate visual style
- **WHEN** the loading indicator is displayed
- **THEN** it SHALL be visually distinct from task content
- **AND** it SHALL use a color or style that indicates a loading/in-progress state (e.g., yellow or gray text)
- **AND** it SHALL be centered or prominently positioned in the content area

### Requirement: Show loading status in status bar

The system SHALL display the loading state in the status bar while fetching assigned tasks.

#### Scenario: Status bar shows loading message
- **WHEN** assigned tasks are being fetched from the API
- **THEN** the status bar SHALL display "Loading assigned tasks..."
- **AND** this message SHALL take priority over the regular status message
- **AND** this message SHALL be visible until loading completes

#### Scenario: Status bar updates on load complete
- **WHEN** assigned tasks have been successfully loaded
- **THEN** the status bar SHALL update to show "Loaded X assigned task(s)" where X is the number of tasks

### Requirement: Hide loading indicator when complete

The system SHALL remove the loading indicator when the API call completes, regardless of success or failure.

#### Scenario: Loading indicator removed on success
- **WHEN** assigned tasks are successfully loaded
- **THEN** the loading indicator SHALL be removed
- **AND** the task list SHALL be displayed in its place

#### Scenario: Loading indicator removed on error
- **WHEN** the API call fails with an error
- **THEN** the loading indicator SHALL be removed
- **AND** the error message SHALL be displayed instead

### Requirement: Support manual refresh with loading indicator

The system SHALL display the loading indicator when the user manually refreshes the assigned tasks list.

#### Scenario: Manual refresh shows loading indicator
- **WHEN** user presses 'r' key while on the "Assigned to Me" screen
- **THEN** the system SHALL clear the current task list
- **AND** display the loading indicator
- **AND** fetch fresh data from the API
- **AND** the status bar SHALL show "Refreshing assigned tasks..."

### Requirement: Loading indicator does not block navigation

The loading state SHALL NOT prevent the user from navigating away from the "Assigned to Me" screen.

#### Scenario: User can navigate away during loading
- **WHEN** assigned tasks are being loaded
- **AND** the loading indicator is displayed
- **WHEN** user presses Esc or navigates to another section
- **THEN** the system SHALL navigate away immediately
- **AND** the loading indicator SHALL be hidden
- **AND** the background API call MAY continue but results will be discarded or cached silently
