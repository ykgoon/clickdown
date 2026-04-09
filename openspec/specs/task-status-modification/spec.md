# task-status-modification Specification

## Purpose
TBD - created by archiving change allow-user-to-change-status-of-a-task. Update Purpose after archive.
## Requirements
### Requirement: User can initiate status change for selected task
The system SHALL allow users to initiate a status change process for the currently selected task using a keyboard shortcut.

#### Scenario: Status change initiated with keyboard shortcut
- **WHEN** user has a task selected in the task list
- **THEN** pressing the 's' key opens the status selection interface

#### Scenario: Status change initiation requires selected task
- **WHEN** no task is selected in the task list
- **THEN** pressing the 's' key has no effect

### Requirement: Status selection interface displays available statuses
The system SHALL display a list of available task statuses when the status change process is initiated.

#### Scenario: Status picker shows space-specific statuses
- **WHEN** user initiates status change for a task
- **THEN** system displays all statuses defined in the task's space
- **AND** each status is shown with its name and color (when available)

#### Scenario: Status picker includes default statuses
- **WHEN** user initiates status change for a task in a space with no custom statuses
- **THEN** system displays the default ClickUp statuses (e.g., "To Do", "In Progress", "Done")

### Requirement: User can select new status from picker
The system SHALL allow users to navigate and select a status from the status picker interface.

#### Scenario: Keyboard navigation in status picker
- **WHEN** status picker is open
- **THEN** user can navigate options using arrow keys or j/k keys
- **AND** pressing Enter selects the highlighted status
- **AND** pressing Esc cancels and closes the picker

#### Scenario: Status selection updates task immediately
- **WHEN** user selects a status from the picker
- **THEN** system begins process to update the task's status via API
- **AND** UI shows loading indicator for the task
- **AND** picker closes upon selection

### Requirement: Task status is updated via ClickUp API
The system SHALL update the task's status using the ClickUp API when a new status is selected.

#### Scenario: Successful API status update
- **WHEN** user selects a new status and system API call succeeds
- **THEN** task's status is updated in ClickUp
- **AND** task list reflects the new status immediately
- **AND** success feedback is shown to user

#### Scenario: Failed API status update handling
- **WHEN** user selects a new status and system API call fails
- **THEN** task's status remains unchanged in UI
- **AND** error message is displayed to user
- **AND** user can retry the operation

#### Scenario: API rate limiting handling
- **WHEN** API returns rate limit error during status update
- **THEN** system shows informative error message
- **AND** suggests user try again later
- **AND** does not repeatedly retry automatically

### Requirement: System supports custom statuses from ClickUp spaces
The system SHALL properly display and handle custom statuses defined in ClickUp spaces.

#### Scenario: Custom statuses appear in picker
- **WHEN** user's space has defined custom statuses
- **THEN** those custom statuses appear in the status picker
- **AND** are displayed with their configured names and colors

#### Scenario: Custom status update via API
- **WHEN** user selects a custom status from picker
- **THEN** system sends the correct status ID to ClickUp API
- **AND** API successfully updates task to that custom status

