//! **Elision has a floor** (bl-5410): the half of QUALITY §1 G1 that
//! [`super`]'s predicate is structurally blind to.
//!
//! That sweep tells a silent cut from a marked one by comparing the width a run
//! was laid at against the width its clip let through, and a galley egui
//! truncated *on purpose* is correctly not reported — it fits its box and ends
//! in `…`. But truncation has a floor, and under it the elision eats the run:
//! `[ … ]` where a verb was, which is bl-bc06's original defect and says
//! strictly less than the clipped `Sto` it replaced. Both are held here, on the
//! same frames, because the two are one loss to the operator.
//!
//! The repair the two halves ask for is different, which is why both are
//! needed: rule 1's `Truncate` answers the first and *causes* the second on a
//! control, and rule 8's wrap ([`row::peers`](crate::shell::row::peers))
//! answers the second. A tree with only the first sweep passes with the
//! composer's `Stop` truncated to nothing at the minimum window.

use super::super::window;
use crate::keymap::{CenterTab, InspectorTab};
use crate::paint_probe::seen_of;

/// **No run on the glass is a bare ellipsis** — the other half of G1, and the
/// half this file's own predicate is blind to (bl-5410).
///
/// A silent cut and a marked one are told apart by comparing laid against shown,
/// and a galley egui truncated *on purpose* is correctly not reported: it fits
/// its box and it ends in `…`. But truncation has a floor, and under it the
/// elision eats the whole run — `[ … ]` where a verb was, which is bl-bc06's
/// original defect and says strictly less than the clipped `Sto` it replaced.
/// The two failures are opposite in the sweep above (one over-wide, one exactly
/// as wide as its box) and identical to the operator, so both are held here.
///
/// Two directions, as everywhere: the sweep must have seen a real frame's worth
/// of runs, or an empty window would satisfy this without painting anything.
#[test]
fn no_run_is_elided_down_to_a_bare_ellipsis() {
    let mut stubs = Vec::new();
    let mut runs = 0usize;
    for (w, h) in super::super::SIZES {
        for tab in [
            CenterTab::Conversation,
            CenterTab::Config,
            CenterTab::Login,
            CenterTab::Search,
        ] {
            for seen in seen_of(&window(w, h, false, tab, InspectorTab::Transcript)) {
                runs += 1;
                if seen.text.trim() == "…" {
                    stubs.push(format!(
                        "a run at {w}x{h} ({tab:?}) says only `…`: {:?}",
                        seen.laid
                    ));
                }
            }
        }
    }
    assert!(
        runs > 200,
        "the sweep saw {runs} runs — it is not reading a real frame"
    );
    assert!(
        stubs.is_empty(),
        "these runs were elided until nothing was left — a control cut to `…` names \
         neither what it is nor what it does (§11 rule 1b):\n{}",
        stubs.join("\n")
    );
}

/// **The composer's verbs survive every window size** (bl-5410). The audit's
/// headline evidence was `Stop`, a *control*, showing 16 of its 25 points at
/// 420x320; the repair is rule 8's wrap, not rule 1's truncation, and this is
/// what tells the two apart — the sweep above would be satisfied by a `Stop`
/// truncated to fit, and the operator would not.
///
/// Asserted by exact glyph equality on a painted run: `Sto…` is not `Stop`, and
/// a needle test with `contains` would accept it. Both verbs, at all four sizes,
/// so the beat cannot pass on the roomy end alone.
#[test]
fn the_composer_verbs_are_painted_whole_at_every_window_size() {
    for (w, h) in super::super::SIZES {
        let painted = seen_of(&window(
            w,
            h,
            false,
            CenterTab::Conversation,
            InspectorTab::Transcript,
        ));
        for verb in ["Message", "Stop"] {
            assert!(
                painted.iter().any(|seen| seen.text == verb),
                "`{verb}` is not on the glass whole at {w}x{h}; what the composer \
                 painted was {:?}",
                painted
                    .iter()
                    .filter(
                        |s| verb.starts_with(s.text.trim_end_matches('…')) || s.text.trim() == "…"
                    )
                    .map(|s| &s.text)
                    .collect::<Vec<_>>()
            );
        }
    }
}
