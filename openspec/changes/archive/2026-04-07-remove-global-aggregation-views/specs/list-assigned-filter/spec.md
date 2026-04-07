## ADDED Requirements

### Requirement: Per-list assigned tasks filter
The system SHALL allow users to filter the task list within a list view to show only tasks assigned to the current user. The filter SHALL use the ClickUp API's `assignees[]` query parameter for efficient server-side filtering.

#### Scenario: Toggle assigned filter on
- **WHEN** user is viewing a list of tasks
- **AND** activates the "Assigned to Me" filter
- **THEN** the task list is re-fetched with `assignees[]={current_user_id}` parameter
- **AND** only tasks where the user is an assignee are displayed

#### Scenario: Toggle assigned filter off
- **WHEN** user is viewing a filtered task list with "Assigned to Me" active
- **AND** deactivates the filter
- **THEN** the full task list is re-fetched without the assignees filter
- **AND** all tasks in the list are displayed

#### Scenario: Filter requires user identity
- **WHEN** user activates the "Assigned to Me" filter
- **AND** the current user's identity is not known
- **THEN** the system fetches the current user profile from the API
- **AND** applies the filter once the user ID is obtained

#### Scenario: Filter persists during session
- **WHEN** user navigates away from a list with the filter active
- **AND** returns to the same list
- **THEN** the filter state is preserved
- **AND** the task list remains filtered

### Requirement: Per-list assigned comments indicator
The system SHALL indicate which tasks in a list have comments where the current user is the assigned commenter. The indicator SHALL be visible alongside the task in the list view.

#### Scenario: Task with assigned comment shows indicator
- **WHEN** a task in the list has comments where the current user is the assigned commenter
- **THEN** a comment icon (💬) is displayed next to the task entry
- **AND** the indicator distinguishes it from tasks without assigned comments

#### Scenario: No assigned comment shows no indicator
- **WHEN** a task has no comments where the current user is the assigned commenter
- **THEN** no comment indicator is displayed for that task

#### Scenario: Fetch assigned comments for visible tasks
- **WHEN** the task list is displayed with the "Assigned to Me" filter or indicator enabled
- **THEN** the system fetches comments for each visible task
- **AND** filters for comments where `assigned_commenter.id == current_user_id`
