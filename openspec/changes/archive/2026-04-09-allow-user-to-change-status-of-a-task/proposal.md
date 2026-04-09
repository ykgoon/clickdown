## Why

Users need to quickly update task statuses while navigating in the terminal interface. Currently, changing a task status requires navigating through the ClickUp web interface, which breaks workflow efficiency. Adding direct status modification capabilities within ClickDown will streamline task management and improve productivity for users who spend significant time in terminal environments.

## What Changes

- Add keyboard shortcut to change status of selected task
- Implement status selection interface (likely a dropdown or picker)
- Update task status via ClickUp API when user selects new status
- Reflect status change immediately in task list view
- Handle API errors gracefully with user feedback
- Support for custom statuses defined in ClickUp spaces

## Capabilities

### New Capabilities
- `task-status-modification`: Ability to change the status of any task through keyboard-driven interface

### Modified Capabilities
- None (this is a new capability, not modifying existing requirements)

## Impact

- **Code**: Changes to task list widget (`src/tui/widgets/task_list.rs`), task detail panel (`src/tui/widgets/task_detail.rs`), and potentially API client (`src/api/`)
- **API**: Will use ClickUp's task update endpoint to modify task status
- **Dependencies**: No new dependencies required
- **UI**: New status picker component, keyboard handling for status change shortcut