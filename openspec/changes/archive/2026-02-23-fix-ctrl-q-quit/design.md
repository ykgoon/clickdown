## Context

The current `ctrl-q` implementation in `src/app.rs` uses `iced::event::listen_with` to capture keyboard events. However, it only listens for `KeyPressed` events with a `Character` key type, which misses cases where:

1. The system doesn't produce a character event for the key combination
2. Different keyboard layouts map `q` differently
3. The iced framework processes the key as a named key instead of a character

The application currently has no centralized keyboard shortcut system - the `ctrl-q` handling is ad-hoc in the main `subscription` method.

## Goals / Non-Goals

**Goals:**
- Make `ctrl-q` reliably quit the application across all keyboard layouts and systems
- Follow iced framework best practices for keyboard event handling
- Maintain the existing application architecture (Elm pattern)
- Keep the implementation simple and maintainable

**Non-Goals:**
- Building a full keyboard shortcut configuration system (future enhancement)
- Adding quit confirmation dialog (can be added later if needed)
- Implementing other keyboard shortcuts (out of scope for this fix)

## Decisions

### Decision 1: Capture Both Character and Named Key Events

**Rationale**: The iced framework can represent the 'q' key as either `Key::Character("q")` or `Key::Named(Named::KeyQ)` depending on the system and keyboard layout. We need to handle both cases.

**Alternatives Considered:**
- Use only `Key::Named(Named::KeyQ)`: Might miss some keyboard layouts
- Use only `Key::Character("q")`: Current broken implementation
- Use a keyboard shortcut library: Overkill for a single shortcut, adds dependency

### Decision 2: Keep Implementation in `subscription` Method

**Rationale**: For a single keyboard shortcut, keeping the logic in the existing `subscription` method is simpler than creating a new module. The logic is straightforward and doesn't warrant abstraction.

**Alternatives Considered:**
- Create a `keyboard` module: Better for multiple shortcuts, but over-engineering for one
- Use iced's `keyboard::key_pressed` subscription: Less flexible than `event::listen_with`

### Decision 3: Direct Exit Without Confirmation

**Rationale**: Matches user expectations for `ctrl-q` (immediate quit). Most desktop applications don't confirm `ctrl-q`. The application doesn't have unsaved state that requires confirmation.

**Alternatives Considered:**
- Add confirmation dialog: Safer but adds friction
- Save state before exit: Future enhancement if needed

## Risks / Trade-offs

**[Risk]**: The `Key::Named(Named::KeyQ)` variant might not be available in older iced versions.
→ **Mitigation**: Test thoroughly; if unavailable, fall back to character-only matching with better documentation.

**[Risk]**: Handling both character and named events might cause double-firing on some systems.
→ **Mitigation**: The pattern match is mutually exclusive (Character vs Named), so this shouldn't occur.

**[Trade-off]**: Not building a reusable keyboard shortcut system now means future shortcuts will need similar ad-hoc implementations.
→ **Mitigation**: This is acceptable for now; can refactor into a module when 3+ shortcuts exist.

## Migration Plan

No migration needed - this is a bug fix that replaces the existing keyboard event handler.

**Steps:**
1. Update the `subscription` method in `src/app.rs` to handle both key event types
2. Test on multiple platforms (Linux, macOS, Windows) if possible
3. Verify no double-firing occurs

**Rollback:**
- Revert the subscription method to the previous implementation

## Open Questions

None - the implementation approach is clear and straightforward.
