## ADDED Requirements

### Requirement: Assignees filter uses array query parameter format

The system SHALL send the `assignees` filter parameter as repeated array query parameters (e.g., `assignees[]=123&assignees[]=456`) when querying tasks from the ClickUp API, matching the ClickUp API v2 specification. Using comma-separated format (`assignees=123,456`) SHALL result in a 400 Bad Request error with code `PUBAPITASK_017`.

#### Scenario: Single assignee filter
- **WHEN** fetching tasks assigned to a single user with ID 123
- **THEN** the API request URL SHALL contain `assignees[]=123`

#### Scenario: Multiple assignees filter
- **WHEN** fetching tasks assigned to multiple users with IDs 123, 456, and 789
- **THEN** the API request URL SHALL contain `assignees[]=123&assignees[]=456&assignees[]=789`

#### Scenario: Empty assignees filter
- **WHEN** no assignees are specified in the filter
- **THEN** the `assignees` parameter SHALL be omitted from the request URL entirely

### Requirement: Assigned task fetch includes closed tasks

The system SHALL set `include_closed=true` when fetching tasks filtered by assignee, ensuring that completed/closed tasks assigned to the user are included in the results.

#### Scenario: Assigned filter includes closed tasks
- **WHEN** fetching tasks with the "Assigned to Me" filter active
- **THEN** the API request URL SHALL contain `include_closed=true`
- **AND** completed tasks assigned to the user SHALL be included in results

#### Scenario: Regular task fetch respects default behavior
- **WHEN** fetching tasks without the "Assigned to Me" filter
- **THEN** `include_closed` SHALL use the default value (not explicitly set)
- **AND** ClickUp API default behavior applies
