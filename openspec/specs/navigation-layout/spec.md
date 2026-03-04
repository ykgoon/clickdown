## ADDED Requirements

### Requirement: Navigation items render without indentation
The system SHALL render all navigation sidebar items flush-left without indentation, regardless of their depth in the workspace hierarchy. The inbox navigation item SHALL be rendered at the top level as a standalone item.

#### Scenario: Workspace items display flush-left
- **WHEN** the sidebar displays workspace items
- **THEN** workspace names are rendered at the left edge with no leading spaces

#### Scenario: Space items display flush-left
- **WHEN** the sidebar displays space items
- **THEN** space names are rendered at the left edge with no leading spaces or indentation

#### Scenario: Folder items display flush-left
- **WHEN** the sidebar displays folder items
- **THEN** folder names are rendered at the left edge with no leading spaces or indentation

#### Scenario: List items display flush-left
- **WHEN** the sidebar displays list items
- **THEN** list names are rendered at the left edge with no leading spaces or indentation

#### Scenario: Inbox item displays flush-left
- **WHEN** the sidebar displays the inbox navigation item
- **THEN** the inbox item is rendered at the left edge with no leading spaces or indentation

### Requirement: Hierarchy level indicated by type label
The system SHALL display a type label or icon prefix for each navigation item to indicate its hierarchy level without using indentation. The inbox item SHALL have a distinct icon to indicate its purpose.

#### Scenario: Workspace type label visible
- **WHEN** a workspace item is displayed
- **THEN** a workspace type indicator (e.g., "WS" or workspace icon) precedes the name

#### Scenario: Space type label visible
- **WHEN** a space item is displayed
- **THEN** a space type indicator (e.g., "SP" or space icon) precedes the name

#### Scenario: Folder type label visible
- **WHEN** a folder item is displayed
- **THEN** a folder type indicator (e.g., "FL" or folder icon) precedes the name

#### Scenario: List type label visible
- **WHEN** a list item is displayed
- **THEN** a list type indicator (e.g., "LI" or list icon) precedes the name

#### Scenario: Inbox type label visible
- **WHEN** the inbox navigation item is displayed
- **THEN** an inbox/mail icon (e.g., 📬 or "IN") precedes the "Inbox" label

### Requirement: Visual separation between hierarchy levels
The system SHALL provide visual separation between different hierarchy levels to maintain clarity without indentation. The inbox section SHALL be visually separated from the workspace hierarchy.

#### Scenario: Spacing between different types
- **WHEN** items of different types are adjacent (e.g., workspace followed by space)
- **THEN** a blank line or visual divider separates the groups

#### Scenario: Consistent item height
- **WHEN** navigation items are rendered
- **THEN** each item occupies the same vertical space regardless of hierarchy level

#### Scenario: Inbox section separated
- **WHEN** inbox item is displayed alongside workspace items
- **THEN** a visual divider or spacing separates inbox from workspace hierarchy
