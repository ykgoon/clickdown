## Context

The current sidebar navigation uses indentation to indicate hierarchy depth:
- Workspaces: no indent
- Spaces: 2 spaces indent
- Folders: 4 spaces indent
- Lists: 6 spaces indent

This approach consumes valuable horizontal space in terminal environments where screen width is typically limited (80-120 columns). Users navigating deep hierarchies lose 6+ characters of horizontal space, which could otherwise display longer task names or metadata.

The `SidebarItem` enum currently stores an `indent: usize` field for Space, Folder, and List variants, though the rendering code uses hardcoded indent values instead of this field.

## Goals / Non-Goals

**Goals:**
- Remove indentation for all sidebar items to preserve horizontal screen space
- Maintain clear visual hierarchy through alternative means (icons, labels, or spacing)
- Keep all existing navigation functionality intact (keyboard bindings, selection, scrolling)
- Minimal code changes - focus on rendering logic only

**Non-Goals:**
- Changing the hierarchy structure or data model
- Modifying keyboard navigation or shortcuts
- Altering sidebar width or layout calculations
- Adding new visual elements like icons or tree characters (can be future enhancement)

## Decisions

### Decision 1: Remove indentation, use type labels instead

**Approach:** Replace indentation with short type prefixes (e.g., "WS", "SP", "FL", "LI") or keep items flush-left with visual separation through spacing.

**Rationale:** 
- Type labels provide explicit hierarchy information without consuming horizontal space
- Users can quickly scan item types without relying on visual position
- Consistent with terminal UI conventions where space is premium

**Alternatives considered:**
- **Tree characters (│├─►)**: More visual clarity but consumes 2-4 chars per level
- **Color coding by type**: Good secondary indicator but not accessible for all users
- **Bold/weight variation**: Subtle but may not be clear enough for deep hierarchies

**Chosen:** Remove indentation entirely, items render flush-left. Hierarchy is evident from context (user navigates down levels sequentially). Type labels can be added as future enhancement if needed.

### Decision 2: Keep `indent` field in SidebarItem enum

**Approach:** Retain the `indent: usize` field in Space, Folder, and List variants even though unused.

**Rationale:**
- No harm in keeping the field, may be useful for future features
- Removing would require API changes and break any code that reads this field
- Minimal memory impact (usize per item is negligible)

**Alternatives considered:**
- Remove the field entirely (cleaner but breaks potential future use)
- Use the field for spacing instead of indentation (defeats the purpose)

**Chosen:** Keep field unused for now, can be removed in future cleanup if never used.

### Decision 3: Update rendering logic only

**Approach:** Modify only the `render_sidebar` function to remove indentation prefixes.

**Rationale:**
- Minimal change surface area
- No changes to navigation logic, state management, or keyboard handling
- Easy to test and verify visually

**Alternatives considered:**
- Refactor entire sidebar module (overkill for this change)
- Add configuration option for indentation preference (premature optimization)

**Chosen:** Simple string literal changes in the match expression within `render_sidebar`.

## Risks / Trade-offs

**[Risk]** Users may find it harder to identify hierarchy level at a glance
→ **Mitigation:** Hierarchy is contextual - users navigate down levels sequentially, so current level is known. Can add type labels later if needed.

**[Risk]** Visual separation between levels less clear
→ **Mitigation:** Consider adding blank lines between levels or using different highlight colors. Can be iterated based on user feedback.

**[Trade-off]** Losing visual hierarchy indicator for gaining horizontal space
→ **Acceptable** because terminal width is the scarcer resource; hierarchy is already known from navigation context.

**[Risk]** Existing users accustomed to indentation may find change disorienting
→ **Mitigation:** Change is consistent across all levels; users adapt quickly to flat layouts (common in modern UIs).

## Migration Plan

No migration required - this is a UI rendering change only:
1. Update `src/tui/widgets/sidebar.rs` - remove indentation prefixes
2. Build and test visually with deep hierarchy
3. No database changes, no API changes, no config changes

**Rollback:** Revert the single file change if issues arise.

## Open Questions

None - straightforward rendering change.
