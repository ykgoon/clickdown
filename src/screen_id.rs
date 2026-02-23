//! Screen ID generation for bug reporting reference
//!
//! Generates deterministic 4-character alphanumeric IDs for each screen
//! based on the screen name/type.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Generate a unique 4-character alphanumeric screen ID from a screen name.
///
/// The ID is deterministic - the same screen name always produces the same ID.
/// Uses base36 encoding (0-9, a-z) for readability.
///
/// # Arguments
///
/// * `screen_name` - A unique identifier for the screen (e.g., "auth", "task-list")
///
/// # Returns
///
/// A 4-character lowercase alphanumeric string (e.g., "a3f9")
///
/// # Example
///
/// ```
/// let id = generate_screen_id("auth");
/// assert_eq!(id.len(), 4);
/// ```
pub fn generate_screen_id(screen_name: &str) -> String {
    let hash = hash_screen_name(screen_name);
    hash_to_base36(hash)
}

/// Hash a screen name using the default hasher.
fn hash_screen_name(screen_name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    screen_name.hash(&mut hasher);
    hasher.finish()
}

/// Convert a hash to a 4-character base36 string.
///
/// Base36 uses digits 0-9 and lowercase letters a-z.
fn hash_to_base36(hash: u64) -> String {
    const BASE36_CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    
    // Use different parts of the hash to generate 4 characters
    let c1 = BASE36_CHARS[((hash >> 0) % 36) as usize] as char;
    let c2 = BASE36_CHARS[((hash >> 8) % 36) as usize] as char;
    let c3 = BASE36_CHARS[((hash >> 16) % 36) as usize] as char;
    let c4 = BASE36_CHARS[((hash >> 24) % 36) as usize] as char;
    
    format!("{}{}{}{}", c1, c2, c3, c4)
}

/// Validate that a screen ID has the correct format.
///
/// # Arguments
///
/// * `id` - The ID to validate
///
/// # Returns
///
/// `true` if the ID is exactly 4 lowercase alphanumeric characters
pub fn validate_screen_id(id: &str) -> bool {
    if id.len() != 4 {
        return false;
    }
    
    id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_has_correct_length() {
        let id = generate_screen_id("test-screen");
        assert_eq!(id.len(), 4);
    }

    #[test]
    fn test_id_contains_valid_characters() {
        let id = generate_screen_id("test-screen");
        assert!(validate_screen_id(&id));
    }

    #[test]
    fn test_id_is_deterministic() {
        let id1 = generate_screen_id("auth");
        let id2 = generate_screen_id("auth");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_different_screens_have_different_ids() {
        let id1 = generate_screen_id("auth");
        let id2 = generate_screen_id("task-list");
        let id3 = generate_screen_id("document-view");
        
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_id_uses_lowercase() {
        let id = generate_screen_id("test-screen");
        assert!(id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_validate_screen_id() {
        assert!(validate_screen_id("a3f9"));
        assert!(validate_screen_id("0000"));
        assert!(validate_screen_id("zzzz"));
        assert!(validate_screen_id("12ab"));
        
        assert!(!validate_screen_id("ABC1")); // uppercase
        assert!(!validate_screen_id("a3f"));  // too short
        assert!(!validate_screen_id("a3f99")); // too long
        assert!(!validate_screen_id("a3-g")); // invalid char
    }
}
