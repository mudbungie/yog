//! STORIES **S7-T2** step-drilldown: a step carrying request / response /
//! staging and a tool call drills into a jsonview row tree that matches the
//! parsed value; a malformed step file renders an **error row** and the sibling
//! tabs still build (STORIES S7.3, DESIGN §11).
//!
//! "Nothing is summarized away; a file yog cannot parse renders as an error row
//! rather than vanishing." The error row is `Doc::Unparsed` — the bytes kept
//! whole, flagged — and the point of the second half is that one bad file
//! costs you that file and nothing else.

#![allow(clippy::unwrap_used)]

use crate::support::{AgentFixture, build_agents, write_message, write_step};
use serde_json::json;
use tempfile::tempdir;
use yog::git_tree::AgentState;
use yog::jsonview::{self, Node};
use yog::steps_view::{self, Doc, UNPARSED};
use yog::transcript;

const REQUEST: &str = r#"{"model":"opus","messages":[{"role":"user","content":"hi"}]}"#;
const STAGING: &str = r#"{"files":["a.rs","b.rs"]}"#;
const RESPONSE: &str =
    "{\"type\":\"usage\",\"input_tokens\":10}\n{\"type\":\"finish\"}\n{\"type\":\"end\"}\n";

/// The fixture both halves stand on: one agent, one complete step carrying
/// meta / request / staging / response and two tool calls.
fn fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    let root = tempdir().unwrap();
    let ws = root.path().join("cobalt");
    std::fs::create_dir_all(&ws).unwrap();
    build_agents(&ws, &[AgentFixture::new("c-1", "work\n")]);
    write_message(
        &ws,
        "c-1",
        "001-opus.json",
        r#"{"content":[{"type":"text","text":"hi"}]}"#,
    );

    // Step 001: complete — meta, request, staging, response, and one tool call.
    write_step(&ws, "c-1", "001", "meta.json", r#"{"commit":"c0ffee"}"#);
    write_step(&ws, "c-1", "001", "request.json", REQUEST);
    write_step(&ws, "c-1", "001", "staging.json", STAGING);
    write_step(&ws, "c-1", "001", "response.json", RESPONSE);
    write_step(
        &ws,
        "c-1",
        "001",
        "tools/toolu_1/input.json",
        r#"{"name":"Read"}"#,
    );
    write_step(
        &ws,
        "c-1",
        "001",
        "tools/toolu_1/output.json",
        r#"{"exit_code":0}"#,
    );
    // A second tool that failed — `is_error` is read off the exit code, not
    // guessed from the text.
    write_step(
        &ws,
        "c-1",
        "001",
        "tools/toolu_2/input.json",
        r#"{"name":"Bash"}"#,
    );
    write_step(
        &ws,
        "c-1",
        "001",
        "tools/toolu_2/output.json",
        r#"{"exit_code":2}"#,
    );
    (root, ws)
}

