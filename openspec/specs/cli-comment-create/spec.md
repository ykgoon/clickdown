# cli-comment-create Specification

## Purpose
Define CLI debug mode capabilities for creating, replying to, and updating comments for debugging purposes.

## Requirements

### Requirement: Create Comment Operation

The system SHALL provide a debug operation to create a new top-level comment on a task.

#### Scenario: Create comment successfully
- **WHEN** user executes `clickdown debug create-comment <task_id> --text "comment text"`
- **THEN** the system SHALL call `create_comment(task_id, request)` API method
- **AND** the system SHALL print "Comment created: <comment_id>" to stdout
- **AND** the system SHALL exit with code 0

#### Scenario: Create comment with JSON output
- **WHEN** user executes `clickdown debug create-comment <task_id> --text "comment text" --json`
- **THEN** the system SHALL output the created comment as JSON to stdout
- **AND** the JSON SHALL include all fields returned by the API

#### Scenario: Create comment fails due to invalid task ID
- **WHEN** user executes `clickdown debug create-comment <invalid_id> --text "text"`
- **THEN** the system SHALL print "Task not found" error to stderr
- **AND** the system SHALL exit with code 1

#### Scenario: Create comment fails due to empty text
- **WHEN** user executes `clickdown debug create-comment <task_id> --text ""`
- **THEN** the system SHALL print "Comment text cannot be empty" to stderr
- **AND** the system SHALL exit with code 2

#### Scenario: Create comment fails due to auth error
- **WHEN** user executes without valid token
- **THEN** the system SHALL print authentication error to stderr
- **AND** the system SHALL exit with code 3

### Requirement: Create Reply Operation

The system SHALL provide a debug operation to create a reply to an existing comment.

#### Scenario: Create reply successfully
- **WHEN** user executes `clickdown debug create-reply <comment_id> --text "reply text"`
- **THEN** the system SHALL call `create_comment_reply(comment_id, request)` API method
- **AND** the system SHALL print "Reply created: <reply_id>" to stdout
- **AND** the system SHALL exit with code 0

#### Scenario: Create reply with JSON output
- **WHEN** user executes `clickdown debug create-reply <comment_id> --text "reply text" --json`
- **THEN** the system SHALL output the created reply as JSON to stdout
- **AND** the JSON SHALL include `parent_id` field set to the parent comment ID

#### Scenario: Create reply fails due to invalid comment ID
- **WHEN** user executes `clickdown debug create-reply <invalid_id> --text "text"`
- **THEN** the system SHALL print "Comment not found" error to stderr
- **AND** the system SHALL exit with code 1

#### Scenario: Create reply fails due to empty text
- **WHEN** user executes `clickdown debug create-reply <comment_id> --text ""`
- **THEN** the system SHALL print "Reply text cannot be empty" to stderr
- **AND** the system SHALL exit with code 2

### Requirement: Update Comment Operation

The system SHALL provide a debug operation to update an existing comment.

#### Scenario: Update comment successfully
- **WHEN** user executes `clickdown debug update-comment <comment_id> --text "updated text"`
- **THEN** the system SHALL call `update_comment(comment_id, request)` API method
- **AND** the system SHALL print "Comment updated: <comment_id>" to stdout
- **AND** the system SHALL exit with code 0

#### Scenario: Update comment with JSON output
- **WHEN** user executes `clickdown debug update-comment <comment_id> --text "text" --json`
- **THEN** the system SHALL output the updated comment as JSON to stdout

#### Scenario: Update comment fails due to invalid comment ID
- **WHEN** user executes `clickdown debug update-comment <invalid_id> --text "text"`
- **THEN** the system SHALL print "Comment not found" error to stderr
- **AND** the system SHALL exit with code 1

#### Scenario: Update comment fails due to empty text
- **WHEN** user executes `clickdown debug update-comment <comment_id> --text ""`
- **THEN** the system SHALL print "Comment text cannot be empty" to stderr
- **AND** the system SHALL exit with code 2

### Requirement: Verbose Logging for Comment Operations

The system SHALL provide detailed logging for comment creation and update operations.

#### Scenario: Verbose mode shows API request details
- **WHEN** user executes `clickdown debug create-comment <task_id> --text "text" --verbose`
- **THEN** the system SHALL log the API endpoint URL to stderr
- **AND** the system SHALL log the HTTP method (POST) to stderr

#### Scenario: Verbose mode shows API response body
- **WHEN** user executes a comment operation with `--verbose`
- **THEN** the system SHALL log the full API response body to stderr
- **AND** the response body SHALL be logged BEFORE parsing attempts

#### Scenario: Verbose mode shows parsing errors
- **WHEN** API response parsing fails
- **THEN** the system SHALL log the parsing error with the first 200 characters of the response body
- **AND** the system SHALL print a user-friendly error message to stderr
