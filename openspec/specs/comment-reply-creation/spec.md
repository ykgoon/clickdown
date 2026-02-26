## ADDED Requirements

### Requirement: Reply form activation in thread view
The system SHALL activate the comment input form when the user presses 'r' in thread view. The form SHALL be ready for immediate text input with clear visual feedback.

#### Scenario: Reply form opened with 'r' key
- **WHEN** user is in thread view (CommentViewMode::InThread)
- **AND** focus is on the comments section
- **AND** user presses 'r'
- **THEN** the comment input form is activated at the bottom of the comments panel
- **AND** `comment_new_text` is cleared for fresh input
- **AND** `comment_editing_index` is set to None (new comment mode)
- **AND** focus moves to the input field
- **AND** status bar shows "Replying to {author} (Ctrl+S save, Esc cancel)"

#### Scenario: Reply form shows author context
- **WHEN** reply form is opened
- **THEN** the status message includes the parent comment author's name
- **AND** the author name is truncated if too long (max 20 chars)

#### Scenario: Reply not available in top-level view
- **WHEN** user is in top-level view (CommentViewMode::TopLevel)
- **AND** user presses 'r'
- **THEN** no form is opened
- **AND** status bar shows "Press Enter to view thread, then 'r' to reply"

### Requirement: Reply input validation
The system SHALL validate reply content before submission to prevent empty or invalid replies.

#### Scenario: Empty reply rejected
- **WHEN** user presses Ctrl+S with empty reply text
- **THEN** reply is NOT sent to API
- **AND** status bar shows "Reply cannot be empty"
- **AND** input form remains open for editing

#### Scenario: Whitespace-only reply rejected
- **WHEN** user presses Ctrl+S with whitespace-only text (spaces, tabs, newlines)
- **THEN** reply is NOT sent to API
- **AND** status bar shows "Reply cannot be empty"
- **AND** input form remains open for editing

#### Scenario: Valid reply accepted
- **WHEN** user enters non-empty text
- **AND** presses Ctrl+S
- **THEN** reply is sent to API for creation

### Requirement: Reply form cancellation
The system SHALL allow users to cancel reply creation without saving.

#### Scenario: Reply cancelled with Esc
- **WHEN** user is editing a reply
- **AND** presses Esc
- **THEN** input form is closed
- **AND** `comment_new_text` is cleared
- **AND** `comment_editing_index` remains None
- **AND** status bar shows "Reply cancelled"
- **AND** thread view remains open

#### Scenario: Reply discarded without saving
- **WHEN** user types text in reply form
- **AND** presses Esc without saving
- **THEN** the typed text is discarded
- **AND** no API call is made

### Requirement: Reply creation state management
The system SHALL manage application state correctly during reply creation to provide smooth UX.

#### Scenario: Loading state during reply creation
- **WHEN** reply is submitted (Ctrl+S with valid text)
- **THEN** `loading` flag is set to true
- **AND** status bar shows "Saving reply..."
- **AND** input form remains open until response received

#### Scenario: Reply added to thread view on success
- **WHEN** API returns successful reply creation
- **THEN** reply is inserted into `self.comments` vector
- **AND** reply appears in thread view immediately
- **AND** `comment_new_text` is cleared
- **AND** `comment_editing_index` is set to None
- **AND** status bar shows "Reply added"
- **AND** `loading` flag is set to false

#### Scenario: Error handling on reply creation failure
- **WHEN** API returns error during reply creation
- **THEN** error message is displayed in status bar
- **AND** input form remains open for retry
- **AND** `loading` flag is set to false
- **AND** error is logged via tracing

### Requirement: Parent ID assignment for replies
The system SHALL correctly assign the parent_comment_id when creating a reply to establish thread hierarchy.

#### Scenario: Parent ID set from thread context
- **WHEN** reply is created in thread view
- **THEN** `parent_id` is set to the parent_comment_id from CommentViewMode::InThread
- **AND** CreateCommentRequest includes this parent_id

#### Scenario: Parent ID preserved during async operation
- **WHEN** reply creation is sent to async task
- **THEN** parent_id is captured and preserved in the request
- **AND** view mode changes during async operation don't affect the reply
