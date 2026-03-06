## Context

The ClickDown application has an "Assigned to Me" navigation feature that displays tasks assigned to the current user. The feature uses the `get_tasks_with_assignee()` method in the API client, which builds query parameters using `TaskFilters::to_query_string()`.

**Current Implementation:**
- `TaskFilters` uses `add_all_ints()` to add assignees to the query string
- This produces: `assignees[]=123&assignees[]=456`
- ClickUp API v2 expects: `assignees=123,456`

**Problem:**
The array-style query parameter format (`assignees[]`) does not match ClickUp API's expected comma-separated format, potentially causing the filter to be ignored or return incorrect results.

## Goals / Non-Goals

**Goals:**
- Fix the assignees filter parameter format to match ClickUp API v2 specification
- Maintain backward compatibility with other filter parameters (statuses, etc.)
- Add minimal code changes to achieve correct parameter format
- Add test coverage for the new format

**Non-Goals:**
- Changing other filter parameter formats (statuses, order_by, etc.)
- Refactoring the entire QueryParams utility
- Adding new assignee-related features beyond the filter fix

## Decisions

### Decision 1: Add specialized method for comma-separated integers

**Approach:** Add a new method `add_comma_separated_ints()` to `QueryParams` for parameters that require comma-separated values instead of repeated array parameters.

**Rationale:**
- Minimal change to existing code
- Clear separation between array-style and comma-separated parameters
- Reusable for other similar parameters if needed in the future

**Alternatives Considered:**
1. *Change `add_all_ints()` behavior*: Would break existing functionality for other parameters that correctly use array format
2. *Hard-code assignees in TaskFilters*: Would duplicate query-building logic and reduce maintainability
3. *Use a separate utility function*: Would add unnecessary indirection

### Decision 2: Update TaskFilters to use comma-separated format for assignees only

**Approach:** Modify `TaskFilters::to_query_string()` to use the new `add_comma_separated_ints()` method specifically for the `assignees` field.

**Rationale:**
- Targeted fix that only affects the problematic parameter
- Other filter parameters (statuses) continue using array format as expected by ClickUp API
- Clear and explicit about which parameters use which format

## Risks / Trade-offs

**[Risk]** ClickUp API may accept both formats, making this a non-issue
→ **Mitigation:** Test with real API calls to verify the fix improves results

**[Risk]** Other query parameters may also need comma-separated format
→ **Mitigation:** The new `add_comma_separated_ints()` method can be reused for other parameters if discovered

**[Risk]** Breaking change if any internal code relies on the array format
→ **Mitigation:** No known internal code parses these query strings; they are only sent to ClickUp API

## Migration Plan

1. Add `add_comma_separated_ints()` method to `QueryParams`
2. Update `TaskFilters::to_query_string()` to use the new method for assignees
3. Add unit tests for the new method and updated TaskFilters
4. Test with real ClickUp API to verify correct behavior
5. No database migrations or config changes required

## Open Questions

None - this is a straightforward parameter format fix.
