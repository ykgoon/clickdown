## Context

The authentication screen (`login_view.rs`) has four critical issues reported on screen c08b:

1. **"Show password" toggle doesn't work** - The checkbox state changes but the password input remains masked
2. **UI components are misaligned** - Form elements don't have consistent alignment
3. **Tab navigation broken** - Focus doesn't move between inputs when pressing Tab
4. **Authentication fails with "parsing error"** - The credential authentication flow fails during response parsing

Current implementation uses iced's `TextInput::secure(true)` for password masking, but the `show_password` state toggle isn't connected to actually change the input's secure mode. The authentication flow in `client.rs` attempts to call an OAuth token endpoint that may not exist or return unexpected data.

## Goals / Non-Goals

**Goals:**
- Fix password visibility toggle to actually reveal/hide password text
- Align all form components (labels, inputs, checkbox, button) consistently
- Enable proper tab focus order: username → password → show checkbox → login button
- Fix authentication parsing error by handling the API response correctly
- Maintain existing dark theme and visual styling

**Non-Goals:**
- Adding new authentication methods (OAuth, SSO)
- Changing the authentication API endpoint
- Adding password validation rules
- Implementing "forgot password" flow

## Decisions

### 1. Password Visibility Toggle Implementation

**Decision**: Use iced's `secure()` method dynamically based on state, not a static `secure(true)`.

**Rationale**: The current code sets `.secure(true)` statically. The `show_password` boolean in state is only used for the checkbox, not wired to the TextInput. We need to conditionally apply `.secure(!state.show_password)`.

**Alternatives considered**:
- Custom text input widget: Too complex, iced provides built-in support
- Two separate inputs (masked/unmasked): Unnecessary state duplication

### 2. Tab Navigation Fix

**Decision**: Use iced's focus system with proper widget ordering and explicit focus hints.

**Rationale**: Iced 0.13 uses a focus-based system. The issue is likely that the password row (containing both input and checkbox) breaks the natural tab order. Solution: ensure widgets are in a single Column with proper focusable wrappers, or use `Focusable` trait implementations.

**Alternatives considered**:
- Manual keyboard event handling: Overly complex, fights the framework
- Restructuring layout to flatten widget hierarchy: Preferred approach

### 3. UI Alignment Fix

**Decision**: Use consistent width constraints and alignment containers for all form elements.

**Rationale**: Current code mixes `Length::Fixed(400.0)` widths with unconstrained elements. Solution: wrap all inputs in a form container with consistent spacing and alignment, ensure the password row doesn't break alignment.

**Alternatives considered**:
- Grid layout: Overkill for a simple form
- Manual positioning: Not responsive, hard to maintain

### 4. Authentication Parsing Error

**Decision**: The parsing error likely stems from the OAuth endpoint returning a different structure than `OAuthTokenResponse` expects, or the endpoint not existing. Need to handle the actual ClickUp API response format.

**Rationale**: ClickUp's actual API uses OAuth 2.0 with authorization codes, not password grant. The current implementation assumes a password grant endpoint exists. The fix requires either:
- Using ClickUp's actual OAuth flow (redirect-based)
- Properly handling the error when the endpoint doesn't exist
- Using API token authentication instead (simpler for desktop apps)

For this fix, we'll improve error handling to show the actual parsing error and fall back to token-based auth if credential auth fails.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Changing password input secure mode may cause focus loss | Test thoroughly; iced should maintain focus when only the secure flag changes |
| Flattening widget hierarchy may break responsive layout | Use Container with constraints instead of fixed widths where possible |
| OAuth flow changes may require backend updates | Keep token-based auth as fallback; credential auth is bonus |
| Tab order fix may require iced version upgrade | Check iced 0.13 documentation for focus APIs |

## Migration Plan

1. Update `login_view.rs` to wire `show_password` state to TextInput's `secure()` method
2. Restructure password row to maintain tab order (possibly flatten hierarchy)
3. Add focus hints to inputs for explicit tab navigation
4. Update `client.rs` to handle OAuth response parsing errors gracefully
5. Test all four fixes together on screen c08b

No database migrations or config changes required.

## Open Questions

- Does ClickUp actually support password grant OAuth flow, or do we need to implement redirect-based OAuth?
- Should we deprecate credential auth entirely and only support API tokens for desktop usage?
