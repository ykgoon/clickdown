## MODIFIED Requirements

### Requirement: Unread messages display
The system SHALL display activities in a list view with newest activities at the top. Each activity SHALL show the activity type icon, title, description, and timestamp.

#### Scenario: Activities displayed
- **WHEN** user enters the inbox view
- **THEN** all activities are displayed in a list

#### Scenario: Newest-first ordering
- **WHEN** activities are displayed
- **THEN** they are ordered by timestamp descending (newest first)

**Note**: This is changed from "oldest-first" to match modern activity feed conventions.

#### Scenario: Activity shows timestamp
- **WHEN** an activity is displayed
- **THEN** the activity timestamp is shown (e.g., "2 hours ago" or date)

#### Scenario: Activity shows title
- **WHEN** an activity is displayed
- **THEN** the activity title is prominently displayed

#### Scenario: Activity shows description
- **WHEN** an activity has a description
- **THEN** a truncated preview of the description is shown

#### Scenario: Activity shows source
- **WHEN** an activity is displayed
- **THEN** the source task/workspace name is indicated

#### Scenario: Activity type icon displayed
- **WHEN** an activity is displayed
- **THEN** an icon indicating the activity type is shown:
  - 📋 for task assignments
  - 💬 for comments
  - 🔄 for status changes
  - ⏰ for due dates
