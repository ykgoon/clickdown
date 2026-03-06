## MODIFIED Requirements

### Requirement: Assigned Tasks screen loading state handling

The system SHALL properly handle and display loading state when navigating to the "Assigned to Me" screen.

#### Scenario: Navigate to Assigned Tasks with empty cache
- **WHEN** user selects "Assigned to Me" from the sidebar
- **AND** the assigned tasks cache is empty or invalid
- **THEN** the system SHALL set the loading state to true
- **AND** display a loading indicator in the main content area
- **AND** fetch assigned tasks from the API asynchronously
- **AND** the status bar SHALL show "Loading assigned tasks..."

#### Scenario: Navigate to Assigned Tasks with valid cache
- **WHEN** user selects "Assigned to Me" from the sidebar
- **AND** the assigned tasks cache is valid (less than 5 minutes old)
- **AND** cached tasks exist
- **THEN** the system SHALL display cached tasks immediately
- **AND** the status bar SHALL show "Loaded X assigned task(s) from cache"
- **AND** no loading indicator SHALL be shown

#### Scenario: Loading completes successfully
- **WHEN** assigned tasks are fetched from the API successfully
- **THEN** the loading state SHALL be set to false
- **AND** the loading indicator SHALL be removed
- **AND** the tasks SHALL be displayed in the task list
- **AND** the status bar SHALL show "Loaded X assigned task(s)"
- **AND** any previous error SHALL be cleared

#### Scenario: Loading fails with error
- **WHEN** assigned tasks fetch from the API fails
- **THEN** the loading state SHALL be set to false
- **AND** the loading indicator SHALL be removed
- **AND** an error message SHALL be displayed in the status bar
- **AND** the main content area SHALL remain empty or show a placeholder

### Requirement: User ID detection for Assigned Tasks

The system SHALL detect the current user's ID to filter assigned tasks, with fallback mechanisms when detection from local data fails.

#### Scenario: User ID detected from local task
- **WHEN** user navigates to "Assigned to Me"
- **AND** the user ID is not already known
- **AND** there are tasks in the current task list
- **THEN** the system SHALL extract the user ID from the creator field of the first task
- **AND** proceed to load assigned tasks

#### Scenario: User ID not found in local tasks
- **WHEN** user navigates to "Assigned to Me"
- **AND** the user ID is not already known
- **AND** the task list is empty
- **THEN** the system SHALL attempt fallback user ID detection (implementation detail)
- **AND** if fallback fails, display an appropriate error message

#### Scenario: User ID already known
- **WHEN** user navigates to "Assigned to Me"
- **AND** the user ID was previously detected
- **THEN** the system SHALL skip user ID detection
- **AND** proceed directly to loading assigned tasks

### Requirement: Manual refresh for Assigned Tasks

The system SHALL allow users to manually refresh the assigned tasks list with the 'r' key.

#### Scenario: Manual refresh with 'r' key
- **WHEN** user is on the "Assigned to Me" screen
- **AND** user presses the 'r' key
- **THEN** the current task list SHALL be cleared
- **AND** the cache SHALL be invalidated
- **AND** the loading indicator SHALL be displayed
- **AND** fresh data SHALL be fetched from the API
- **AND** the status bar SHALL show "Refreshing assigned tasks..."

#### Scenario: Manual refresh during loading
- **WHEN** assigned tasks are currently being loaded
- **AND** user presses the 'r' key
- **THEN** the current load SHALL be cancelled or ignored
- **AND** a fresh load SHALL be initiated
- **AND** the loading indicator SHALL remain visible
