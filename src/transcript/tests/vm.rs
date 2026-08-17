//! Origin-classification and forgiving-parse coverage, through [`build`].

use super::{AGENT, write_msg};
use crate::transcript::{Block, EntryKind, build};
use tempfile::tempdir;

/// Build a one-file transcript and return that entry's classified kind. Every
/// name here opens the counter at `001`, so the listing holds nothing but the
/// file — a higher one also reveals a compaction mark ([`super::compaction`]).
fn kind_of(name: &str, bytes: &[u8]) -> EntryKind {
    let dir = tempdir().unwrap();
    write_msg(dir.path(), name, bytes);
    let mut t = build(dir.path(), AGENT);
    assert_eq!(t.entries.len(), 1, "one entry expected for {name}");
    t.entries.remove(0).kind
}

#[test]
fn delivered_md_strips_the_deposit_envelope_off_the_body() {
    // Delivery renames the deposit file into `messages/` with its
    // frontmatter untouched (ARCH §2.11), so the bytes open with the
    // envelope. The parsed body is the message — never the `---` fence.
    assert_eq!(
        kind_of(
            "001-user.md",
            b"---\nfrom: user\ndeposited_at: 2026-08-02T04:00:00Z\n---\nis this thing on?\n"
        ),
        EntryKind::Delivered {
            sender: "user".into(),
            epitaph: None,
            body: "is this thing on?\n".into()
        }
    );
}

#[test]
fn delivered_md_without_an_envelope_is_the_whole_file() {
    // The forgiving read: no frontmatter means no envelope to strip.
    assert_eq!(
        kind_of("001-alice.md", b"hi there\n"),
        EntryKind::Delivered {
            sender: "alice".into(),
            epitaph: None,
            body: "hi there\n".into()
        }
    );
}

#[test]
fn delivered_result_message_with_no_content_has_an_empty_body() {
    // A result message can be envelope-only (ARCH §2.6) — the epitaph is
    // asserted, the child never spoke.
    assert_eq!(
        kind_of(
            "001-kid.md",
            b"---\nfrom: kid\ndeposited_at: t\nepitaph: died\nterminal_ref: sha\n---\n"
        ),
        EntryKind::Delivered {
            sender: "kid".into(),
            epitaph: Some(crate::inboxview::Epitaph::Died),
            body: String::new()
        }
    );
}

#[test]
fn delivered_sender_keeps_internal_hyphens() {
    let EntryKind::Delivered { sender, .. } = kind_of("001-claude-fable.md", b"x") else {
        panic!("expected delivered");
    };
    assert_eq!(sender, "claude-fable");
}

#[test]
fn model_bare_block_array_parses_text_and_thinking() {
    let k = kind_of(
        "001-opus.json",
        br#"[{"type":"text","text":"hello"},{"type":"thinking","thinking":"hmm"}]"#,
    );
    let EntryKind::Model {
        model_id,
        blocks,
        usage,
    } = k
    else {
        panic!("expected model");
    };
    assert_eq!(model_id, "opus");
    assert_eq!(
        blocks,
        vec![Block::Text("hello".into()), Block::Thinking("hmm".into())]
    );
    assert!(usage.is_empty(), "the legacy bare array carries no usage");
}

#[test]
fn model_usage_sibling_is_read_verbatim_and_non_counters_are_skipped() {
    // The lernie ≥0.0.4 entry shape: the provider's report beside `content`.
    // Integer counters ride through under their committed names; a non-integer
    // sibling (not a token count) is skipped, never invented into one.
    let k = kind_of(
        "001-m.json",
        br#"{"content":[{"type":"text","text":"hi"}],"usage":{"input_tokens":5,"output_tokens":3,"service_tier":"standard"}}"#,
    );
    let EntryKind::Model { usage, .. } = k else {
        panic!("expected model");
    };
    assert_eq!(
        usage.into_iter().collect::<Vec<_>>(),
        vec![
            ("input_tokens".to_string(), 5),
            ("output_tokens".to_string(), 3)
        ]
    );
}

