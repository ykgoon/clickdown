## Context

The authentication screen in ClickDown uses a TUI (Terminal UI) built with ratatui. Users must enter their ClickUp API token to authenticate, but the current implementation has several usability issues:

1. **No visible feedback**: Token input is fully masked with bullet characters (•), but users report seeing nothing when typing
2. **Paste broken**: Ctrl+V and Ctrl+Shift+V trigger the quit confirmation dialog instead of pasting clipboard content
3. **Quit shortcut too broad**: The quit detection catches modifier combinations it shouldn't

The codebase uses:
- `ratatui` for terminal UI rendering
- `crossterm` for terminal event handling
- Custom input event system in `src/tui/input.rs`
- Auth widget in `src/tui/widgets/auth.rs`
- Main app state in `src/tui/app.rs`

**Constraints:**
- Must work within ratatui/crossterm event model
- Should not break existing keyboard shortcuts
- Minimal changes to preserve existing architecture

## Goals / Non-Goals

**Goals:**
- Users can see their token input (at least partially) as they type
- Ctrl+V and Ctrl+Shift+V paste clipboard content into token field
- Only Ctrl+Q triggers quit confirmation
- Maintain security by not fully exposing the token (partial masking)

**Non-Goals:**
- Full token visibility (security risk)
- Changing the authentication flow itself
- Adding new authentication methods
- Refactoring the entire input system

## Decisions

### 1. Token Display Strategy: Partial Masking

**Decision**: Show first 4 characters unmasked, mask remaining characters with bullets.

**Rationale**: 
- Users can verify they started typing correctly (catches copy/paste errors)
- Still provides security by hiding most of the token
- Common pattern in password fields (e.g., "pass••••••")

**Alternatives considered:**
- Full masking (current): Rejected - users can't verify input at all
- Full visibility: Rejected - security risk, tokens are sensitive
- Toggle visibility: Too complex for TUI, adds UI clutter

### 2. Paste Handling: Intercept Ctrl+V at App Level

**Decision**: Handle Ctrl+V and Ctrl+Shift+V in `update_auth()` before they reach quit detection logic.

**Rationale**:
- crossterm reports Ctrl+V as `KeyCode::Char('v')` with `KeyModifiers::CONTROL`
- Can detect and extract clipboard content using `arboard` crate (cross-platform clipboard)
- Intercepting at auth screen level keeps changes localized

**Alternatives considered:**
- Global paste handler: Rejected - adds complexity, only auth needs it
- Raw keyboard events: Rejected - crossterm doesn't expose raw events reliably

### 3. Quit Shortcut: Exact Match on Ctrl+Q Only

**Decision**: Change quit detection from catching any 'q' with CONTROL to exact match on `KeyCode::Char('q')` with only CONTROL modifier (no SHIFT).

**Rationale**:
- Current code uses `key.modifiers.contains(KeyModifiers::CONTROL)` which matches any combination including Ctrl+Shift
- Should use exact modifier match: `key.modifiers == KeyModifiers::CONTROL`
- Prevents Ctrl+Shift+V from being misinterpreted

**Alternatives considered:**
- Add explicit paste detection first: Done in combination with this fix
- Use different quit shortcut: Rejected - Ctrl+Q is standard

### 4. Clipboard Library: Use `arboard`

**Decision**: Add `arboard` crate for cross-platform clipboard access.

**Rationale**:
- Works on Linux, macOS, Windows
- Simple API: `Clipboard::new()?.get_text()`
- Actively maintained, minimal dependencies

**Alternatives considered:**
- `copypasta`: Older, less maintained
- `wl-clipboard-rs`: Linux-only (Wayland)
- System calls: Not cross-platform

## Risks / Trade-offs

**[Risk] Clipboard access may fail on some systems** → Mitigation: Gracefully handle errors, show "Paste failed" message instead of crashing

**[Risk] Partial masking reduces security** → Mitigation: Only show 4 chars, document trade-off, users can still paste if concerned

**[Risk] crossterm paste detection inconsistent across terminals** → Mitigation: Test on common terminals, fall back to manual entry if paste fails

**[Trade-off] Adding dependency (arboard)** → Acceptable for core functionality fix, minimal overhead

## Migration Plan

1. Add `arboard` dependency to `Cargo.toml`
2. Update `is_quit()` to use exact modifier match
3. Add paste handling to `update_auth()` in `src/tui/app.rs`
4. Update auth widget to show partial token
5. Test on Linux (primary platform), verify macOS/Windows behavior

**Rollback**: Revert changes, remove dependency - no data migration needed

## Open Questions

- Should we support right-click paste in terminals that allow it? (Low priority, may not work in all terminals)
- Should we add a "show/hide token" toggle? (Nice-to-have, not critical for this fix)
