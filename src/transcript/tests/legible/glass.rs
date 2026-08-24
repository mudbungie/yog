//! **How a run is read off the glass** — the probe every bl-7654 assertion
//! spends, split off [`super`] at the cap on the seam three modules already
//! share (`legible`, `parity` and `speaker` all drive it).
//!
//! Every reader here goes through [`crate::paint_probe::seen_of`]: the glyphs
//! that reached the glass, the rect they were laid into, the part of that rect
//! the clip let through, and the ink they were painted in. Never `Row::preview`
//! / `Row::body` — the standing hazard on this surface is a probe that reads the
//! projection and therefore cannot see elision at all (bl-bc06: 1815 tests
//! passed while covering no truncation).

use std::collections::HashSet;

use super::super::render::{entry, tx};
use crate::paint_probe::{Seen, screen_sized, seen_of};
use crate::transcript::{AutoExpand, Block, EntryKind, Transcript, Usage};

/// The two window sizes this surface has broken at.
pub(crate) const SIZES: [(f32, f32); 2] = [(420.0, 320.0), (800.0, 500.0)];

/// The toggle glyphs, and the mark a row with nothing to fold wears instead.
pub(crate) const OPEN: &str = "▼";
pub(crate) const SHUT: &str = "▶";
pub(super) const NO_FOLD: &str = "·";

/// A payload that fits its line and has nothing to fold.
pub(super) const SHORT: &str = "pong";

/// A single-line model answer far wider than either window.
pub(crate) fn long() -> String {
    "abcdefghij".repeat(40)
}

/// Every galley one settled render of `t` put on the glass. `bounded` is
/// whether the seat carries the centre's ambient `Truncate`; `false` is a seat
/// that declares nothing, where egui's horizontal default is `Extend`.
pub(crate) fn seen(t: &Transcript, auto: AutoExpand, bounded: bool, w: f32, h: f32) -> Vec<Seen> {
    let ctx = egui::Context::default();
    let mut folds = HashSet::new();
    let mut frame = || {
        ctx.run(screen_sized(w, h), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                if bounded {
                    // Verbatim what `shell::row::bounded` puts at the centre
                    // panel root (bl-5410) — the transcript's real seat.
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                }
                super::super::plain(ui, t, false, auto, &mut folds);
            });
        })
    };
    let _ = frame();
    seen_of(&frame())
}

/// A run is **whole** when its seat showed all of what was laid into it. A run
/// laid wider than its clip rect is one the operator reads the front of, with
/// no ellipsis to say so — the galley was never truncated, so egui never added
/// one.
pub(crate) fn whole(seen: &Seen) -> bool {
    seen.shown.width() >= seen.laid.width() - 1.0
}

/// The run carrying `needle`'s glyphs, whichever seat painted it.
pub(crate) fn run(painted: &[Seen], needle: &str) -> Seen {
    let seats: Vec<&String> = painted.iter().map(|s| &s.text).collect();
    let hit = painted.iter().find(|s| s.text.contains(needle)).cloned();
    assert!(
        hit.is_some(),
        "nothing on the glass carries {needle:?}: {seats:?}"
    );
    hit.expect("asserted present just above")
}

/// A transcript of one model turn whose single text block is `payload`.
pub(crate) fn answer(payload: &str) -> Transcript {
    tx(vec![entry(EntryKind::Model {
        model_id: "opus".into(),
        usage: Usage::default(),
        blocks: vec![Block::Text(payload.into())],
    })])
}
