# cli-debug-mode Specification (Delta)

## Purpose
Extend CLI debug mode to support comment write operations for debugging.

## ADDED Requirements

### Requirement: Create Comment Operation

The system SHALL provide a debug operation to create a new top-level comment on a task.

#### Scenario: Create comment successfully
- **WHEN** user executes `clickdown debug create-comment <task_id> --text "comment text"`
- **THEN** the system SHALL fetch tasks from the specified list
- **AND** the system SHALL call the ClickUp API to create a comment
- **AND** the system SHALL print "Comment created: <comment_id>" to stdout

#### Scenario: Create comment with JSON output
- **WHEN** user executes `clickdown debug create-comment <task_id> --text "text" --json`
- **THEN** the system SHALL output the created comment as a JSON object to stdout

### Requirement: Create Reply Operation

The system SHALL provide a debug operation to create a reply to an existing comment.

#### Scenario: Create reply successfully
- **WHEN** user executes `clickdown debug create-reply <comment_id> --text "reply text"`
- **THEN** the system SHALL call the ClickUp API to create a reply
- **AND** the system SHALL print "Reply created: <reply_id>" to stdout

#### Scenario: Create reply with JSON output
- **WHEN** user executes `clickdown debug create-reply <comment_id> --text "text" --json`
- **THEN** the system SHALL output the created reply as a JSON object to stdout

### Requirement: Update Comment Operation

The system SHALL provide a debug operation to update an existing comment.

#### Scenario: Update comment successfully
- **WHEN** user executes `clickdown debug update-comment <comment_id> --text "updated text"`
- **THEN** the system SHALL call the ClickUp API to update the comment
- **AND** the system SHALL print "Comment updated: <comment_id>" to stdout

#### Scenario: Update comment with JSON output
- **WHEN** user executes `clickdown debug update-comment <comment_id> --text "text" --json`
- **THEN** the system SHALL output the updated comment as a JSON object to stdout

### Requirement: Comment Operation Options

The system SHALL support options for comment operations.

#### Scenario: Specify comment text
- **WHEN** user executes `clickdown debug create-comment <task_id> --text "text"`
- **THEN** the `--text` option SHALL specify the comment content
- **AND** the option SHALL be required

#### Scenario: Specify parent ID for threading
- **WHEN** user executes `clickdown debug create-comment <task_id> --text "text" --parent-id <comment_id>`
- **THEN** the `--parent-id` option SHALL create a reply to the specified comment

#### Scenario: Specify assignee
- **WHEN** user executes `clickdown debug create-comment <task_id> --text "text" --assignee <user_id>`
- **THEN** the `--assignee` option SHALL assign the comment to the specified user

#### Scenario: Specify assigned commenter
- **WHEN** user executes `clickdown debug create-comment <task_id> --text "text" --assigned-commenter <user_id>`
- **THEN** the `--assigned-commenter` option SHALL set who assigned the comment
