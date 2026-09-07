//! [`super::super::detail`]'s half of the view-model tests: one step's records
//! read on demand — what parses, what stays raw bytes, what is honestly absent,
//! and the `is_error` derived off each tool's exit code.
//!
//! Split from [`vm`](super::vm) at §12's budget on the read seam §12 already
//! names for the production side: `mod` is the listing, `detail` the drill-in.

use tempfile::tempdir;

use super::{AGENT, step_dir, write_file, write_tool};
use crate::steps_view::{Doc, detail};

#[test]
fn detail_parses_records_splits_response_and_derives_tool_is_error() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "001", "meta.json", br#"{"commit":"c"}"#);
    write_file(ws, "001", "request.json", br#"{"model":"opus"}"#);
    write_file(
        ws,
        "001",
        "staging.json",
        br#"[{"type":"text","text":"hi"}]"#,
    );
    write_file(
        ws,
        "001",
        "response.json",
        b"{\"type\":\"message_start\"}\nnot json\n{\"type\":\"end\"}\n",
    );
    // exit 0 → ok; exit 2 → error; malformed output → raw, not an error;
    // output with no exit_code → not an error.
    write_tool(ws, "001", "toolu_1", b"{}", Some(br#"{"exit_code":0}"#));
    write_tool(ws, "001", "toolu_2", b"{}", Some(br#"{"exit_code":2}"#));
    write_tool(ws, "001", "toolu_0", b"{}", Some(b"oops"));
    write_tool(ws, "001", "toolu_3", b"{}", Some(br#"{"stdout":"x"}"#));
    // A stray file at the tools/ level is not a tool call.
    std::fs::write(step_dir(ws, "001").join("tools").join(".keep"), b"").unwrap();

    let d = detail(ws, AGENT, "001").unwrap();
    assert!(matches!(d.meta, Doc::Json { .. }));
    assert!(matches!(d.request, Doc::Json { .. }));
    assert!(matches!(d.staging, Doc::Json { .. }));

    // response: three events, the middle one kept raw; the empty trailing
    // line dropped.
    assert_eq!(d.response.len(), 3);
    assert!(matches!(d.response[0], Doc::Json { .. }));
    assert_eq!(d.response[1], Doc::Unparsed(b"not json".to_vec()));
    assert!(matches!(d.response[2], Doc::Json { .. }));

    // tools sorted by id; is_error derived from exit_code.
    let ids: Vec<&str> = d.tools.iter().map(|t| t.tool_id.as_str()).collect();
    assert_eq!(ids, vec!["toolu_0", "toolu_1", "toolu_2", "toolu_3"]);
    let errs: Vec<bool> = d.tools.iter().map(|t| t.is_error).collect();
    assert_eq!(errs, vec![false, false, true, false]);
    assert_eq!(d.tools[0].output, Doc::Unparsed(b"oops".to_vec()));
}

/// STORIES **S7-T2** / DESIGN §11, view-model half (bl-307f): the drive's own
/// repro — a real step whose `request.json` is overwritten with `{ this is not
/// json`. The doc is [`Doc::Unparsed`], *not* [`Doc::Absent`] and not silently
/// raw: the two are distinct facts here so the renderer can frame one and not
/// the other. Every byte is kept, and the sibling records still build.
#[test]
fn a_malformed_record_is_unparsed_not_absent_and_siblings_still_build() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "001", "request.json", b"{ this is not json");
    write_file(ws, "001", "meta.json", br#"{"commit":"c0ffee"}"#);
    // A file of pure whitespace parses as nothing either — malformed, not absent.
    write_file(ws, "001", "staging.json", b"   \n");

    let d = detail(ws, AGENT, "001").unwrap();
    assert_eq!(
        d.request,
        Doc::Unparsed(b"{ this is not json".to_vec()),
        "malformed keeps its bytes AND says it is malformed"
    );
    assert_eq!(d.staging, Doc::Unparsed(b"   \n".to_vec()));
    assert!(
        matches!(d.meta, Doc::Json { .. }),
        "the sibling record parses"
    );
    assert!(d.response.is_empty(), "no response.json ⇒ no events");
}

#[test]
fn detail_is_forgiving_when_every_file_is_absent() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    // The record directory exists and holds nothing — a step litany opened and
    // never filled. Absent records are that fact, and this is the one shape
    // entitled to wear it.
    std::fs::create_dir_all(step_dir(ws, "001")).unwrap();
    let d = detail(ws, AGENT, "001").unwrap();
    assert_eq!(d.meta, Doc::Absent);
    assert_eq!(d.request, Doc::Absent);
    assert_eq!(d.staging, Doc::Absent);
    assert!(d.response.is_empty());
    assert!(d.tools.is_empty());
    assert_eq!(d.seq, "001");
}

/// bl-136f: the number the numbered listing invites. `lernie steps` prints
/// `003`, so `3` is what an operator types — and joined verbatim it named a
/// directory that is not there, which read back as a step whose every record
/// was absent, under `ok:true`. The seq is the number now, however it is
/// spelled, and the answer is the records.
#[test]
fn a_seq_typed_as_a_bare_number_addresses_the_padded_record() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "003", "meta.json", br#"{"commit":"c0ffee"}"#);
    for typed in ["3", "03", "003", "0003"] {
        let d = detail(ws, AGENT, typed).unwrap();
        assert!(
            matches!(d.meta, Doc::Json { .. }),
            "{typed:?} must address the record"
        );
        assert_eq!(d.seq, "003", "and answer under the record's own name");
    }
}

/// The other half: a seq that names no record **refuses, naming the ones that
/// do** — never `ok:true` over a path that was never opened.
#[test]
fn a_seq_that_names_no_record_refuses_and_names_the_ones_that_do() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "001", "meta.json", b"{}");
    write_file(ws, "002", "meta.json", b"{}");
    assert_eq!(
        detail(ws, AGENT, "9").unwrap_err(),
        "unknown step \"9\": steps here are 001, 002"
    );
    // Non-numeric needles are carried to the listing verbatim rather than
    // mangled into a number they never were, and so is the empty one.
    for typed in ["latest", ""] {
        assert_eq!(
            detail(ws, AGENT, typed).unwrap_err(),
            format!("unknown step {typed:?}: steps here are 001, 002")
        );
    }
    // Zero is a number like any other — `0` is the record `000`, which no
    // conversation has, so it refuses rather than reading the first step.
    assert_eq!(
        detail(ws, AGENT, "0").unwrap_err(),
        "unknown step \"0\": steps here are 001, 002"
    );
}

/// And with no steps at all the refusal says so, rather than listing nothing.
#[test]
fn a_conversation_with_no_steps_refuses_by_saying_it_has_none() {
    let dir = tempdir().unwrap();
    assert_eq!(
        detail(dir.path(), AGENT, "1").unwrap_err(),
        "unknown step \"1\": this conversation has taken no step"
    );
}
