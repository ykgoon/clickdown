//! User model for ClickUp API

use serde::{Deserialize, Serialize};

use crate::utils::deserializers::{null_to_default_id, null_to_empty_string, null_to_empty_vec};

/// User reference (assignee, creator, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    #[serde(default, deserialize_with = "null_to_default_id")]
    pub id: i64,
    #[serde(default, deserialize_with = "null_to_empty_string")]
    pub username: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default, rename = "profilePicture")]
    pub profile_picture: Option<String>,
    #[serde(default)]
    pub initials: Option<String>,
}

/// API response for getting list/task members
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MembersResponse {
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub members: Vec<User>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_deserialize_full() {
        let json = "{
            \"id\": 12345,
            \"username\": \"testuser\",
            \"color\": \"#7b68ee\",
            \"email\": \"test@example.com\",
            \"profilePicture\": \"https://example.com/pic.jpg\",
            \"initials\": \"TU\"
        }";

        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, 12345);
        assert_eq!(user.username, "testuser");
        assert_eq!(user.color, Some("#7b68ee".to_string()));
        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert_eq!(
            user.profile_picture,
            Some("https://example.com/pic.jpg".to_string())
        );
        assert_eq!(user.initials, Some("TU".to_string()));
    }

    #[test]
    fn test_user_deserialize_minimal() {
        let json = "{
            \"id\": 12345,
            \"username\": \"testuser\"
        }";

        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, 12345);
        assert_eq!(user.username, "testuser");
        assert_eq!(user.color, None);
        assert_eq!(user.email, None);
        assert_eq!(user.profile_picture, None);
        assert_eq!(user.initials, None);
    }

    #[test]
    fn test_user_deserialize_null_fields() {
        let json = "{
            \"id\": null,
            \"username\": null,
            \"color\": null,
            \"email\": null,
            \"profilePicture\": null,
            \"initials\": null
        }";

        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, 0); // null_to_default_id returns 0
        assert_eq!(user.username, "");
        assert_eq!(user.color, None);
        assert_eq!(user.email, None);
        assert_eq!(user.profile_picture, None);
        assert_eq!(user.initials, None);
    }

    #[test]
    fn test_members_response_deserialize() {
        let json = "{
            \"members\": [
                {
                    \"id\": 123,
                    \"username\": \"Alice\",
                    \"email\": \"alice@example.com\",
                    \"color\": \"#FF0000\",
                    \"initials\": \"A\",
                    \"profilePicture\": null
                },
                {
                    \"id\": 456,
                    \"username\": \"Bob\",
                    \"email\": null,
                    \"color\": null,
                    \"initials\": \"B\",
                    \"profilePicture\": null
                }
            ]
        }";

        let response: MembersResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.members.len(), 2);
        assert_eq!(response.members[0].id, 123);
        assert_eq!(response.members[0].username, "Alice");
        assert_eq!(
            response.members[0].email,
            Some("alice@example.com".to_string())
        );
        assert_eq!(response.members[1].id, 456);
        assert_eq!(response.members[1].username, "Bob");
        assert_eq!(response.members[1].email, None);
    }

    #[test]
    fn test_members_response_empty() {
        let json = r#"{"members": []}"#;
        let response: MembersResponse = serde_json::from_str(json).unwrap();
        assert!(response.members.is_empty());
    }

    #[test]
    fn test_members_response_null_members() {
        let json = r#"{"members": null}"#;
        let response: MembersResponse = serde_json::from_str(json).unwrap();
        // null_to_empty_vec behavior would depend on serde config; with #[serde(default)]
        // on Vec field, null becomes empty vec
        assert!(response.members.is_empty());
    }
}
