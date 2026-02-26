## Why

When debugging comment creation issues (particularly the "failed to parse" error when saving replies), developers currently have no way to reproduce the issue outside the TUI. The ClickUp API may return different response formats for create/update operations vs. fetch operations, but without a debug command to create comments, we cannot see the actual API responses to diagnose parsing failures.

## What Changes

- **New CLI debug command**: `create-comment <task_id>` - Create a new top-level comment on a task
- **New CLI debug command**: `create-reply <comment_id>` - Create a reply to an existing comment
- **New CLI debug command**: `update-comment <comment_id>` - Update an existing comment
- **New options**: `--text`, `--parent-id`, `--assignee`, `--assigned-commenter` for comment creation
- **Enhanced verbose logging**: Show full API response body when parsing fails
- **JSON output support**: All new commands support `--json` for programmatic use

## Capabilities

### New Capabilities
- `cli-comment-create`: Debug commands for creating, replying to, and updating comments via CLI
- `cli-comment-options`: Support for comment creation options (parent_id, assignee, assigned_commenter)

### Modified Capabilities
- `cli-debug-mode`: Extending existing debug mode to support comment write operations

## Impact

- **Files**: `src/cli/args.rs`, `src/cli/run.rs`, `src/commands/debug_ops.rs`
- **API**: No changes to ClickUp API client - uses existing `create_comment()`, `create_comment_reply()`, `update_comment()` methods
- **Dependencies**: None - builds on existing CLI debug infrastructure
- **Breaking**: None - additive change only
