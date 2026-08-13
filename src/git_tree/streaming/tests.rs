//! The stream fold's tables: the text half every seat already read, the delta
//! half the §11 live mark added, and the disk reads behind both.

use super::*;

use tempfile::tempdir;

/// The text half of the fold — what every pre-existing case here asserts,
/// spelled once so the fold's other half does not have to be threaded
/// through them.
fn text(bytes: &[u8]) -> Option<String> {
    fold_stream(bytes).text
}

fn write(path: &Path, contents: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, contents).unwrap();
}

#[test]
fn accumulates_text_in_order_across_indices() {
    let jsonl = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"content_delta","index":0,"delta":{"text_delta":"hel"}}
{"type":"content_delta","index":0,"delta":{"text_delta":"lo"}}
{"type":"content_delta","index":0,"delta":{"text_delta":" world"}}
"#;
    assert_eq!(text(jsonl).as_deref(), Some("hello world"));
}

#[test]
fn accumulates_brazen_content_delta_text() {
    // brazen v=1: text rides `content_delta.delta.text_delta`
    // (bl-507a dual vocabulary).
    let jsonl = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"content_start","index":0,"kind":{"text":{}}}
{"type":"content_delta","index":0,"delta":{"text_delta":"Hel"}}
{"type":"content_delta","index":0,"delta":{"text_delta":"lo"}}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;
    assert_eq!(text(jsonl).as_deref(), Some("Hello"));
}

#[test]
fn ignores_brazen_non_text_deltas() {
    // Tool-argument (`json_delta`) and `thinking_delta` fragments
    // carry no display text.
    let jsonl = br#"{"type":"content_delta","index":1,"delta":{"json_delta":"{\"a\":"}}
{"type":"content_delta","index":0,"delta":{"thinking_delta":"hmm"}}
"#;
    assert!(text(jsonl).is_none());
}

#[test]
fn ignores_non_text_events() {
    let jsonl = br#"{"type":"message_start"}
{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}
{"type":"tool_use_delta","index":1,"partial_json":"{}"}
{"type":"content_block_stop","index":0}
"#;
    assert!(text(jsonl).is_none());
}

#[test]
fn tolerates_malformed_lines_without_aborting() {
    let jsonl =
        b"not json\n{\"type\":\"content_delta\",\"index\":0,\"delta\":{\"text_delta\":\"hi\"}}\n{partial";
    assert_eq!(text(jsonl).as_deref(), Some("hi"));
}

#[test]
fn empty_payload_returns_none() {
    assert!(text(b"").is_none());
    assert!(text(b"\n\n").is_none());
}

#[test]
fn content_delta_without_text_delta_is_skipped() {
    let jsonl = br#"{"type":"content_delta","index":0,"delta":{"json_delta":"{}"}}
{"type":"content_delta","index":0,"delta":{"text_delta":"x"}}
"#;
    assert_eq!(text(jsonl).as_deref(), Some("x"));
}

#[test]
fn the_fold_reads_the_latest_steps_response() {
    let dir = tempdir().unwrap();
    let conv = "20260427T120000Z-aaaa";
    let steps = dir.path().join(STEPS_DIR).join(conv);
    write(
        &steps.join("001").join(RESPONSE_FILE),
        b"{\"type\":\"content_delta\",\"index\":0,\"delta\":{\"text_delta\":\"first\"}}\n",
    );
    write(
        &steps.join("002").join(RESPONSE_FILE),
        b"{\"type\":\"content_delta\",\"index\":0,\"delta\":{\"text_delta\":\"second\"}}\n",
    );
    assert_eq!(
        stream_from_disk(dir.path(), conv).text.as_deref(),
        Some("second")
    );
}

#[test]
fn the_fold_says_nothing_when_the_steps_dir_is_absent() {
    let dir = tempdir().unwrap();
    assert!(stream_from_disk(dir.path(), "no-such-conv").text.is_none());
}

