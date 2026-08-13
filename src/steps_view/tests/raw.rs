//! S7-T1, Steps half: the Raw toggle yields the underlying record file's bytes
//! **unaltered**.
//!
//! Every fixture record is written with spacing `serde_json` would never emit,
//! so "verbatim" is a claim the assertion can actually catch: the parsed view
//! renders the jsonview tree and never the literal bytes, the raw view renders
//! the literal bytes and never the tree's framing.

use std::collections::HashSet;

use super::{AGENT, write_file, write_tool};
use crate::steps_view::render::{StepTab, render};
use crate::steps_view::{StepDetail, StepsView, UNPARSED, detail};

/// The odd-spaced bytes of each record — no serializer produces these.
const META: &str = r#"{ "commit" :   "c0ffee" }"#;
const REQUEST: &str = r#"{ "model" :   "opus" }"#;
const STAGING: &str = "not json at all";
const EVENT: &str = r#"{ "type" :   "end" }"#;
const TOOL_IN: &str = r#"{ "name" :   "Read" }"#;
const TOOL_OUT: &str = r#"{ "exit_code" :   0 }"#;

fn painted(detail: &StepDetail, tab: StepTab, raw: bool) -> String {
    let mut collapsed = HashSet::new();
    crate::paint_probe::paint(|ui| {
        render(
            ui,
            &StepsView::default(),
            None,
            Some(detail),
            tab,
            &mut collapsed,
            raw,
        );
    })
}

/// A step whose five records are all present, each written with the odd
/// spacing above.
fn populated() -> (tempfile::TempDir, StepDetail) {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "001", "meta.json", META.as_bytes());
    write_file(ws, "001", "request.json", REQUEST.as_bytes());
    write_file(ws, "001", "staging.json", STAGING.as_bytes());
    write_file(ws, "001", "response.json", format!("{EVENT}\n").as_bytes());
    write_tool(
        ws,
        "001",
        "toolu_1",
        TOOL_IN.as_bytes(),
        Some(TOOL_OUT.as_bytes()),
    );
    let d = detail(ws, AGENT, "001");
    (dir, d)
}

#[test]
fn raw_mode_paints_every_records_bytes_unaltered() {
    let (_dir, d) = populated();
    for (tab, bytes) in [
        (StepTab::Meta, META),
        (StepTab::Request, REQUEST),
        (StepTab::Response, EVENT),
    ] {
        let raw = painted(&d, tab, true);
        assert!(raw.contains(bytes), "bytes not verbatim in raw:\n{raw}");
        let parsed = painted(&d, tab, false);
        assert!(
            !parsed.contains(bytes),
            "parsed view should render the tree, not the bytes:\n{parsed}"
        );
    }
}

#[test]
fn raw_mode_paints_both_halves_of_a_tool_call() {
    let (_dir, d) = populated();
    let raw = painted(&d, StepTab::Tools, true);
    assert!(raw.contains(TOOL_IN), "tool input not verbatim:\n{raw}");
    assert!(raw.contains(TOOL_OUT), "tool output not verbatim:\n{raw}");
    // The tool's own framing — the id and the ok/error badge — still heads the
    // bytes: raw is an escape from the *parse*, not from the row's identity.
    assert!(raw.contains("toolu_1"), "tool id missing in raw:\n{raw}");
}

/// An unparseable record already shows its bytes under the [`UNPARSED`] error
/// row; under Raw it is the bytes alone, because the framing is the parsed
/// view's word about the file and Raw is the file itself.
#[test]
fn raw_mode_drops_the_unparsed_framing_and_keeps_the_bytes() {
    let (_dir, d) = populated();
    let parsed = painted(&d, StepTab::Staging, false);
    assert!(parsed.contains(UNPARSED), "error row missing:\n{parsed}");
    let raw = painted(&d, StepTab::Staging, true);
    assert!(raw.contains(STAGING), "bytes missing in raw:\n{raw}");
    assert!(!raw.contains(UNPARSED), "framing kept in raw:\n{raw}");
}

/// A record with no bytes has none to show — Raw says "(absent)" exactly as the
/// parsed view does, rather than painting a blank the reader must interpret.
#[test]
fn raw_mode_says_absent_when_the_record_has_no_bytes() {
    let dir = tempfile::tempdir().unwrap();
    write_file(dir.path(), "001", "meta.json", META.as_bytes());
    let d = detail(dir.path(), AGENT, "001");
    let raw = painted(&d, StepTab::Request, true);
    assert!(raw.contains("(absent)"), "absent record unsaid:\n{raw}");
}

/// The response record is a JSONL list, so Raw shows every line's bytes — the
/// per-event split is the reader's, not a summary.
#[test]
fn raw_mode_paints_every_response_event() {
    let dir = tempfile::tempdir().unwrap();
    write_file(
        dir.path(),
        "001",
        "response.json",
        format!("{EVENT}\n{{ \"type\" :   \"usage\" }}\n").as_bytes(),
    );
    let d = detail(dir.path(), AGENT, "001");
    let raw = painted(&d, StepTab::Response, true);
    assert!(raw.contains(EVENT), "first event missing:\n{raw}");
    assert!(
        raw.contains(r#"{ "type" :   "usage" }"#),
        "second event missing:\n{raw}"
    );
}
