## Why

`j/k` move one item at a time; long task lists and comment threads are slow to traverse. Standard pager keys (`Ctrl+F`/`Ctrl+U`, `PageDown`/`PageUp`) should move a full viewport page.

## What Changes

- **New**: `Ctrl+F`/`PageDown` scrolls down a page; `Ctrl+U`/`PageUp` scrolls up a page, for: grouped task list, comment panel selection, expanded description/comments panels, and the document screen
- **New**: `GroupedTaskList` learns the last rendered viewport height and exposes `select_next_page` / `select_previous_page` that move the selection by a page, skipping header rows
- **New**: Comment panel records its visible height at render; page keys move the comment selection by that many rows
- **Modified**: Expanded-panel `j/k` line scrolling now also honors page keys with a page-sized step

## Capabilities

### Modified Capabilities
- `comment-panel-scrolling`: page scrolling requirements added (also covers task list, expanded panels, document)

## Impact

- `src/tui/widgets/task_list.rs`, `src/tui/widgets/comments.rs`: visible-height tracking + page navigation
- `src/tui/app.rs`: new key arms in `update_tasks`, `update_task_detail`, `update_document`
- New unit tests for page math
