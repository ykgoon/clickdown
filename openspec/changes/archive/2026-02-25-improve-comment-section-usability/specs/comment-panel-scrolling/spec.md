## ADDED Requirements

### Requirement: Comment panel height ratio
The system SHALL allocate vertical space in the task detail view using a 3:7 ratio between the description panel and the comment panel. The comment panel SHALL receive 70% of the available vertical space after accounting for the title bar and status bar.

#### Scenario: Layout renders with correct ratio
- **WHEN** the task detail view is displayed on a terminal with sufficient height (≥24 rows)
- **THEN** the description panel occupies approximately 30% of the available content area
- **AND** the comment panel occupies approximately 70% of the available content area
- **AND** the ratio is maintained when the terminal is resized

#### Scenario: Minimum height enforcement
- **WHEN** the terminal height is below the minimum threshold (24 rows)
- **THEN** a warning message is displayed indicating the minimum size requirement
- **AND** the layout gracefully degrades to a stacked arrangement if possible

### Requirement: Independent panel scrolling
The system SHALL enable independent scrolling for both the description panel and the comment panel. Each panel SHALL maintain its own scroll offset and scroll state.

#### Scenario: Description panel scrolls independently
- **WHEN** description content exceeds the description panel's allocated space
- **THEN** the description panel becomes scrollable
- **AND** scrolling the description does not affect the comment panel's scroll position
- **AND** a visual scroll indicator appears on the description panel

#### Scenario: Comment panel scrolls independently
- **WHEN** comment content exceeds the comment panel's allocated space
- **THEN** the comment panel becomes scrollable
- **AND** scrolling the comments does not affect the description panel's scroll position
- **AND** a visual scroll indicator appears on the comment panel

#### Scenario: Scroll indicators visible
- **WHEN** a panel has content that extends beyond its visible area
- **THEN** a scroll indicator (scroll bar or arrow) is displayed on the right edge of the panel
- **AND** the indicator shows the relative position within the scrollable content
- **AND** the indicator updates as the user scrolls

### Requirement: Auto-scroll on comment selection
The system SHALL automatically adjust the comment panel's scroll position when keyboard navigation moves the selection to a comment that is outside the visible area.

#### Scenario: Auto-scroll when navigating down
- **WHEN** the user presses 'j' to navigate to the next comment
- **AND** the selected comment is below the visible area
- **THEN** the comment panel automatically scrolls down to make the selected comment visible
- **AND** the scroll animation is smooth (if supported by the terminal)

#### Scenario: Auto-scroll when navigating up
- **WHEN** the user presses 'k' to navigate to the previous comment
- **AND** the selected comment is above the visible area
- **THEN** the comment panel automatically scrolls up to make the selected comment visible
- **AND** the scroll animation is smooth (if supported by the terminal)

#### Scenario: No auto-scroll when selection is visible
- **WHEN** the selected comment is already within the visible area
- **THEN** the scroll position remains unchanged
- **AND** no unnecessary scrolling occurs

#### Scenario: Boundary behavior
- **WHEN** the user navigates past the last comment with 'j'
- **THEN** the selection wraps to the first comment OR stays at the last comment (based on existing navigation pattern)
- **AND** when navigating past the first comment with 'k'
- **THEN** the selection wraps to the last comment OR stays at the first comment

## MODIFIED Requirements

The following requirements from the `task-comments` specification are modified to include the new scrolling and layout behavior:

### Requirement: Comments scroll independently
**MODIFIED**: The system SHALL provide independent scrolling for the comments section when the comments list exceeds available vertical space. The comments panel SHALL maintain its own scroll offset separate from the description panel.

#### Scenario: Comments section scrolls independently
- **WHEN** comments list exceeds the comment panel's allocated vertical space
- **THEN** the comment panel scrolls independently of the task description panel
- **AND** scroll position is indicated with a visual scroll bar on the right edge
- **AND** the scroll position persists while navigating other task fields

#### Scenario: Auto-scroll keeps selection visible
- **WHEN** user navigates comments with j/k keys
- **AND** the selected comment moves outside the visible area
- **THEN** the comment panel automatically scrolls to keep the selection visible
- **AND** the auto-scroll maintains the 3:7 panel ratio

### Requirement: Comment navigation
**MODIFIED**: The system SHALL support keyboard navigation through the comments list with automatic scroll management. Navigation SHALL trigger auto-scroll when the selected comment moves outside the visible bounds.

#### Scenario: Navigate comments with j/k with auto-scroll
- **WHEN** focus is on the comments section
- **THEN** pressing 'j' moves selection to the next comment (down)
- **AND** pressing 'k' moves selection to the previous comment (up)
- **AND** if the selected comment is outside the visible area, the panel auto-scrolls
- **AND** the scroll position updates to show the selected comment

#### Scenario: Comment selection highlighted with scrolling
- **WHEN** a comment is selected
- **THEN** it is highlighted with a different background color or border
- **AND** the highlight remains visible as the panel scrolls
- **AND** the selection is preserved when switching focus away and back
