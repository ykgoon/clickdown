## Why

Users navigating deep workspace hierarchies (Workspaces → Spaces → Folders → Lists) lose valuable horizontal screen real estate to indentation. Terminal screens are typically wider than they are tall, and preserving horizontal space is critical for displaying task titles, status indicators, and other metadata without truncation.

## What Changes

- Remove left-margin indentation for items at deeper hierarchy levels in the navigation sidebar
- All items display flush-left regardless of their depth in the hierarchy
- Hierarchy level is still indicated by icon or text prefix (e.g., workspace/space/folder/list labels)
- Visual separation between levels maintained through spacing or dividers instead of indentation

## Capabilities

### New Capabilities
- `navigation-layout`: Controls visual layout of navigation items, including indentation behavior and hierarchy indicators

### Modified Capabilities
- `tui-navigation`: Updated to specify that navigation items render without indentation to preserve horizontal screen space

## Impact

- **Modified**: `src/tui/widgets/sidebar.rs` - Remove indentation logic in rendering
- **Modified**: `src/tui/layout.rs` - May adjust sidebar width calculations
- **Unchanged**: Navigation logic, hierarchy traversal, keyboard bindings
- **User-facing**: More horizontal space for task titles and metadata in sidebar
