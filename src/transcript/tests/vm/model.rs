//! The **model** side of the forgiving parse: an assistant file's content,
//! however it is spelled. A bare block array, an object wrapping one, a plain
//! string, a `usage` sibling read verbatim, a `tool_use` chip summarized from
//! its input, and every block shape the reader is meant to skip rather than
//! choke on — down to invalid JSON, which falls to `Raw` instead of vanishing.

use super::kind_of;
use crate::transcript::{Block, EntryKind};

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
