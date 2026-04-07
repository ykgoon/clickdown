# assigned-comments-aggregation Specification (Delta)

## Purpose
Update the "Assigned to Me" feature to handle unknown user identity by fetching the user profile on-demand instead of showing an error immediately.

## Requirements

### MODIFIED Requirements

#### Requirement: Handle unknown user identity
The system SHALL handle cases where the current user's identity (`current_user_id`) is not available when loading assigned items. Instead of displaying an error immediately, the system SHALL initiate an on-demand fetch of the user profile from the ClickUp API.

##### Scenario: On-demand user fetch when loading assigned items
- **WHEN** user navigates to "Assigned to Me" view
- **AND** `current_user_id` is `None` (user identity not yet detected)
- **THEN** the system calls `fetch_current_user_and_load_tasks()` instead of showing error
- **AND** a loading indicator is displayed with message "Loading user profile..."
- **AND** assigned items are fetched after user profile completes
- **AND** user ID is stored for subsequent navigations

##### Scenario: Error shown if on-demand fetch fails
- **WHEN** on-demand user profile fetch fails (network error, API error)
- **THEN** an error message is displayed: "Failed to load user profile. Please check your connection and try again."
- **AND** the user can retry by pressing 'r' to refresh
- **AND** task-based detection remains available as fallback

##### Scenario: Proactive fetch prevents on-demand fetch
- **WHEN** user profile was fetched proactively during app initialization
- **AND** user navigates to "Assigned to Me" view
- **THEN** `current_user_id` is already available
- **AND** assigned items load immediately without additional user profile fetch
- **AND** no loading indicator for user profile is shown
