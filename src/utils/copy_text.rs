//! Plain-text extraction for copying task fields from the TUI.
//!
//! ClickUp stores comments and descriptions as rich-text blocks/JSON, so a raw
//! copy would put escaped JSON on the clipboard. These helpers flatten the
//! content to the text a reader actually sees, which is what belongs in the
//! clipboard when the user asks to copy a description or a comment.

use serde_json::Value;

/// Convert a ClickUp rich-text comment body to plain text.
///
/// `Comment::text` arrives in several shapes depending on where the comment
/// came from:
/// - plain text (`"hello @Ada"`)
/// - a rich-text JSON document (`{"type":"doc","content":[...]}`)
/// - a bare array of blocks (`[{"text":"..."}]`)
///
/// Anything that does not parse as JSON is returned unchanged, so plain
/// comments keep their text exactly as written.
pub fn comment_text_to_plain(text: &str) -> String {
    let trimmed = text.trim();
    if !(trimmed.starts_with('{') || trimmed.starts_with('[')) {
        return text.to_string();
    }

    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return text.to_string();
    };

    let mut out = String::new();
    collect_rich_text(&value, &mut out);
    let flattened = tidy(&out);

    // A block document that carries no readable text (e.g. an unknown node
    // shape) is better handed back verbatim than silently emptied.
    if flattened.is_empty() {
        text.to_string()
    } else {
        flattened
    }
}

/// Recursively collect text from a rich-text node or block array.
fn collect_rich_text(value: &Value, out: &mut String) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_rich_text(item, out);
            }
        }
        Value::Object(map) => {
            let node_type = map.get("type").and_then(Value::as_str).unwrap_or_default();
            if node_type == "text" {
                if let Some(text) = map.get("text").and_then(Value::as_str) {
                    out.push_str(text);
                }
                return;
            }

            // Blocks carry their content under `content`; mentions keep their
            // label under `text`/`attrs` and are handled by the leaf arms below.
            if let Some(content) = map.get("content") {
                collect_rich_text(content, out);
            } else if let Some(text) = map.get("text").and_then(Value::as_str) {
                out.push_str(text);
            }

            // Block-level nodes end a visual line.
            match node_type {
                "paragraph" | "heading" | "blockquote" | "codeBlock" | "listItem"
                | "bulletList" | "orderedList" | "code" | "divider" => {
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                }
                _ => {}
            }
        }
        Value::String(s) => out.push_str(s),
        _ => {}
    }
}

/// Collapse runs of blank lines and trim trailing whitespace per line.
fn tidy(text: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut blank_run = 0usize;

    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed.trim().is_empty() {
            blank_run += 1;
            continue;
        }
        // Keep at most one blank line between paragraphs.
        if blank_run > 0 && !lines.is_empty() {
            lines.push(String::new());
        }
        blank_run = 0;
        lines.push(trimmed.to_string());
    }

    lines.join("\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_passes_through_unchanged() {
        let text = "Just a normal comment with @mention";
        assert_eq!(comment_text_to_plain(text), text);
    }

    #[test]
    fn doc_json_is_flattened_to_readable_text() {
        let json = r#"{"type":"doc","content":[
            {"type":"paragraph","content":[{"type":"text","text":"First line"}]},
            {"type":"paragraph","content":[{"type":"text","text":"Second line"}]}
        ]}"#;
        assert_eq!(comment_text_to_plain(json), "First line\nSecond line");
    }

    #[test]
    fn bare_block_array_is_flattened() {
        let json = r#"[{"type":"paragraph","content":[{"type":"text","text":"Only block"}]}]"#;
        assert_eq!(comment_text_to_plain(json), "Only block");
    }

    #[test]
    fn json_without_readable_text_falls_back_to_raw() {
        let json = r#"{"type":"doc","content":[]}"#;
        assert_eq!(comment_text_to_plain(json), json);
    }

    #[test]
    fn plain_non_json_braces_are_not_mangled() {
        let text = "{not json at all";
        assert_eq!(comment_text_to_plain(text), text);
    }
}
