## ADDED Requirements

### Requirement: Reply API endpoint integration
The system SHALL use the ClickUp API's reply endpoint to create threaded comments with proper parent_id linkage.

#### Scenario: Reply sent to correct API endpoint
- **WHEN** creating a reply with parent_id
- **THEN** `create_comment_reply(parent_comment_id, &request)` is called
- **AND** NOT `create_comment(task_id, &request)`
- **AND** parent_comment_id is passed as the thread's parent comment ID

#### Scenario: CreateCommentRequest includes parent_id
- **WHEN** building CreateCommentRequest for a reply
- **THEN** `parent_id` field is set to Some(parent_comment_id)
- **AND** `comment_text` contains the reply content
- **AND** `assignee` and `assigned_commenter` are None (unless explicitly set)

#### Scenario: API response parsed correctly
- **WHEN** API returns successful reply creation response
- **THEN** response is deserialized to Comment struct
- **AND** parent_id is extracted from response
- **AND** comment is added to local comments vector

### Requirement: API error handling for replies
The system SHALL handle API errors gracefully when creating replies to provide clear feedback to users.

#### Scenario: Network error during reply creation
- **WHEN** network request fails (timeout, connection lost)
- **THEN** error message "Failed to create reply: {error}" is shown
- **AND** input form remains open for retry
- **AND** error is logged via tracing::error!

#### Scenario: Authentication error during reply creation
- **WHEN** API returns 401 Unauthorized
- **THEN** error message includes authentication failure
- **AND** user is prompted to re-authenticate
- **AND** input form is closed

#### Scenario: Rate limit error during reply creation
- **WHEN** API returns 429 Too Many Requests
- **THEN** error message "Rate limit exceeded. Please wait." is shown
- **AND** input form remains open for retry
- **AND** error is logged with rate limit details

#### Scenario: Invalid parent_id error
- **WHEN** API returns error for invalid parent_comment_id
- **THEN** error message "Invalid thread. Please reload comments." is shown
- **AND** user is advised to reload the task
- **AND** input form is closed

### Requirement: Async reply creation
The system SHALL create replies asynchronously to maintain UI responsiveness during API calls.

#### Scenario: Reply creation spawned as async task
- **WHEN** user submits reply (Ctrl+S)
- **THEN** reply creation is spawned with `tokio::spawn`
- **AND** UI remains responsive during API call
- **AND** message channel sends CommentCreated result when complete

#### Scenario: Message channel communication
- **WHEN** async reply creation completes
- **THEN** AppMessage::CommentCreated(Ok(comment)) is sent on success
- **OR** AppMessage::CommentCreated(Err(error)) is sent on failure
- **AND** message is received and processed by main update loop

#### Scenario: Client cloned for async operation
- **WHEN** spawning async reply creation
- **THEN** API client is cloned before spawn
- **AND** message sender channel is cloned before spawn
- **AND** both are moved into async task

### Requirement: API client method for replies
The system SHALL provide a dedicated API client method for creating comment replies.

#### Scenario: create_comment_reply method exists
- **WHEN** reply creation is needed
- **THEN** `ClickUpApi::create_comment_reply(parent_comment_id, &request)` is available
- **AND** method signature: `async fn create_comment_reply(&self, parent_comment_id: &str, comment: &CreateCommentRequest) -> Result<Comment>`

#### Scenario: Reply endpoint URL construction
- **WHEN** constructing API endpoint for reply
- **THEN** URL follows pattern: `/comment/{parent_comment_id}/comment`
- **AND** uses POST HTTP method
- **AND** includes authentication headers

#### Scenario: Mock client supports reply creation
- **WHEN** testing reply creation
- **THEN** MockClickUpClient implements create_comment_reply
- **AND** returns configured mock Comment response
- **AND** tests can run without network calls
