## Why

The authentication screen (c08b) has multiple critical usability and functionality issues that prevent users from successfully logging in. The "parsing error" during authentication is blocking all usage, while UI/UX problems (misaligned components, broken tab navigation, non-functional password reveal) degrade the user experience.

## What Changes

- **Fix authentication parsing error**: The credential-based authentication flow is failing with a "parsing error" - likely due to incorrect API endpoint handling or response parsing
- **Fix "Show password" toggle**: The checkbox toggle does not reveal the password in plain text as expected
- **Fix UI component alignment**: Login form components are misaligned, creating a visually broken interface
- **Fix tab navigation**: Pressing Tab does not move focus between inputs (username → password → login button), breaking keyboard accessibility

## Capabilities

### New Capabilities

- `auth-screen-ui`: Complete authentication screen UI with proper form layout, input focus management, password visibility toggle, and aligned components

### Modified Capabilities

- `auth-flow`: The authentication flow needs fixing to properly handle credential-based login and parse the token response without errors

## Impact

- **Modified files**: `src/ui/login_view.rs`, `src/api/client.rs`, `src/app.rs`
- **Dependencies**: No new dependencies required; uses existing iced widget capabilities
- **Breaking changes**: None - fixes only, no API changes
- **Testing impact**: Login view tests need to verify password toggle behavior and tab focus order
