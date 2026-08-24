//! **A strip of peers is all of them or none of them** (§11 rule 8, bl-b531 —
//! *"four of six inspector tabs are off-window"*), split from [`super`]'s G1
//! sweep at §12's budget on the seam between two different claims over the same
//! frame: that sweep asks whether any run was cut without saying so, this asks
//! whether a **set** of controls reached the glass whole and together.
//!
//! Neither subsumes the other. Every tab on the strip can be uncut while two of
//! them were never laid out at all — the state that has no seat to hover, no
//! rect to click and no ellipsis to warn you — and that absence is invisible to
//! a predicate over the runs a frame did paint.

use super::super::window;
use crate::keymap::{CenterTab, InspectorTab};
use crate::paint_probe::seen_of;

/// How far below the strip's first row a wrapped second row can land — one row
/// of controls plus the spacing between them (§11 rule 8, `row::peers`).
const WRAPPED: f32 = 40.0;

/// **A strip of peers is all of them or none of them** (§11 rule 8, bl-b531 —
/// *"four of six inspector tabs are off-window"*), asserted through the **real**
/// window rather than the synthetic strip `super::super::super::row`'s own test builds.
///
/// That distinction is the finding, not a detail: `row::peers`' test seats six
/// `selectable_label`s in a `CentralPanel` of exactly the width the audit
/// measured, which is the ambient state production does not guarantee. The
/// shipped strip sits inside the conversation pane, inside a bounded viewport
/// whose width is whatever the roster column left it, and only this render can
/// say whether the rule survives the trip.
///
/// **All or none**, because §11 rule 6 is the other half of the answer: at
/// 420x320 nothing fits, so the centre column is a scrolling viewport and the
/// whole altitude-2 inspector is below the fold — reachable, and correctly not
/// on the glass. What must never happen is the in-between: *some* of the strip
/// shown while the rest was silently never laid out, which is the state that has
/// no seat to hover, no rect to click and no ellipsis to warn you.
///
/// Two more predicates on each tab that IS shown — whole (not cut to a stub) and
/// inside the window (not laid past its edge) — because a strip can hold all six
/// and still be unusable in either of those ways. And the strip is required to
/// be up at some size, or the whole test would pass on a window that had lost
/// the inspector outright.
#[test]
fn the_inspector_strip_shows_all_of_its_peers_or_none_of_them() {
    let want: Vec<&str> = InspectorTab::all().iter().map(|t| t.label()).collect();
    let mut ever = false;
    for (w, h) in super::super::SIZES {
        let out = window(
            w,
            h,
            false,
            CenterTab::Conversation,
            InspectorTab::Transcript,
        );
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(w, h));
        let painted = seen_of(&out);
        // The strip is located by `Transcript`, the one label no other strip in
        // the window wears — the centre's own tab bar spells `Config` too, and a
        // search by name alone would count that one and report a strip of one.
        let Some(head) = painted.iter().find(|seen| seen.text == "Transcript") else {
            continue;
        };
        ever = true;
        // Its band: the head's row, plus the room `row::peers` wraps a second
        // line into at a narrow pane. A peer found outside it is on some other
        // surface and is not evidence about this one.
        let band = head.laid.top() - 1.0..=head.laid.bottom() + WRAPPED;
        for label in &want {
            let seat = painted
                .iter()
                .find(|seen| &seen.text == label && band.contains(&seen.laid.top()))
                .unwrap_or_else(|| {
                    panic!(
                        "the strip is up at {w}x{h} but `{label}` is not on it: {:?}",
                        painted
                            .iter()
                            .filter(|s| band.contains(&s.laid.top()))
                            .map(|s| &s.text)
                            .collect::<Vec<_>>()
                    )
                });
            assert!(
                seat.shown.width() >= seat.laid.width() - 1.0,
                "the `{label}` tab is cut at {w}x{h}: {} pt of {} pt shown",
                seat.shown.width(),
                seat.laid.width()
            );
            assert!(
                screen.contains_rect(seat.laid),
                "the `{label}` tab is laid off the window at {w}x{h}: {:?}",
                seat.laid
            );
        }
    }
    assert!(
        ever,
        "no window size painted the inspector strip at all — the property above \
         would have held vacuously"
    );
}
