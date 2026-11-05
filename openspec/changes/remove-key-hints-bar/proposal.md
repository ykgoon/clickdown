## Why

The bottom status bar currently renders a command-key hint bar (`j/k: Navigate | Enter: Select | ...`). With all command keys moving behind the SPACEmenu (see `space-command-menu`), the hint bar is redundant, consumes two extra terminal rows, and rots quickly. Users discover commands via `SPACE` instead.

## What Changes

- **Removed**: The command-key hints text and the bordered 3-row hint bar at the bottom of the screen
- **Modified**: `TuiLayout` status area shrinks from 3 rows to 1 borderless status-message row
- **Removed**: `get_hints()` and per-screen hint strings in `app.rs`; `render_status` takes only the status message
- **Kept**: The status message itself (feedback, mode indicators) still renders at the bottom
- **Kept**: `?` help dialog remains the discoverability surface for shortcuts

## Capabilities

### Modified Capabilities
- `keyboard-shortcuts`: hints bar requirement removed; help dialog is the shortcut reference

## Impact

- `src/tui/layout.rs`: status area constraint 3 -> 1; `render_status` signature simplified
- `src/tui/app.rs`: `get_hints` deleted; render path passes only status
- Snapshot tests referencing hint text must be regenerated
