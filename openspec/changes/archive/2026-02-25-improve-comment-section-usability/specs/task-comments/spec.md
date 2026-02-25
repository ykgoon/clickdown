## MODIFIED Requirements

The following requirements from the existing `task-comments` specification are modified to include the new panel height ratio and independent scrolling behavior.

### Requirement: Task comments display
**MODIFIED**: The system SHALL display a list of comments for each task in the task detail view. The comments section SHALL occupy 70% of the vertical space in the task detail panel (3:7 ratio with description). Comments SHALL be shown in reverse chronological order (newest first) with proper text wrapping and independent scrolling.

#### Scenario: Comments section rendered with correct layout
- **WHEN** task detail view is displayed
- **THEN** a comments section is visible below the task description
- **AND** the comment panel occupies 70% of the available vertical space
- **AND** the description panel occupies 30% of the available vertical space
- **AND** comments are listed with most recent first
- **AND** each comment shows author name, timestamp, and content

#### Scenario: Comments scroll independently with auto-scroll
- **WHEN** comments list exceeds the comment panel's available vertical space
- **THEN** the comments section scrolls independently of the task description
- **AND** a visual scroll indicator appears on the right edge of the comment panel
- **AND** when navigating with j/k, the panel auto-scrolls to keep the selected comment visible

### Requirement: Comment navigation
**MODIFIED**: The system SHALL support keyboard navigation through the comments list with automatic scroll management. Navigation SHALL maintain the selected comment's visibility through auto-scroll behavior.

#### Scenario: Navigate comments with j/k with auto-scroll
- **WHEN** focus is on the comments section
- **THEN** pressing 'j' moves selection to the next comment (down)
- **AND** pressing 'k' moves selection to the previous comment (up)
- **AND** if the selected comment scrolls out of the visible area, the panel automatically scrolls to keep it visible
- **AND** the auto-scroll behavior is smooth and predictable

#### Scenario: Scroll position indicated
- **WHEN** the comment list is scrollable
- **THEN** a scroll bar or indicator is displayed on the right edge of the comment panel
- **AND** the indicator shows the current position within the full comment list
- **AND** the indicator updates as the user scrolls or navigates
