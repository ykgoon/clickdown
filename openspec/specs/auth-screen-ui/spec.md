# auth-screen-ui Specification

## Purpose
Define the user interface requirements for the authentication screen, including form layout, input focus management, password visibility toggle, and visual alignment.

## Requirements

### Requirement: Password visibility toggle
The system SHALL allow users to toggle password visibility to reveal or mask the password text.

#### Scenario: User toggles password visibility on
- **WHEN** user checks the "Show" checkbox
- **THEN** password input displays characters in plain text
- **AND** checkbox remains checked until user unchecks it

#### Scenario: User toggles password visibility off
- **WHEN** user unchecks the "Show" checkbox
- **THEN** password input masks characters (shows bullets/dots)
- **AND** checkbox remains unchecked until user checks it

#### Scenario: Password visibility persists during input
- **WHEN** password visibility is toggled on
- **AND** user continues typing in the password field
- **THEN** newly typed characters remain visible (not masked)

### Requirement: Tab navigation order
The system SHALL support keyboard navigation using the Tab key with a logical focus order.

#### Scenario: Tab moves focus from username to password
- **WHEN** username input has focus
- **AND** user presses Tab key
- **THEN** password input receives focus

#### Scenario: Tab moves focus from password to show checkbox
- **WHEN** password input has focus
- **AND** user presses Tab key
- **THEN** "Show" checkbox receives focus

#### Scenario: Tab moves focus from checkbox to login button
- **WHEN** "Show" checkbox has focus
- **AND** user presses Tab key
- **THEN** Login button receives focus

#### Scenario: Enter submits form from password field
- **WHEN** password input has focus
- **AND** user presses Enter key
- **THEN** login form is submitted (same as clicking Login button)

### Requirement: Form component alignment
The system SHALL display all form components with consistent visual alignment.

#### Scenario: Labels align vertically
- **WHEN** login form renders
- **THEN** "Email" and "Password" labels left-align with each other

#### Scenario: Input fields align vertically
- **WHEN** login form renders
- **THEN** username and password input fields have equal width
- **AND** input fields left-align with each other

#### Scenario: Password row maintains alignment
- **WHEN** password row renders (input + checkbox)
- **THEN** password input aligns with username input above it
- **AND** checkbox is vertically centered with the password input

#### Scenario: Login button aligns with inputs
- **WHEN** login form renders
- **THEN** Login button has the same width as input fields
- **AND** button left-aligns with input fields

### Requirement: Input focus indicators
The system SHALL provide clear visual feedback when an input has keyboard focus.

#### Scenario: Username input receives focus
- **WHEN** user clicks or tabs to username input
- **THEN** input displays a visible focus border/highlight

#### Scenario: Password input receives focus
- **WHEN** user clicks or tabs to password input
- **THEN** input displays a visible focus border/highlight

#### Scenario: Checkbox receives focus
- **WHEN** user tabs to "Show" checkbox
- **THEN** checkbox displays a visible focus indicator

### Requirement: Error message display
The system SHALL display authentication errors clearly below the form.

#### Scenario: Error message appears after failed login
- **WHEN** authentication fails
- **THEN** error message appears in red text below the login button
- **AND** error message is clearly visible and readable

#### Scenario: Error message cleared on new login attempt
- **WHEN** user starts entering credentials after a failed attempt
- **THEN** previous error message is cleared
