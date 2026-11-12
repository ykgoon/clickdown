//! @mention support for comment and description text.
//!
//! ClickUp does not tag anyone from plain `@Name` text. A real mention is a
//! structured block in the comment payload, or a `#user_mention#` markdown
//! link in a task description. Both require the user's numeric id, which plain
//! text does not carry — so the editor keeps a small registry of the people
//! picked from the `@` picker (plus whoever the list-members fetch returned)
//! and resolves `@Name` back to them at save time.
//!
//! Offsets into the editing buffer shift constantly while typing, so nothing
//! here stores positions. Resolution scans the finished text instead.

use crate::models::comment::{BlockAttributes, CommentBlock, MentionUser};

/// One person that `@Name` in a buffer may refer to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MentionEntry {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub initials: String,
}

impl MentionEntry {
    pub fn from_user(user: &crate::models::User) -> Option<Self> {
        // A mention without an id could never be tagged, so skip those.
        if user.id <= 0 || user.username.is_empty() {
            return None;
        }
        Some(Self {
            id: user.id,
            name: user.username.clone(),
            email: user.email.clone().unwrap_or_default(),
            initials: user.initials.clone().unwrap_or_else(|| {
                user
                    .username
                    .split_whitespace()
                    .filter_map(|word| word.chars().next())
                    .take(2)
                    .collect::<String>()
                    .to_uppercase()
            }),
        })
    }

    /// The ClickUp user shape embedded in a `tag` block.
    pub fn user_json(&self) -> MentionUser {
        MentionUser {
            id: self.id,
            username: self.name.clone(),
            email: self.email.clone(),
            initials: self.initials.clone(),
        }
    }
}

fn text_block(text: String) -> CommentBlock {
    CommentBlock::Text {
        text,
        attributes: BlockAttributes {},
    }
}

/// Split `text` into ClickUp comment blocks, tagging every `@Name` that
/// resolves in `registry`.
///
/// Returns `None` when nothing resolves, so callers can keep sending plain
/// `comment_text` for ordinary comments.
pub fn build_comment_blocks(
    text: &str,
    registry: &[MentionEntry],
) -> Option<Vec<CommentBlock>> {
    let matches = find_mentions(text, registry);
    if matches.is_empty() {
        return None;
    }

    let mut blocks = Vec::new();
    let mut cursor = 0usize;
    for (start, end, entry) in matches {
        if start > cursor {
            blocks.push(text_block(text[cursor..start].to_string()));
        }
        blocks.push(CommentBlock::Tag {
            kind: "mention",
            user: entry.user_json(),
            text: format!("@{}", entry.name),
        });
        cursor = end;
    }
    if cursor < text.len() {
        blocks.push(text_block(text[cursor..].to_string()));
    }
    Some(blocks)
}

/// Render `text` as ClickUp markdown, turning resolvable `@Name` into
/// `[@Name](#user_mention#<id>)` — the form the web app shows as a real tag in
/// task descriptions. Returns the input unchanged when nothing resolves.
pub fn to_markdown(text: &str, registry: &[MentionEntry]) -> String {
    let matches = find_mentions(text, registry);
    if matches.is_empty() {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len() + matches.len() * 32);
    let mut cursor = 0usize;
    for (start, end, entry) in matches {
        out.push_str(&text[cursor..start]);
        out.push_str(&format!("[@{}](#user_mention#{})", entry.name, entry.id));
        cursor = end;
    }
    out.push_str(&text[cursor..]);
    out
}

/// True when `text` resolves at least one mention in `registry`.
pub fn contains_mention(text: &str, registry: &[MentionEntry]) -> bool {
    !find_mentions(text, registry).is_empty()
}

/// Byte ranges of `@Name` occurrences, longest name first so that
/// `@Ann Lee` wins over `@Ann`.
fn find_mentions(text: &str, registry: &[MentionEntry]) -> Vec<(usize, usize, MentionEntry)> {
    if registry.is_empty() || !text.contains('@') {
        return Vec::new();
    }

    let mut ordered: Vec<&MentionEntry> = registry.iter().collect();
    ordered.sort_by_key(|entry| std::cmp::Reverse(entry.name.len()));

    let mut found: Vec<(usize, usize, MentionEntry)> = Vec::new();
    let mut search_from = 0usize;

    while let Some(rel) = text[search_from..].find('@') {
        let start = search_from + rel;
        // Skip escaped markers such as `\@` and doubled `@@`.
        if start > 0 && text[..start].ends_with('\\') {
            search_from = start + 1;
            continue;
        }

        let rest = &text[start + 1..];
        let hit = ordered.iter().find(|entry| {
            rest.starts_with(&entry.name)
                && rest[entry.name.len()..]
                    .chars()
                    .next()
                    .map(|c| !is_name_char(c))
                    .unwrap_or(true)
        });

        match hit {
            Some(entry) => {
                let end = start + 1 + entry.name.len();
                found.push((start, end, (*entry).clone()));
                // A name may itself contain '@'-free text; continue after it.
                search_from = end;
            }
            None => {
                search_from = start + 1;
            }
        }
    }

    found
}