#[test]
fn model_object_content_array_or_string() {
    let arr = kind_of(
        "001-m.json",
        br#"{"role":"assistant","content":[{"type":"text","text":"x"}]}"#,
    );
    assert!(
        matches!(arr, EntryKind::Model { blocks, .. } if blocks == vec![Block::Text("x".into())])
    );
    let s = kind_of("001-m.json", br#"{"content":"just text"}"#);
    assert!(
        matches!(s, EntryKind::Model { blocks, .. } if blocks == vec![Block::Text("just text".into())])
    );
}

#[test]
fn model_object_without_content_has_no_blocks() {
    let k = kind_of("001-m.json", br#"{"role":"assistant"}"#);
    assert!(matches!(k, EntryKind::Model { blocks, .. } if blocks.is_empty()));
}

#[test]
fn model_tool_use_chip_summarizes_input() {
    let k = kind_of(
        "001-m.json",
        br#"[{"type":"tool_use","id":"toolu_1","name":"Read","input":{"path":"/x"}}]"#,
    );
    let EntryKind::Model { blocks, .. } = k else {
        panic!("expected model");
    };
    assert_eq!(
        blocks,
        vec![Block::ToolUse {
            id: "toolu_1".into(),
            name: "Read".into(),
            input_summary: r#"{"path":"/x"}"#.into(),
        }]
    );
}

#[test]
fn model_tool_use_without_input_is_empty_summary() {
    let k = kind_of(
        "001-m.json",
        br#"[{"type":"tool_use","id":"t","name":"Now"}]"#,
    );
    let EntryKind::Model { blocks, .. } = k else {
        panic!("expected model");
    };
    assert!(matches!(&blocks[0], Block::ToolUse { input_summary, .. } if input_summary.is_empty()));
}

#[test]
fn model_tool_use_long_input_is_truncated() {
    let big = "a".repeat(500);
    let json = format!(r#"[{{"type":"tool_use","id":"t","name":"N","input":{{"k":"{big}"}}}}]"#);
    let EntryKind::Model { blocks, .. } = kind_of("001-m.json", json.as_bytes()) else {
        panic!("expected model");
    };
    let Block::ToolUse { input_summary, .. } = &blocks[0] else {
        panic!("expected tool_use");
    };
    assert!(
        input_summary.ends_with('…'),
        "not truncated: {input_summary}"
    );
}

#[test]
fn model_untyped_and_unknown_type_blocks_are_skipped() {
    // A block with no `type` and a block with an unrecognized `type` are
    // both dropped from the parsed view (still inspectable via Raw).
    let k = kind_of(
        "001-m.json",
        br#"[{"foo":1},{"type":"image","x":1},{"type":"text","text":"y"}]"#,
    );
    assert!(
        matches!(k, EntryKind::Model { blocks, .. } if blocks == vec![Block::Text("y".into())])
    );
}

#[test]
fn model_invalid_json_falls_to_raw() {
    assert_eq!(kind_of("001-m.json", b"not json"), EntryKind::Raw);
}

#[test]
fn tool_result_direct_fields() {
    assert_eq!(
        kind_of(
            "001-tool.json",
            br#"{"tool_use_id":"t","content":"done","is_error":false}"#
        ),
        EntryKind::ToolResult {
            tool_use_id: "t".into(),
            content: "done".into(),
            is_error: false,
        }
    );
}

#[test]
fn tool_result_error_flag_and_array_and_scalar_content() {
    assert!(matches!(
        kind_of(
            "001-tool.json",
            br#"{"tool_use_id":"t","content":"boom","is_error":true}"#
        ),
        EntryKind::ToolResult { is_error: true, .. }
    ));
    assert!(matches!(
        kind_of("001-tool.json", br#"{"tool_use_id":"t","content":[{"type":"text","text":"a"},{"type":"text","text":"b"}]}"#),
        EntryKind::ToolResult { content, .. } if content == "ab"
    ));
    assert!(matches!(
        kind_of("001-tool.json", br#"{"tool_use_id":"t","content":[{"type":"image","x":1}]}"#),
        EntryKind::ToolResult { content, .. } if content == r#"{"type":"image","x":1}"#
    ));
    assert!(matches!(
        kind_of("001-tool.json", br#"{"tool_use_id":"t","content":42}"#),
        EntryKind::ToolResult { content, .. } if content == "42"
    ));
}

#[test]
fn tool_result_absent_and_null_content_are_empty() {
    assert!(matches!(
        kind_of("001-tool.json", br#"{"tool_use_id":"t"}"#),
        EntryKind::ToolResult { content, .. } if content.is_empty()
    ));
    assert!(matches!(
        kind_of("001-tool.json", br#"{"tool_use_id":"t","content":null}"#),
        EntryKind::ToolResult { content, .. } if content.is_empty()
    ));
}

#[test]
fn tool_result_wrapped_in_content_array_is_found() {
    assert!(matches!(
        kind_of("001-tool.json", br#"{"role":"user","content":[{"tool_use_id":"t","content":"z"}]}"#),
        EntryKind::ToolResult { tool_use_id, content, .. } if tool_use_id == "t" && content == "z"
    ));
}

#[test]
fn tool_json_without_result_block_is_raw() {
    // no content key; content not an array; unparseable bytes — each falls
    // through find_tool_result / the JSON parse to the Raw bucket.
    assert_eq!(kind_of("001-tool.json", br#"{"foo":1}"#), EntryKind::Raw);
    assert_eq!(
        kind_of("001-tool.json", br#"{"content":"x"}"#),
        EntryKind::Raw
    );
    assert_eq!(kind_of("001-tool.json", b"nope"), EntryKind::Raw);
}

#[test]
fn unparseable_names_go_to_raw_bucket_not_dropped() {
    let dir = tempdir().unwrap();
    // no dot / no hyphen / empty counter / non-digit counter / empty origin
    // / unknown extension — all preserved as Raw, never dropped.
    for n in [
        "README",
        "readme.md",
        "-x.md",
        "abc-x.md",
        "003-.md",
        "001-note.txt",
    ] {
        write_msg(dir.path(), n, b"body");
    }
    let t = build(dir.path(), AGENT);
    assert_eq!(t.entries.len(), 6, "none dropped");
    assert!(t.entries.iter().all(|e| e.kind == EntryKind::Raw));
    assert!(t.entries.iter().any(|e| e.name == "001-note.txt"));
}

#[test]
fn raw_entry_keeps_verbatim_bytes() {
    let dir = tempdir().unwrap();
    write_msg(dir.path(), "README", b"\x00\x01raw");
    let t = build(dir.path(), AGENT);
    assert_eq!(t.entries[0].raw, b"\x00\x01raw");
}
