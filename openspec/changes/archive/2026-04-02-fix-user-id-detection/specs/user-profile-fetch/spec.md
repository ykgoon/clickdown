# user-profile-fetch Specification

## Purpose
Define proactive fetching of the current user's profile during application initialization to enable assignee filtering across all features without requiring user navigation to trigger detection.

## Requirements

### ADDED Requirements

#### Requirement: Proactive user profile fetch at initialization
The system SHALL fetch the current user's profile from the ClickUp API during application initialization, after workspaces are successfully loaded. The fetch SHALL occur asynchronously in the background without blocking UI rendering.

##### Scenario: User profile fetched after workspace load
- **WHEN** workspaces are successfully loaded from the API
- **THEN** the system initiates a background fetch of the current user profile via `get_current_user()`
- **AND** the fetch completes asynchronously without blocking the UI
- **AND** the user's ID is stored in `current_user_id` upon successful fetch

##### Scenario: User profile fetch failure is silent
- **WHEN** the user profile fetch fails during initialization
- **THEN** the failure is logged at debug level but no error is shown to the user
- **AND** the system continues normal operation
- **AND** task-based user ID detection remains available as fallback

##### Scenario: No fetch if API client unavailable
- **WHEN** the API client is not available during initialization
- **THEN** the user profile fetch is skipped
- **AND** a warning is logged
- **AND** task-based detection remains available as fallback

#### Requirement: User ID storage for assignee filtering
The system SHALL store the fetched user ID in the `current_user_id` field for use by assignee filtering mechanisms across all features.

##### Scenario: User ID available for assigned items fetch
- **WHEN** user navigates to "Assigned to Me" view after successful user profile fetch
- **THEN** `current_user_id` contains the user's ID
- **AND** assigned items are fetched using this ID without additional user profile API calls

##### Scenario: User ID persists across navigation
- **WHEN** user navigates between different views after user profile fetch
- **THEN** `current_user_id` remains available
- **AND** features requiring user ID (assigned items, smart inbox) work immediately

#### Requirement: On-demand user profile fetch fallback
The system SHALL fetch the user profile on-demand if `current_user_id` is unavailable when a feature explicitly requires it (e.g., loading "Assigned to Me" view).

##### Scenario: On-demand fetch when loading assigned items
- **WHEN** user navigates to "Assigned to Me" view
- **AND** `current_user_id` is `None`
- **THEN** the system initiates `fetch_current_user_and_load_tasks()` instead of showing error
- **AND** a loading indicator is shown
- **AND** assigned items are loaded after user profile fetch completes

##### Scenario: Error shown if on-demand fetch fails
- **WHEN** on-demand user profile fetch fails
- **THEN** an error message is shown: "Failed to load user profile. Please check your connection and try again."
- **AND** the user can retry by refreshing the view

### MODIFIED Requirements

None - this is a new capability that does not modify existing requirement specifications.

## Scenarios

#### Scenario: First-time user immediate access to "Assigned to Me"
- **WHEN** user launches the application for the first time (no session state)
- **AND** user clicks "Assigned to Me" immediately without navigating to any task list
- **THEN** user profile was fetched during initialization
- **AND** `current_user_id` is available
- **AND** assigned items load and display successfully
- **AND** no error message is shown

#### Scenario: Session restore with user ID
- **WHEN** user has a saved session with `user_id` in session state
- **AND** application restores the session on startup
- **THEN** `current_user_id` is restored from session state
- **AND** proactive user profile fetch is skipped (already have user ID)
- **AND** "Assigned to Me" works immediately from cache or background refresh

#### Scenario: Session restore without user ID
- **WHEN** user has a saved session without `user_id` (previous session quit before detection)
- **AND** application restores the session on startup
- **THEN** `current_user_id` is `None` after restore
- **AND** proactive user profile fetch is initiated
- **AND** "Assigned to Me" works after fetch completes
