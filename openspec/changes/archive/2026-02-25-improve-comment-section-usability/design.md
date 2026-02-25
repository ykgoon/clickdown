## Context

The current task detail view renders description and comments in a fixed layout without independent scrolling. When tasks have many comments, users struggle to navigate and read them efficiently. The existing implementation in `src/tui/widgets/task_detail.rs` and `src/tui/widgets/comments.rs` uses a simple vertical split without proper scroll management.

**Current State:**
- Description panel takes most of the space with `Constraint::Min(3)`
- Comments section has no dedicated scrollable area
- Navigation with j/k doesn't auto-scroll to keep selected comment visible
- No visual indication of scrollable content

**Stakeholders:**
- End users who work with tasks containing multiple comments
- Developers maintaining the TUI layout and widget code

## Goals / Non-Goals

**Goals:**
- Implement 3:7 height ratio between description and comment panels
- Make both panels independently scrollable when content overflows
- Add auto-scroll behavior to keep selected comment visible during j/k navigation
- Add visual scroll indicators to show when content extends beyond visible area
- Maintain backward compatibility with existing task detail functionality

**Non-Goals:**
- Changing the comment data model or API integration
- Modifying comment creation/editing behavior
- Adding new comment features (reactions, threading, etc.)
- Changing keyboard shortcuts for navigation

## Decisions

### Decision 1: Layout Ratio Implementation

**Approach:** Use ratatui's `Constraint::Percentage` to enforce the 3:7 ratio.

**Alternatives Considered:**
- **Fixed height ratios**: Rejected because it wouldn't adapt to different terminal sizes.
- **Dynamic ratio based on content**: Rejected for complexity; users expect consistent layout.
- **User-configurable ratio**: Out of scope; can be added later if needed.

**Rationale:** Percentage constraints provide predictable behavior across terminal sizes while maintaining the desired emphasis on comments (70% of available space).

### Decision 2: Independent Scrolling

**Approach:** Implement scroll state tracking for each panel using a scroll offset that adjusts when content exceeds available space.

**Alternatives Considered:**
- **Use ratatui's Scrollable widget**: Rejected because we need fine-grained control over scroll behavior tied to selection.
- **Single scroll state for entire task detail**: Rejected because it doesn't allow independent scrolling of panels.
- **No scroll indicators**: Rejected because users need visual feedback about hidden content.

**Rationale:** Manual scroll offset management gives us control to implement auto-scroll behavior and visual indicators. This pattern is consistent with existing TUI navigation patterns in the codebase.

### Decision 3: Auto-Scroll on Selection Change

**Approach:** When selected comment index changes, check if it's outside visible bounds and adjust scroll offset accordingly.

**Alternatives Considered:**
- **Always center selected item**: Rejected because it can be disorienting with rapid navigation.
- **Scroll only at boundaries**: Chosen approach - scroll only when selection moves outside visible area.
- **No auto-scroll**: Rejected because it defeats the purpose of keyboard navigation.

**Rationale:** Boundary-based auto-scroll provides smooth navigation while minimizing disorientation. This matches user expectations from vim-style navigation.

### Decision 4: Scroll Indicator Placement

**Approach:** Add scroll bar on the right edge of each scrollable panel using Unicode box-drawing characters or simple `│` symbols.

**Alternatives Considered:**
- **Show line numbers**: Rejected for space efficiency.
- **Use colors only**: Rejected because it's not accessible for colorblind users.
- **No indicators**: Rejected because users need to know content is hidden.

**Rationale:** Simple scroll bars are space-efficient and universally understood. They can be rendered alongside the existing border.

## Risks / Trade-offs

**[Risk] Layout breaks on small terminals** → Mitigation: Add minimum height checks and gracefully degrade to stacked layout when terminal is too small.

**[Risk] Auto-scroll feels janky** → Mitigation: Implement smooth scrolling if ratatui supports it; otherwise ensure scroll jumps are minimal and predictable.

**[Risk] Scroll indicators reduce content width** → Mitigation: Account for indicator width in text wrapping calculations; use single character width.

**[Trade-off] Manual scroll management is more code** → Acceptable because it provides better control and user experience than relying on widget-level scrolling.

**[Trade-off] 3:7 ratio may not suit all use cases** → Acceptable because comments are the primary focus of this improvement; can be made configurable later.

## Migration Plan

1. **Update layout.rs**: Add new layout splitting function that supports percentage-based constraints for task detail view.
2. **Update task_detail.rs**: Refactor to use new layout with 3:7 ratio and independent scrollable areas.
3. **Update comments.rs**: Add scroll state management and auto-scroll logic tied to selection changes.
4. **Update input handling**: Ensure j/k navigation triggers scroll position updates.
5. **Test**: Verify layout on various terminal sizes and with different comment counts.

**Rollback Strategy:** Revert to previous layout constraints if issues arise; no database or API changes means simple code rollback.

## Open Questions

- Should the 3:7 ratio be configurable by users in the future?
- Should scroll indicators use Unicode box characters or ASCII-only for maximum compatibility?
- Should there be a minimum number of visible comment lines regardless of ratio?
