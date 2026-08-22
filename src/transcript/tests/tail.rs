//! §11 tail idiom, both halves. On a screen too short to hold the conversation
//! the view rides the newest rows (bl-5cdb); on one too *tall* the conversation
//! still sits on the bottom edge, empty space above it (bl-8c13). One complaint
//! twice: text that arrives at the top and grows down, so following live output
//! means scrolling — from the very first line, not just after the screen fills.

use crate::transcript::{AutoExpand, Entry, EntryKind, Transcript};
use std::collections::HashSet;

/// A screen far too short for `long()`, so the anchor decides what is seen.
const NARROW: (f32, f32) = (400.0, 120.0);

/// Enough one-line rows that only a handful fit the viewport at once.
/// Delivered messages, because a delivery is a turn boundary and never a step
/// inside one — so what the anchor decides is tested over rows the turn rollup
/// leaves exactly as they are.
fn long() -> Transcript {
    let entries = (0..40)
        .map(|i| Entry {
            name: format!("{i:03}-user.md"),
            raw: Vec::new(),
            kind: EntryKind::Delivered {
                sender: "user".into(),
                epitaph: None,
                body: format!("line-{i:03}"),
            },
        })
        .collect();
    Transcript { entries }
}

/// Paint the transcript into a scrolling viewport, settled (frame two).
fn settled(t: &Transcript) -> String {
    let mut folds = HashSet::new();
    crate::paint_probe::paint_settled(NARROW.0, NARROW.1, |ui| {
        super::plain(ui, t, false, AutoExpand::default(), &mut folds);
    })
}

/// The same settled frame read for geometry: the topmost and bottommost pixel
/// the transcript painted.
fn settled_span(t: &Transcript) -> (f32, f32) {
    let mut folds = HashSet::new();
    let painted = crate::paint_probe::painted_settled(NARROW.0, NARROW.1, |ui| {
        super::plain(ui, t, false, AutoExpand::default(), &mut folds);
    });
    crate::paint_probe::span(&painted)
}

/// The first two rows of `long()` — far shorter than the viewport.
fn underfull() -> Transcript {
    Transcript {
        entries: long().entries.into_iter().take(2).collect(),
    }
}

/// One frame on `ctx` at [`NARROW`] carrying `events`, returning what it
/// painted — the interaction driver behind the stick-yield tests below.
fn event_frame(ctx: &egui::Context, t: &Transcript, events: Vec<egui::Event>) -> String {
    let mut folds = HashSet::new();
    let input = egui::RawInput {
        events,
        ..crate::paint_probe::screen_sized(NARROW.0, NARROW.1)
    };
    let output = ctx.run(input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            super::plain(ui, t, false, AutoExpand::default(), &mut folds);
        });
    });
    crate::paint_probe::text_of(&output)
}

/// A wheel turn of `dy` points with the pointer over the transcript. The
/// pointer sits near the left edge: the auto-shrunk scroll area is only as
/// wide as its widest row, and a wheel is consumed only under the pointer.
fn wheel(dy: f32) -> Vec<egui::Event> {
    vec![
        egui::Event::PointerMoved(egui::pos2(20.0, NARROW.1 / 2.0)),
        egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, dy),
            modifiers: egui::Modifiers::default(),
        },
    ]
}

