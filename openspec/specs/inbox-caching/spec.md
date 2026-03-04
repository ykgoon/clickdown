## ADDED Requirements

### Requirement: Notification storage schema
The system SHALL create a notifications table in the SQLite cache database to store fetched notifications.

#### Scenario: Table creation on startup
- **WHEN** the application starts
- **THEN** the notifications table exists or is created

#### Scenario: Table has required columns
- **WHEN** the notifications table is created
- **THEN** it has columns: id, workspace_id, title, description, created_at, read_at, fetched_at

#### Scenario: Primary key on notification ID
- **WHEN** the notifications table is created
- **THEN** the id column is the primary key

### Requirement: Cache notifications on fetch
The system SHALL store fetched notifications in the local SQLite cache when retrieved from the API.

#### Scenario: Store fetched notifications
- **WHEN** notifications are fetched from the API
- **THEN** they are inserted or updated in the cache

#### Scenario: Update existing notifications
- **WHEN** a notification already exists in cache
- **THEN** it is updated with latest data from API

#### Scenario: Record fetch timestamp
- **WHEN** notifications are cached
- **THEN** the fetched_at timestamp is recorded

### Requirement: Mark notification as read in cache
The system SHALL update the read_at timestamp when a notification is cleared by the user.

#### Scenario: Mark single notification as read
- **WHEN** user clears a notification
- **THEN** the read_at field is set to current timestamp

#### Scenario: Mark all notifications as read
- **WHEN** user clears all notifications
- **THEN** all unread notifications have read_at set

### Requirement: Query unread notifications
The system SHALL query the cache to retrieve unread notifications for display.

#### Scenario: Fetch unread only
- **WHEN** inbox view is displayed
- **THEN** only notifications with NULL read_at are returned

#### Scenario: Order by creation date
- **WHEN** unread notifications are fetched
- **THEN** they are ordered by created_at ascending (oldest first)

#### Scenario: Limit results
- **WHEN** unread notifications are fetched
- **THEN** results are limited to most recent N notifications (configurable)

### Requirement: Cache cleanup
The system SHALL periodically clean up old read notifications to prevent database bloat.

#### Scenario: Archive old read notifications
- **WHEN** notification has been read for more than N days
- **THEN** it can be archived or deleted (configurable)

#### Scenario: Cleanup on startup
- **WHEN** application starts
- **THEN** old read notifications are cleaned up (optional)
