## ADDED Requirements

### Requirement: Page scrolling for overflowing lists
Item lists that overflow their viewport SHALL scroll by a full page with `Ctrl+F`/`PageDown` (down) and `Ctrl+U`/`PageUp` (up).

#### Scenario: Task list page down
- **GIVEN** a task list with more tasks than fit on screen
- **WHEN** the user presses `Ctrl+F` or `PageDown`
- **THEN** the selection moves down by one viewport page (never onto a header row)

#### Scenario: Comments page up
- **GIVEN** the comment panel focused with more comments than fit
- **WHEN** the user presses `Ctrl+U` or `PageUp`
- **THEN** the selection moves up by one viewport page

#### Scenario: Expanded panel page scroll
- **WHEN** the description or comments panel is expanded full-screen and the user presses `Ctrl+F`/`Ctrl+U`
- **THEN** the panel scroll offset moves by a page of lines

#### Scenario: Document page scroll
- **WHEN** the document screen shows overflowing content and the user presses `Ctrl+F`/`Ctrl+U`
- **THEN** the document scrolls by a page
