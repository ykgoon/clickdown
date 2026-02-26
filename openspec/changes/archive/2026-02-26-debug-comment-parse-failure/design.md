## Context

The ClickDown application uses serde for JSON deserialization of ClickUp API responses. The Comment model includes custom deserializers for handling null values, flexible timestamps, and optional fields. However, users report "failed to parse" errors when saving comment replies in the TUI, even though the API call succeeds.

The CLI debug commands (`create-comment`, `create-reply`, `update-comment`) have been implemented and provide the tooling needed to reproduce and diagnose this issue. The current Comment model handles many edge cases, but the actual root cause of the parsing failure is unknown and requires investigation with real API responses.

**Current State:**
- Comment model exists with custom deserializers for null-safety
- CLI debug mode supports comment operations with `--verbose` and `--json` flags
- Error messages include truncated response body (200 chars) but lack field-level diagnostics
- No systematic logging of which specific field caused deserialization to fail

**Constraints:**
- Must maintain backward compatibility with existing API responses
- Cannot modify ClickUp API behavior - must handle what it returns
- Investigation requires real API access (cannot be fully tested with mocks)

## Goals / Non-Goals

**Goals:**
- Identify the specific field(s) or format causing deserialization failures
- Implement robust deserializers that handle all observed API response variations
- Provide clear diagnostic information when parsing fails
- Add comprehensive test coverage for edge cases
- Document known API response formats and handling strategies

**Non-Goals:**
- Changing the Comment struct's public API (field names, types visible to rest of codebase)
- Implementing comment attachments or reactions deserialization (separate concern)
- Modifying how comments are displayed or navigated in the TUI
- Changing the API client's error handling patterns

## Decisions

### 1. Investigation Approach

**Decision:** Use CLI debug mode with `--verbose --json` to capture actual API responses that fail parsing

**Rationale:**
- CLI debug commands are already implemented and tested
- Verbose mode logs full response body without truncation
- JSON output allows inspection with tools like `jq`
- Can reproduce issue in controlled manner before testing in TUI
- Alternative: Add logging to TUI code path (more invasive, harder to isolate)

**Alternatives Considered:**
- Add network packet capture (overkill, requires external tools)
- Mock server to intercept responses (complex setup, doesn't help with existing issue)

### 2. Error Message Enhancement

**Decision:** Enhance parse error messages to include field-level diagnostics using serde's error context

**Rationale:**
- Current error: "Failed to parse response: {truncated JSON}" lacks specificity
- Serde provides line/column information for parse errors
- Can use `serde_path_to_error` crate for field-level error paths
- Helps developers quickly identify problematic fields without code inspection

**Implementation:**
```rust
// Wrap serde_json deserialization with better error reporting
let mut deserializer = serde_json::Deserializer::from_str(&body);
let result: Result<T, _> = serde_path_to_error::deserialize(&mut deserializer);
```

**Alternatives Considered:**
- Custom Deserialize implementation for Comment (too verbose, hard to maintain)
- Log full response and let developer inspect manually (already possible with --verbose)

### 3. Unknown Fields Handling

**Decision:** Explicitly allow unknown fields using `#[serde(deny_unknown_fields)]` removal (default behavior)

**Rationale:**
- Serde ignores unknown fields by default - this is correct behavior
- ClickUp API may add fields without notice; robustness principle applies
- No code change needed - current behavior is correct
- Document this behavior to prevent future "fixes" that add deny_unknown_fields

### 4. Reactions Field Strategy

**Decision:** Keep `reaction: String` field as-is; do not implement `reactions: Vec<String>` deserialization yet

**Rationale:**
- Current code has `reaction: String` (singular) - tests mention `reactions` array but struct doesn't have it
- Unknown fields are ignored by serde, so `reactions` array won't cause parse failures
- If `reaction` field is the issue, need to understand what format API actually returns
- Implement reactions deserialization only if/when the field is needed by the application

**Alternatives Considered:**
- Add `reactions: Option<Vec<String>>` field (premature, not required by current features)
- Change `reaction` to handle both string and array (complexity without clear benefit)

### 5. Timestamp Deserializer Enhancement

**Decision:** Extend `flexible_timestamp` to handle additional formats if investigation reveals they're needed

**Rationale:**
- Current deserializer handles i64 and string representations
- May need to handle ISO 8601 strings or other formats
- Decision deferred until investigation reveals actual API format variations
- Add test cases for any new formats discovered

### 6. Testing Strategy

**Decision:** Use fixture-based testing with real API response samples captured during investigation

**Rationale:**
- Mock client tests verify code paths but not actual API compatibility
- Fixture tests with real response JSON ensure deserializer handles actual data
- Can capture edge cases (null fields, missing fields, type variations)
- Tests serve as documentation of expected API behavior

**Implementation:**
```rust
#[test]
fn test_comment_parse_api_response_sample_1() {
    let json = r#"{...captured from real API...}"#;
    let comment: Comment = serde_json::from_str(json).unwrap();
    // assertions
}
```

## Risks / Trade-offs

**[Risk] Investigation requires real API access** → Mitigation: CLI debug commands already implemented; can test with user's API token or capture responses from TUI

**[Risk] ClickUp API response format varies by workspace/plan** → Mitigation: Test with multiple workspaces if possible; document any known variations; use most permissive deserializers

**[Risk] Fix may mask future API changes** → Mitigation: Enhanced logging will catch new issues quickly; test coverage reduces regression risk

**[Risk] Adding dependencies for better error messages** → Mitigation: `serde_path_to_error` is small, well-maintained; only adds ~5 dependencies; benefit outweighs cost

**[Trade-off] Not implementing reactions deserialization** → Keeps scope focused; can add later if needed; unknown fields are safely ignored

**[Trade-off] Not adding comprehensive ISO 8601 timestamp support** → Only add formats that are actually needed based on investigation findings

## Migration Plan

Not applicable - this is a bug fix/investigation change with no migration required.

## Open Questions

1. **What specific field(s) are causing the parse failure?** - Requires investigation with real API responses
2. **Does the failure occur only with replies, or also with top-level comments?** - Need to test both scenarios
3. **Is the issue workspace-specific or universal?** - May need to test with multiple workspaces
4. **Does ClickUp API return different formats for different comment operations (create vs. update vs. fetch)?** - Need to capture responses from all operations
