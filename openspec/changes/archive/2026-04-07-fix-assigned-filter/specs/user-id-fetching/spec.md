## ADDED Requirements

### Requirement: Fetch current user profile during TUI initialization

The system SHALL fetch the current authenticated user's profile during TUI application initialization and store the user ID for use in assignee filtering operations.

#### Scenario: Successful user profile fetch on startup
- **WHEN** the TUI application initializes with a valid API token
- **THEN** the system SHALL call `get_current_user()` in the background
- **AND** store the returned user's ID in `current_user_id`
- **AND** log the detected user ID and username at info level

#### Scenario: Failed user profile fetch
- **WHEN** the user profile API call fails during initialization
- **THEN** `current_user_id` SHALL remain `None`
- **AND** the failure SHALL be logged at debug level
- **AND** the application SHALL continue normal operation (non-fatal)

#### Scenario: User ID available for assignee filtering
- **WHEN** the user presses `a` to toggle the "Assigned to Me" filter
- **AND** `current_user_id` is `Some(id)`
- **THEN** the system SHALL use that ID to filter tasks by assignee

#### Scenario: User ID unavailable for assignee filtering
- **WHEN** the user presses `a` to toggle the "Assigned to Me" filter
- **AND** `current_user_id` is `None`
- **THEN** the system SHALL display an error message "User ID not available for filtering"
- **AND** no API call SHALL be made
