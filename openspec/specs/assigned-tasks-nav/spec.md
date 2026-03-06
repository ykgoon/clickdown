# assigned-tasks-nav Specification

## Purpose
Define the navigation, data fetching, caching, and display of assigned tasks view in the sidebar, enabling users to quickly access and manage tasks assigned to them across all workspaces.

## Requirements
### Requirement: Assigned tasks navigation item
The system SHALL display an "Assigned to Me" navigation item in the sidebar at the top level, above the workspace hierarchy. The item SHALL show the current user's assigned tasks count as a badge.

#### Scenario: Assigned tasks item visible in sidebar
- **WHEN** the sidebar is displayed
- **THEN** an "Assigned to Me" item is visible at the top of the navigation list
- **AND** the item displays a count badge showing number of assigned tasks

#### Scenario: Assigned tasks item has distinct icon
- **WHEN** the assigned tasks item is rendered
- **THEN** it displays a user/inbox icon (👤 or 📬) to distinguish it from workspace items

#### Scenario: Assigned tasks count updates
- **WHEN** assigned tasks are loaded or refreshed
- **THEN** the count badge updates to reflect the current number of assigned tasks

#### Scenario: Assigned tasks item is selectable
- **WHEN** user navigates to the assigned tasks item using j/k keys
- **THEN** the item is highlighted to indicate selection
- **AND** pressing Enter opens the assigned tasks view

### Requirement: Fetch assigned tasks from all accessible lists
The system SHALL fetch tasks from all accessible lists across all workspaces and filter for tasks where the current user is listed in the `assignees` field. The system SHALL cache the results for performance.

#### Scenario: Fetch tasks across all lists
- **WHEN** user selects the "Assigned to Me" navigation item
- **THEN** the system fetches tasks from all accessible lists in parallel
- **AND** filters tasks where current user ID matches an assignee

#### Scenario: Cache assigned tasks
- **WHEN** assigned tasks are fetched
- **THEN** results are stored in the cache with a timestamp
- **AND** subsequent requests within 5 minutes use cached data

#### Scenario: Handle unknown user identity
- **WHEN** the current user's identity cannot be determined
- **THEN** the system displays a message indicating identity detection failed
- **AND** offers to fetch all tasks for manual filtering

#### Scenario: Limit initial fetch
- **WHEN** fetching assigned tasks
- **THEN** the initial fetch is limited to 100 tasks
- **AND** a "Load More" option is available if more tasks exist

### Requirement: Assigned tasks view display
The system SHALL display assigned tasks in a list view similar to the existing task list view, showing task name, status, priority, and due date.

#### Scenario: Display assigned tasks list
- **WHEN** assigned tasks view is opened
- **THEN** tasks are displayed in a scrollable list
- **AND** each task shows name, status indicator, priority indicator, and due date

#### Scenario: Empty assigned tasks state
- **WHEN** no tasks are assigned to the current user
- **THEN** a message "No tasks assigned to you" is displayed
- **AND** the count badge shows 0

#### Scenario: Task selection in assigned view
- **WHEN** user selects a task using j/k and presses Enter
- **THEN** the task detail view opens with the selected task's information

### Requirement: Refresh assigned tasks
The system SHALL allow users to manually refresh the assigned tasks list to fetch the latest data from the API.

#### Scenario: Manual refresh with keyboard
- **WHEN** user presses `r` in the assigned tasks view
- **THEN** the cache is invalidated
- **AND** fresh tasks are fetched from the API

#### Scenario: Refresh indicator
- **WHEN** assigned tasks are being refreshed
- **THEN** a loading indicator is displayed
- **AND** the task count badge shows a spinner or loading state

#### Scenario: Refresh preserves selection
- **WHEN** refresh completes
- **THEN** the view returns to assigned tasks list
- **AND** previously selected task remains selected if still in the list

