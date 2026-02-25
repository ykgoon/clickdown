# comment-panel-scrolling Specification

## Purpose
Defines the layout ratio and independent scrolling behavior for the description and comment panels in the task detail view.

## Requirements

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
