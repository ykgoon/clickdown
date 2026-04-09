## Why

The help dialog currently renders all 9 shortcut sections in a single scrollable view at 70%×70% of the terminal. This requires ~43 content lines but only fits ~15 on a minimum-size terminal (80×24), causing consistent truncation of lower sections (Comments, Forms, Session). Users on small terminals cannot see all shortcuts without resizing.

## What Changes

- Replace the single-page help dialog with a 3-page paginated view
- Page 1 becomes contextual: shows shortcuts relevant to the current screen and focus state
- Page 2 is global: Navigation, Global actions, Actions, and Forms (always the same content)
- Page 3 is reference: all remaining sections not shown on page 1
- Add pagination controls (page indicator + navigation hints) in the dialog footer
- Replace the "any key closes" behavior with explicit `Esc`/`?` close — `j/k` now paginate instead of closing

## Capabilities

### New Capabilities
- `help-dialog-pagination`: Paginated help dialog with contextual first page, global anchor page, and reference page. Navigation via j/k or arrow keys. Explicit close via Esc/?.

### Modified Capabilities
- `help-dialog`: The "any key closes" requirement is replaced by explicit close (Esc/?). The "Press any key to close" hint is replaced with pagination navigation hints. Content organization changes from single-page to 3-page.
- `keyboard-shortcuts`: The "Question mark toggles help dialog" requirement remains, but the "Any key closes help dialog" requirement is modified — only Esc and ? close the dialog; j/k/arrow keys paginate.

## Impact

- `src/tui/widgets/help.rs` — Primary rewrite: pagination state, multi-page rendering, footer nav
- `src/tui/app.rs` — Update help key handling (j/k paginate instead of close), update `get_hints()` for pagination footer
- `openspec/specs/help-dialog/spec.md` — Modify "any key closes" and "content organization" requirements
- `openspec/specs/keyboard-shortcuts/spec.md` — Modify "Any key closes help dialog" requirement