/// §11 tail idiom, the yield half (bl-e90a): the bottom anchor must release
/// on the operator's own scroll-up immediately, must NOT be re-imposed by the
/// transcript growing while they read, and re-engages only when they return
/// to the tail. The anchor is egui's `stick_to_bottom`; this pins the whole
/// gesture so a regression in `crate::tail` or an egui upgrade cannot bring
/// back the fought scroll the operator reported as "sticky".
#[test]
fn scrolling_up_releases_the_tail_and_growth_does_not_recapture_it() {
    let mut t = long();
    let ctx = egui::Context::default();
    // Settle on the tail.
    let _ = event_frame(&ctx, &t, Vec::new());
    let painted = event_frame(&ctx, &t, Vec::new());
    assert!(painted.contains("line-039"), "settled on the tail");
    // The operator scrolls up: one wheel turn, then the smoothing drains.
    let _ = event_frame(&ctx, &t, wheel(3000.0));
    let mut painted = String::new();
    for _ in 0..20 {
        painted = event_frame(&ctx, &t, Vec::new());
    }
    assert!(
        !painted.contains("line-039"),
        "a scroll-up must leave the tail at once:\n{painted}"
    );
    // The conversation keeps streaming: new rows must NOT drag the view back.
    t.entries.push(Entry {
        name: "040-x.json".into(),
        raw: Vec::new(),
        kind: EntryKind::Streaming {
            thinking: String::new(),
            text: "fresh tail".into(),
        },
    });
    painted = event_frame(&ctx, &t, Vec::new());
    assert!(
        !painted.contains("fresh tail"),
        "growth must not recapture a released view:\n{painted}"
    );
    // Returning to the tail re-engages the anchor: further growth rides again.
    // The gesture must be **drained**, not merely landed. egui hands a wheel
    // turn to the view over many frames (`unprocessed_scroll_delta`, flushed
    // whole only once it decays under one point), and a leftover point spent
    // on the frame the transcript GROWS is a scroll gesture in egui's eyes —
    // it un-sticks the anchor at the one moment stick-to-bottom was about to
    // re-seat it, and the view then parks a row short of the tail forever. So
    // this loop is sized to outlast the decay, not to settle the paint.
    let _ = event_frame(&ctx, &t, wheel(-9000.0));
    for _ in 0..40 {
        painted = event_frame(&ctx, &t, Vec::new());
    }
    assert!(
        painted.contains("fresh tail"),
        "back on the tail:\n{painted}"
    );
    t.entries.push(Entry {
        name: "041-x.json".into(),
        raw: Vec::new(),
        kind: EntryKind::Streaming {
            thinking: String::new(),
            text: "freshest tail".into(),
        },
    });
    // Re-engaged, further growth rides again — within the same few-frame
    // settle the smooth-scroll tail always costs (egui's wheel smoothing
    // releases its last points over the frames after the gesture).
    for _ in 0..8 {
        painted = event_frame(&ctx, &t, Vec::new());
    }
    assert!(
        painted.contains("freshest tail"),
        "re-engaged: the view rides new rows again:\n{painted}"
    );
}

#[test]
fn overflowing_transcript_shows_its_newest_rows_not_its_oldest() {
    let painted = settled(&long());
    assert!(
        painted.contains("line-039"),
        "newest row must be on screen:\n{painted}"
    );
    assert!(
        !painted.contains("line-000"),
        "oldest row must have scrolled off the top:\n{painted}"
    );
}

#[test]
fn growing_transcript_keeps_the_new_tail_on_screen() {
    // The live case: the same view, one row longer, still shows the last row.
    let mut grown = long();
    grown.entries.push(Entry {
        name: "040-x.json".into(),
        raw: Vec::new(),
        kind: EntryKind::Streaming {
            thinking: String::new(),
            text: "partial output".into(),
        },
    });
    let painted = settled(&grown);
    assert!(painted.contains("partial output"), "got:\n{painted}");
    assert!(!painted.contains("line-000"), "got:\n{painted}");
}

#[test]
fn a_transcript_that_fits_shows_everything_it_has() {
    let painted = settled(&underfull());
    assert!(painted.contains("line-000"), "got:\n{painted}");
    assert!(painted.contains("line-001"), "got:\n{painted}");
}

#[test]
fn a_transcript_that_fits_still_sits_on_the_bottom_edge() {
    // The rule bl-5cdb left out and bl-8c13 rules in: two lines land at the
    // same bottom pixel forty do. Not "the anchor has nothing to hide" — the
    // first line of a new conversation appears where every later line will.
    let (_, overfull_bottom) = settled_span(&long());
    let (_, bottom) = settled_span(&underfull());
    assert!(
        (bottom - overfull_bottom).abs() < 1.0,
        "an underfull transcript must end on the same edge as a full one: \
         {bottom} vs {overfull_bottom}"
    );
}

#[test]
fn the_space_an_underfull_transcript_leaves_is_above_it() {
    // The other direction of the same fact: the two rows are pushed *down*, so
    // the empty part of the viewport is over them, not under.
    let (overfull_top, _) = settled_span(&long());
    let (top, _) = settled_span(&underfull());
    assert!(
        top - overfull_top > 40.0,
        "two rows must start well below where forty start: {top} vs {overfull_top}"
    );
}
