## Why

The status bar at the bottom of the TUI has limited space, but ClickDown is adding more keyboard shortcuts over time. Displaying all shortcuts in the status bar is no longer feasible. Users need a way to discover and reference available keyboard shortcuts without cluttering the interface.

## What Changes

- **New**: A help dialog that displays all available keyboard shortcuts
- **New**: `?` key toggles the help dialog (open/close)
- **Modified**: Status bar will show `?` hint to indicate help is available
- **New**: Help dialog with organized sections for different shortcut categories
- **New**: Modal dialog behavior (blocks interaction with underlying UI until closed)

## Capabilities

### New Capabilities
- `help-dialog`: A modal dialog component that displays keyboard shortcuts in an organized, scrollable interface with categories for navigation, task management, and application controls

### Modified Capabilities
- `keyboard-shortcuts`: Adding new shortcut for toggling help dialog, expanding beyond just Ctrl-Q
- `tui-widgets`: Adding new modal dialog widget type
- `tui-layouts`: Adding centered modal overlay layout pattern

## Impact

- **TUI rendering**: New modal overlay rendering that dims/blocks background content
- **Input handling**: Global `?` shortcut that works across all states
- **Status bar**: Added `?` hint to all screens
- **Dependencies**: ratatui for modal rendering, crossterm for input handling
- **No breaking changes**: Existing shortcuts continue to work
