## ADDED Requirements

### Requirement: Screen ID Generation
The system SHALL generate a unique 4-character alphanumeric identifier for each screen in the application. The ID SHALL be deterministic, producing the same ID for the same screen across application sessions.

#### Scenario: Generate ID for authentication screen
- **WHEN** the authentication screen is rendered
- **THEN** the system generates a consistent 4-character alphanumeric ID (e.g., "a3f9")

#### Scenario: Generate ID for workspace list screen
- **WHEN** the workspace list screen is rendered
- **THEN** the system generates a consistent 4-character alphanumeric ID different from other screens

#### Scenario: ID consistency across sessions
- **WHEN** the application is restarted and the same screen is displayed
- **THEN** the screen displays the same ID as in the previous session

### Requirement: Screen ID Display
The system SHALL display the screen ID in the bottom-left corner of each screen. The ID SHALL be rendered in small, unobtrusive text that does not interfere with primary UI elements.

#### Scenario: ID visible on authentication screen
- **WHEN** the authentication screen is displayed
- **THEN** the screen ID is visible in the bottom-left corner

#### Scenario: ID visible on task list screen
- **WHEN** the task list screen is displayed
- **THEN** the screen ID is visible in the bottom-left corner

#### Scenario: ID does not obscure content
- **WHEN** any screen is displayed with content near the bottom-left corner
- **THEN** the screen ID does not overlap or obscure primary UI elements

### Requirement: Screen ID Format
The screen ID SHALL consist of exactly 4 alphanumeric characters using lowercase letters (a-z) and digits (0-9). The ID SHALL be displayed in a monospace font for consistent width.

#### Scenario: ID contains valid characters
- **WHEN** a screen ID is generated
- **THEN** it contains only characters from the set [0-9a-z]

#### Scenario: ID has correct length
- **WHEN** a screen ID is generated
- **THEN** it is exactly 4 characters long

#### Scenario: ID uses lowercase
- **WHEN** a screen ID is displayed
- **THEN** all alphabetic characters are lowercase

### Requirement: Screen Coverage
The system SHALL display screen IDs on all primary screens including: authentication, workspace navigation, space view, folder view, list view, task list, task detail, and document view.

#### Scenario: ID on workspace navigation
- **WHEN** the user is viewing the workspace hierarchy
- **THEN** a screen ID is displayed

#### Scenario: ID on task detail view
- **WHEN** the user is viewing or editing a task
- **THEN** a screen ID is displayed

#### Scenario: ID on document view
- **WHEN** the user is viewing a document
- **THEN** a screen ID is displayed
