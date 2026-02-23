## Why

When reporting bugs, users and developers struggle to precisely identify which screen has an issue. This creates ambiguity and slows down bug triage and resolution. Adding unique screen IDs enables accurate screen references in bug reports.

## What Changes

- Each screen in the application displays a unique 4-character alphanumeric ID in the bottom-left corner
- IDs are rendered in small, unobtrusive text to avoid interfering with the UI
- Screen IDs are generated deterministically for consistency across sessions
- The ID display is visible in all major screens: authentication, workspace navigation, task list, task detail, and document views

## Capabilities

### New Capabilities
- `screen-identification`: Unique 4-character alphanumeric ID displayed on each screen for bug reporting reference

### Modified Capabilities
<!-- Existing capabilities whose REQUIREMENTS are changing -->

## Impact

- **UI Components**: All screen views need to render the screen ID overlay
- **Screen Module**: New utility to generate and manage screen IDs
- **Styling**: Minimal CSS/styling for unobtrusive ID display
- **No Breaking Changes**: This is a purely additive feature
