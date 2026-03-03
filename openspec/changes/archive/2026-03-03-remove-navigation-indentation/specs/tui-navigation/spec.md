## MODIFIED Requirements

### Requirement: Workspace hierarchy navigation
The system SHALL support navigation through the workspace hierarchy (Workspaces → Spaces → Folders → Lists) using keyboard. Navigation items SHALL render without indentation to preserve horizontal screen space. Hierarchy level SHALL be indicated by type labels or icons rather than visual indentation.

#### Scenario: Navigate into workspace
- **WHEN** user selects a workspace and presses `Enter`
- **THEN** spaces within that workspace are displayed
- **AND** all items render flush-left without indentation

#### Scenario: Navigate into space
- **WHEN** user selects a space and presses `Enter`
- **THEN** folders within that space are displayed
- **AND** all items render flush-left without indentation

#### Scenario: Navigate into folder
- **WHEN** user selects a folder and presses `Enter`
- **THEN** lists within that folder are displayed
- **AND** all items render flush-left without indentation

#### Scenario: Navigate into list
- **WHEN** user selects a list and presses `Enter`
- **THEN** tasks within that list are displayed
- **AND** all items render flush-left without indentation

#### Scenario: Go back in hierarchy
- **WHEN** user presses `Esc` or `Backspace`
- **THEN** parent level in hierarchy is displayed
- **AND** items at the parent level render flush-left without indentation

#### Scenario: Hierarchy level identifiable without indentation
- **WHEN** viewing items at any hierarchy level
- **THEN** the type of each item (workspace/space/folder/list) is identifiable via type label or icon prefix
- **AND** horizontal space is preserved for displaying item names and metadata
