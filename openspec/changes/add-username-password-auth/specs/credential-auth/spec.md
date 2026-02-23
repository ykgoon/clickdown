## ADDED Requirements

### Requirement: User can authenticate with username and password
The system SHALL allow users to log in using their ClickUp username (email) and password, exchanging credentials for an API token.

#### Scenario: Successful authentication
- **WHEN** user enters valid username and password
- **AND** user clicks "Login" button
- **THEN** system exchanges credentials for an API token
- **AND** token is stored securely for future sessions
- **AND** user is navigated to the main application view

#### Scenario: Invalid credentials
- **WHEN** user enters incorrect username or password
- **AND** user clicks "Login" button
- **THEN** system displays error message "Invalid username or password"
- **AND** user remains on the login screen
- **AND** password field is cleared for re-entry

#### Scenario: Network error during authentication
- **WHEN** network is unavailable during login attempt
- **THEN** system displays error message "Unable to connect to ClickUp. Please check your internet connection."
- **AND** user remains on the login screen

#### Scenario: Account locked or disabled
- **WHEN** user's ClickUp account is locked or disabled
- **THEN** system displays error message "Your account has been locked. Please contact ClickUp support."
- **AND** user remains on the login screen

### Requirement: Credentials are handled securely
The system SHALL handle user credentials with appropriate security measures.

#### Scenario: Credentials in memory
- **WHEN** user enters username and password
- **THEN** credentials are stored only in memory during the authentication request
- **AND** credentials are cleared from memory after token exchange completes

#### Scenario: Credentials not logged
- **WHEN** authentication request is processed
- **THEN** username and password MUST NOT appear in application logs
- **AND** password MUST NOT appear in error messages

#### Scenario: Token storage unchanged
- **WHEN** authentication succeeds
- **THEN** the obtained token is stored using existing AuthManager mechanism
- **AND** token storage format and location remain unchanged

### Requirement: User can log out
The system SHALL provide a logout mechanism that clears the stored token and returns to the login screen.

#### Scenario: User initiates logout
- **WHEN** user clicks "Logout" button
- **THEN** stored token is cleared from disk
- **AND** user is navigated back to the login screen
- **AND** username and password fields are cleared

## MODIFIED Requirements

### Requirement: Graceful Error Handling

All error-prone operations MUST propagate errors gracefully rather than panicking.

#### Scenario: Client operation fails

- **WHEN** an API client returns an error
- **THEN** the error MUST be propagated via the `?` operator or match expression
- **AND** the error MUST be converted to an appropriate Message variant

#### Scenario: Config initialization fails

- **WHEN** ConfigManager::default() cannot create the config directory
- **THEN** the error MUST be logged and a fallback instance returned
- **AND** the application MUST continue to start

#### Scenario: Auth initialization fails

- **WHEN** AuthManager::default() cannot access the token path
- **THEN** the error MUST be logged and a fallback instance returned
- **AND** the application MUST continue to start

#### Scenario: Credential authentication fails

- **WHEN** credential exchange endpoint returns an error
- **THEN** the error MUST be converted to `Message::AuthError` with user-friendly message
- **AND** the UI MUST display the error without crashing
- **AND** the login form MUST remain accessible for retry
