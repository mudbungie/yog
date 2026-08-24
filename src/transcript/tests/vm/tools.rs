//! The **tool-result** side of the forgiving parse: the direct fields, the
//! error flag, content spelled as an array or a scalar, content that is absent
//! or null, a result buried inside a `content` array — and a tool file with no
//! result block at all, which is `Raw` rather than an empty success.

use super::kind_of;
use crate::transcript::EntryKind;

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
