## REMOVED Requirements

### Requirement: Screen ID Generation
**Reason**: Replaced by screen title system which provides more descriptive and debuggable context. Screen titles are more human-readable and provide better debugging information than 4-character IDs.

**Migration**: 
- Remove screen ID generation logic from `src/ui/` modules
- Remove screen ID display from bottom-left corner of screens
- Implement screen title display at top of each screen using format "ClickDown - {Context}"
- Update tests to verify screen titles instead of screen IDs

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
**Reason**: Screen titles at the top of each screen provide better visibility and context than small IDs in the corner.

**Migration**: Remove screen ID rendering from bottom-left corner; implement screen title rendering at top center/top left of each screen.

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
**Reason**: No longer applicable as screen IDs are being removed entirely.

**Migration**: N/A - feature removed.

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
**Reason**: Screen titles replace screen IDs on all screens with more descriptive labels.

**Migration**: Implement screen titles on all screens as specified in `tui-layouts` spec.

#### Scenario: ID on workspace navigation
- **WHEN** the user is viewing the workspace hierarchy
- **THEN** a screen ID is displayed

#### Scenario: ID on task detail view
- **WHEN** the user is viewing or editing a task
- **THEN** a screen ID is displayed

#### Scenario: ID on document view
- **WHEN** the user is viewing a document
- **THEN** a screen ID is displayed
