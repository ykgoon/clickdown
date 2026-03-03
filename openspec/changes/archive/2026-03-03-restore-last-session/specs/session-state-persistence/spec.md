## ADDED Requirements

### Requirement: Session state persistence on exit
The system SHALL save the current application state to the SQLite cache database when the user initiates a graceful shutdown. The saved state SHALL include the current screen type and all navigation hierarchy selections.

#### Scenario: Save state on graceful exit
- **WHEN** user initiates graceful shutdown via ctrl-q or quit confirmation
- **THEN** current screen type is saved (Auth, Workspaces, Spaces, Folders, Lists, Tasks, TaskDetail, Document)
- **AND** current workspace ID is saved (if applicable)
- **AND** current space ID is saved (if applicable)
- **AND** current folder ID is saved (if applicable)
- **AND** current list ID is saved (if applicable)
- **AND** current task ID is saved (if in TaskDetail view)
- **AND** current document ID is saved (if in Document view)

#### Scenario: Skip save on crash or forced exit
- **WHEN** application terminates due to crash or forced kill
- **THEN** no session state is saved
- **AND** application uses fallback behavior on next startup

### Requirement: Session state restoration on startup
The system SHALL restore the user to their last viewed location when the application starts, if valid session state exists in the cache database.

#### Scenario: Restore to last viewed screen
- **WHEN** application starts with valid session state in cache
- **THEN** user is navigated to the saved screen type
- **AND** saved navigation hierarchy is restored
- **AND** status message indicates what was restored (e.g., "Restored to Tasks view")

#### Scenario: Handle deleted workspace
- **WHEN** saved workspace ID no longer exists on the server
- **THEN** application falls back to Workspaces screen
- **AND** session state is cleared
- **AND** status message indicates fallback (e.g., "Saved workspace not found, showing all workspaces")

#### Scenario: Handle deleted space
- **WHEN** saved space ID no longer exists
- **THEN** application falls back to parent Spaces screen
- **AND** session state is cleared or updated
- **AND** status message indicates fallback

#### Scenario: Handle deleted folder
- **WHEN** saved folder ID no longer exists
- **THEN** application falls back to parent Folders screen (in the saved space)
- **AND** session state is cleared or updated
- **AND** status message indicates fallback

#### Scenario: Handle deleted list
- **WHEN** saved list ID no longer exists
- **THEN** application falls back to parent Lists screen (in the saved folder/space)
- **AND** session state is cleared or updated
- **AND** status message indicates fallback

#### Scenario: Handle deleted task
- **WHEN** saved task ID no longer exists (in TaskDetail view)
- **THEN** application falls back to Tasks list view
- **AND** session state is updated to Tasks screen
- **AND** status message indicates fallback

#### Scenario: Handle deleted document
- **WHEN** saved document ID no longer exists (in Document view)
- **THEN** application falls back to previous screen
- **AND** session state is cleared or updated
- **AND** status message indicates fallback

#### Scenario: First launch (no saved state)
- **WHEN** application starts with no session state in cache
- **THEN** application follows normal initialization flow
- **AND** starts at Auth screen (if not authenticated) or Workspaces screen (if authenticated)

### Requirement: Session state database schema
The system SHALL use a dedicated table in the SQLite cache database to store session state.

#### Scenario: Create session_state table
- **WHEN** application initializes database
- **THEN** `session_state` table is created if not exists
- **AND** table has columns: key (TEXT PRIMARY KEY), value (TEXT NOT NULL)

#### Scenario: Store state as JSON
- **WHEN** session state is saved
- **THEN** state is serialized as JSON
- **AND** stored with key 'current_session'

### Requirement: Session state update on navigation
The system SHALL update the in-memory session state as the user navigates, so the latest state is always available for saving on exit.

#### Scenario: Update state on workspace selection
- **WHEN** user selects a workspace
- **THEN** in-memory session state is updated with workspace ID
- **AND** screen type is set to Spaces

#### Scenario: Update state on space selection
- **WHEN** user selects a space
- **THEN** in-memory session state is updated with space ID
- **AND** screen type is set to Folders

#### Scenario: Update state on folder selection
- **WHEN** user selects a folder
- **THEN** in-memory session state is updated with folder ID
- **AND** screen type is set to Lists

#### Scenario: Update state on list selection
- **WHEN** user selects a list
- **THEN** in-memory session state is updated with list ID
- **AND** screen type is set to Tasks

#### Scenario: Update state on task selection
- **WHEN** user selects a task
- **THEN** in-memory session state is updated with task ID
- **AND** screen type is set to TaskDetail

#### Scenario: Update state on document selection
- **WHEN** user selects a document
- **THEN** in-memory session state is updated with document ID
- **AND** screen type is set to Document

### Requirement: Session state clearing
The system SHALL provide a mechanism to clear saved session state.

#### Scenario: Clear state after successful restore
- **WHEN** session state is successfully restored and validated
- **THEN** state remains saved (for next session)
- **AND** is updated as user navigates

#### Scenario: Clear state on logout
- **WHEN** user logs out
- **THEN** session state is cleared from database
- **AND** next startup begins at Auth screen
