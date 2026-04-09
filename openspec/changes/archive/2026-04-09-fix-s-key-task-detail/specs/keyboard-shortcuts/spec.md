## ADDED Requirements

### Requirement: S opens status picker from task detail
The system SHALL allow users to open the status picker overlay by pressing the `s` key while on the task detail screen. This shortcut SHALL only be active when a task is loaded in the task detail view. The behavior SHALL be consistent with the `s` key behavior in the task list view (opens status picker for selected task).

#### Scenario: User presses s on task detail with task loaded
- **WHEN** user is viewing task detail with a task loaded
- **AND** presses `s`
- **THEN** the status picker overlay opens showing available status options
- **AND** the current task's status is indicated in the picker

#### Scenario: S is inactive without task loaded
- **WHEN** user is on task detail screen
- **AND** no task is loaded in the detail view
- **AND** presses `s`
- **THEN** the status picker does not open
- **AND** a status message indicates no task is selected

#### Scenario: S does not interfere with comment typing
- **WHEN** user is actively typing text in the comment input field
- **AND** presses `s`
- **THEN** the letter 's' is entered into the comment text
- **AND** the status picker does not open
