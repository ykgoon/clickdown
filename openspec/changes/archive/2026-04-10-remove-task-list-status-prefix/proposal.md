## Why

The task list view currently displays a single-letter status prefix (e.g., "T", "D", "I") before each task name, like `[•] T Sample task`. Since status groupings are already shown via collapsible group headers (e.g., "▸ TODO (3)", "▸ DONE (1)"), the per-task status letter is redundant and adds visual noise. Removing it will produce a cleaner, less cluttered task list.

## What Changes

- Remove the single-letter status indicator from individual task rows in the task list view
- Task rows will still show the priority indicator (e.g., `[•]`, `[↑]`, `[↓]`, `[⚡]`) followed directly by the task name
- Group headers continue to provide status context for all tasks within each group

## Capabilities

### New Capabilities
<!-- No new capabilities introduced -->

### Modified Capabilities
- `task-list`: Remove per-task status letter from task row rendering; status context is provided by group headers only

## Impact

- `src/tui/widgets/task_list.rs`: Remove `status_str` computation and its `Span` from the task row rendering
- No API changes, no breaking changes to data models or caching
- Visual/UI change only — task data and grouping logic remain unchanged