/// Characters that may not immediately follow a mention's name, otherwise
/// `@Ann` would match inside `@Anne`.
fn is_name_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: i64, name: &str) -> MentionEntry {
        MentionEntry {
            id,
            name: name.to_string(),
            email: format!("{}@example.com", name.to_lowercase().replace(' ', ".")),
            initials: "XX".to_string(),
        }
    }

    fn registry() -> Vec<MentionEntry> {
        vec![entry(38650140, "Y.K. Goon"), entry(7, "Ann"), entry(8, "Ann Lee")]
    }

    #[test]
    fn no_registry_means_no_blocks() {
        assert!(build_comment_blocks("hello @Ann", &[]).is_none());
    }

    #[test]
    fn text_without_mention_means_no_blocks() {
        assert!(build_comment_blocks("hello there", &registry()).is_none());
    }

    #[test]
    fn unknown_name_means_no_blocks() {
        assert!(build_comment_blocks("hello @Nobody", &registry()).is_none());
    }

    #[test]
    fn single_mention_mid_text_splits_into_three_blocks() {
        let blocks = build_comment_blocks("hi @Ann please look", &registry()).unwrap();
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0], text_block("hi ".to_string()));
        match &blocks[1] {
            CommentBlock::Tag { kind, user, text } => {
                assert_eq!(*kind, "mention");
                assert_eq!(user.id, 7);
                assert_eq!(text, "@Ann");
            }
            other => panic!("expected tag, got {other:?}"),
        }
        assert_eq!(blocks[2], text_block(" please look".to_string()));
    }

    #[test]
    fn mention_at_start_and_end_are_both_tagged() {
        let blocks = build_comment_blocks("@Ann and @Y.K. Goon", &registry()).unwrap();
        let tags = blocks
            .iter()
            .filter(|b| matches!(b, CommentBlock::Tag { .. }))
            .count();
        assert_eq!(tags, 2);
    }

    #[test]
    fn longest_name_wins() {
        let blocks = build_comment_blocks("@Ann Lee is here", &registry()).unwrap();
        match &blocks[0] {
            CommentBlock::Tag { user, text, .. } => {
                assert_eq!(user.id, 8, "should match 'Ann Lee', not 'Ann'");
                assert_eq!(text, "@Ann Lee");
            }
            other => panic!("expected tag, got {other:?}"),
        }
    }

    #[test]
    fn name_prefix_does_not_match_inside_longer_word() {
        // "@Anne" is a different person; "@Ann" must not match it.
        assert!(build_comment_blocks("@Anne hi", &registry()).is_none());
    }

    #[test]
    fn blocks_reconstruct_the_original_text() {
        let text = "a @Ann b @Y.K. Goon c";
        let blocks = build_comment_blocks(text, &registry()).unwrap();
        let rebuilt: String = blocks
            .iter()
            .map(|b| match b {
                CommentBlock::Text { text, .. } => text.clone(),
                CommentBlock::Tag { text, .. } => text.clone(),
            })
            .collect();
        assert_eq!(rebuilt, text);
    }

    #[test]
    fn tag_block_serializes_with_type_and_user() {
        let blocks = build_comment_blocks("@Ann hi", &registry()).unwrap();
        let json = serde_json::to_string(&blocks).unwrap();
        assert!(json.contains("\"type\":\"mention\""), "{json}");
        assert!(json.contains("\"id\":7"), "{json}");
        assert!(json.contains("\"text\":\"@Ann\""), "{json}");
    }

    #[test]
    fn markdown_uses_user_mention_scheme() {
        let md = to_markdown("hey @Ann, ping @Y.K. Goon", &registry());
        assert_eq!(
            md,
            "hey [@Ann](#user_mention#7), ping [@Y.K. Goon](#user_mention#38650140)"
        );
    }

    #[test]
    fn markdown_leaves_plain_text_alone() {
        assert_eq!(to_markdown("no mentions here", &registry()), "no mentions here");
    }

    #[test]
    fn contains_mention_reports_resolution() {
        assert!(contains_mention("hi @Ann", &registry()));
        assert!(!contains_mention("hi @Nobody", &registry()));
    }
}
