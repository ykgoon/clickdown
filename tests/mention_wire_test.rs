use clickdown::models::comment::{BlockAttributes, CommentBlock, CreateCommentRequest, MentionUser};
use clickdown::tui::mention::{build_comment_blocks, to_markdown, MentionEntry};

fn registry() -> Vec<MentionEntry> {
    vec![MentionEntry {
        id: 38650140,
        name: "Y.K. Goon".to_string(),
        email: "yk@kakitangan.com".to_string(),
        initials: "YG".to_string(),
    }]
}

#[test]
fn serialized_request_matches_the_live_api_shape() {
    let text = "@Y.K. Goon lorem ipsum please review";
    let blocks = build_comment_blocks(text, &registry()).expect("mention resolves");

    let request = CreateCommentRequest {
        comment_text: String::new(),
        comment: Some(blocks),
        assignee: None,
        assigned_commenter: None,
        parent_id: None,
    };
    let json = serde_json::to_string(&request).unwrap();
    println!("REQUEST: {json}");

    assert!(json.contains("\"comment_text\":\"\""));
    assert!(json.contains("\"type\":\"mention\""));
    assert!(json.contains("\"id\":38650140"));
    assert!(json.contains("lorem ipsum please review"));
}

#[test]
fn description_markdown_matches_the_live_api_shape() {
    let md = to_markdown(
        "Hey @Y.K. Goon lorem ipsum check this",
        &registry(),
    );
    println!("MARKDOWN: {md}");
    assert_eq!(
        md,
        "Hey [@Y.K. Goon](#user_mention#38650140) lorem ipsum check this"
    );
}

#[test]
fn plain_comment_still_sends_plain_text() {
    let blocks = build_comment_blocks("no mentions at all", &registry());
    assert!(blocks.is_none());
    let request = CreateCommentRequest {
        comment_text: "no mentions at all".to_string(),
        comment: None,
        assignee: None,
        assigned_commenter: None,
        parent_id: None,
    };
    let json = serde_json::to_string(&request).unwrap();
    assert!(!json.contains("\"comment\""));
}

#[test]
fn blocks_type_is_reusable_for_text_runs() {
    let b = CommentBlock::Text {
        text: "hi".to_string(),
        attributes: BlockAttributes {},
    };
    let json = serde_json::to_string(&b).unwrap();
    assert_eq!(json, "{\"text\":\"hi\",\"attributes\":{}}");
}

#[test]
fn mention_user_serializes_all_fields() {
    let u = MentionUser {
        id: 1,
        username: "A".to_string(),
        email: "a@b.c".to_string(),
        initials: "A".to_string(),
    };
    let json = serde_json::to_string(&u).unwrap();
    assert!(json.contains("\"username\":\"A\""));
    assert!(json.contains("\"email\":\"a@b.c\""));
}