#[test]
fn the_fold_says_nothing_when_the_response_is_absent() {
    let dir = tempdir().unwrap();
    let conv = "20260427T120000Z-bbbb";
    std::fs::create_dir_all(dir.path().join(STEPS_DIR).join(conv).join("001")).unwrap();
    assert!(stream_from_disk(dir.path(), conv).text.is_none());
}

#[test]
fn latest_step_dir_skips_non_step_entries() {
    // Stray file at the conv-id level (e.g. an editor backup) and a
    // dir that doesn't match `<NNN>` shape must both be ignored.
    let dir = tempdir().unwrap();
    let conv = "20260427T120000Z-cccc";
    let conv_steps = dir.path().join(STEPS_DIR).join(conv);
    std::fs::create_dir_all(conv_steps.join("001")).unwrap();
    std::fs::create_dir_all(conv_steps.join("notes")).unwrap();
    std::fs::write(conv_steps.join(".keep"), b"").unwrap();
    std::fs::write(conv_steps.join("01a"), b"").unwrap();
    let latest = latest_step_dir(&conv_steps).unwrap();
    assert!(latest.ends_with("001"));
}

#[test]
fn last_delta_is_none_before_anything_comes_back() {
    // The request went out and the stream opened; nothing has landed.
    // That `None` is what the mark paints as "waiting for the API".
    let jsonl = br#"{"type":"message_start","v":1,"role":"assistant"}
"#;
    assert_eq!(fold_stream(jsonl).last_delta, None);
    assert_eq!(fold_stream(b"").last_delta, None);
}

#[test]
fn last_delta_ends_on_the_newest_delta_either_way() {
    let thinking_last = br#"{"type":"content_delta","index":0,"delta":{"text_delta":"hi"}}
{"type":"content_delta","index":1,"delta":{"thinking_delta":"hmm"}}
"#;
    let folded = fold_stream(thinking_last);
    assert_eq!(folded.last_delta, Some(Delta::Thinking));
    // Thinking displays nothing, so the text half is unmoved by it.
    assert_eq!(folded.text.as_deref(), Some("hi"));

    let text_last = br#"{"type":"content_delta","index":0,"delta":{"thinking_delta":"hmm"}}
{"type":"content_delta","index":1,"delta":{"text_delta":"so"}}
"#;
    assert_eq!(fold_stream(text_last).last_delta, Some(Delta::Text));
}

#[test]
fn json_delta_moves_neither_half() {
    // Tool arguments are the model composing a call; the §5.1 #10 tool
    // records say that better, so this seam ignores them outright.
    let jsonl = br#"{"type":"content_delta","index":1,"delta":{"json_delta":"{\"a\":"}}
"#;
    assert_eq!(fold_stream(jsonl), Stream::default());
}

#[test]
fn stream_from_disk_carries_both_halves_of_the_latest_step() {
    let dir = tempdir().unwrap();
    let conv = "20260427T120000Z-dddd";
    let steps = dir.path().join(STEPS_DIR).join(conv);
    write(
        &steps.join("001").join(RESPONSE_FILE),
        b"{\"type\":\"content_delta\",\"index\":0,\"delta\":{\"text_delta\":\"old\"}}\n",
    );
    write(
        &steps.join("002").join(RESPONSE_FILE),
        b"{\"type\":\"content_delta\",\"index\":0,\"delta\":{\"thinking_delta\":\"now\"}}\n",
    );
    assert_eq!(
        stream_from_disk(dir.path(), conv),
        Stream {
            text: None,
            thinking: Some("now".to_owned()),
            last_delta: Some(Delta::Thinking),
        }
    );
}

#[test]
fn stream_from_disk_defaults_when_there_is_nothing_to_read() {
    let dir = tempdir().unwrap();
    assert_eq!(
        stream_from_disk(dir.path(), "no-such-conv"),
        Stream::default()
    );
    let conv = "20260427T120000Z-eeee";
    std::fs::create_dir_all(dir.path().join(STEPS_DIR).join(conv).join("001")).unwrap();
    assert_eq!(stream_from_disk(dir.path(), conv), Stream::default());
}
