## ADDED Requirements

### Requirement: Enter thread navigation
The system SHALL allow users to "enter" a top-level comment thread by pressing Enter. This action SHALL switch the comment panel from top-level view to thread view.

#### Scenario: Enter thread from top-level view
- **WHEN** user has focus on the comments section in top-level view
- **AND** a top-level comment is selected
- **AND** user presses Enter
- **THEN** the comment panel switches to thread view mode
- **AND** the breadcrumb updates to show "Comments > {author}"
- **AND** the selected comment's thread is displayed (parent + replies)
- **AND** the status bar shows "'r' reply, Esc go back"

#### Scenario: Enter on comment with no replies
- **WHEN** user presses Enter on a top-level comment with no replies
- **THEN** thread view opens showing only the parent comment
- **AND** a message "No replies yet" is displayed
- **AND** the reply form is accessible

#### Scenario: Enter on reply in thread view (no-op)
- **WHEN** user is already in thread view
- **AND** user presses Enter on a reply
- **THEN** no action occurs (Enter has no effect on replies)
- **AND** focus remains on the selected reply

### Requirement: Exit thread navigation
The system SHALL allow users to exit a thread view and return to top-level view by pressing Esc. This action SHALL preserve the previously selected top-level comment for easy re-entry.

#### Scenario: Exit thread with Esc
- **WHEN** user is in thread view
- **AND** user presses Esc
- **THEN** the comment panel returns to top-level view
- **AND** the breadcrumb returns to "Comments"
- **AND** the parent comment remains selected
- **AND** the status bar returns to top-level hints

#### Scenario: Exit thread preserves scroll position
- **WHEN** user exits a thread and re-enters it
- **THEN** the thread view shows the same scroll position as before
- **AND** the same reply remains selected (if any was selected)

#### Scenario: Esc in top-level view (no-op)
- **WHEN** user is in top-level view
- **AND** user presses Esc
- **THEN** no navigation occurs (Esc may have other functions like canceling forms)
- **AND** top-level view remains unchanged

### Requirement: Breadcrumb navigation context
The system SHALL display a breadcrumb in the comment panel title to indicate the current navigation context. The breadcrumb SHALL update dynamically when entering or exiting threads.

#### Scenario: Breadcrumb shows top-level context
- **WHEN** viewing top-level comments
- **THEN** the panel title shows "Comments"
- **AND** no breadcrumb suffix is present

#### Scenario: Breadcrumb shows thread context
- **WHEN** viewing a thread for a comment by user "John Doe"
- **THEN** the panel title shows "Comments > John Doe"
- **AND** the breadcrumb is truncated if username is too long (e.g., "Comments > John D...")

#### Scenario: Breadcrumb updates on thread switch
- **WHEN** user exits one thread and enters another
- **THEN** the breadcrumb updates to show the new thread's author
- **AND** the update is immediate (no flicker or delay)

### Requirement: Reply action in thread view
The system SHALL provide a "Reply" action (r key) that creates a nested reply to the current thread's parent comment. The reply form SHALL show context about which comment is being replied to.

#### Scenario: Reply form opened in thread view
- **WHEN** user is in thread view
- **AND** user presses 'r'
- **THEN** a reply form appears at the bottom of the comment panel
- **AND** the form shows "Replying to {parent_author}..." as context
- **AND** focus moves to the reply input field
- **AND** the status bar shows "Ctrl+S save, Esc cancel"

#### Scenario: Reply saved successfully
- **WHEN** user fills out the reply form and presses Ctrl+S
- **THEN** the reply is sent to the ClickUp API with parent_id set to the thread's parent comment id
- **AND** a "Saving..." indicator is shown
- **AND** on success, the reply appears in the thread view
- **AND** the reply form is closed
- **AND** status bar shows "Reply added"

#### Scenario: Reply cancelled
- **WHEN** user presses Esc while editing a reply
- **THEN** a confirmation dialog appears "Discard reply?"
- **AND** on "Yes", the form is closed and content is discarded
- **AND** on "No", editing continues

#### Scenario: Reply in top-level view (no-op)
- **WHEN** user is in top-level view
- **AND** user presses 'r'
- **THEN** no action occurs (reply only available in thread view)
- **AND** a hint message may appear "Press Enter to view thread, then 'r' to reply"

### Requirement: New comment action in top-level view
The system SHALL provide a "New comment" action (n key) that creates a top-level comment. This action SHALL always create a comment with parent_id = None, regardless of current view.

#### Scenario: New comment form in top-level view
- **WHEN** user is in top-level view
- **AND** user presses 'n'
- **THEN** a new comment form appears
- **AND** the form shows "New comment (markdown supported)..."
- **AND** focus moves to the input field
- **AND** the status bar shows "Ctrl+S save, Esc cancel"

#### Scenario: New comment saved
- **WHEN** user fills out the new comment form and presses Ctrl+S
- **THEN** the comment is sent to the ClickUp API with parent_id = None
- **AND** on success, the comment appears at the top of the top-level list
- **AND** the form is closed
- **AND** status bar shows "Comment added"

#### Scenario: New comment from thread view
- **WHEN** user is in thread view
- **AND** user presses 'n'
- **THEN** a new top-level comment form appears (not a reply)
- **AND** on save, the comment is added to top-level list
- **AND** the view returns to top-level to show the new comment

### Requirement: Keyboard navigation within thread view
The system SHALL support keyboard navigation (j/k) through replies in thread view. Navigation SHALL maintain the selected reply's visibility through auto-scroll behavior.

#### Scenario: Navigate replies with j/k
- **WHEN** focus is on the comments section in thread view
- **THEN** pressing 'j' moves selection to the next reply (down)
- **AND** pressing 'k' moves selection to the previous reply (up)
- **AND** if the selected reply scrolls out of the visible area, the panel automatically scrolls to keep it visible

#### Scenario: Navigate from parent to first reply
- **WHEN** parent comment is selected in thread view
- **AND** user presses 'j'
- **THEN** selection moves to the first reply
- **AND** the panel scrolls if needed to show the first reply

#### Scenario: Navigate from reply to parent
- **WHEN** first reply is selected in thread view
- **AND** user presses 'k'
- **THEN** selection moves to the parent comment
- **AND** the panel scrolls if needed to show the parent

### Requirement: Focus switching in thread view
The system SHALL support Tab key to switch focus between task form fields and the comments section, regardless of view mode (top-level or thread view).

#### Scenario: Tab into thread view comments
- **WHEN** focus is on task form fields
- **AND** user is in thread view
- **AND** user presses Tab
- **THEN** focus moves to the comments section (thread view)
- **AND** the first reply or parent comment is selected
- **AND** the focused section is visually indicated

#### Scenario: Tab out of thread view comments
- **WHEN** focus is on comments section in thread view
- **AND** user presses Tab
- **THEN** focus moves to the first task form field
- **AND** comment selection is preserved
