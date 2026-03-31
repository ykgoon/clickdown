# assigned-comments-aggregation Specification

## Purpose
Define the unified view for assigned work items (tasks and comments) in the "Assigned to Me" navigation section, enabling users to see all work assigned to them in one place.

## Requirements

### ADDED Requirements

#### Requirement: Unified assigned work items view
The system SHALL display both assigned tasks and assigned comments in a single unified list in the "Assigned to Me" view. The system SHALL use an `AssignedItem` enum to represent heterogeneous items (tasks and comments) in the list.

##### Scenario: Assigned items list contains both tasks and comments
- **WHEN** user opens the "Assigned to Me" view
- **THEN** the list displays both tasks where user is an assignee AND comments where user is an assigned commenter
- **AND** items are sorted by updated_at descending (most recent first)
- **AND** the list uses a type-safe enum to represent items

##### Scenario: Assigned items list handles empty state
- **WHEN** user has no assigned tasks and no assigned comments
- **THEN** a message "No tasks or comments assigned to you" is displayed
- **AND** the count badge shows 0

#### Requirement: Visual distinction between tasks and comments
The system SHALL visually distinguish assigned comments from assigned tasks using icons and styling. Comments SHALL display a comment icon (💬) while tasks display a task icon (✓).

##### Scenario: Task item displays task icon
- **WHEN** an assigned task is rendered in the list
- **THEN** a task icon (✓ or similar) is displayed to the left of the task name
- **AND** the item shows task status indicator and priority indicator

##### Scenario: Comment item displays comment icon
- **WHEN** an assigned comment is rendered in the list
- **THEN** a comment icon (💬 or similar) is displayed to the left of the comment preview
- **AND** the item shows the parent task name as context
- **AND** the item shows the comment author and timestamp

##### Scenario: Comment item shows comment preview
- **WHEN** an assigned comment is rendered
- **THEN** the first line or 50 characters of comment content is shown as preview
- **AND** long previews are truncated with ellipsis

#### Requirement: Combined count badge
The system SHALL display a combined count of assigned tasks and assigned comments in the badge next to the "Assigned to Me" navigation item. The badge SHALL show the total count (tasks + comments).

##### Scenario: Badge shows combined count
- **WHEN** user has 3 assigned tasks and 2 assigned comments
- **THEN** the badge displays "5"
- **AND** the badge is visible in the sidebar navigation

##### Scenario: Badge shows zero when empty
- **WHEN** user has no assigned items
- **THEN** the badge displays "0" or is hidden (per existing badge behavior)

#### Requirement: Filter by item type
The system SHALL allow users to filter the assigned items list by type: All, Tasks Only, or Comments Only. The filter SHALL be accessible via a keyboard shortcut or UI toggle.

##### Scenario: Filter shows all items by default
- **WHEN** user opens the "Assigned to Me" view
- **THEN** all assigned items (tasks and comments) are displayed
- **AND** the filter is set to "All" by default

##### Scenario: Filter tasks only
- **WHEN** user activates the "Tasks Only" filter (via keyboard shortcut or UI)
- **THEN** only assigned tasks are displayed
- **AND** assigned comments are hidden
- **AND** the badge still shows combined count

##### Scenario: Filter comments only
- **WHEN** user activates the "Comments Only" filter
- **THEN** only assigned comments are displayed
- **AND** assigned tasks are hidden

##### Scenario: Filter state persists during session
- **WHEN** user changes the filter
- **THEN** the filter state persists until user changes it or exits the view
- **AND** refresh preserves the filter state

#### Requirement: Navigate to parent task from assigned comment
The system SHALL navigate to the parent task and open the comment thread when user selects an assigned comment and presses Enter.

##### Scenario: Selecting comment opens parent task
- **WHEN** user navigates to an assigned comment with j/k keys
- **AND** presses Enter
- **THEN** the task detail view opens for the parent task
- **AND** the comments section is focused
- **AND** the comment thread view is opened for the selected comment

##### Scenario: Comment navigation with missing parent task
- **WHEN** user selects an assigned comment whose parent task is not accessible or deleted
- **THEN** an error message "Parent task not available" is displayed
- **AND** user can remove the orphaned comment from the list

#### Requirement: Fetch assigned comments from all workspaces
The system SHALL fetch comments where the current user is listed in the `assigned_commenters` field from all accessible lists across all workspaces. The fetch SHALL be performed in parallel with assigned tasks.

##### Scenario: Fetch comments across all lists
- **WHEN** user opens the "Assigned to Me" view
- **THEN** the system fetches comments from all accessible lists in parallel
- **AND** filters comments where current user ID matches an assigned commenter
- **AND** includes parent task information in the response

##### Scenario: Parallel fetch with tasks
- **WHEN** assigned items are fetched
- **THEN** tasks and comments are fetched in parallel using tokio::join!
- **AND** the view displays items as soon as both fetches complete
- **AND** if one fetch fails, the other still displays with a warning

##### Scenario: Handle assigned commenters API variations
- **WHEN** the ClickUp API returns assigned_commenters in different formats
- **THEN** the system handles: array of user IDs, array of user objects, null, or missing field
- **AND** uses flexible deserializers to parse the response
- **AND** logs warnings for unexpected formats

#### Requirement: Cache assigned comments
The system SHALL cache assigned comments with their parent task associations for offline access and performance. The cache SHALL have a 5-minute TTL.

##### Scenario: Cache assigned comments
- **WHEN** assigned comments are fetched
- **THEN** results are stored in SQLite cache with timestamp
- **AND** parent task information is cached with the comment
- **AND** subsequent requests within 5 minutes use cached data

