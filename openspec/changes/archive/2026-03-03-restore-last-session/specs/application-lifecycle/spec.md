## MODIFIED Requirements

### Requirement: Application shutdown
The system SHALL support graceful shutdown when requested by the user through UI or keyboard shortcuts. The application MUST exit cleanly without leaving orphaned processes or corrupted state. Before exiting, the system SHALL persist the current session state to enable restoration on next startup.

#### Scenario: User requests quit via ctrl-q
- **WHEN** user presses `ctrl-q` keyboard shortcut
- **THEN** current session state is saved to cache database
- **AND** system calls `std::process::exit(0)` to terminate the application

#### Scenario: Application exits with success code
- **WHEN** application shuts down via ctrl-q
- **THEN** exit code is 0 indicating successful termination
- **AND** session state has been persisted

#### Scenario: No cleanup required on exit
- **WHEN** application exits
- **THEN** no file handles, network connections, or database connections are left open
- **AND** database connection is properly closed after saving session state

#### Scenario: Save session state on exit
- **WHEN** user initiates graceful shutdown
- **THEN** current screen type is serialized
- **AND** current navigation hierarchy IDs are serialized (workspace, space, folder, list, task, document as applicable)
- **AND** state is saved to `session_state` table in cache database
- **AND** state is saved with key 'current_session'

### Requirement: Application initialization
The system SHALL initialize the application state on startup. If valid session state exists in the cache database, the system SHALL restore the user to their last viewed location.

#### Scenario: Initialize without saved session
- **WHEN** application starts with no saved session state
- **THEN** application follows normal initialization flow
- **AND** starts at Auth screen (if not authenticated) or Workspaces screen (if authenticated)

#### Scenario: Initialize with saved session
- **WHEN** application starts with valid session state in cache
- **THEN** session state is loaded from `session_state` table
- **AND** user is navigated to the saved screen type
- **AND** saved navigation hierarchy is restored
- **AND** status message indicates what was restored

#### Scenario: Handle invalid saved state
- **WHEN** saved session state references resources that no longer exist
- **THEN** application falls back to the nearest valid parent screen
- **AND** status message indicates the fallback
- **AND** session state is updated or cleared
