## MODIFIED Requirements

### Requirement: Task comments display
The system SHALL display a list of comments for each task in the task detail view. The comments section SHALL occupy 70% of the vertical space in the task detail panel (3:7 ratio with description). Comments SHALL be shown in two view modes:

1. **Top-level view**: Only top-level comments (parent_id = None) are shown, sorted by created_at descending (newest first)
2. **Thread view**: When a user "enters" a top-level comment, the parent comment and all its replies (parent_id = top-level comment id) are shown, with replies sorted ascending (oldest first)

The system SHALL provide visual indicators for thread navigation, including breadcrumb display and reply counts.

#### Scenario: Top-level comments rendered
- **WHEN** task detail view is displayed and comments are loaded
- **THEN** only top-level comments (parent_id = None) are visible
- **AND** comments are sorted newest first
- **AND** each comment shows author name, timestamp, and content
- **AND** comments with replies show a reply count indicator (e.g., "2 replies")
- **AND** the comment panel title shows "Comments"

#### Scenario: Thread view opened
- **WHEN** user presses Enter on a selected top-level comment
- **THEN** the panel switches to thread view mode
- **AND** the parent comment is displayed at the top
- **AND** all replies are shown below in chronological order (oldest first)
- **AND** the panel title shows "Comments > {parent_author}"
- **AND** the parent comment has a distinct visual style (e.g., bold border)

#### Scenario: Comments scroll independently with auto-scroll
- **WHEN** comments list exceeds the comment panel's available vertical space
- **THEN** the comments section scrolls independently of the task description
- **AND** a visual scroll indicator appears on the right edge of the comment panel
- **AND** when navigating with j/k, the panel auto-scrolls to keep the selected comment visible
- **AND** auto-scroll works in both top-level view and thread view

#### Scenario: Empty top-level state
- **WHEN** task has no top-level comments
- **THEN** a message "No comments yet. Press 'n' to add one." is displayed
- **AND** the comment input form is accessible

### Requirement: Comment creation
The system SHALL allow users to create new comments on tasks via a text input form. The form SHALL support multi-line markdown input. New comments SHALL always be created as top-level comments (parent_id = None) unless explicitly created as a reply in thread view.

#### Scenario: New top-level comment accessible
- **WHEN** user presses 'n' in task detail view (top-level or thread view)
- **THEN** a multi-line text input form appears at the bottom of the comments section
- **AND** placeholder text "New comment (markdown supported)..." is shown
- **AND** focus moves to the input field
- **AND** the comment will be created with parent_id = None

#### Scenario: Reply to thread accessible
- **WHEN** user is in thread view
- **AND** user presses 'r'
- **THEN** a reply form appears with context "Replying to {author}..."
- **AND** focus moves to the input field
- **AND** the reply will be created with parent_id = selected thread's parent comment id

#### Scenario: Comment input validation
- **WHEN** user attempts to save a comment or reply with empty or whitespace-only content
- **THEN** an error message "Comment cannot be empty" is displayed
- **AND** the form remains open for editing

#### Scenario: Comment saved successfully
- **WHEN** user presses Ctrl+S with valid comment content
- **THEN** the comment is sent to the ClickUp API with appropriate parent_id
- **AND** a "Saving..." indicator is shown
- **AND** on success, the comment appears in the appropriate view (top-level or thread)
- **AND** the input form is closed
- **AND** status bar shows "Comment added" or "Reply added"

### Requirement: Comment navigation
The system SHALL support keyboard navigation through the comments list with automatic scroll management. Navigation SHALL include entering and exiting comment threads.

#### Scenario: Navigate comments with j/k with auto-scroll
- **WHEN** focus is on the comments section
- **THEN** pressing 'j' moves selection to the next comment (down)
- **AND** pressing 'k' moves selection to the previous comment (up)
- **AND** if the selected comment scrolls out of the visible area, the panel automatically scrolls to keep it visible
- **AND** navigation works in both top-level view and thread view

#### Scenario: Enter thread view
- **WHEN** a top-level comment is selected in top-level view
- **AND** user presses Enter
- **THEN** the panel switches to thread view for that comment
- **AND** the breadcrumb updates to show "Comments > {author}"
- **AND** status bar shows "'r' reply, Esc go back"

#### Scenario: Exit thread view
- **WHEN** user is in thread view
- **AND** user presses Esc
- **THEN** the panel returns to top-level view
- **AND** the breadcrumb returns to "Comments"
- **AND** the parent comment remains selected

#### Scenario: Focus switching with Tab
- **WHEN** user presses Tab in task detail view
- **THEN** focus switches between task form fields and comments section
- **AND** the focused section is visually indicated (border highlight or cursor)
- **AND** Tab works in both top-level and thread view

### Requirement: Comment API integration
The system SHALL fetch, create, and update comments via the ClickUp API. API calls SHALL include parent_id field for threaded comments. API responses SHALL be parsed to extract parent_id relationships.

#### Scenario: Comments fetched with parent_id
- **WHEN** a task detail view is opened
- **THEN** a request is made to fetch comments for that task
- **AND** the API response includes parent_id for each comment (if available)
- **AND** comments are cached with their parent_id relationships
- **AND** a "Loading comments..." indicator is shown

#### Scenario: Comment created with parent_id
- **WHEN** user creates a new top-level comment
- **THEN** a POST request is made to the ClickUp API endpoint
- **AND** the request includes parent_id = None (or omitted if API doesn't support)
- **AND** the response is parsed and the new comment is added to top-level list

#### Scenario: Reply created with parent_id
- **WHEN** user creates a reply in thread view
- **THEN** a POST request is made to the ClickUp API endpoint
- **AND** the request includes parent_id = parent comment's id
- **AND** the response is parsed and the reply is added to the thread view
- **AND** the parent comment's reply count is incremented

#### Scenario: API error handling for threaded comments
- **WHEN** an API call for comments fails (network error, auth error, rate limit)
- **THEN** an appropriate error message is displayed
- **AND** the error is logged via tracing
- **AND** the user can retry the operation
- **AND** if parent_id is not supported by API, a warning is logged and comments are treated as flat list
