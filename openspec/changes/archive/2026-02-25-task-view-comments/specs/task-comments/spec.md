## ADDED Requirements

### Requirement: Task comments display
The system SHALL display a list of comments for each task in the task detail view. Comments SHALL be shown in reverse chronological order (newest first) with proper text wrapping.

#### Scenario: Comments section rendered
- **WHEN** task detail view is displayed and comments are loaded
- **THEN** a comments section is visible below the task description
- **AND** comments are listed with most recent first
- **AND** each comment shows author name, timestamp, and content

#### Scenario: Comment text wrapping
- **WHEN** comment text exceeds the widget width
- **THEN** text wraps to the next line without horizontal overflow
- **AND** no content is hidden or requires horizontal scrolling

#### Scenario: Comment metadata displayed
- **WHEN** a comment is rendered
- **THEN** the following metadata is shown:
  - Commenter username
  - Creation date (formatted as "Created: YYYY-MM-DD HH:MM")
  - Updated indicator if comment was edited ("(edited)")

#### Scenario: Empty comments state
- **WHEN** task has no comments
- **THEN** a message "No comments yet. Press 'n' to add one." is displayed
- **AND** the comment input form is accessible

#### Scenario: Comments scroll independently
- **WHEN** comments list exceeds available vertical space
- **THEN** comments section scrolls independently of task detail form
- **AND** scroll position is indicated with a scroll bar or line numbers

### Requirement: Comment creation
The system SHALL allow users to create new comments on tasks via a text input form. The form SHALL support multi-line markdown input.

#### Scenario: New comment form accessible
- **WHEN** user presses 'n' in task detail view
- **THEN** a multi-line text input form appears at the bottom of the comments section
- **AND** placeholder text "Write a comment (markdown supported)..." is shown
- **AND** focus moves to the input field

#### Scenario: Comment input validation
- **WHEN** user attempts to save a comment with empty or whitespace-only content
- **THEN** an error message "Comment cannot be empty" is displayed
- **AND** the form remains open for editing

#### Scenario: Comment saved successfully
- **WHEN** user presses Ctrl+s with valid comment content
- **THEN** the comment is sent to the ClickUp API
- **AND** a "Saving..." indicator is shown
- **AND** on success, the comment appears at the top of the comments list
- **AND** the input form is closed
- **AND** status bar shows "Comment added"

#### Scenario: Comment save error
- **WHEN** API call to create comment fails
- **THEN** an error message is displayed in the status bar
- **AND** the form remains open with content preserved
- **AND** user can retry or cancel

#### Scenario: Comment creation cancelled
- **WHEN** user presses Esc while editing a new comment
- **THEN** a confirmation dialog appears "Discard comment?"
- **AND** on "Yes", the form is closed and content is discarded
- **AND** on "No", editing continues

### Requirement: Comment editing
The system SHALL allow users to edit their own comments. Comments created by other users SHALL be read-only.

#### Scenario: Edit action available
- **WHEN** user navigates to their own comment with j/k keys
- **THEN** the status bar shows "Press 'e' to edit, 'n' for new comment"
- **AND** the comment is visually indicated as selectable (highlight on focus)

#### Scenario: Edit action restricted
- **WHEN** user navigates to another user's comment
- **THEN** the status bar shows "View only. Press 'n' for new comment"
- **AND** pressing 'e' shows a message "You can only edit your own comments"

#### Scenario: Comment edit form opened
- **WHEN** user presses 'e' on their own selected comment
- **THEN** an inline edit form appears with the comment text pre-filled
- **AND** focus moves to the input field
- **AND** the original comment is temporarily hidden

#### Scenario: Comment edit saved
- **WHEN** user presses Ctrl+s after editing
- **THEN** the updated comment is sent to the ClickUp API
- **AND** a "Saving..." indicator is shown
- **AND** on success, the comment is updated in the list with "(edited)" indicator
- **AND** the edit form is closed
- **AND** status bar shows "Comment updated"

#### Scenario: Comment edit cancelled
- **WHEN** user presses Esc while editing
- **THEN** a confirmation dialog appears "Discard changes?"
- **AND** on "Yes", the form is closed and original comment is restored
- **AND** on "No", editing continues

### Requirement: Comment navigation
The system SHALL support keyboard navigation through the comments list. Navigation SHALL be consistent with vim-style keybindings.

#### Scenario: Navigate comments with j/k
- **WHEN** focus is on the comments section
- **THEN** pressing 'j' moves selection to the next comment (down)
- **AND** pressing 'k' moves selection to the previous comment (up)
- **AND** selection wraps at boundaries (optional, based on existing pattern)

#### Scenario: Focus switching with Tab
- **WHEN** user presses Tab in task detail view
- **THEN** focus switches between task form fields and comments section
- **AND** the focused section is visually indicated (border highlight or cursor)

#### Scenario: Comment selection highlighted
- **WHEN** a comment is selected
- **THEN** it is highlighted with a different background color or border
- **AND** the selection is visible even when scrolling

#### Scenario: Scroll to keep selection visible
- **WHEN** selected comment scrolls out of visible area
- **THEN** the comments list automatically scrolls to keep selection visible
- **AND** the scroll animation is smooth (if supported by terminal)

### Requirement: Comment API integration
The system SHALL fetch, create, and update comments via the ClickUp API. API calls SHALL follow existing patterns for error handling and caching.

#### Scenario: Comments fetched on task load
- **WHEN** a task detail view is opened
- **THEN** a request is made to fetch comments for that task
- **AND** a "Loading comments..." indicator is shown
- **AND** on success, comments are displayed in the comments section
- **AND** on error, an error message is shown with retry option

#### Scenario: Comments cached locally
- **WHEN** comments are fetched from the API
- **THEN** they are stored in the SQLite cache with timestamp
- **AND** subsequent requests within 5 minutes use cached data
- **AND** cache is invalidated after 5 minutes

#### Scenario: Comment created via API
- **WHEN** user creates a new comment
- **THEN** a POST request is made to the ClickUp API endpoint
- **AND** the request includes the task ID and comment text
- **AND** the response is parsed and the new comment is added to the list

#### Scenario: Comment updated via API
- **WHEN** user edits an existing comment
- **THEN** a PUT request is made to the ClickUp API endpoint
- **AND** the request includes the comment ID and updated text
- **AND** the response is parsed and the comment is updated in the list

#### Scenario: API error handling
- **WHEN** an API call for comments fails (network error, auth error, rate limit)
- **THEN** an appropriate error message is displayed
- **AND** the error is logged via tracing
- **AND** the user can retry the operation

### Requirement: Comment markdown rendering
The system SHALL render markdown formatting in comment content. Rendering SHALL be consistent with document markdown rendering.

#### Scenario: Basic markdown rendered
- **WHEN** comment contains markdown formatting
- **THEN** the following are rendered correctly:
  - Bold (**text** or __text__)
  - Italic (*text* or _text_)
  - Inline code (`code`)
  - Links ([text](url))
  - Lists (- item or 1. item)

#### Scenario: Code blocks rendered
- **WHEN** comment contains code blocks (```language ... ```)
- **THEN** code is displayed in monospace font
- **AND** syntax highlighting is applied if available
- **AND** code blocks are scrollable if they exceed widget width

#### Scenario: Long URLs wrapped
- **WHEN** comment contains a long URL
- **THEN** the URL is wrapped at word boundaries or with hyphenation
- **AND** the URL does not cause horizontal overflow

#### Scenario: Markdown input preview
- **WHEN** user is editing a comment
- **THEN** a hint "Markdown supported" is shown
- **AND** (future) a preview toggle could show rendered markdown
