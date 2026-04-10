## Context

The current task list view (`TaskListState` + `render_task_list()`) renders tasks as a flat list, sorted by status priority (In Progress → Todo → Done → Fallback) and then by `updated_at` descending. There are no visual group headers — tasks from different status groups flow together without separation.

The `SelectableList<T>` helper handles keyboard navigation (j/k) with wrap-around, treating all items as equally selectable. Navigation logic doesn't distinguish between "header" rows and "task" rows.

Both the unfiltered view and the "Assigned to Me" filtered view use the same rendering code path.

## Goals / Non-Goals

**Goals:**
- Add visual status group headers (e.g., "▸ IN PROGRESS (3)") between task groups
- Group tasks under their `status_group` field using the existing `StatusGroupPriority` ordering
- Keep headers non-selectable — j/k navigation should skip over them
- Work identically for both unfiltered and filtered ("Assigned to Me") views
- Maintain existing sort order within each group (by `updated_at` descending)
- Collapse empty status groups (don't render headers for groups with zero tasks)

**Non-Goals:**
- No collapsible groups (that's a future enhancement)
- No changes to how tasks are fetched from the API
- No changes to the "Assigned to Me" filtering logic itself — only how results are displayed
- No drag-and-drop or reordering of groups

## Decisions

### 1. Grouped data model: `GroupedTaskList` struct

**Decision:** Create a new `GroupedTaskList` struct in `src/tui/widgets/task_list.rs` that replaces `TaskListState` for grouped rendering. It stores:
```rust
pub struct GroupedTaskList {
    groups: Vec<TaskGroup>,
    selected_group: usize,
    selected_item_in_group: usize,
}

pub struct TaskGroup {
    pub status_group: StatusGroupPriority,
    pub label: String,       // e.g., "IN PROGRESS"
    pub tasks: Vec<Task>,
}
```

**Rationale:** Using a grouped structure lets us:
- Track selection as `(group_index, item_index)` rather than a flat index
- Render headers between groups cleanly
- Skip empty groups entirely

**Alternatives considered:**
- **Flat list with header sentinel items**: Insert special "header" tasks into the flat `Vec<Task>`, mark them with a sentinel (e.g., empty `id`). This avoids changing selection logic but is fragile — header rows could be accidentally selected or treated as real tasks.
- **Computed grouping at render time**: Keep flat `Vec<Task>` sorted, compute group boundaries during rendering. This is simpler but duplicates grouping logic and makes selection mapping harder.

### 2. Selection model: Virtual index mapping

**Decision:** Keep `j/k` navigation working on a "virtual" flat index that excludes header rows. When rendering, we map from virtual index → `(group, item)` pair.

```
Virtual index:  0  1  2     3  4     5  6  7
Rendered:      [T1][T2][HEADER][T3][T4][HEADER][T5][T6]
                 ↑ Group boundary
```

**Rationale:** The ratatui `ListState` works with flat indices. We can maintain a `Vec<ListRow>` where each row is either `Header(GroupLabel)` or `Task(Task)`, and use ratatui's native selection on it. The `selected()` method then extracts the task if the selected row is a task.

**Alternatives considered:**
- **Two-level selection**: Track group index + item index separately. More complex — requires handling two selection states and doesn't integrate cleanly with ratatui's `ListState`.

### 3. `ListRow` enum for rendering

**Decision:** Introduce a `ListRow` enum:
```rust
pub enum ListRow {
    Header { label: String, count: usize },
    Task(Task),
}
```

The `GroupedTaskList` builds a `Vec<ListRow>` for rendering. Navigation (j/k) only selects `Task` rows; `Header` rows are visually distinct (different style, no highlight).

**Rationale:** This is the cleanest approach — headers are part of the rendered list but semantically distinct. Selection logic can skip headers by checking the enum variant.

### 4. Navigation: Skip header rows

**Decision:** Override `select_next()` and `select_previous()` to skip `Header` variants. When moving to the next row, if it's a header, advance one more step.

```rust
pub fn select_next(&mut self) {
    loop {
        let next = (current + 1) % rows.len();
        if matches!(rows[next], ListRow::Task(_)) {
            self.list.select(Some(next));
            break;
        }
        // Skip header, continue loop
    }
}
```

### 5. Group labels: Use existing `StatusGroupPriority` display

**Decision:** Derive group labels from the `StatusGroupPriority` enum's display representation. Use uppercase with count: `"IN PROGRESS (3)"`, `"TODO (5)"`, `"DONE (2)"`.

**Rationale:** Consistent with the existing status priority system. No new string constants needed.

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| **Header rows break selection** if navigation logic doesn't skip them properly | Add unit tests for `select_next`/`select_previous` with mixed header/task lists |
| **Empty task lists** should not show any headers | Only create groups that have at least one task |
| **Single-task lists** should still show the group header | Always show headers when the group has ≥1 task |
| **Performance** with large task lists (100+ tasks) | Grouping is O(n) — negligible for terminal UI scale |
| **Backwards compatibility** — existing code referencing `TaskListState` | Replace `TaskListState` entirely with `GroupedTaskList`; update all callers in `app.rs` |
