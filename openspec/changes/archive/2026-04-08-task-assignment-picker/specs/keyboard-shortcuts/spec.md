## ADDED Requirements

### Requirement: A opens assignee picker from task detail
The system SHALL allow users to open the assignee picker overlay by pressing the `A` key while on the task detail screen. This shortcut SHALL only be active when a task is selected and the task's parent list ID is known in application state.

#### Scenario: User presses A on task detail
- **WHEN** user is viewing task detail
- **AND** presses `A`
- **AND** the task's parent list ID is available in application state
- **THEN** the assignee picker overlay opens

#### Scenario: A is inactive without list context
- **WHEN** user is viewing task detail
- **AND** presses `A`
- **AND** the task's parent list ID is NOT available
- **THEN** the assignee picker does not open
- **AND** a status message indicates list context is not available

#### Scenario: A does not trigger in other screens
- **WHEN** user is on the task list screen (not task detail)
- **AND** presses `A`
- **THEN** no action is taken
- **AND** no error is shown
