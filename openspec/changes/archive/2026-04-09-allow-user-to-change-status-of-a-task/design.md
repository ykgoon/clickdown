## Context

ClickDown is a terminal-based ClickUp client that follows the Elm Architecture pattern. Users currently need to navigate to the ClickUp web interface to change task statuses, which disrupts workflow efficiency. The application already has infrastructure for keyboard handling, API communication via trait-based dependency injection, and UI components built with ratatui.

Current relevant code:
- Task list widget (`src/tui/widgets/task_list.rs`) handles keyboard input and displays tasks
- Task detail panel (`src/tui/widgets/task_detail.rs`) shows detailed task information
- API client (`src/api/client.rs`) implements the ClickUpApi trait for HTTP communication
- Message enum (`src/app.rs`) defines all state transitions
- Terminal handling and rendering loop in `src/tui/`

## Goals / Non-Goals

**Goals:**
- Enable users to change task status via keyboard shortcut within ClickDown
- Maintain consistency with existing UI patterns and keyboard navigation
- Provide immediate visual feedback of status changes
- Handle API errors gracefully with user notifications
- Support custom statuses from ClickUp spaces
- Follow existing code patterns and architectural principles

**Non-Goals:**
- Changing multiple task statuses simultaneously (bulk operations)
- Modifying task statuses through bulk edit or rule engines
- Creating new statuses or modifying status definitions
- Integration with ClickUp automation or dependencies

## Decisions

### Keyboard Shortcut Selection
**Decision:** Use 's' key to initiate status change when a task is selected
**Rationale:** 
- 's' is intuitively associated with "status"
- Not currently used in task list view (checking existing shortcuts: j/k navigate, Enter selects, a toggles assignee filter, ? shows help)
- Consistent with single-key shortcuts used elsewhere in the application
- Alternative considered: 't' for "task status" but 's' is more direct
- Alternative considered: Ctrl+S but prefer single-key shortcuts for frequency of use

### Status Selection Interface
**Decision:** Implement a modal dropdown/picker that appears over the current view
**Rationale:**
- Follows existing pattern of modals (auth view, help dialog, comment panels)
- Allows users to see available statuses without losing context
- Keyboard navigable with j/k or arrow keys, Enter to select, Esc to cancel
- Consistent with other selection interfaces in the application
- Alternative considered: Inline expansion but would complicate task list layout
- Alternative considered: Command palette but overkill for this simple interaction

### API Integration Approach
**Decision:** Use existing ClickUpApi trait to update task status via PATCH endpoint
**Rationale:**
- Leverages existing dependency injection pattern for testability
- Reuses authenticated HTTP client infrastructure
- Follows established error handling patterns
- Alternative considered: Direct HTTP calls but would break testability and consistency
- The UpdateTaskRequest struct already supports status field modification

### State Management
**Decision:** Add new Message variants for status change flow and update TuiApp state
**Rationale:**
- Follows Elm Architecture pattern used throughout the application
- Keeps state transitions explicit and traceable
- Enables proper loading/error states
- Alternative considered: Direct widget state but would complicate testing and consistency

### Error Handling and Feedback
**Decision:** Use existing status bar and notification mechanisms for API errors
**Rationale:**
- Consistent with how other API errors are displayed (authentication, task creation, etc.)
- Provides immediate feedback without requiring modal dialogs for every error
- Alternative considered: Modal error dialogs but would be disruptive for frequent operation

## Risks / Trade-offs

[Risk] API rate limiting if users frequently change status → Mitigation: Implement client-side debouncing for rapid successive changes
[Risk] Status list becomes very long in spaces with many custom statuses → Mitigation: Add filtering/search to status picker if needed
[Risk] Inconsistent status colors between ClickUp and ClickDown → Mitigation: Use ClickUp-provided colors when available, fallback to defaults
[Risk] Offline caching may show stale status after change → Mitigation: Invalidate relevant cache entries after successful status update
[Risk] Conflict between local UI update and actual API result → Mitigation: Optimistic UI update with rollback on failure

## Open Questions

- Should the status picker show status colors alongside names for better visual identification?
- What should be the default behavior when escaping from the status picker (cancel vs keep current)?
- Should we show a loading indicator in the task list while the API request is in flight?