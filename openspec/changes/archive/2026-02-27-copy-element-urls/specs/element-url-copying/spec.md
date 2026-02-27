## ADDED Requirements

### Requirement: URL generation for ClickUp elements
The system SHALL generate ClickUp web app URLs for all element types using the standard URL patterns. The URL generation SHALL be deterministic and based solely on element IDs.

#### Scenario: Generate workspace URL
- **WHEN** the system has a workspace with ID `workspace123`
- **THEN** the generated URL is `https://app.clickup.com/workspace123`

#### Scenario: Generate space URL
- **WHEN** the system has a space with ID `space456` in workspace `workspace123`
- **THEN** the generated URL is `https://app.clickup.com/workspace123/s/space456`

#### Scenario: Generate folder URL
- **WHEN** the system has a folder with ID `folder789` in workspace `workspace123`
- **THEN** the generated URL is `https://app.clickup.com/workspace123/f/folder789`

#### Scenario: Generate list URL
- **WHEN** the system has a list with ID `list012` in workspace `workspace123`
- **THEN** the generated URL is `https://app.clickup.com/workspace123/l/list012`

#### Scenario: Generate task URL
- **WHEN** the system has a task with ID `task345` in list `list012` within workspace `workspace123`
- **THEN** the generated URL is `https://app.clickup.com/workspace123/l/list012/t/task345`

#### Scenario: Generate comment URL
- **WHEN** the system has a comment with ID `comment678` on task `task345` in list `list012` within workspace `workspace123`
- **THEN** the generated URL is `https://app.clickup.com/workspace123/l/list012/t/task345/comment/comment678`

#### Scenario: Generate document URL
- **WHEN** the system has a document with ID `doc901` in workspace `workspace123`
- **THEN** the generated URL is `https://app.clickup.com/workspace123/d/doc901`

#### Scenario: URL generation handles missing context gracefully
- **WHEN** the system attempts to generate a URL without required context (e.g., task URL without list ID)
- **THEN** the URL generation returns `None` or an error indicating missing context

### Requirement: Copy URL to clipboard
The system SHALL copy the generated URL to the system clipboard when the user triggers the copy action. The clipboard operation SHALL work across all supported platforms (Linux, macOS, Windows).

#### Scenario: User copies URL successfully
- **WHEN** the user triggers the copy URL action in a view with valid context
- **THEN** the URL is copied to the system clipboard
- **AND** the clipboard content matches the generated URL exactly

#### Scenario: Copy URL on Linux with X11
- **WHEN** the user triggers the copy URL action on Linux with X11
- **THEN** the URL is copied to the CLIPBOARD selection (not PRIMARY)
- **AND** the URL can be pasted with Ctrl+V or middle-click in other applications

#### Scenario: Copy URL on Linux with Wayland
- **WHEN** the user triggers the copy URL action on Linux with Wayland
- **THEN** the URL is copied to the system clipboard
- **AND** the URL can be pasted in other applications

#### Scenario: Copy URL on macOS
- **WHEN** the user triggers the copy URL action on macOS
- **THEN** the URL is copied to the system clipboard
- **AND** the URL can be pasted with Cmd+V in other applications

#### Scenario: Copy URL on Windows
- **WHEN** the user triggers the copy URL action on Windows
- **THEN** the URL is copied to the system clipboard
- **AND** the URL can be pasted with Ctrl+V in other applications

#### Scenario: Clipboard unavailable (headless SSH)
- **WHEN** the user triggers the copy URL action in a headless SSH session without clipboard support
- **THEN** the system displays an error message: "Failed to copy URL: clipboard unavailable"
- **AND** the application continues running without crashing

#### Scenario: Clipboard unavailable (Wayland without portal)
- **WHEN** the user triggers the copy URL action on Wayland without clipboard portal access
- **THEN** the system displays an error message with the failure reason
- **AND** the application continues running without crashing

### Requirement: Keyboard shortcut for copying URL
The system SHALL provide a keyboard shortcut to trigger the copy URL action. The shortcut SHALL be `Ctrl+Shift+C` on Linux/Windows and `Cmd+Shift+C` on macOS.

#### Scenario: User presses Ctrl+Shift+C on Linux/Windows
- **WHEN** the user presses `Ctrl`, `Shift`, and `C` simultaneously
- **THEN** the URL for the current context is copied to clipboard

#### Scenario: User presses Cmd+Shift+C on macOS
- **WHEN** the user presses `Cmd`, `Shift`, and `C` simultaneously on macOS
- **THEN** the URL for the current context is copied to clipboard

