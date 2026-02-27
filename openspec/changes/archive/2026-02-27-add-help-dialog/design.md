## Context

The help dialog widget already exists in `src/tui/widgets/help.rs` and is toggled with the `?` key. However, there are integration issues:

1. **Status bar clutter**: The status bar shows context-specific hints but doesn't indicate the `?` shortcut for help
2. **Help close behavior**: The help widget displays "Press any key to close" but the close logic needs proper integration
3. **Help visibility**: When help is open, underlying UI hints should still be visible for reference

Current state:
- Help widget exists with hardcoded shortcuts
- `?` key toggles `self.help.visible`
- Help renders as a modal overlay
- No hint in status bar about the help shortcut

## Goals / Non-Goals

**Goals:**
- Add `?` hint to status bar across all screens
- Ensure help dialog closes on any key press when visible
- Keep help dialog content maintainable (not hardcoded)
- Maintain modal behavior (blocks interaction with underlying UI)

**Non-Goals:**
- Redesigning the help dialog UI
- Adding search/filter to help dialog
- Changing existing keyboard shortcuts
- Making help context-sensitive (same shortcuts shown everywhere)

## Decisions

### Decision 1: Show `?` hint in status bar
**Approach**: Add `| ?` to all status bar hints via `get_hints()`

**Rationale**: Users need to discover the help shortcut. Following the pattern of other global hints shown in status bar.

**Alternatives considered:**
- Show `?` hint only on first launch → Too complex, users might miss it
- Never show in status bar, rely on muscle memory → Poor discoverability

### Decision 2: Close help on any key press
**Approach**: When `help.visible == true`, any key press calls `help.hide()` and returns early

**Rationale**: Simple, intuitive behavior. Matches the "Press any key to close" message in the help dialog.

**Alternatives considered:**
- Only `Esc` or `?` closes help → Less intuitive, inconsistent with dialog message
- Click outside to close → Not applicable in TUI

### Decision 3: Keep shortcuts hardcoded in help widget
**Approach**: Maintain current approach of hardcoded shortcuts in `help.rs`

**Rationale**: Single source of truth for shortcuts. Adding a central registry would be over-engineering for current codebase size.

**Alternatives considered:**
- Central shortcut registry with descriptions → More maintainable long-term but adds complexity
- Generate help from actual key handlers → Complex, requires refactoring input handling

### Decision 4: Help blocks underlying input
**Approach**: When help is visible, don't process other shortcuts

**Rationale**: Prevents accidental actions while help is open. Standard modal behavior.

## Risks / Trade-offs

**[Risk]** Hardcoded shortcuts in help.rs may drift from actual shortcuts
→ **Mitigation**: Code review checklist should verify help.rs matches actual bindings

**[Risk]** Status bar gets cluttered with too many hints
→ **Mitigation**: `?` hint is short; status bar has space for it

**[Trade-off]** Help dialog is not context-sensitive
→ **Acceptable**: Global shortcuts are the same everywhere; context-specific ones are shown in status bar

**[Trade-off]** Help uses fixed 70% size
→ **Acceptable**: Works well for current content; can be adjusted later if needed
