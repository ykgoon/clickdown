## Context

ClickDown currently operates as a TUI application using ratatui and crossterm. There is no CLI interface for debugging or headless operation. The application uses tokio for async operations, reqwest for HTTP calls, and tracing for logging.

The change introduces a CLI debug mode that allows users to run ClickDown commands from the terminal without launching the full TUI, making it easier to reproduce bugs and debug issues.

## Goals / Non-Goals

**Goals:**
- Add a `debug` subcommand to ClickDown for headless debugging operations
- Support common debugging operations: workspace listing, task fetching, document viewing
- Enable verbose logging output for troubleshooting
- Provide JSON output format for programmatic consumption
- Integrate with existing tracing/logging infrastructure
- Maintain compatibility with existing TUI mode

**Non-Goals:**
- Full-featured CLI application (this is debug-focused)
- Interactive CLI REPL mode
- Modifying ClickUp API integration (just new entry points)
- Replacing the TUI interface

## Decisions

### Decision 1: Use crossterm's CLI capabilities vs. external crate

**Choice:** Use existing crossterm crate (already in dependencies) for terminal operations, no new CLI parser dependency.

**Rationale:** ClickDown already uses crossterm for terminal operations. For a debug-focused CLI, we can parse simple subcommands manually without adding clap or similar. This keeps dependencies minimal and aligns with the "debug mode" scope.

**Alternatives considered:**
- `clap`: Industry standard but adds ~20 dependencies for simple use case
- `structopt`: Derive-based but adds complexity
- Manual parsing: Lightweight, sufficient for 3-4 subcommands

### Decision 2: Subcommand structure

**Choice:** `clickdown debug <operation> [options]`

**Rationale:** Keeps debug mode isolated from potential future CLI commands. Operations include:
- `clickdown debug workspaces` - List all workspaces
- `clickdown debug tasks <list_id>` - Fetch tasks from a list
- `clickdown debug docs <search_query>` - Search documents
- `clickdown debug auth-status` - Check authentication status

**Options:**
- `--json` - Output in JSON format
- `--verbose` / `-v` - Enable verbose logging
- `--token <token>` - Override stored token (for testing)

### Decision 3: Logging integration

**Choice:** Leverage existing tracing-subscriber with environment variable control

**Rationale:** ClickDown already uses tracing. Debug mode will set `RUST_LOG=debug` or `RUST_LOG=trace` when `--verbose` is passed. No new logging infrastructure needed.

**Implementation:** 
```rust
if verbose {
    std::env::set_var("RUST_LOG", "debug");
}
```

### Decision 4: Error handling and exit codes

**Choice:** Use standard Unix exit codes with stderr output

**Rationale:** Aligns with CLI conventions:
- `0` - Success
- `1` - General error
- `2` - Invalid arguments
- `3` - Authentication error
- `4` - Network error

Errors go to stderr, data goes to stdout (when using `--json`).

## Risks / Trade-offs

**[Risk]** Manual argument parsing may become unwieldy if CLI expands beyond debug mode.

→ **Mitigation:** Document that this is debug-only. If full CLI is needed later, migrate to clap.

**[Risk]** Debug mode may expose sensitive data (tokens, API responses) in logs.

→ **Mitigation:** Add warning message when `--verbose` is used. Never log tokens even in verbose mode.

**[Risk]** Headless mode may not reproduce TUI-specific bugs.

→ **Mitigation:** Document that debug mode is for API/data issues, not rendering bugs.

**[Risk]** Duplicating code between TUI and CLI for fetching data.

→ **Mitigation:** Factor out data-fetching logic into shared module (e.g., `src/commands/`).

## Migration Plan

1. Create `src/cli/` module with subcommand parsing
2. Add `src/commands/` for shared debug operations
3. Modify `src/main.rs` to route based on subcommand
4. Update error handling to support exit codes
5. Test with existing mock client infrastructure

**Rollback strategy:** Feature is additive (new subcommand). If issues arise, users can ignore the subcommand. No breaking changes.

## Open Questions

- Should debug mode support write operations (create/update tasks) for testing?
  - Leaning toward: Yes, but mark as "advanced debug" with confirmation prompts
- Should we cache results like the TUI does, or always fetch fresh?
  - Leaning toward: Always fetch fresh for debugging accuracy
