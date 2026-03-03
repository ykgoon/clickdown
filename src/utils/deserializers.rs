//! Shared serde deserializer helpers
//!
//! This module provides reusable deserializer functions for handling common
//! API response variations like null values, type flexibility, and missing fields.

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

/// Helper function to deserialize null as empty string
///
/// Used for string fields that may be null or missing in API responses.
pub fn null_to_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

/// Helper function to deserialize null as empty vec for Vec<T> fields
pub fn null_to_empty_vec<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

/// Helper function to deserialize null as false for bool fields
///
/// Used for boolean fields that may be null in API responses.
pub fn null_to_false<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<bool>::deserialize(deserializer).map(|opt| opt.unwrap_or(false))
}

/// Flexible timestamp deserializer that handles both i64 or string
///
/// The ClickUp API may return timestamps in two formats:
/// - Integer: `1234567890000` (milliseconds since epoch)
/// - String: `"1234567890000"` (numeric string)
///
/// This deserializer accepts both formats and returns `Option<i64>`.
/// Null or missing fields return `None`.
pub fn flexible_timestamp<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
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
        Some(TimestampValue::String(s)) => s.parse::<i64>().map(Some).map_err(de::Error::custom),
    }
}

/// Flexible deserializer for integer fields that can be either i32 or string
pub fn flexible_int<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IntValue {
        Int(i32),
        String(String),
    }

    let opt = Option::<IntValue>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(IntValue::Int(v)) => Ok(Some(v)),
        Some(IntValue::String(s)) => s.parse::<i32>().map(Some).map_err(de::Error::custom),
    }
}

/// Flexible deserializer for i64 fields that can be either i64 or string
pub fn flexible_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum I64Value {
        Int(i64),
        String(String),
    }

    let opt = Option::<I64Value>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(I64Value::Int(v)) => Ok(Some(v)),
        Some(I64Value::String(s)) => s.parse::<i64>().map(Some).map_err(de::Error::custom),
    }
}

/// Flexible deserializer for ID fields that can be either string or integer
///
/// The ClickUp API may return IDs as either strings or integers.
/// This deserializer converts integers to strings and handles null/missing values.
pub fn flexible_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(i64),
    }

    let opt = Option::<StringOrInt>::deserialize(deserializer)?;
    match opt {
        None => Ok(String::new()),
        Some(StringOrInt::String(s)) => Ok(s),
        Some(StringOrInt::Int(i)) => Ok(i.to_string()),
    }
}

/// Helper function to deserialize null as default ID (0)
///
/// Used for user ID fields that may be null in API responses.
pub fn null_to_default_id<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<i64>::deserialize(deserializer).map(|opt| opt.unwrap_or(0))
}

/// Flexible deserializer for resolved field that can be either bool or i64
///
/// ClickUp API may return resolved as either format.
pub fn flexible_resolved<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ResolvedValue {
        Bool(bool),
        Int(i64),
    }

    let opt = Option::<ResolvedValue>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(ResolvedValue::Bool(b)) => Ok(Some(if b { 1 } else { 0 })),
        Some(ResolvedValue::Int(v)) => Ok(Some(v)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn test_null_to_empty_string() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(default, deserialize_with = "null_to_empty_string")]
            value: String,
        }

        let json = r#"{"value": null}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.value, "");

        let json = r#"{"value": "hello"}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.value, "hello");

        let json = r#"{}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.value, "");
    }

    #[test]
    fn test_null_to_empty_vec() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(default, deserialize_with = "null_to_empty_vec")]
            values: Vec<String>,
        }

        let json = r#"{"values": null}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert!(test.values.is_empty());

        let json = r#"{"values": ["a", "b"]}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.values, vec!["a", "b"]);

        let json = r#"{}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert!(test.values.is_empty());
    }

    #[test]
    fn test_null_to_false() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(default, deserialize_with = "null_to_false")]
            flag: bool,
        }

        let json = r#"{"flag": null}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert!(!test.flag);

        let json = r#"{"flag": true}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert!(test.flag);

        let json = r#"{}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert!(!test.flag);
    }

    #[test]
    fn test_flexible_timestamp_int() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(default, deserialize_with = "flexible_timestamp")]
            timestamp: Option<i64>,
        }

        let json = r#"{"timestamp": 1234567890}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.timestamp, Some(1234567890));

        let json = r#"{"timestamp": "1234567890"}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.timestamp, Some(1234567890));

        let json = r#"{"timestamp": null}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.timestamp, None);

        let json = r#"{}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.timestamp, None);
    }

    #[test]
    fn test_flexible_int() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(default, deserialize_with = "flexible_int")]
            value: Option<i32>,
        }

        let json = r#"{"value": 42}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.value, Some(42));

        let json = r#"{"value": "42"}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.value, Some(42));

        let json = r#"{"value": null}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.value, None);
    }

    #[test]
    fn test_flexible_i64() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(default, deserialize_with = "flexible_i64")]
            value: Option<i64>,
        }

        let json = r#"{"value": 1234567890123}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.value, Some(1234567890123));

        let json = r#"{"value": "1234567890123"}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.value, Some(1234567890123));
    }

    #[test]
    fn test_flexible_string() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(default, deserialize_with = "flexible_string")]
            id: String,
        }

        let json = r#"{"id": "abc123"}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.id, "abc123");

        let json = r#"{"id": 123456}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.id, "123456");

        let json = r#"{"id": null}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.id, "");

        let json = r#"{}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.id, "");
    }

    #[test]
    fn test_null_to_default_id() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(default, deserialize_with = "null_to_default_id")]
            id: i64,
        }

        let json = r#"{"id": null}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.id, 0);

        let json = r#"{"id": 123}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.id, 123);

        let json = r#"{}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.id, 0);
    }

    #[test]
    fn test_flexible_resolved() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(default, deserialize_with = "flexible_resolved")]
            resolved: Option<i64>,
        }

        let json = r#"{"resolved": true}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.resolved, Some(1));

        let json = r#"{"resolved": false}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.resolved, Some(0));

        let json = r#"{"resolved": 42}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.resolved, Some(42));

        let json = r#"{"resolved": null}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(test.resolved, None);
    }
}
