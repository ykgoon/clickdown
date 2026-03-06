//! Query parameter builder
//!
//! This module provides a helper for building URL query parameter strings.

/// Query parameter builder for constructing URL query strings
///
/// This helper reduces duplication in building query parameter strings
/// from optional fields.
///
/// # Example
///
/// ```
/// use clickdown::utils::QueryParams;
///
/// let mut params = QueryParams::new();
/// params.add_opt("archived", Some(true));
/// params.add_opt("page", Some(2));
/// params.add_all("statuses", &["todo", "in progress"]);
///
/// assert_eq!(params.to_query_string(), "?archived=true&page=2&statuses[]=todo&statuses[]=in progress");
/// ```
#[derive(Debug, Clone, Default)]
pub struct QueryParams {
    params: Vec<String>,
}

impl QueryParams {
    /// Create a new empty query params builder
    pub fn new() -> Self {
        Self { params: Vec::new() }
    }

    /// Add a parameter if the value is Some
    pub fn add_opt<T: std::fmt::Display>(&mut self, key: &str, value: Option<T>) -> &mut Self {
        if let Some(v) = value {
            self.params.push(format!("{}={}", key, v));
        }
        self
    }

    /// Add a string parameter with URL encoding if the value is Some
    pub fn add_opt_encoded(&mut self, key: &str, value: Option<&str>) -> &mut Self {
        if let Some(v) = value {
            self.params
                .push(format!("{}={}", key, urlencoding::encode(v)));
        }
        self
    }

    /// Add all values from a slice as repeated parameters (e.g., statuses[]=a&statuses[]=b)
    pub fn add_all<T: std::fmt::Display>(&mut self, key: &str, values: &[T]) -> &mut Self {
        for v in values {
            self.params.push(format!("{}[]={}", key, v));
        }
        self
    }

    /// Add all values from a slice of integers as comma-separated parameter (e.g., assignees=123,456)
    ///
    /// This is used for ClickUp API parameters that expect comma-separated values
    /// instead of repeated array parameters.
    pub fn add_comma_separated_ints(&mut self, key: &str, values: &[i64]) -> &mut Self {
        if !values.is_empty() {
            let comma_separated = values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",");
            self.params.push(format!("{}={}", key, comma_separated));
        }
        self
    }

    /// Check if there are any parameters
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Build the query string with leading "?" if not empty
    pub fn to_query_string(&self) -> String {
        if self.params.is_empty() {
            String::new()
        } else {
            format!("?{}", self.params.join("&"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let params = QueryParams::new();
        assert!(params.is_empty());
        assert_eq!(params.to_query_string(), "");
    }

    #[test]
    fn test_add_opt_some() {
        let mut params = QueryParams::new();
        params.add_opt("archived", Some(true));
        params.add_opt("page", Some(2));

        assert!(!params.is_empty());
        assert_eq!(params.to_query_string(), "?archived=true&page=2");
    }

    #[test]
    fn test_add_opt_none() {
        let mut params = QueryParams::new();
        params.add_opt("archived", None::<bool>);

        assert!(params.is_empty());
        assert_eq!(params.to_query_string(), "");
    }

    #[test]
    fn test_add_opt_encoded() {
        let mut params = QueryParams::new();
        params.add_opt_encoded("query", Some("hello world"));

        assert!(params.to_query_string().contains("query="));
        assert!(params.to_query_string().contains("hello%20world"));
    }

    #[test]
    fn test_add_opt_encoded_none() {
        let mut params = QueryParams::new();
        params.add_opt_encoded("query", None);

        assert!(params.is_empty());
    }

    #[test]
    fn test_add_all() {
        let mut params = QueryParams::new();
        params.add_all("statuses", &["todo", "in progress"]);

        assert_eq!(
            params.to_query_string(),
            "?statuses[]=todo&statuses[]=in progress"
        );
    }

    #[test]
    fn test_add_all_empty() {
        let mut params = QueryParams::new();
        params.add_all("statuses", &[] as &[&str]);

        assert!(params.is_empty());
    }

    #[test]
    fn test_add_comma_separated_ints() {
        let mut params = QueryParams::new();
        params.add_comma_separated_ints("assignees", &[123, 456, 789]);

        assert_eq!(params.to_query_string(), "?assignees=123,456,789");
    }

    #[test]
    fn test_add_comma_separated_ints_single() {
        let mut params = QueryParams::new();
        params.add_comma_separated_ints("assignees", &[123]);

        assert_eq!(params.to_query_string(), "?assignees=123");
    }

    #[test]
    fn test_add_comma_separated_ints_empty() {
        let mut params = QueryParams::new();
        params.add_comma_separated_ints("assignees", &[]);

        assert!(params.is_empty());
        assert_eq!(params.to_query_string(), "");
    }

    #[test]
    fn test_mixed_params() {
        let mut params = QueryParams::new();
        params.add_opt("archived", Some(false));
        params.add_opt("page", Some(1));
        params.add_opt_encoded("query", Some("test search"));
        params.add_all("statuses", &["todo"]);
        params.add_comma_separated_ints("assignees", &[789]);

        let query = params.to_query_string();
        assert!(query.contains("archived=false"));
        assert!(query.contains("page=1"));
        assert!(query.contains("query=test%20search"));
        assert!(query.contains("statuses[]=todo"));
        assert!(query.contains("assignees=789"));
    }
}
