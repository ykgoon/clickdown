## 1. CLI Argument Parsing

- [x] 1.1 Add `CreateComment`, `CreateReply`, and `UpdateComment` variants to `DebugOperation` enum in `src/cli/args.rs`
- [x] 1.2 Add `--text`, `--parent-id`, `--assignee`, and `--assigned-commenter` option parsing in `parse_debug_command()`
- [x] 1.3 Add argument validation for comment operations (task_id/comment_id required, text required)
- [x] 1.4 Update help text in `print_usage()` to document new comment commands

## 2. Debug Operations Implementation

- [x] 2.1 Implement `create_comment()` method in `src/commands/debug_ops.rs`
- [x] 2.2 Implement `create_comment_json()` method for JSON output
- [x] 2.3 Implement `create_reply()` method in `src/commands/debug_ops.rs`
- [x] 2.4 Implement `create_reply_json()` method for JSON output
- [x] 2.5 Implement `update_comment()` method in `src/commands/debug_ops.rs`
- [x] 2.6 Implement `update_comment_json()` method for JSON output

## 3. CLI Run Integration

- [x] 3.1 Add routing for `CreateComment` operation in `src/cli/run.rs`
- [x] 3.2 Add routing for `CreateReply` operation in `src/cli/run.rs`
- [x] 3.3 Add routing for `UpdateComment` operation in `src/cli/run.rs`
- [x] 3.4 Ensure error handling uses appropriate exit codes (AUTH_ERROR, NETWORK_ERROR, GENERAL_ERROR)

## 4. Testing

- [x] 4.1 Test `create-comment` command with valid task ID and text
- [x] 4.2 Test `create-comment` command with `--json` output
- [x] 4.3 Test `create-reply` command with valid comment ID and text
- [x] 4.4 Test `update-comment` command with valid comment ID and text
- [x] 4.5 Test `--verbose` mode shows API request/response details
- [x] 4.6 Test error cases (invalid IDs, empty text, auth errors)
- [x] 4.7 Test `--parent-id`, `--assignee`, `--assigned-commenter` options

## 5. Documentation

- [x] 5.1 Verify help text displays correctly with `clickdown debug --help`
- [x] 5.2 Update AGENTS.md or README.md with new debug commands if needed