/// STORIES **S7-T2** step-drilldown — the drilldown half: the row tree IS the
/// parsed value, walked.
#[test]
fn s7_t2_the_drilldown_row_tree_matches_the_parsed_value() {
    let (_root, ws) = fixture();
    let detail = steps_view::detail(&ws, "c-1", "001");
    assert_eq!(detail.seq, "001");

    // --- Every record parsed, none summarized away.
    assert!(matches!(detail.request, Doc::Json { .. }));
    assert!(matches!(detail.staging, Doc::Json { .. }));
    assert_eq!(detail.response.len(), 3, "one row per response event");
    assert_eq!(detail.tools.len(), 2);
    assert_eq!(detail.tools[0].tool_id, "toolu_1");
    assert!(!detail.tools[0].is_error);
    assert!(detail.tools[1].is_error, "exit_code 2 is an error");

    // --- The jsonview row tree IS the parsed value, walked. Same shape, same
    // order, same scalars — the widget adds nothing and drops nothing.
    let Doc::Json { value, .. } = &detail.request else {
        panic!("request parsed")
    };
    assert_eq!(
        *value,
        serde_json::from_str::<serde_json::Value>(REQUEST).unwrap()
    );
    let collapsed = std::collections::HashSet::new();
    let rows = jsonview::flatten(value, "001/request", &collapsed);
    assert_eq!(rows[0].label, "$");
    assert_eq!(rows[0].path, "001/request");
    assert_eq!(rows[0].node, Node::Object(2), "two keys at the root");
    let labels: Vec<&str> = rows.iter().map(|r| r.label.as_str()).collect();
    assert!(labels.contains(&"model") && labels.contains(&"messages"));
    // An array element is labelled `[i]` and pathed `/i` — the label is for the
    // eye, the path is the identity a collapse is keyed on.
    let elem = rows.iter().find(|r| r.label == "[0]").unwrap();
    assert_eq!(elem.path, "001/request/messages/0");
    // A scalar renders as compact JSON, so a string keeps its quotes and can
    // never be confused with a bare identifier.
    let model = rows.iter().find(|r| r.label == "model").unwrap();
    assert_eq!(model.node, Node::Scalar("\"opus\"".to_owned()));

    // Collapsing a container emits its own row and no descendants — the tree is
    // a query over the value plus the collapse set, with nothing stored.
    let collapsed = std::collections::HashSet::from(["001/request/messages".to_owned()]);
    let folded = jsonview::flatten(value, "001/request", &collapsed);
    assert!(folded.len() < rows.len());
    assert!(
        !folded.iter().any(|r| r.label == "[0]"),
        "a collapsed array hides its elements"
    );
    assert!(folded.iter().any(|r| r.label == "messages" && r.collapsed));
    // The same value, uncollapsed, is the same tree again — pure over inputs.
    assert_eq!(
        jsonview::flatten(value, "001/request", &std::collections::HashSet::new()),
        rows
    );
    // Flatten is total over any value, arrays and bare scalars included.
    assert_eq!(
        jsonview::flatten(&json!(7), "r", &std::collections::HashSet::new())[0].node,
        Node::Scalar("7".to_owned())
    );
}

/// STORIES **S7-T2** step-drilldown — the malformed half: a file yog cannot
/// parse is an error row, and its siblings still build.
#[test]
fn s7_t2_a_malformed_file_is_a_row_and_its_siblings_still_build() {
    let (_root, ws) = fixture();
    // --- A malformed file renders an ERROR ROW, not a hole. Step 002's request
    // is not JSON at all.
    write_step(&ws, "c-1", "002", "meta.json", r#"{"commit":"deadbee"}"#);
    write_step(&ws, "c-1", "002", "request.json", "this is not json {{{");
    write_step(&ws, "c-1", "002", "response.json", RESPONSE);
    let bad = steps_view::detail(&ws, "c-1", "002");
    let Doc::Unparsed(raw) = &bad.request else {
        panic!("a bad file is Unparsed, never Absent: {:?}", bad.request)
    };
    assert_eq!(raw.as_slice(), b"this is not json {{{", "bytes kept whole");
    assert!(
        !UNPARSED.is_empty(),
        "and the row says so in words, not by omission"
    );
    // An absent record is a different value from an unreadable one — "there is
    // no staging" and "the staging will not parse" are not the same fact.
    assert!(matches!(bad.staging, Doc::Absent));

    // --- The sibling tabs still build. One bad file costs that file only.
    assert!(
        matches!(bad.meta, Doc::Json { .. }),
        "the good sibling parsed"
    );
    assert_eq!(bad.response.len(), 3);
    let steps = steps_view::build(&ws, "c-1", AgentState::Quiescent);
    assert_eq!(steps.steps.len(), 2, "both steps still listed");
    assert!(
        !steps.steps[1].wound.wounded(),
        "a step with a response is not a wound, whatever its request says"
    );
    assert_eq!(transcript::build(&ws, "c-1").entries.len(), 1);
}
