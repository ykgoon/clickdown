## ADDED Requirements

### Requirement: Unread messages display
The system SHALL display unread notifications in a list view with oldest messages at the top. Each notification SHALL show essential information for quick scanning.

#### Scenario: Unread notifications displayed
- **WHEN** user enters the inbox view
- **THEN** all unread notifications are displayed in a list

#### Scenario: Oldest-first ordering
- **WHEN** notifications are displayed
- **THEN** they are ordered chronologically with oldest at the top

#### Scenario: Notification shows timestamp
- **WHEN** a notification is displayed
- **THEN** the creation timestamp is shown (e.g., "2 hours ago" or date)

#### Scenario: Notification shows title
- **WHEN** a notification is displayed
- **THEN** the notification title is prominently displayed

#### Scenario: Notification shows description
- **WHEN** a notification has a description
- **THEN** a truncated preview of the description is shown

#### Scenario: Notification shows source
- **WHEN** a notification is displayed
- **THEN** the source workspace or context is indicated

### Requirement: Empty inbox state
The system SHALL display an empty state message when there are no unread notifications.

#### Scenario: No unread notifications
- **WHEN** all notifications have been cleared
- **THEN** a message "Inbox is empty" is displayed

#### Scenario: Empty state is encouraging
- **WHEN** inbox is empty
- **THEN** the message indicates this is a positive state (e.g., "All caught up!")

### Requirement: Notification list navigation
The system SHALL support vim-style keyboard navigation through the notification list.

#### Scenario: Move selection down
- **WHEN** user presses j or ↓
- **THEN** selection moves to the next notification in the list

#### Scenario: Move selection up
- **WHEN** user presses k or ↑
- **THEN** selection moves to the previous notification in the list

#### Scenario: Selection wraps around
- **WHEN** user presses j on the last item
- **THEN** selection wraps to the first item (and vice versa for k)

#### Scenario: Selected notification highlighted
- **WHEN** a notification is selected
- **THEN** it is visually highlighted with a different background or border

### Requirement: Long notification handling
The system SHALL handle notifications that exceed available width gracefully.

#### Scenario: Long title truncation
- **WHEN** a notification title exceeds available width
- **THEN** it is truncated with ellipsis (...)

#### Scenario: Multi-line description
- **WHEN** a notification has a long description
- **THEN** it is truncated to 1-2 lines with ellipsis
