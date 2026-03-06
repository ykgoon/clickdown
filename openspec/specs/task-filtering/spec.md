# task-filtering Specification

## Purpose
TBD - created by archiving change fix-assigned-tasks-api-filter. Update Purpose after archive.
## Requirements
### Requirement: Assignees filter uses comma-separated format

The system SHALL send the `assignees` filter parameter as a comma-separated list of user IDs (e.g., `assignees=123,456`) when querying tasks from the ClickUp API, matching the ClickUp API v2 specification.

#### Scenario: Single assignee filter
- **WHEN** fetching tasks assigned to a single user with ID 123
- **THEN** the API request URL SHALL contain `assignees=123` (not `assignees[]=123`)

#### Scenario: Multiple assignees filter
- **WHEN** fetching tasks assigned to multiple users with IDs 123, 456, and 789
- **THEN** the API request URL SHALL contain `assignees=123,456,789` (not `assignees[]=123&assignees[]=456&assignees[]=789`)

#### Scenario: Empty assignees filter
- **WHEN** no assignees are specified in the filter
- **THEN** the `assignees` parameter SHALL be omitted from the request URL entirely

#### Scenario: Assigned tasks navigation uses correct filter
- **WHEN** the user navigates to "Assigned to Me" view
- **THEN** the system SHALL fetch tasks using `assignees=<current_user_id>` format