##### Scenario: Cache invalidation
- **WHEN** user presses 'r' to refresh in the assigned view
- **THEN** the cache is invalidated for both tasks and comments
- **AND** fresh data is fetched from the API

##### Scenario: Graceful degradation on cache miss
- **WHEN** cache is empty or expired
- **THEN** the system fetches from API
- **AND** displays a loading indicator
- **AND** updates cache on success

#### Requirement: Refresh assigned items
The system SHALL allow users to manually refresh the assigned items list to fetch the latest data from the API. Refresh SHALL update both tasks and comments.

##### Scenario: Manual refresh with keyboard
- **WHEN** user presses `r` in the assigned items view
- **THEN** the cache is invalidated for both tasks and comments
- **AND** fresh data is fetched from the API
- **AND** the list is re-sorted by updated_at

##### Scenario: Refresh indicator
- **WHEN** assigned items are being refreshed
- **THEN** a loading indicator is displayed
- **AND** the count badge shows a spinner or loading state

##### Scenario: Refresh preserves filter state
- **WHEN** refresh completes
- **THEN** the view returns to assigned items list
- **AND** the previously active filter (All/Tasks/Comments) is preserved
- **AND** previously selected item remains selected if still in the list

### MODIFIED Requirements

#### Requirement: Assigned tasks navigation item (MODIFIED from assigned-tasks-nav)
The system SHALL display an "Assigned to Me" navigation item in the sidebar at the top level, above the workspace hierarchy. The item SHALL show the combined count of assigned tasks AND assigned comments as a badge.

##### Scenario: Assigned tasks item visible in sidebar
- **WHEN** the sidebar is displayed
- **THEN** an "Assigned to Me" item is visible at the top of the navigation list
- **AND** the item displays a count badge showing combined number of assigned tasks and comments

##### Scenario: Assigned tasks item has distinct icon
- **WHEN** the assigned tasks item is rendered
- **THEN** it displays a user/inbox icon (👤 or 📬) to distinguish it from workspace items

##### Scenario: Assigned tasks count updates
- **WHEN** assigned items are loaded or refreshed
- **THEN** the count badge updates to reflect the combined count of assigned tasks and comments

##### Scenario: Assigned tasks item is selectable
- **WHEN** user navigates to the assigned tasks item using j/k keys
- **THEN** the item is highlighted to indicate selection
- **AND** pressing Enter opens the unified assigned items view

#### Requirement: Fetch assigned tasks from all accessible lists (MODIFIED from assigned-tasks-nav)
The system SHALL fetch tasks from all accessible lists across all workspaces and filter for tasks where the current user is listed in the `assignees` field. The system SHALL ALSO fetch comments where the current user is listed in `assigned_commenters`. The system SHALL cache the results for performance.

##### Scenario: Fetch tasks and comments in parallel
- **WHEN** user selects the "Assigned to Me" navigation item
- **THEN** the system fetches tasks from all accessible lists in parallel
- **AND** fetches comments from all accessible lists in parallel
- **AND** filters tasks where current user ID matches an assignee
- **AND** filters comments where current user ID matches an assigned commenter

##### Scenario: Cache assigned items
- **WHEN** assigned items are fetched
- **THEN** results are stored in the cache with a timestamp
- **AND** subsequent requests within 5 minutes use cached data
- **AND** cache includes both tasks and comments

##### Scenario: Handle unknown user identity
- **WHEN** the current user's identity cannot be determined
- **THEN** the system displays a message indicating identity detection failed
- **AND** offers to fetch all tasks for manual filtering
- **AND** shows a message that comment filtering requires user identity

##### Scenario: Limit initial fetch
- **WHEN** fetching assigned items
- **THEN** the initial fetch is limited to 100 items total (tasks + comments)
- **AND** a "Load More" option is available if more items exist

#### Requirement: Assigned tasks view display (MODIFIED from assigned-tasks-nav)
The system SHALL display assigned items in a unified list view showing tasks and comments with appropriate visual distinction. Tasks SHALL show name, status, priority, and due date. Comments SHALL show comment preview, parent task name, author, and timestamp.

##### Scenario: Display assigned items list
- **WHEN** assigned items view is opened
- **THEN** items are displayed in a scrollable list sorted by updated_at descending
- **AND** tasks show name, status indicator, priority indicator, and due date
- **AND** comments show comment preview, parent task name, author, and timestamp
- **AND** each item type has a distinct icon

##### Scenario: Empty assigned items state
- **WHEN** no items are assigned to the current user
- **THEN** a message "No tasks or comments assigned to you" is displayed
- **AND** the count badge shows 0

##### Scenario: Task selection in assigned view
- **WHEN** user selects a task using j/k and presses Enter
- **THEN** the task detail view opens with the selected task's information

##### Scenario: Comment selection in assigned view
- **WHEN** user selects a comment using j/k and presses Enter
- **THEN** the task detail view opens for the parent task
- **AND** the comment thread view is opened for the selected comment

#### Requirement: Refresh assigned tasks (MODIFIED from assigned-tasks-nav)
The system SHALL allow users to manually refresh the assigned items list to fetch the latest data from the API. Refresh SHALL update both tasks and comments.

##### Scenario: Manual refresh with keyboard
- **WHEN** user presses `r` in the assigned items view
- **THEN** the cache is invalidated for both tasks and comments
- **AND** fresh data is fetched from the API

##### Scenario: Refresh indicator
- **WHEN** assigned items are being refreshed
- **THEN** a loading indicator is displayed
- **AND** the count badge shows a spinner or loading state

##### Scenario: Refresh preserves selection
- **WHEN** refresh completes
- **THEN** the view returns to assigned items list
- **AND** previously selected item remains selected if still in the list
