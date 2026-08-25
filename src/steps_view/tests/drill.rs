//! The Steps drill-in's headless shape-walk — the record trees below the table:
//! every tab header present, a parsed doc as a tree, an unparseable one as the
//! error row over its verbatim bytes, an absent one as `(absent)` and never the
//! error row, and each tool call's outcome said in words beside its glyph.
//!
//! Split from [`render`](super::render) at §12's budget on the seam the
//! production paints are already cut on: `render` is the table, `drill` the
//! records under it. The fixtures are still that file's — one `painted`, one
//! `summary`.

use super::render::{painted, summary};
use crate::git_tree::Framing;
use crate::steps_view::render::StepTab;
use crate::steps_view::{Doc, Orphan, StepDetail, StepsView, ToolIo, UNPARSED};

fn detail_fixture() -> StepDetail {
    StepDetail {
        seq: "001".into(),
        meta: Doc::of_bytes(br#"{"commit": "c0ffee"}"#.to_vec()),
        request: Doc::of_bytes(br#"{"model": "opus"}"#.to_vec()),
        staging: Doc::Unparsed(b"raw-staging".to_vec()),
        response: vec![
            Doc::of_bytes(br#"{"type": "end"}"#.to_vec()),
            Doc::Unparsed(b"bad-line".to_vec()),
        ],
        tools: vec![
            ToolIo {
                tool_id: "toolu_ok".into(),
                input: Doc::of_bytes(br#"{"name": "Read"}"#.to_vec()),
                output: Doc::of_bytes(br#"{"exit_code": 0}"#.to_vec()),
                is_error: false,
            },
            ToolIo {
                tool_id: "toolu_err".into(),
                input: Doc::of_bytes(br#"{"name": "Bash"}"#.to_vec()),
                output: Doc::Absent,
                is_error: true,
            },
        ],
        // The capture logs are [`super::logs`]'s half of this walk.
        stderr: None,
        driver: None,
    }
}

#[test]
fn detail_tabs_render_their_records() {
    let view = StepsView {
        steps: vec![summary("001", Framing::Complete)],
        orphan: Orphan::default(),
    };
    let d = detail_fixture();
    // Every tab header is always present; the active one is still text.
    let meta = painted(&view, Some(0), Some(&d), StepTab::Meta);
    for header in ["meta", "request", "staging", "response", "tools"] {
        assert!(meta.contains(header), "missing tab {header}:\n{meta}");
    }
    // bl-3ffc: the five are on-disk file names, so the picker says what the
    // row of them *is* before the operator has to guess from the words.
    assert!(meta.contains("Records:"), "unlabelled picker:\n{meta}");
    assert!(meta.contains("commit:"), "meta tree:\n{meta}");

    let request = painted(&view, Some(0), Some(&d), StepTab::Request);
    assert!(request.contains("model:"));

    // Staging is unparseable here → the error row above the verbatim bytes.
    let staging = painted(&view, Some(0), Some(&d), StepTab::Staging);
    assert!(staging.contains("raw-staging"));
    assert!(staging.contains(UNPARSED), "error row missing:\n{staging}");

    // Response: one parsed event tree, one malformed line — framed the same way.
    let response = painted(&view, Some(0), Some(&d), StepTab::Response);
    assert!(response.contains("type:"));
    assert!(response.contains("bad-line"));
    assert!(response.contains(UNPARSED));

    // Tools: ids, ok/error glyphs, and the input/output section labels.
    let tools = painted(&view, Some(0), Some(&d), StepTab::Tools);
    // The opaque provider id is named rather than left bare (bl-3ffc).
    assert!(tools.contains("call"), "unlabelled tool id:\n{tools}");
    assert!(tools.contains("toolu_ok"));
    assert!(tools.contains("toolu_err"));
    assert!(tools.contains("input"));
    assert!(tools.contains("output"));
    assert!(tools.contains('✔'));
    assert!(tools.contains('✖'));
    // §11 glyph doctrine (bl-4305): the outcome is said outright at this seat,
    // not carried by ✔/✖ and the hue alone.
    let (_, _, ok) = crate::theme::tool_result_badge(false);
    let (_, _, err) = crate::theme::tool_result_badge(true);
    assert!(tools.contains(ok), "unsaid ok outcome:\n{tools}");
    assert!(tools.contains(err), "unsaid error outcome:\n{tools}");
}

#[test]
fn empty_drill_ins_and_absent_docs_show_their_placeholders() {
    let view = StepsView {
        steps: vec![summary("001", Framing::Complete)],
        orphan: Orphan::default(),
    };
    let empty = StepDetail {
        seq: "001".into(),
        meta: Doc::Absent,
        request: Doc::Absent,
        staging: Doc::Absent,
        response: Vec::new(),
        tools: Vec::new(),
        stderr: None,
        driver: None,
    };
    // Absent doc under the Meta tab — "(absent)", and pointedly NOT the
    // malformed error row: a missing file is not a broken one (S7-T2).
    let meta = painted(&view, Some(0), Some(&empty), StepTab::Meta);
    assert!(meta.contains("(absent)"), "got:\n{meta}");
    assert!(!meta.contains(UNPARSED), "absent ≠ unparseable:\n{meta}");
    // Empty event list and empty tool list.
    let response = painted(&view, Some(0), Some(&empty), StepTab::Response);
    assert!(response.contains("(no events)"));
    let tools = painted(&view, Some(0), Some(&empty), StepTab::Tools);
    assert!(tools.contains("(no tool calls)"));
}
