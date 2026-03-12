## ADDED Requirements

### Requirement: Smart inbox activity feed
The system SHALL display a polling-based activity feed in the Inbox view that aggregates recent activity from multiple ClickUp API endpoints. The activity feed SHALL simulate notifications by showing assignments, comments, status changes, and due dates.

#### Scenario: Fetch tasks assigned to user
- **WHEN** the smart inbox refreshes
- **THEN** it fetches tasks recently assigned to the current user from `/api/v2/team/{team_id}/task?assignees={user_id}`

#### Scenario: Fetch comments on user's tasks
- **WHEN** the smart inbox refreshes
- **THEN** it fetches recent comments on tasks assigned to the current user

#### Scenario: Fetch tasks with status changes
- **WHEN** the smart inbox refreshes
- **THEN** it fetches tasks with recent status updates

#### Scenario: Fetch tasks with approaching due dates
- **WHEN** the smart inbox refreshes
- **THEN** it fetches tasks with due dates within the next 7 days

#### Scenario: Merge and sort activities
- **WHEN** multiple activity types are fetched
- **THEN** they are merged into a single list sorted by timestamp (newest first)

### Requirement: Activity type identification
The system SHALL identify and display different activity types with distinct visual indicators.

#### Scenario: Assignment activity type
- **WHEN** a task is newly assigned to the user
- **THEN** it is displayed with an assignment icon (📋 or similar)

#### Scenario: Comment activity type
- **WHEN** a comment is posted on user's task
- **THEN** it is displayed with a comment icon (💬 or similar)

#### Scenario: Status change activity type
- **WHEN** a task's status changes
- **THEN** it is displayed with a status icon (🔄 or similar)

#### Scenario: Due date activity type
- **WHEN** a task has an approaching due date
- **THEN** it is displayed with a due date icon (⏰ or similar)

### Requirement: Activity data model
The system SHALL define an `InboxActivity` model that normalizes different activity types into a unified structure.

#### Scenario: Activity has unique ID
- **WHEN** an activity is created
- **THEN** it has a unique identifier (combines source type + source ID)

#### Scenario: Activity has type field
- **WHEN** an activity is created
- **THEN** it has a type field indicating the activity category (assignment, comment, status, due_date)

#### Scenario: Activity has title
- **WHEN** an activity is created
- **THEN** it has a human-readable title (e.g., "Task assigned to you", "New comment on Task X")

#### Scenario: Activity has description
- **WHEN** an activity is created
- **THEN** it has an optional description with additional context

#### Scenario: Activity has timestamp
- **WHEN** an activity is created
- **THEN** it has a timestamp indicating when the activity occurred

#### Scenario: Activity has source reference
- **WHEN** an activity is created
- **THEN** it has a reference to the source (task ID, comment ID, etc.)

#### Scenario: Activity has workspace reference
- **WHEN** an activity is created
- **THEN** it has the workspace/team ID for context

### Requirement: Incremental polling
The system SHALL fetch only new or updated activity since the last check to minimize API calls.

#### Scenario: Track last check timestamp
- **WHEN** the inbox is refreshed
- **THEN** the current timestamp is stored as `last_inbox_check`

#### Scenario: Fetch only new activity
- **WHEN** fetching activity on subsequent refreshes
- **THEN** only activity with timestamps after `last_inbox_check` is fetched

#### Scenario: First-time fetch
- **WHEN** `last_inbox_check` does not exist
- **THEN** fetch activity from the last 7 days

### Requirement: Manual refresh mechanism
The system SHALL provide a manual refresh mechanism to fetch new activity from the API.

#### Scenario: Refresh on inbox enter
- **WHEN** user enters the inbox view
- **THEN** activity is fetched from the API (cache-first strategy)

#### Scenario: Manual refresh shortcut
- **WHEN** user presses 'r' in inbox view
- **THEN** activity is refreshed from the API

#### Scenario: Refresh indicator
- **WHEN** refresh is in progress
- **THEN** a loading indicator is displayed

### Requirement: Handle API errors gracefully
The system SHALL handle API errors gracefully when fetching activity.

#### Scenario: Partial failure
- **WHEN** one activity endpoint fails but others succeed
- **THEN** display available activities and log the error

#### Scenario: Complete failure
- **WHEN** all activity endpoints fail
- **THEN** show cached activity and display an error message

#### Scenario: Network error
- **WHEN** network is unavailable
- **THEN** show cached activity with a message indicating stale data
