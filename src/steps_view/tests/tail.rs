//! §11 tail idiom on the Steps tab, in both directions. The table sits on its
//! newest step while the table is all there is — riding that bottom when the
//! steps overflow (bl-5cdb), pushed down onto it when they do not (bl-8c13) —
//! and the whole idiom comes off the moment a drill-in hangs below it: the
//! bottom of a detail is not the tail, so riding it would carry the step rows
//! off the top of the viewport the instant the operator picked one, and padding
//! down to it would push the same rows out of reach the other way.

use std::collections::HashSet;

use crate::git_tree::Framing;
use crate::steps_view::render::{StepTab, render};
use crate::steps_view::{Doc, StepDetail, StepSummary, StepsView, Wound};

/// A viewport far shorter than `many()`, so the anchor decides what is seen.
const NARROW: (f32, f32) = (900.0, 160.0);

/// A viewport taller than either a two-step table or that table plus a drill-in,
/// so what is being read is where an *underfull* body sits, not what fits.
const ROOMY: (f32, f32) = (900.0, 400.0);

fn many() -> StepsView {
    steps(1..=40)
}

/// A table far shorter than `ROOMY`.
fn few() -> StepsView {
    steps(1..=2)
}

fn steps(seqs: std::ops::RangeInclusive<u32>) -> StepsView {
    let steps = seqs
        .map(|i| StepSummary {
            seq: format!("{i:03}"),
            framing: Framing::Complete,
            attempts: 1,
            tokens: crate::budgets::BudgetSpend::default(),
            commit: None,
            started_at: None,
            ended_at: None,
            auth_failed: crate::login::auth::AuthFailure::No,
            wound: Wound::None,
        })
        .collect();
    StepsView { steps }
}

fn settled(view: &StepsView, detail: Option<&StepDetail>) -> String {
    let mut collapsed = HashSet::new();
    crate::paint_probe::paint_settled(NARROW.0, NARROW.1, |ui| {
        render(
            ui,
            view,
            Some(0),
            detail,
            StepTab::Meta,
            &mut collapsed,
            false,
        );
    })
}

fn detail_of(seq: &str) -> StepDetail {
    StepDetail {
        seq: seq.into(),
        meta: Doc::of_bytes(br#"{"commit": "c0ffee"}"#.to_vec()),
        request: Doc::Absent,
        staging: Doc::Absent,
        response: Vec::new(),
        tools: Vec::new(),
    }
}

#[test]
fn a_bare_step_table_rides_its_newest_step() {
    let painted = settled(&many(), None);
    assert!(
        painted.contains("040"),
        "newest step must be seen:\n{painted}"
    );
    assert!(
        !painted.contains("001"),
        "oldest step must have scrolled off:\n{painted}"
    );
}

#[test]
fn an_open_drill_in_releases_the_anchor() {
    // With a detail below the table the body's bottom is the detail's end, so
    // the view stays at the top where the step rows — and the header — are.
    let painted = settled(&many(), Some(&detail_of("001")));
    assert!(
        painted.contains("001"),
        "the picked step must stay in view:\n{painted}"
    );
    assert!(
        !painted.contains("c0ffee"),
        "the view must not be parked at the bottom of the detail:\n{painted}"
    );
}

/// Where a table starts painting in the roomy viewport — the top of its header
/// row.
fn top_in_roomy(view: &StepsView, detail: Option<&StepDetail>) -> f32 {
    let mut collapsed = HashSet::new();
    let painted = crate::paint_probe::painted_settled(ROOMY.0, ROOMY.1, |ui| {
        render(
            ui,
            view,
            Some(0),
            detail,
            StepTab::Meta,
            &mut collapsed,
            false,
        );
    });
    crate::paint_probe::span(&painted).0
}

#[test]
fn a_short_step_table_is_pushed_down_onto_the_bottom_edge() {
    // Two steps in a 400pt viewport: the newest step is still the bottom row,
    // so it is still where a fortieth would be — the header is pushed down with
    // it and the empty space is above (bl-8c13).
    let short = top_in_roomy(&few(), None);
    let full = top_in_roomy(&many(), None);
    assert!(
        short - full > 100.0,
        "a two-row table must start far below a forty-row one: {short} vs {full}"
    );
}

#[test]
fn an_open_drill_in_releases_the_bottom_alignment_too() {
    // The same two steps with a detail open: the body is no longer a tail, so
    // it is not seated on the bottom either — the table stays at the top, where
    // the operator who just picked a row is looking.
    let drilled = top_in_roomy(&few(), Some(&detail_of("001")));
    let bare = top_in_roomy(&few(), None);
    assert!(
        bare - drilled > 100.0,
        "a drilled-in table must sit at the top, not be padded down: \
         {drilled} vs {bare}"
    );
}
