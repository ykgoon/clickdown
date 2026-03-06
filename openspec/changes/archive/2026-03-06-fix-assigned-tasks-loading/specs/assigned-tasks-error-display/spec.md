## ADDED Requirements

### Requirement: Display authentication error for assigned tasks

The system SHALL display a clear error message when the user is not authenticated and attempts to view assigned tasks.

#### Scenario: User not authenticated
- **WHEN** user navigates to "Assigned to Me" section
- **AND** the user is not authenticated (no valid API token)
- **THEN** the system SHALL display the error message "Not authenticated" in the status bar
- **AND** the main content area SHALL remain empty or show a placeholder

#### Scenario: User recovers from authentication error
- **WHEN** the authentication error is displayed
- **AND** the user authenticates successfully
- **AND** the user navigates to "Assigned to Me" again
- **THEN** the system SHALL attempt to load assigned tasks
- **AND** the error message SHALL be cleared

### Requirement: Display user detection failure error

The system SHALL display a helpful error message when the user ID cannot be detected and no fallback is available.

#### Scenario: User ID not detected and no fallback available
- **WHEN** user navigates to "Assigned to Me" section
- **AND** the user ID has not been detected from any local task
- **AND** the fallback user ID detection fails or is not implemented
- **THEN** the system SHALL display the error message "User identity not detected. Please open a task you created first." in the status bar
- **AND** the main content area SHALL remain empty

#### Scenario: User opens a task after detection failure
- **WHEN** the user detection failure error is displayed
- **AND** the user navigates to and opens a task they created
- **THEN** the system SHALL detect the user ID from the task creator field
- **AND** the error message SHALL be cleared
- **AND** the user can now navigate to "Assigned to Me" successfully

### Requirement: Display API error for assigned tasks

The system SHALL display an error message when the API call to fetch assigned tasks fails.

#### Scenario: API call fails with network error
- **WHEN** the system attempts to fetch assigned tasks
- **AND** the API call fails due to a network error
- **THEN** the system SHALL display an error message in the status bar containing "Failed to load assigned tasks"
- **AND** the main content area SHALL remain empty or show the last cached data if available

#### Scenario: API call fails with authorization error
- **WHEN** the system attempts to fetch assigned tasks
- **AND** the API returns an authorization error (401/403)
- **THEN** the system SHALL display an appropriate error message
- **AND** the user SHOULD be prompted to re-authenticate

### Requirement: Clear error on successful load

The system SHALL clear any previous error state when assigned tasks are successfully loaded.

#### Scenario: Successful load after previous error
- **WHEN** assigned tasks are successfully loaded from the API
- **AND** there was a previous error displayed
- **THEN** the system SHALL clear the error message
- **THEN** the system SHALL display the loaded tasks
- **AND** the status bar SHALL show "Loaded X assigned task(s)"
