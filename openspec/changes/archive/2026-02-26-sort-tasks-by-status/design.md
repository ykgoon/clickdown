## Context

ClickDown displays tasks in the order returned by the ClickUp API without any client-side sorting. Users must manually scan through all tasks to identify active work items. The task list widget (`src/tui/widgets/task_list.rs`) renders tasks directly from the `TaskListState.tasks` vector without reordering.

The Task model already includes `status` (with status group information) and `updated_at` fields, providing all necessary data for sorting.

## Goals / Non-Goals

**Goals:**
- Automatically sort tasks by status priority: in-progress → to-do → done
- Within each status group, sort by most recently active (updated_at descending)
- Maintain existing visual indicators (status colors, priority badges)
- Zero user configuration - sorting is automatic and transparent
- Preserve selection index after sorting when possible

**Non-Goals:**
- User-configurable sort orders or preferences
- Sorting by other criteria (due date, assignee, priority)
- Changing the API request - sorting is client-side only
- Modifying how tasks are stored or cached

## Decisions

### Decision: Implement sorting in the model layer, not the view

**Approach:** Add a `sort_tasks()` method to the task model or create a sorting utility function that operates on `Vec<Task>`. The TUI widget will receive pre-sorted tasks.

**Rationale:**
- Separation of concerns: view layer handles rendering, model layer handles data organization
- Reusability: sorting logic can be used by other components (CLI debug mode, future features)
- Testability: sorting logic can be unit tested independently of UI
- Performance: sort once when data loads, not on every render

**Alternatives considered:**
- Sort in the view layer during render: Would couple sorting logic to UI, harder to test
- Sort via API parameters: ClickUp API has limited sorting options, doesn't support our custom status priority

### Decision: Use status_group field for status categorization

**Approach:** The `TaskStatus` struct has a `status_group` field that categorizes statuses into groups like "in_progress", "todo", "done". We'll map these groups to sort priorities.

**Rationale:**
- ClickUp already provides the categorization we need
- More robust than matching status names (which can be customized per workspace)
- Future-proof: new statuses in the same group automatically get correct priority

**Alternatives considered:**
- Match status names directly: Fragile, doesn't handle custom statuses
- Add configuration for status mapping: Overkill for this use case

### Decision: Handle missing updated_at gracefully

**Approach:** Tasks without `updated_at` will be treated as having the oldest timestamp (sorted last within their status group).

**Rationale:**
- Prevents panics on malformed data
- Predictable behavior for edge cases
- Consistent with "recently active" intent

## Risks / Trade-offs

**[Performance with large task lists]** → Sorting is O(n log n), but task lists are typically <100 items. Impact is negligible compared to API call latency.

**[Status group values may vary]** → ClickUp's status_group values might not be consistent. Mitigation: Use case-insensitive matching with fallback to status name matching.

**[Selection index shifts after sort]** → If tasks are re-sorted while user has a selection, the selected task may move. Mitigation: Track selected task ID, not index, and restore selection after re-sort.
