## Purpose

The keyboard-shortcuts capability defines global keyboard shortcuts that provide quick access to common application actions. These shortcuts work across all application states and follow desktop application conventions.
## Requirements
### Requirement: Ctrl-Q quits the application
The system SHALL allow users to quit the application by pressing `ctrl-q` keyboard shortcut. The shortcut MUST work reliably across different keyboard layouts and operating systems.

#### Scenario: User presses ctrl-q
- **WHEN** user presses `ctrl` and `q` keys simultaneously
- **THEN** the application exits immediately without confirmation dialog

#### Scenario: Ctrl-Q works with non-US keyboard layouts
- **WHEN** user presses `ctrl` and `q` keys on a non-US keyboard layout
- **THEN** the application exits regardless of keyboard layout mapping

#### Scenario: Ctrl-Q works with both character and named key events
- **WHEN** the system generates either `Key::Character("q")` or `Key::Named(Named::KeyQ)` event
- **THEN** both event types trigger application exit when ctrl modifier is pressed

#### Scenario: Ctrl-Q does not interfere with normal typing
- **WHEN** user types the letter 'q' without holding ctrl
- **THEN** the application continues running normally

#### Scenario: Ctrl-Q works in all application states
- **WHEN** user presses `ctrl-q` during any application state (loading, authenticated, unauthenticated)
- **THEN** the application exits in all states

### Requirement: Question mark toggles help dialog
The system SHALL allow users to toggle the help dialog by pressing the `?` key. This shortcut SHALL work globally across all application states and screens.

#### Scenario: User presses question mark to open help
- **WHEN** user presses `?` key
- **THEN** the help dialog opens showing all keyboard shortcuts

#### Scenario: User presses question mark to close help
- **WHEN** the help dialog is visible AND user presses `?` key
- **THEN** the help dialog closes

#### Scenario: Question mark works in all states
- **WHEN** user presses `?` during any application state (loading, authenticated, unauthenticated)
- **THEN** the help dialog toggles appropriately

#### Scenario: Question mark does not interfere with other input
- **WHEN** user is typing in a form field AND presses `?`
- **THEN** the `?` character is entered in the field instead of toggling help

### Requirement: Any key closes help dialog
The system SHALL close the help dialog when the user presses `Esc` or `?`. Other keypresses while the dialog is visible SHALL NOT close the dialog. The `j`, `k`, `↑`, `↓`, `←`, and `→` keys SHALL navigate between help dialog pages instead of closing it.

#### Scenario: Pressing Esc closes help
- **WHEN** the help dialog is visible AND user presses `Esc`
- **THEN** the help dialog closes without processing the key

#### Scenario: Pressing ? closes help
- **WHEN** the help dialog is visible AND user presses `?`
- **THEN** the help dialog closes (toggles off)

#### Scenario: Pressing j/k navigates pages instead of closing
- **WHEN** the help dialog is visible AND user presses `j` or `k`
- **THEN** the help dialog advances to the next/previous page AND does not close

#### Scenario: Pressing navigation keys does not close help
- **WHEN** the help dialog is visible AND user presses `Enter`, `n`, `e`, `d`, or other non-navigation keys
- **THEN** the help dialog remains open AND no action is taken

### Requirement: A opens assignee picker from task detail
The system SHALL allow users to open the assignee picker overlay by pressing the `A` key while on the task detail screen. This shortcut SHALL only be active when a task is selected and the task's parent list ID is known in application state.

#### Scenario: User presses A on task detail
- **WHEN** user is viewing task detail
- **AND** presses `A`
- **AND** the task's parent list ID is available in application state
- **THEN** the assignee picker overlay opens

#### Scenario: A is inactive without list context
- **WHEN** user is viewing task detail
- **AND** presses `A`
- **AND** the task's parent list ID is NOT available
- **THEN** the assignee picker does not open
- **AND** a status message indicates list context is not available

#### Scenario: A does not trigger in other screens
- **WHEN** user is on the task list screen (not task detail)
- **AND** presses `A`
- **THEN** no action is taken
- **AND** no error is shown

### Requirement: S opens status picker from task detail
The system SHALL allow users to open the status picker overlay by pressing the `s` key while on the task detail screen. This shortcut SHALL only be active when a task is loaded in the task detail view. The behavior SHALL be consistent with the `s` key behavior in the task list view (opens status picker for selected task).

#### Scenario: User presses s on task detail with task loaded
- **WHEN** user is viewing task detail with a task loaded
- **AND** presses `s`
- **THEN** the status picker overlay opens showing available status options
- **AND** the current task's status is indicated in the picker

#### Scenario: S is inactive without task loaded
- **WHEN** user is on task detail screen
- **AND** no task is loaded in the detail view
- **AND** presses `s`
- **THEN** the status picker does not open
- **AND** a status message indicates no task is selected

#### Scenario: S does not interfere with comment typing
- **WHEN** user is actively typing text in the comment input field
- **AND** presses `s`
- **THEN** the letter 's' is entered into the comment text
- **AND** the status picker does not open

### Requirement: Keyboard shortcut chord support
The system SHALL support two-key chord shortcuts where the first key acts as a prefix leader. After the leader key is pressed, a brief pending state awaits the second key. If the second key matches the expected chord completion, the associated action is triggered. If a non-matching key is pressed, the second key is processed as normal input.

#### Scenario: g then u opens URL navigation
- **WHEN** the user presses `g`
- **AND** then presses `u` within the chord timeout
- **THEN** the URL input dialog opens

#### Scenario: g then non-u passes through second key
- **WHEN** the user presses `g`
- **AND** then presses `j` (not part of a `g` chord)
- **THEN** the `j` key is processed as normal navigation input (scroll down)

#### Scenario: Chord pending state times out
- **WHEN** the user presses `g`
- **AND** no second key is pressed within the chord timeout window
- **THEN** the pending state is cleared
- **AND** the next key is processed as normal input

#### Scenario: Esc cancels chord pending state
- **WHEN** the user presses `g`
- **AND** then presses `Esc`
- **THEN** the pending state is cleared
- **AND** no action is taken

