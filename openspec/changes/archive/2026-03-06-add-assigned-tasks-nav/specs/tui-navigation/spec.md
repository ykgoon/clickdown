## ADDED Requirements

### Requirement: Assigned tasks view navigation
The system SHALL implement keyboard navigation for the assigned tasks view, consistent with existing task list navigation patterns.

#### Scenario: Navigate assigned tasks with j/k
- **WHEN** user is in the assigned tasks view
- **AND** user presses `j` or `↓` (down arrow)
- **THEN** selection moves to the next task in the list
- **AND** view scrolls if selection moves out of visible area

#### Scenario: Navigate assigned tasks with up arrow
- **WHEN** user is in the assigned tasks view
- **AND** user presses `k` or `↑` (up arrow)
- **THEN** selection moves to the previous task in the list
- **AND** view scrolls if selection moves out of visible area

#### Scenario: Select assigned task
- **WHEN** user has a task selected in assigned tasks view
- **AND** user presses `Enter`
- **THEN** the task detail view opens showing the selected task's information

#### Scenario: Return from assigned tasks
- **WHEN** user is in the assigned tasks view
- **AND** user presses `Esc`
- **THEN** the assigned tasks view closes
- **AND** the previous view (e.g., workspace list) is displayed

### Requirement: Assigned tasks view state
The system SHALL maintain an `AssignedTasksView` state in the navigation state machine, allowing users to navigate to and from the assigned tasks view.

#### Scenario: Enter assigned tasks view from sidebar
- **WHEN** user selects the "Assigned to Me" item in the sidebar
- **AND** presses `Enter`
- **THEN** the navigation state changes to `AssignedTasksView`
- **AND** assigned tasks are fetched and displayed

#### Scenario: Assigned tasks view is distinct from list view
- **WHEN** user is in the assigned tasks view
- **THEN** the view state is `AssignedTasksView`, not `ListView`
- **AND** the header displays "Assigned to Me" instead of list name

#### Scenario: Return to assigned tasks from task detail
- **WHEN** user opens a task detail from assigned tasks view
- **AND** user presses `Esc` to return
- **THEN** the view returns to assigned tasks list, not workspace navigation

### Requirement: Refresh assigned tasks
The system SHALL support manual refresh of assigned tasks using the `r` key.

#### Scenario: Refresh assigned tasks
- **WHEN** user is in the assigned tasks view
- **AND** user presses `r`
- **THEN** the cache is invalidated
- **AND** fresh tasks are fetched from the API
- **AND** a loading indicator is displayed during refresh

#### Scenario: Refresh preserves view state
- **WHEN** refresh completes
- **THEN** the view remains in `AssignedTasksView` state
- **AND** the task list is updated with fresh data
