## 1. Module Setup

- [x] 1.1 Create `src/cli/` directory structure
- [x] 1.2 Create `src/commands/` directory for shared operations
- [x] 1.3 Add module declarations in `src/lib.rs` or `src/main.rs`

## 2. CLI Argument Parsing

- [x] 2.1 Implement subcommand parser for `debug` command
- [x] 2.2 Implement `--json` flag parsing
- [x] 2.3 Implement `--verbose` / `-v` flag parsing
- [x] 2.4 Implement `--token <token>` override flag parsing
- [x] 2.5 Add usage/help message display for invalid commands

## 3. Debug Operations Implementation

- [x] 3.1 Implement `workspaces` operation (list all workspaces)
- [x] 3.2 Implement `tasks <list_id>` operation (fetch tasks from list)
- [x] 3.3 Implement `docs <query>` operation (search documents)
- [x] 3.4 Implement `auth-status` operation (check authentication)
- [x] 3.5 Add JSON output formatting for each operation
- [x] 3.6 Add human-readable output formatting for each operation

## 4. Logging Integration

- [x] 4.1 Add logging configuration in CLI entry point
- [x] 4.2 Implement `--verbose` flag to set RUST_LOG to "debug"
- [x] 4.3 Ensure sensitive data redaction in logs (tokens, credentials)
- [x] 4.4 Configure stderr for log output, stdout for data output

## 5. Error Handling and Exit Codes

- [x] 5.1 Define exit code constants (0=success, 1=error, 2=invalid args, 3=auth, 4=network)
- [x] 5.2 Implement error-to-exit-code mapping
- [x] 5.3 Add CLI-specific error message formatting
- [x] 5.4 Implement verbose error details with `--verbose` flag

## 6. Main Entry Point Integration

- [x] 6.1 Modify `src/main.rs` to detect CLI subcommands
- [x] 6.2 Route to TUI mode when no subcommand provided
- [x] 6.3 Route to CLI debug mode when `debug` subcommand provided
- [x] 6.4 Ensure proper cleanup and exit code return

## 7. Testing

- [x] 7.1 Add unit tests for argument parsing
- [x] 7.2 Add integration tests using MockClickUpClient
- [x] 7.3 Test each debug operation with mock data
- [x] 7.4 Test exit codes for error scenarios
- [x] 7.5 Manual testing: verify JSON output format
- [x] 7.6 Manual testing: verify verbose logging output

## 8. Documentation

- [x] 8.1 Add CLI debug mode section to README.md
- [x] 8.2 Document available subcommands and options
- [x] 8.3 Add examples for common debugging scenarios
- [x] 8.4 Update TESTING.md with CLI testing instructions
- [x] 8.5 Add "Debugging with CLI Mode" section to AGENTS.md
- [x] 8.6 Include copy-paste ready examples in AGENTS.md
- [x] 8.7 Document common debugging workflows in AGENTS.md
