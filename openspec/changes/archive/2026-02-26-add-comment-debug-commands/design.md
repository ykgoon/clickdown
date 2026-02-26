## Context

The ClickDown CLI debug mode currently supports read-only operations (fetching workspaces, tasks, comments, etc.). When debugging comment creation issues—specifically the "failed to parse" error that occurs when saving replies in the TUI—developers cannot reproduce the issue in headless mode to see the actual API response.

The existing CLI infrastructure in `src/cli/` provides:
- Argument parsing in `src/cli/args.rs` with `DebugOperation` enum
- Command routing in `src/cli/run.rs`
- Operation implementations in `src/commands/debug_ops.rs`

The API client already has the necessary methods: `create_comment()`, `create_comment_reply()`, and `update_comment()`.

## Goals / Non-Goals

**Goals:**
- Add CLI debug commands for creating, replying to, and updating comments
- Support all comment creation options (text, parent_id, assignee, assigned_commenter)
- Enable verbose logging to show full API responses for debugging
- Support both human-readable and JSON output formats
- Follow existing CLI patterns and conventions

**Non-Goals:**
- Deleting comments (not needed for parsing debugging)
- Creating comments with attachments or reactions (API limitations)
- Changing the API client implementation
- Modifying TUI behavior

## Decisions

### 1. Command Structure

**Decision:** Use separate commands for create-comment, create-reply, and update-comment

**Rationale:**
- Clear separation of concerns
- Easier to discover via `--help`
- Matches existing CLI patterns (e.g., separate commands for different entity types)
- Alternative: Single `comment` command with sub-actions (more complex, less discoverable)

### 2. Option Naming

**Decision:** Use `--text` for comment content, `--parent-id` for threading, `--assignee` and `--assigned-commenter` for assignment

**Rationale:**
- `--text` is clear and concise (vs. `--comment-text` which is redundant)
- `--parent-id` matches the API field name
- `--assignee` and `--assigned-commenter` match API terminology
- Alternative: Use positional arguments (less flexible, harder to extend)

### 3. Required vs. Optional Arguments

**Decision:** `--text` is required for all write operations; task_id/comment_id are positional required arguments

**Rationale:**
- Empty comments are invalid (API rejects them)
- Clear error messages for missing required args
- Consistent with existing CLI commands

### 4. Output Format

**Decision:** Support both human-readable and JSON output via `--json` flag

**Rationale:**
- Human-readable for interactive debugging
- JSON for scripting and inspection with tools like `jq`
- Matches existing pattern (e.g., `tasks --json`, `comments --json`)

### 5. Error Handling

**Decision:** Use existing error handling in `run_cli()` with specific error messages for comment operations

**Rationale:**
- Consistent with existing debug commands
- Leverages existing exit codes (AUTH_ERROR, NETWORK_ERROR, GENERAL_ERROR)
- Verbose mode shows full error details

## Risks / Trade-offs

**[Risk] Command line length limits for long comments** → Mitigation: Document that very long comments should use JSON input file or stdin (future enhancement)

**[Risk] Token exposure in logs when using --verbose** → Mitigation: Already handled by existing logging that masks token values

**[Risk] API rate limiting during testing** → Mitigation: Use `--token` option for testing with different tokens; document rate limits

**[Trade-off] Not implementing stdin input** → Keeps initial implementation simple; can add later if needed

**[Trade-off] Not implementing batch operations** → Focus on single-comment debugging; batch can be added later
