## ADDED Requirements

### Requirement: Notifications API endpoint
The system SHALL integrate with ClickUp's notifications API to fetch unread notifications for a workspace.

#### Scenario: Fetch notifications for workspace
- **WHEN** the application requests notifications for a workspace
- **THEN** a GET request is made to the ClickUp notifications endpoint

#### Scenario: API authentication
- **WHEN** notifications API is called
- **THEN** the stored API token is used for authentication

#### Scenario: Handle API errors
- **WHEN** the notifications API returns an error
- **THEN** an error message is displayed to the user

### Requirement: Notification data model
The system SHALL define a Notification model that captures essential fields from the ClickUp API response.

#### Scenario: Notification has unique ID
- **WHEN** a notification is received from the API
- **THEN** it has a unique identifier field

#### Scenario: Notification has title
- **WHEN** a notification is received from the API
- **THEN** it has a title field

#### Scenario: Notification has description
- **WHEN** a notification is received from the API
- **THEN** it has an optional description field

#### Scenario: Notification has timestamp
- **WHEN** a notification is received from the API
- **THEN** it has a creation timestamp field

#### Scenario: Notification has workspace reference
- **WHEN** a notification is received from the API
- **THEN** it has a workspace ID reference

### Requirement: Flexible notification deserialization
The system SHALL handle variations in the ClickUp API notification response format gracefully.

#### Scenario: Handle missing optional fields
- **WHEN** API response lacks optional fields
- **THEN** deserialization succeeds with default values

#### Scenario: Handle timestamp format variations
- **WHEN** API returns timestamps in different formats (milliseconds, ISO 8601)
- **THEN** deserialization handles all formats correctly

#### Scenario: Log parsing errors
- **WHEN** notification parsing fails
- **THEN** the error is logged with field path information

### Requirement: Manual refresh mechanism
The system SHALL provide a manual refresh mechanism to fetch new notifications from the API.

#### Scenario: Refresh on inbox enter
- **WHEN** user enters the inbox view
- **THEN** notifications are fetched from the API

#### Scenario: Manual refresh shortcut
- **WHEN** user presses r in inbox view
- **THEN** notifications are refreshed from the API

#### Scenario: Refresh indicator
- **WHEN** refresh is in progress
- **THEN** a loading indicator is displayed
