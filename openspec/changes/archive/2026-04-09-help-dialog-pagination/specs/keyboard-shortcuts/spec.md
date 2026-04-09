## MODIFIED Requirements

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
