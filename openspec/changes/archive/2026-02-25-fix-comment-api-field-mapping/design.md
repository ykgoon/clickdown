# Design: Fix Comment API Field Mapping

## Context

The Comment model incorrectly maps ClickUp API response fields, causing all comment data to deserialize as empty/null values. The model was written with assumed field names that don't match the actual API response.

**Current API Response Format** (verified from ClickUp API documentation):
```json
{
  "comments": [
    {
      "id": "458",
      "comment": [{"text": "Task comment content"}],
      "comment_text": "Task comment content",
      "user": {
        "id": 183,
        "username": "John Doe",
        "email": "johndoe@gmail.com",
        "color": "#827718",
        "profilePicture": "https://...",
        "initials": "JD"
      },
      "resolved": false,
      "assignee": {...},
      "assigned_by": {...},
      "reactions": [],
      "date": "1568036964079"
    }
  ]
}
```

## Goals

**Goals:**
- Fix Comment model to correctly deserialize ClickUp API responses
- Maintain backward compatibility with existing code that uses Comment fields
- Update all tests to use correct API response format
- Handle edge cases (null fields, string vs i64 timestamps)

**Non-Goals:**
- Adding new comment features (replies, reactions, etc.)
- Changing comment rendering logic
- Modifying API client interface

## Decisions

### 1. Field Rename Mappings

**Decision:** Update serde rename attributes to match actual API field names:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    
    #[serde(default, deserialize_with = "null_to_empty_string", rename = "comment_text")]
    pub text: String,
    
    #[serde(default, deserialize_with = "null_to_empty_string", rename = "text_preview")]
    pub text_preview: String,
    
    #[serde(default, rename = "user")]
    pub commenter: Option<User>,
    
    #[serde(default, rename = "date", deserialize_with = "flexible_string_timestamp")]
    pub created_at: Option<i64>,
    
    #[serde(default, rename = "date_updated", deserialize_with = "flexible_string_timestamp")]
    pub updated_at: Option<i64>,
    
    #[serde(default, rename = "assignee")]
    pub assigned_commenter: Option<User>,
    
    #[serde(default, rename = "assigned_by")]
    pub assigned_by: Option<User>,
    
    #[serde(default, deserialize_with = "null_to_false", rename = "resolved")]
    pub assigned: bool,
    
    #[serde(default, deserialize_with = "reactions_to_string")]
    pub reaction: String,
}
```

**Rationale:** Preserves existing code that uses `comment.text`, `comment.commenter`, etc. while correctly mapping from API field names.

### 2. String Timestamp Deserializer

**Decision:** Create new deserializer that handles string timestamps:

```rust
fn flexible_string_timestamp<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TimestampValue {
        Int(i64),
        String(String),
    }
    
    let opt = Option::<TimestampValue>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(TimestampValue::Int(v)) => Ok(Some(v)),
        Some(TimestampValue::String(s)) => {
            s.parse::<i64>().map(Some).map_err(D::Error::custom)
        }
    }
}
```

**Rationale:** API returns `date` as string (e.g., `"1568036964079"`), but may return i64 in some cases. Flexible deserializer handles both.

### 3. Reactions Array to String

**Decision:** Convert reactions array to string representation:

```rust
fn reactions_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<Vec<String>>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default().join(","))
}
```

**Rationale:** API returns `reactions` as array `["thumbsup", "heart"]`, but model uses single string. Join with commas for compatibility.

### 4. Text Preview Field

**Decision:** Keep `text_preview` with `rename = "text_preview"` but mark as optional.

**Rationale:** API may or may not include this field. It's a truncated preview of comment text.

### 5. Updated Field Semantics

| Field | API Source | Notes |
|-------|-----------|-------|
| `text` | `comment_text` | Main comment body |
| `text_preview` | `text_preview` | Optional truncated preview |
| `commenter` | `user` | Comment author |
| `created_at` | `date` | String timestamp |
| `updated_at` | `date_updated` | May not exist for all comments |
| `assigned_commenter` | `assignee` | User assigned via comment |
| `assigned_by` | `assigned_by` | User who made assignment |
| `assigned` | `resolved` | Using `resolved` field as proxy |
| `reaction` | `reactions` | Array → comma-separated string |

## Risks / Trade-offs

**[Breaking Change Risk]** → If any code directly serializes Comment objects with old field names, JSON output will change. **Mitigation:** This is internal model, serialization not used externally.

**[Timestamp Format Variance]** → API might return timestamps in different formats. **Mitigation:** Flexible deserializer handles both string and i64.

**[Missing Fields]** → Some API responses might omit certain fields. **Mitigation:** All fields use `#[serde(default)]` for safe deserialization.

**[Assigned vs Resolved Semantics]** → Using `resolved` field for `assigned` may not be semantically correct. **Investigation needed:** Verify if `resolved` and `assigned` have different meanings.

## Test Strategy

### Unit Tests
- Test deserialization from actual API response format
- Test string timestamp parsing
- Test reactions array conversion
- Test null/missing field handling

### Integration Tests
- Test with mock API responses matching real ClickUp format
- Verify comment display in TUI with correctly deserialized data
- Test comment CRUD operations

### Fixtures Update
Update `tests/fixtures.rs` to create test comments with correct field structure matching API response.