#### Scenario: Copy URL with lowercase 'c'
- **WHEN** the user presses `Ctrl+Shift+c` (lowercase)
- **THEN** the URL is copied (case-insensitive for the letter key)

#### Scenario: Copy URL does not interfere with typing 'C'
- **WHEN** the user presses `C` without modifiers
- **THEN** no copy action is triggered
- **AND** normal input continues

#### Scenario: Copy URL does not interfere with Ctrl+C
- **WHEN** the user presses `Ctrl+C` without Shift
- **THEN** no copy URL action is triggered
- **AND** Ctrl+C retains its existing behavior (if any)

### Requirement: Context-aware URL copying
The system SHALL determine which URL to copy based on the current view context. The copied URL SHALL be the most relevant URL for the user's current location in the application.

#### Scenario: Copy URL from workspace list view
- **WHEN** the user is viewing the workspace list with a workspace selected
- **AND** the user triggers the copy URL action
- **THEN** the selected workspace's URL is copied

#### Scenario: Copy URL from space list view
- **WHEN** the user is viewing spaces within a workspace with a space selected
- **AND** the user triggers the copy URL action
- **THEN** the selected space's URL is copied

#### Scenario: Copy URL from folder list view
- **WHEN** the user is viewing folders within a space with a folder selected
- **AND** the user triggers the copy URL action
- **THEN** the selected folder's URL is copied

#### Scenario: Copy URL from list view
- **WHEN** the user is viewing lists within a folder with a list selected
- **AND** the user triggers the copy URL action
- **THEN** the selected list's URL is copied

#### Scenario: Copy URL from task list view
- **WHEN** the user is viewing tasks within a list with a task selected
- **AND** the user triggers the copy URL action
- **THEN** the selected task's URL is copied

#### Scenario: Copy URL from task detail view
- **WHEN** the user is viewing task details
- **AND** the user triggers the copy URL action
- **THEN** the current task's URL is copied

#### Scenario: Copy URL from comment thread view
- **WHEN** the user is viewing a comment thread with a specific comment selected
- **AND** the user triggers the copy URL action
- **THEN** the selected comment's URL (including task context) is copied

#### Scenario: Copy URL from document view
- **WHEN** the user is viewing a document
- **AND** the user triggers the copy URL action
- **THEN** the current document's URL is copied

#### Scenario: Copy URL without selection
- **WHEN** the user triggers the copy URL action without any item selected
- **THEN** a message is shown: "No item selected"
- **AND** no URL is copied

### Requirement: Visual feedback for URL copy
The system SHALL provide visual feedback when a URL is copied to confirm the action succeeded. The feedback SHALL be displayed in the status bar and SHALL be non-intrusive.

#### Scenario: Successful URL copy feedback
- **WHEN** the URL is successfully copied to clipboard
- **THEN** the status bar displays: "Copied: <URL>" (truncated to 60 characters if longer)
- **AND** the message is visible for at least 2 seconds or until next action

#### Scenario: URL truncation in feedback
- **WHEN** the copied URL is longer than 60 characters
- **THEN** the feedback message shows the first 57 characters followed by "..."
- **AND** the full URL is still copied to clipboard

#### Scenario: Failed URL copy feedback
- **WHEN** the clipboard operation fails
- **THEN** the status bar displays: "Failed to copy URL: <reason>"
- **AND** the error reason is specific (e.g., "clipboard unavailable", "access denied")

#### Scenario: Feedback does not block interaction
- **WHEN** the feedback message is displayed
- **THEN** the user can continue interacting with the application
- **AND** the message does not require dismissal

### Requirement: URL generation is testable
The system SHALL expose URL generation logic in a way that allows unit testing without clipboard access. The URL generation SHALL be pure (no side effects) and deterministic.

#### Scenario: Test workspace URL generation
- **WHEN** unit tests call the URL generator with a workspace ID
- **THEN** the correct URL is returned without clipboard interaction

#### Scenario: Test comment URL generation with full context
- **WHEN** unit tests call the URL generator with comment, task, list, and workspace IDs
- **THEN** the full comment URL is returned with all path segments

#### Scenario: Test URL generation error cases
- **WHEN** unit tests call the URL generator with missing required IDs
- **THEN** the generator returns an error or `None` indicating what's missing

#### Scenario: Test clipboard abstraction
- **WHEN** unit tests mock the clipboard interface
- **THEN** clipboard operations can be verified without actual system clipboard access
