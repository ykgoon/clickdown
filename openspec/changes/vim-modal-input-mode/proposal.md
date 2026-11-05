## Why

Single-key commands (pre-`space-command-menu`) fired while users typed text: comments contained stray `d`/`e` effects and commands swallowed input. A vim-style modal model fixes this and is the prerequisite for `space-command-menu`. Core implementation (INSERT/normal modes, `InsertTarget`, `Ctrl+S` commit, `Esc` abort, `@` mention picker) already landed in commit 7c761a2; this change formalizes the spec and closes verification gaps.

## What Changes

- **Formalized**: INPUT/NORMAL mode contract as a spec requirement (behavior unchanged)
- **Verified**: existing `tests/modal_input_test.rs` coverage mapped to the spec scenarios
- **New tests**: NORMAL-mode `SPACE` opens menu only when no input target is active; `Esc` aborts buffer; INPUT mode does not interpret `SPACE` as menu opener

## Capabilities

### Modified Capabilities
- `text-input-guards`: modal input mode requirement added

## Impact

- No functional code changes expected; tests only (plus fixes if verification finds gaps)
