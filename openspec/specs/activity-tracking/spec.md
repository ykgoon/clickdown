## ADDED Requirements

### Requirement: Last check timestamp storage
The system SHALL store and retrieve the last inbox check timestamp in the SQLite cache.

#### Scenario: Store timestamp after fetch
- **WHEN** activity is successfully fetched from the API
- **THEN** the current Unix timestamp (milliseconds) is stored as `last_inbox_check`

#### Scenario: Retrieve timestamp on startup
- **WHEN** the application starts
- **THEN** the `last_inbox_check` timestamp is loaded from cache if it exists

#### Scenario: Timestamp stored per workspace
- **WHEN** storing `last_inbox_check`
- **THEN** it is scoped to the current workspace ID

### Requirement: Incremental activity fetch
The system SHALL use the last check timestamp to fetch only new or updated activity.

#### Scenario: Filter tasks by date_updated
- **WHEN** fetching tasks assigned to user
- **THEN** use `date_updated_gt={last_check_timestamp}` query parameter

#### Scenario: Filter comments by date
- **WHEN** fetching comments
- **THEN** filter to comments created after `last_check_timestamp`

#### Scenario: Default time range for first fetch
- **WHEN** `last_inbox_check` does not exist (first time)
- **THEN** fetch activity from the last 7 days

### Requirement: Activity deduplication
The system SHALL deduplicate activities that may appear from multiple sources.

#### Scenario: Same task from multiple queries
- **WHEN** a task appears in both "assigned tasks" and "status changed" queries
- **THEN** it is deduplicated, keeping the most recent activity type

#### Scenario: Comment on already-assigned task
- **WHEN** a task was assigned earlier and now has a new comment
- **THEN** show as separate activities (assignment + comment)

### Requirement: Activity expiration
The system SHALL expire old activities to prevent unbounded growth.

#### Scenario: Activity retention period
- **WHEN** storing activities in cache
- **THEN** retain activities for a maximum of 30 days

#### Scenario: Cleanup on fetch
- **WHEN** new activities are fetched
- **THEN** activities older than 30 days are removed from cache
