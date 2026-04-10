## Context

The task list widget (`src/tui/widgets/task_list.rs`) renders each task with three visual components:
1. A priority indicator in brackets: `[•]`, `[↑]`, `[↓]`, `[⚡]`
2. A single-letter status prefix: `T`, `D`, `I`, etc. (colored and bold)
3. The task name

Since the introduction of status groupings (group headers like "▸ TODO (3)"), the per-task status letter has become redundant. The group header already communicates the status of all tasks within that group.

## Goals / Non-Goals

**Goals:**
- Remove the single-letter status prefix from individual task rows
- Keep the priority indicator and task name
- Maintain existing group headers (no change to grouping logic)
- Keep all existing tests passing; add a snapshot or rendering test if practical

**Non-Goals:**
- No changes to task grouping, sorting, or navigation logic
- No changes to the Task model or API layer
- No changes to the task detail view (only the list view)

## Decisions

**Remove status Span entirely rather than replacing with whitespace**
- The `render_task_list` function builds a `Line` from multiple `Span` elements. The status `Span` will be removed entirely, not replaced with an empty or space `Span`, to keep the code minimal.
- The `get_status_color` function will remain unused in this file but is kept in case it's referenced elsewhere or useful for future features (e.g., coloring the task name by status).

**Keep `get_status_color` function (dead code for now)**
- Marking with `#[allow(dead_code)]` rather than deleting, as it may be useful for future visual enhancements like coloring task names by status.

## Risks / Trade-offs

- **Visual regression**: Removing the status letter may make the list look too sparse if group headers are collapsed or not visible. → Mitigation: group headers are always rendered; this is a purely visual simplification with no functional impact.
- **Unused code**: `get_status_color` becomes unused. → Mitigation: allow dead code annotation; can be removed in a future cleanup if still unused.
