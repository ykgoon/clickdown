## ADDED Requirements

### Requirement: Task list sorting by status and activity
The system SHALL automatically sort tasks in the task list view by status priority and recency. Tasks SHALL be grouped by status with the following priority order: in-progress → to-do → done. Within each status group, tasks SHALL be sorted by most recently active (updated_at descending). Tasks without updated_at SHALL be sorted last within their status group.

#### Scenario: Tasks sorted by status priority
- **WHEN** tasks are loaded from the API or cache
- **THEN** the system sorts tasks into status groups: in-progress, to-do, done
- **AND** in-progress tasks appear first
- **AND** to-do tasks appear second
- **AND** done tasks appear last
- **AND** tasks with unrecognized status groups are placed after done tasks

#### Scenario: Tasks sorted by recency within status group
- **WHEN** multiple tasks share the same status group
- **THEN** tasks are sorted by updated_at in descending order (most recent first)
- **AND** tasks with newer updated_at appear before older tasks
- **AND** tasks without updated_at appear last within their status group

#### Scenario: Sorting applied automatically on load
- **WHEN** tasks are fetched from the API
- **AND** the task list is populated
- **THEN** sorting is applied immediately before rendering
- **AND** no user action is required to trigger sorting
- **AND** the sorted order is consistent across sessions with the same data

#### Scenario: Sorting handles missing status gracefully
- **WHEN** a task has no status field (status = None)
- **THEN** the task is placed in a fallback group after all recognized status groups
- **AND** within the fallback group, tasks are still sorted by updated_at descending

#### Scenario: Sorting preserves task selection
- **WHEN** tasks are re-sorted (e.g., after refresh or status change)
- **AND** a task was previously selected
- **THEN** the system attempts to maintain selection on the same task by ID
- **AND** if the selected task no longer exists, selection moves to the nearest task

### Requirement: Status group mapping
The system SHALL map ClickUp status_group values to sorting priorities. The mapping SHALL be case-insensitive and handle common variations.

#### Scenario: Standard status groups recognized
- **WHEN** task status_group is "in_progress" or "in progress"
- **THEN** the task is assigned to the in-progress group (priority 1)
- **WHEN** task status_group is "todo" or "to_do"
- **THEN** the task is assigned to the to-do group (priority 2)
- **WHEN** task status_group is "done" or "complete"
- **THEN** the task is assigned to the done group (priority 3)

#### Scenario: Unknown status groups handled
- **WHEN** task status_group is not recognized
- **THEN** the task is assigned to a fallback group (priority 4)
- **AND** a debug log message is generated with the unknown status_group value
