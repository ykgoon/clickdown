## MODIFIED Requirements

### Requirement: Task list widget
The system SHALL render a task list widget displaying tasks with priority indicators. The list SHALL support selection, sorting indicators, and keyboard navigation. Status context SHALL be communicated via group headers rather than per-task status indicators.

#### Scenario: Task displayed with priority
- **WHEN** task list is rendered
- **THEN** each task shows:
  - Task name
  - Priority indicator (urgent ⚡, high ↑, normal •, low ↓)
- **AND** tasks are organized under status group headers (e.g., "▸ TODO (3)", "▸ IN PROGRESS (1)")

#### Scenario: Task selection
- **WHEN** user navigates task list with `j/k` keys
- **THEN** selected task is highlighted
- **AND** task detail can be opened with `Enter`

#### Scenario: Task list scroll
- **WHEN** task list exceeds visible area
- **THEN** list scrolls to keep selection visible
- **AND** scroll position indicator is shown
