## ADDED Requirements

### Requirement: S opens status picker from task detail
The system SHALL allow users to open the status picker overlay by pressing the `s` key while on the task detail screen. This shortcut SHALL only be active when a task is loaded in the task detail view. The behavior SHALL be consistent with the `s` key behavior in the task list view.

#### Scenario: User presses s on task detail with task loaded
- **WHEN** user is viewing task detail with a task loaded
- **AND** presses `s`
- **THEN** the status picker overlay opens
- **AND** the status bar displays navigation instructions for the status picker

#### Scenario: S is inactive without task loaded
- **WHEN** user is on task detail screen
- **AND** no task is loaded in the detail view
- **AND** presses `s`
- **THEN** the status picker does not open
- **AND** a status message indicates no task is selected

#### Scenario: Status picker shows same options from task detail as from task list
- **WHEN** user opens status picker from task detail
- **THEN** the same status options are presented as when opened from task list view
- **AND** the same navigation keys (j/k) and selection (Enter/Esc) work identically

#### Scenario: S does not interfere with comment typing
- **WHEN** user is typing a comment in the comment form
- **AND** presses `s`
- **THEN** the letter 's' is entered into the comment text
- **AND** the status picker does not open
