## Why

Command keys (`n`, `e`, `d`, `a`, `s`, `u`, `v`, `r`, `A`) currently fire directly in NORMAL mode. This conflicts with vim-style modal input (typing text must never trigger commands) and wastes single keys that users may expect for motion. Emacs/Spacemacs-style transient menus solve this: every command is a two-key chord behind `SPACE`.

## What Changes

- **Modified**: Single-key command bindings removed from NORMAL mode; commands are only reachable via `SPACE <group> <key>`
- **Extended SPACEmenu groups**:
  - `SPC g`: `g u` navigate to URL
  - `SPC t`: `t n` new task, `t e` edit task, `t d` delete task, `t s` change status, `t a` assign people, `t f` toggle assigned-to-me filter, `t v` expand/collapse description
  - `SPC c` (new): `c n` new comment, `c e` edit comment, `c r` reply in thread, `c v` expand/collapse comments
  - `SPC y` (new): `y u` copy task URL
  - `SPC w`: `w q` quit
- **Kept**: Navigation keys (`j`, `k`, arrows, `Enter`, `Esc`, `Tab`, `?`) stay direct — they are motions, not commands
- **Modified**: Menu overlay lists the new groups/commands

## Capabilities

### Modified Capabilities
- `keyboard-shortcuts`: command keys moved behind SPACE leader; menu contents specified

## Impact

- `src/tui/app.rs`: `update_tasks`, `update_task_detail` lose direct command keys; `handle_command_menu_input` and `CommandMenuLevel` extended
- `src/tui/widgets/command_menu.rs`: menu text updated
- Tests invoking single-key commands updated to drive `SPACE <group> <key>`
