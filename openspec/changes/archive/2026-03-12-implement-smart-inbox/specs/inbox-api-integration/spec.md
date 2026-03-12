## REMOVED Requirements

### Requirement: Notifications API endpoint
**Reason**: ClickUp API v2 does not have a notifications endpoint. The endpoint `/team/{workspace_id}/notifications` returns 404 "Route not found".

**Migration**: Use the new smart inbox activity feed that polls multiple existing API endpoints:
- `/api/v2/team/{team_id}/task?assignees={user_id}` for task assignments
- `/api/v2/task/{task_id}/comment` for comments on user's tasks
- Task queries with status filters for status changes
- Task queries with due_date filters for approaching deadlines

See `smart-inbox/spec.md` for the new activity-based approach.

### Requirement: Notification data model
**Reason**: The Notification model was designed for a non-existent API endpoint.

**Migration**: Use the new `InboxActivity` model defined in `smart-inbox/spec.md` which normalizes activity from multiple sources into a unified structure.

### Requirement: Flexible notification deserialization
**Reason**: No longer deserializing ClickUp notification responses (endpoint doesn't exist).

**Migration**: Activity deserialization uses standard Task and Comment models which already have flexible deserializers.

## MODIFIED Requirements

### Requirement: Manual refresh mechanism
The system SHALL provide a manual refresh mechanism to fetch new activity from multiple API endpoints.

#### Scenario: Refresh on inbox enter
- **WHEN** user enters the inbox view
- **THEN** activity is fetched from multiple API endpoints (assignments, comments, status changes, due dates)

#### Scenario: Manual refresh shortcut
- **WHEN** user presses 'r' in inbox view
- **THEN** activity is refreshed from the API using incremental polling (only new activity since last check)

#### Scenario: Refresh indicator
- **WHEN** refresh is in progress
- **THEN** a loading indicator is displayed

#### Scenario: Handle API errors
- **WHEN** one or more activity endpoints return errors
- **THEN** display available activities and show a warning message (partial success)
