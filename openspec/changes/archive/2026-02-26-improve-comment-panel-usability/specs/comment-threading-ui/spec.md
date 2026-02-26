## ADDED Requirements

### Requirement: Top-level comments display
The system SHALL display only top-level comments (comments with no parent_id) when viewing a task's comment panel. Top-level comments SHALL be shown in reverse chronological order (newest first). Each top-level comment SHALL indicate if it has replies.

#### Scenario: Top-level comments rendered
- **WHEN** a task detail view is opened and comments are loaded
- **THEN** only comments with parent_id = null or None are displayed
- **AND** comments are sorted by created_at in descending order (newest first)
- **AND** each comment shows author, timestamp, and content
- **AND** comments with replies show a reply count indicator (e.g., "2 replies")

#### Scenario: Empty top-level state
- **WHEN** a task has no top-level comments (only replies or no comments at all)
- **THEN** the message "No comments yet. Press 'n' to add one." is displayed
- **AND** the comment input form is accessible

#### Scenario: Reply count displayed
- **WHEN** a top-level comment has one or more replies
- **THEN** a reply count is shown in the comment header (e.g., "2 replies")
- **AND** the reply count is visually distinct from comment content (dimmed color)
- **AND** clicking or pressing Enter on the comment opens the thread view

### Requirement: Thread view display
The system SHALL display all replies to a selected top-level comment when the user "enters" a thread. Replies SHALL be shown in chronological order (oldest first) below the parent comment. The parent comment SHALL be shown at the top of the thread view for context.

#### Scenario: Thread view opened
- **WHEN** user presses Enter on a selected top-level comment
- **THEN** the comment panel switches to thread view mode
- **AND** the parent comment is displayed at the top of the thread
- **AND** all replies (comments with parent_id = selected comment's id) are shown below
- **AND** replies are sorted by created_at in ascending order (oldest first)
- **AND** the breadcrumb shows "Comments > {parent_author}"

#### Scenario: Thread view with no replies
- **WHEN** user enters a top-level comment that has no replies
- **THEN** only the parent comment is shown
- **AND** a message "No replies yet. Press 'r' to reply." is displayed below
- **AND** the reply form is accessible

#### Scenario: Reply text wrapping
- **WHEN** reply text exceeds the widget width
- **THEN** text wraps to the next line without horizontal overflow
- **AND** no content is hidden or requires horizontal scrolling

### Requirement: Visual thread indicators
The system SHALL provide visual indicators to distinguish between top-level comments and replies, and to show the current view mode (top-level vs. thread view).

#### Scenario: Top-level comment styling
- **WHEN** viewing top-level comments
- **THEN** each comment has a consistent visual style
- **AND** comments with replies show a reply count badge
- **AND** the selected comment is highlighted with a border or background color

#### Scenario: Thread view parent comment styling
- **WHEN** viewing a thread
- **THEN** the parent comment at the top has a distinct visual style (e.g., bold border or different background)
- **AND** the text "Parent comment" or similar indicator is shown
- **AND** replies are visually indented or have a thread line connecting them to parent

#### Scenario: Breadcrumb navigation display
- **WHEN** in thread view
- **THEN** the comment panel title shows "Comments > {parent_author}"
- **AND** the breadcrumb is visually distinct from regular panel titles
- **AND** the breadcrumb updates when navigating to different threads

### Requirement: Comment filtering by parent_id
The system SHALL filter comments based on parent_id to show appropriate comments in each view mode. Filtering SHALL be performed client-side on already-loaded comments.

#### Scenario: Filter top-level comments
- **WHEN** in top-level view mode
- **THEN** only comments where parent_id is None or null are displayed
- **AND** comments are cached with their parent_id for fast filtering

#### Scenario: Filter thread replies
- **WHEN** in thread view for parent comment with id "abc123"
- **THEN** only comments where parent_id = "abc123" are displayed as replies
- **AND** the parent comment itself is also displayed at the top
- **AND** filtering is performed in O(n) time on the comment list

#### Scenario: Parent comment not found
- **WHEN** user enters a thread but the parent comment is not in the loaded list
- **THEN** an error message "Parent comment not found" is displayed
- **AND** an option to reload comments is provided
- **AND** the user can press Esc to return to top-level view
