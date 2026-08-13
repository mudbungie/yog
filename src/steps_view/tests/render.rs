//! Headless shape-walk tests for the Steps tab widget — assert user-visible
//! text (framing badges, summary figures, tab headers, drill-in trees) lands
//! in the paint output. Structs are built directly, so the render path is
//! exercised without touching disk; each [`Doc`] is built from bytes through
//! the module's own classifier, because a record *is* its bytes plus what they
//! parsed to (§11 Raw). The Raw toggle's own half of this walk is [`super::raw`].

use std::collections::HashSet;

use crate::git_tree::Framing;
use crate::steps_view::render::{StepTab, render};
use crate::steps_view::{Doc, StepDetail, StepSummary, StepsView, ToolIo, UNPARSED, Wound};

pub(super) fn painted(
    view: &StepsView,
    selected: Option<usize>,
    detail: Option<&StepDetail>,
    tab: StepTab,
) -> String {
    let mut collapsed = HashSet::new();
    crate::paint_probe::paint(|ui| {
        render(ui, view, selected, detail, tab, &mut collapsed, false);
    })
}

fn summary(seq: &str, framing: Framing) -> StepSummary {
    StepSummary {
        seq: seq.into(),
        framing,
        attempts: 2,
        tokens: crate::budgets::spend_from_bytes(
            br#"{"type":"usage","input_tokens":7,"output_tokens":0}"#,
        ),
        commit: Some("abcdef1234567890".into()),
        started_at: Some("t-start".into()),
        ended_at: Some("t-end".into()),
        auth_failed: crate::login::auth::AuthFailure::No,
        wound: Wound::None,
    }
}

#[test]
fn empty_view_shows_placeholder() {
    let text = painted(&StepsView::default(), None, None, StepTab::Meta);
    assert!(text.contains("(no steps yet)"), "got:\n{text}");
}

#[test]
fn list_paints_badges_selection_and_figures() {
    let view = StepsView {
        steps: vec![
            summary("001", Framing::Complete),
            summary("002", Framing::Failed),
            StepSummary {
                seq: "003".into(),
                framing: Framing::Killed,
                attempts: 0,
                tokens: crate::budgets::BudgetSpend::default(),
                commit: None,
                started_at: None,
                ended_at: None,
                auth_failed: crate::login::auth::AuthFailure::No,
                wound: Wound::None,
            },
        ],
    };
    let text = painted(&view, Some(0), None, StepTab::Meta);
    // One badge per outcome — and per §11 the words are on the row outright,
    // not on hover: delete the glyph and the outcome still reads.
    assert!(text.contains("✔ complete"), "got:\n{text}");
    assert!(text.contains("✖ failed"), "got:\n{text}");
    assert!(text.contains("■ no clean end"), "got:\n{text}");
    // Selected step 0 carries the ▶ marker; the figures render.
    assert!(text.contains("▶ 001"));
    assert!(text.contains('2'));
    assert!(text.contains('7'));
    // Short commit (7 chars) for step 001, and none for the commit-less 003.
    assert!(text.contains("abcdef1"));
    assert!(text.contains("t-start"));
    assert!(text.contains("t-end"));
}

#[test]
fn every_column_is_headed_and_explained() {
    use crate::steps_view::columns::COLUMNS;
    let view = StepsView {
        steps: vec![summary("001", Framing::Complete)],
    };
    let text = painted(&view, Some(0), None, StepTab::Meta);
    // bl-3ffc: the list was seven bare values with nothing saying which was
    // which. Every column now paints its own name above the values, and every
    // name carries a sentence of explanation for an operator meeting it cold.
    let mut headers = std::collections::HashSet::new();
    for column in COLUMNS {
        assert!(
            text.contains(column.header),
            "unlabelled column {:?}:\n{text}",
            column.header
        );
        assert!(
            headers.insert(column.header),
            "duplicate {:?}",
            column.header
        );
        assert!(!column.hint.is_empty(), "unexplained {:?}", column.header);
    }
    // The header row exists only over a table: no steps, no headings.
    let empty = painted(&StepsView::default(), None, None, StepTab::Meta);
    assert!(
        !empty.contains("Attempts"),
        "headers over nothing:\n{empty}"
    );
}

#[test]
fn every_framing_says_itself_in_distinct_words() {
    use crate::steps_view::render::{framing_badge, summary_badge};
    let mut phrases = std::collections::HashSet::new();
    for framing in [Framing::Complete, Framing::Failed, Framing::Killed] {
        let (glyph, _, phrase) = framing_badge(framing);
        // The glyph doctrine (§11): every outcome says itself in words, so the
        // badge is never the outcome's only carrier. A phrase that is empty —
        // or shared with another outcome — puts the load back on the glyph.
        assert!(!phrase.is_empty(), "unsaid framing {framing:?}");
        assert!(
            phrases.insert(phrase),
            "duplicate phrase {phrase:?} for {framing:?}"
        );
        assert!(!glyph.is_empty(), "badgeless framing {framing:?}");
    }
    // The wound outranks the framing read and borrows Failed's ✖ and hue, so
    // its words are the only thing telling the two apart — exactly the load the
    // doctrine moves off the glyph.
    let wound = summary_badge(&StepSummary {
        wound: Wound::Mute,
        ..summary("001", Framing::Killed)
    });
    let failed = framing_badge(Framing::Failed);
    assert_eq!((wound.0, wound.1), (failed.0, failed.1));
    assert!(phrases.insert(wound.2), "the wound speaks for itself");
}

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
    }
}

#[test]
fn detail_tabs_render_their_records() {
    let view = StepsView {
        steps: vec![summary("001", Framing::Complete)],
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
    };
    let empty = StepDetail {
        seq: "001".into(),
        meta: Doc::Absent,
        request: Doc::Absent,
        staging: Doc::Absent,
        response: Vec::new(),
        tools: Vec::new(),
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
