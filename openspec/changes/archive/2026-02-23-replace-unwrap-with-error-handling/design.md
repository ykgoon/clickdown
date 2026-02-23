## Context

The codebase has 7 instances of `.unwrap()` or `.expect()` calls that could panic:
- 4 in `app.rs` for client operations
- 1 in `config/mod.rs` for ConfigManager initialization
- 1 in `api/auth.rs` for AuthManager initialization

## Goals / Non-Goals

**Goals:**
- Replace panic-prone code with proper error propagation
- Maintain backward compatibility with existing behavior
- Follow Rust idioms for error handling

**Non-Goals:**
- Refactoring error types or error handling strategy
- Adding new error variants beyond what's needed
- Changing the overall application architecture

## Decisions

### Decision 1: Use `?` operator for client operations

In `app.rs`, the `load_*` methods return `iced::Task<Message>`. We'll use `.ok_or_else()` to convert `Option` to `Result`, then `?` to propagate, mapping errors to `Message::AuthError`.

**Rationale:** Minimal code change, maintains existing error flow to UI.

### Decision 2: Change default() to return Result

For `ConfigManager` and `AuthManager`, we'll change `default()` to return `Result<Self, Error>` instead of panicking.

**Rationale:** Callers can decide how to handle initialization failure. This is a breaking change but more correct.
