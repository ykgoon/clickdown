## ADDED Requirements

### Requirement: Session State Persistence
The system SHALL persist the user's navigation state when the application exits gracefully and restore it on the next startup, returning the user to their last viewed context.

#### Scenario: Session saved on graceful exit
- **WHEN** the user quits the application using Ctrl+Q
- **THEN** the current screen type and navigation IDs are saved to the cache database

#### Scenario: Session restored on startup
- **WHEN** the user launches the application after a previous graceful exit
- **THEN** the application restores the saved screen type and navigation context

#### Scenario: No session on first launch
- **WHEN** the user launches the application for the first time
- **THEN** no session state exists and the application starts at the Workspaces screen

### Requirement: Navigation Chain Replay
The system SHALL replay the full navigation chain when restoring a session, loading data at each hierarchy level and selecting the restored item before proceeding to the next level.

#### Scenario: Full navigation chain replay
- **WHEN** a session is restored with workspace_id, space_id, folder_id, and list_id
- **THEN** the system loads workspaces, selects the saved workspace, loads spaces, selects the saved space, loads folders, selects the saved folder, loads lists, selects the saved list, and finally loads tasks

#### Scenario: Partial navigation context
- **WHEN** a session is restored with only workspace_id and space_id (no folder_id or list_id)
- **THEN** the system loads workspaces, selects the saved workspace, loads spaces, selects the saved space, and stops at the Spaces screen

### Requirement: Selection Restore at Each Level
The system SHALL select the saved item at each navigation level instead of defaulting to the first item.

#### Scenario: Workspace selection restore
- **WHEN** workspaces are loaded during session restore
- **AND** a saved workspace_id exists
- **THEN** the sidebar selects the workspace with the matching ID instead of the first workspace

#### Scenario: Space selection restore
- **WHEN** spaces are loaded during session restore
- **AND** a saved space_id exists
- **THEN** the sidebar selects the space with the matching ID instead of the first space

#### Scenario: Folder selection restore
- **WHEN** folders are loaded during session restore
- **AND** a saved folder_id exists
- **THEN** the sidebar selects the folder with the matching ID instead of the first folder

#### Scenario: List selection restore
- **WHEN** lists are loaded during session restore
- **AND** a saved list_id exists
- **THEN** the sidebar selects the list with the matching ID instead of the first list

### Requirement: Fallback for Invalid IDs
The system SHALL detect when a saved ID no longer exists and fall back to the nearest valid parent level.

#### Scenario: Saved workspace no longer exists
- **WHEN** the saved workspace_id does not match any loaded workspace
- **THEN** the application shows the Workspaces screen with a status message "Saved workspace not found, showing workspaces"

#### Scenario: Saved space no longer exists
- **WHEN** the saved space_id does not match any loaded space
- **THEN** the application navigates to the Spaces screen with a status message "Saved space not found, showing spaces"

#### Scenario: Saved folder no longer exists
- **WHEN** the saved folder_id does not match any loaded folder
- **THEN** the application navigates to the Folders screen with a status message "Saved folder not found, showing folders"

#### Scenario: Saved list no longer exists
- **WHEN** the saved list_id does not match any loaded list
- **THEN** the application navigates to the Lists screen with a status message "Saved list not found, showing lists"

#### Scenario: Saved task no longer exists
- **WHEN** the saved task_id does not match any loaded task
- **THEN** the application shows the Tasks screen with a status message "Saved task not found, showing tasks"

### Requirement: User Feedback During Restore
The system SHALL provide clear status messages to inform the user about the restore process and any fallback that occurs.

#### Scenario: Successful restore message
- **WHEN** a session is successfully restored without fallback
- **THEN** the status bar shows "Restored to [Screen] view"

#### Scenario: Fallback message
- **WHEN** a saved ID is invalid and fallback occurs
- **THEN** the status bar shows a message explaining the fallback (e.g., "Saved list not found, showing lists")

#### Scenario: Restore in progress indication
- **WHEN** session restore is in progress and data is loading
- **THEN** the loading indicator shows with appropriate messages (e.g., "Loading workspaces...")

### Requirement: Session Clear on Logout
The system SHALL clear the saved session state when the user logs out to ensure the next login starts fresh.

#### Scenario: Session cleared on logout
- **WHEN** the user logs out
- **THEN** the session state is removed from the cache database

#### Scenario: Fresh start after logout
- **WHEN** the user logs in after logging out
- **THEN** no session state is restored and the application starts at the Workspaces screen
