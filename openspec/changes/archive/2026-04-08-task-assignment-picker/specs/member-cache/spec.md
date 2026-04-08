## ADDED Requirements

### Requirement: In-memory member cache
The system SHALL maintain an in-memory cache of list members in `TuiApp` state, stored as a `HashMap<String, Vec<User>>` keyed by list ID. The cache SHALL be populated on first fetch per list per session and served from cache on subsequent accesses.

#### Scenario: First access triggers fetch
- **WHEN** assignee picker is opened for list "list_123"
- **AND** the cache does not contain "list_123"
- **THEN** the system calls `get_list_members("list_123")` from the API
- **AND** stores the result in the cache under key "list_123"
- **AND** displays the picker with the fetched members

#### Scenario: Subsequent access uses cache
- **WHEN** assignee picker is opened for list "list_123"
- **AND** the cache already contains "list_123"
- **THEN** the system uses the cached members directly
- **AND** no API call is made

### Requirement: Cache lifetime
The member cache SHALL live for the duration of the application session. It SHALL be cleared when the application exits or restarts. No persistence to disk or database is required.

#### Scenario: Cache cleared on restart
- **WHEN** the application restarts
- **THEN** the member cache is empty
- **AND** the next picker open triggers a fresh API call

### Requirement: Cache loading state
When fetching members from the API, the system SHALL display a loading indicator in the picker overlay until the response arrives or an error occurs.

#### Scenario: Loading members from API
- **WHEN** cache miss triggers an API fetch
- **THEN** the picker displays "Loading members..."
- **AND** user input is blocked until response arrives

#### Scenario: API error during fetch
- **WHEN** API fetch fails with an error
- **THEN** the picker displays the error message
- **AND** offers to retry or close with Esc

### Requirement: Lazy per-list caching
The cache SHALL be populated lazily — only for lists whose picker has been opened. It SHALL NOT pre-fetch members for all lists in the workspace.

#### Scenario: Only opened lists are cached
- **WHEN** user opens picker for "list_123" but never opens picker for "list_456"
- **THEN** the cache contains only "list_123"
- **AND** no API call was made for "list_456"
