## Context

ClickDown is built with the iced GUI framework following the Elm architecture. Screens are rendered as views in the main application module (`app.rs`) and UI components (`ui/` directory). The application currently has multiple screens: authentication, workspace navigation, task list, task detail, and document views. There is no existing mechanism for screen identification.

## Goals / Non-Goals

**Goals:**
- Display a unique 4-character alphanumeric ID on each screen
- IDs are deterministic (same screen = same ID across sessions)
- Unobtrusive visual presentation (small, bottom-left corner)
- Minimal code changes to existing screen rendering
- Support all current and future screens

**Non-Goals:**
- User configuration of ID visibility (always on)
- Customizable ID format or position
- ID copy-to-clipboard functionality
- Integration with external bug tracking systems

## Decisions

### 1. ID Generation Strategy
**Decision:** Use deterministic hash-based IDs derived from screen names/types.

**Rationale:** 
- Consistency: Same screen always shows same ID
- No persistence needed: IDs survive restarts without storage
- Simple implementation: Hash screen identifier, take first 4 chars

**Alternatives Considered:**
- Random IDs with persistence: Requires database storage, adds complexity
- Sequential numbering: Fragile across versions, not screen-specific
- UUID: Too long for unobtrusive display

### 2. ID Format
**Decision:** 4 alphanumeric characters (base36: 0-9, a-z)

**Rationale:**
- 4 chars provides ~1.6M combinations (36^4), sufficient for foreseeable screens
- Alphanumeric is readable and easy to communicate verbally
- Lowercase for consistency

### 3. Implementation Approach
**Decision:** Create a `screen_id` module that generates IDs, add overlay rendering to each view function.

**Rationale:**
- Minimal intrusion: Existing view logic unchanged
- Reusable: Any new screen can add ID with one line
- Testable: ID generation is pure function

**Alternatives Considered:**
- Wrapper component: More abstraction, less flexible
- Global overlay in main view: Harder to position per-screen

### 4. Visual Styling
**Decision:** Small monospace font, low contrast gray, fixed position at bottom-left with padding.

**Rationale:**
- Visible but unobtrusive
- Monospace ensures consistent width
- Low contrast prevents distraction

## Risks / Trade-offs

**[Visual Clutter]** → Keep font size small (10-12px) and color low contrast (#666 or similar). User feedback will validate if this is acceptable.

**[Screen Real Estate on Small Displays]** → Fixed positioning with minimal padding. 4 chars in small font is ~30-40px width, negligible impact.

**[Hash Collisions]** → With 36^4 combinations and <100 screens, collision probability is extremely low (<0.01%). Acceptable risk.

**[Maintenance Burden]** → New screens must remember to add ID. Mitigation: Add to code review checklist, consider future lint rule.

## Migration Plan

Not applicable - this is a client-side UI enhancement with no deployment complexity or rollback concerns.

## Open Questions

None - implementation approach is clear.
