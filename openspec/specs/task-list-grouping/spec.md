## ADDED Requirements

### Requirement: Task list displays status group headers
The system SHALL render visual status group headers above each group of tasks sharing the same `status_group`. Headers SHALL display the group label in uppercase and the count of tasks in that group (e.g., "▸ IN PROGRESS (3)"). Headers SHALL be rendered with a distinct visual style (dimmed/bold text, separator line) to differentiate them from task rows.

#### Scenario: Status group header rendered for In Progress tasks
- **WHEN** the task list contains tasks with `status_group` of "in_progress"
- **THEN** a header row "▸ IN PROGRESS (N)" SHALL be rendered above those tasks
- **AND** the header SHALL be visually distinct from task rows

#### Scenario: Status group header rendered for Todo tasks
- **WHEN** the task list contains tasks with `status_group` of "todo"
- **THEN** a header row "▸ TODO (N)" SHALL be rendered above those tasks

#### Scenario: Status group header rendered for Done tasks
- **WHEN** the task list contains tasks with `status_group` of "done"
- **THEN** a header row "▸ DONE (N)" SHALL be rendered above those tasks

#### Scenario: Status group header rendered for fallback/unknown tasks
- **WHEN** the task list contains tasks with an unrecognized or missing `status_group`
- **THEN** a header row "▸ OTHER (N)" SHALL be rendered above those tasks

### Requirement: Empty status groups are collapsed
The system SHALL NOT render headers for status groups that contain zero tasks. Only groups with at least one task SHALL be displayed.

#### Scenario: No header for empty In Progress group
- **WHEN** no tasks have `status_group` of "in_progress"
- **THEN** no "IN PROGRESS" header SHALL be rendered
- **AND** the task list SHALL proceed directly to the next non-empty group

#### Scenario: Only one group with tasks
- **WHEN** all tasks belong to a single status group
- **THEN** only that group's header SHALL be rendered
- **AND** all tasks appear under it

### Requirement: Group headers are not selectable
The system SHALL exclude status group header rows from keyboard navigation. Pressing `j` (down) or `k` (up) SHALL skip over header rows and select only task rows.

#### Scenario: Navigation skips header when moving down
- **WHEN** the currently selected task is the last task in a group
- **AND** the user presses `j` (down)
- **THEN** selection SHALL move to the first task in the next group
- **AND** the header row between groups SHALL NOT be selected

#### Scenario: Navigation skips header when moving up
- **WHEN** the currently selected task is the first task in a group
- **AND** the user presses `k` (up)
- **THEN** selection SHALL move to the last task in the previous group
- **AND** the header row between groups SHALL NOT be selected

#### Scenario: No selection when only headers exist
- **WHEN** the task list is loaded but contains zero tasks
- **THEN** no row SHALL be selected
- **AND** no group headers SHALL be rendered

### Requirement: Tasks maintain sort order within groups
The system SHALL maintain the existing sort order within each status group. Tasks within the same group SHALL be sorted by `updated_at` descending (most recent first), consistent with the current behavior.

#### Scenario: Tasks sorted by recency within In Progress group
- **WHEN** multiple tasks have `status_group` of "in_progress"
- **THEN** tasks SHALL be ordered by `updated_at` descending
- **AND** the most recently updated task appears first under the header

#### Scenario: Tasks with missing updated_at sort last
- **WHEN** a task within a group has no `updated_at` value
- **THEN** it SHALL appear after all tasks with `updated_at` in that group

### Requirement: Grouping applies to both filtered and unfiltered views
The system SHALL apply status group headers in both the unfiltered task list view and the "Assigned to Me" filtered view. The grouping behavior SHALL be identical regardless of filter state.

#### Scenario: Grouping in unfiltered view
- **WHEN** viewing all tasks in a list (no filter active)
- **THEN** tasks SHALL be grouped by status with headers

#### Scenario: Grouping in Assigned to Me view
- **WHEN** viewing tasks filtered by assignee ("Assigned to Me" active)
- **THEN** tasks SHALL be grouped by status with headers
- **AND** the view title SHALL still indicate "(Assigned to Me)"

### Requirement: Group order follows status priority
The system SHALL render status groups in priority order: In Progress, Todo, Done, Fallback. This order SHALL be consistent regardless of which groups contain tasks.

#### Scenario: Full group order
- **WHEN** tasks exist in all four status groups
- **THEN** groups SHALL appear in order: In Progress → Todo → Done → Other

#### Scenario: Partial group order
- **WHEN** tasks exist only in Todo and Done groups
- **THEN** groups SHALL appear in order: Todo → Done
- **AND** In Progress and Other headers SHALL NOT be shown
