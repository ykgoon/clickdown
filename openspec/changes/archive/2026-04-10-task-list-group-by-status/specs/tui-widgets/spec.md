## MODIFIED Requirements

### Requirement: Task list widget
The system SHALL render a task list widget displaying tasks grouped by status with visual group headers. Each group header SHALL show the status group label and task count (e.g., "▸ IN PROGRESS (3)"). Tasks within each group SHALL display status and priority indicators. The list SHALL support selection and keyboard navigation, where group headers are non-selectable and navigation (j/k) skips over them.

#### Scenario: Task displayed with status in grouped view
- **WHEN** task list is rendered
- **THEN** tasks are organized under status group headers (e.g., "▸ IN PROGRESS (N)", "▸ TODO (N)", "▸ DONE (N)", "▸ OTHER (N)")
- **AND** each task shows:
  - Task name
  - Status indicator (color-coded: todo, in progress, complete)
  - Priority indicator (urgent, high, normal, low)
- **AND** empty status groups are not rendered

#### Scenario: Task selection skips group headers
- **WHEN** user navigates task list with `j/k` keys
- **THEN** selected task is highlighted
- **AND** group header rows are skipped during navigation
- **AND** task detail can be opened with `Enter`

#### Scenario: Task list scroll
- **WHEN** task list exceeds visible area
- **THEN** list scrolls to keep selection visible
- **AND** scroll position indicator is shown
