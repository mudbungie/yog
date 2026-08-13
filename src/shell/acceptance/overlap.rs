//! QUALITY §1 **G4** (*"Resize sanity … nothing overlaps at either"*), pinned
//! on the paint layer at the window sizes the audit captured (bl-9551).
//!
//! This machine cannot screenshot yog, so the evidence is the frame itself:
//! render the whole shell off-screen at a named size, take every galley it
//! painted **clipped to what was actually visible**, and assert no two of them
//! share pixels. That is the audit's own finding stated as an arithmetic
//! property — its crop `crop-s6-overlap.png` was six independent text runs
//! between y=440 and y=545, which is exactly a non-empty answer from
//! [`overlaps`].
//!
//! The clip matters: a galley's own rect says where the text *would* be, and
//! egui emits shapes it then clips away. Reading the unclipped rects reports
//! collisions nobody can see; reading the clipped ones reports what is on the
//! glass.
//!
//! Measured before the fix, for the record — the same walk over the same
//! fixture: 239 colliding pairs at 420x320 with the activity trail open, 124 at
//! 800x500, 107 at 420x320 with it closed, 7 at 800x500 with it closed.

use super::input;
use crate::keymap::{CenterTab, InspectorTab};
use crate::paint_probe::Painted;

/// Every galley one finished frame put on the glass, each rect narrowed to the
/// part its clip rect actually let through — [`crate::paint_probe::seen_of`]
/// with the laid rect dropped, because a collision is a fact about what
/// reached the glass and nothing else.
pub(super) fn visible(output: &egui::FullOutput) -> Vec<Painted> {
    crate::paint_probe::seen_of(output)
        .into_iter()
        .map(|seen| (seen.text, seen.shown))
        .collect()
}

/// Pairs of visible galleys sharing pixels — the G4 defect, enumerated. The
/// tolerance is a point in each axis: adjacent rows and side-by-side words abut
/// by design and their rects touch, which is not two runs on the same pixels.
pub(super) fn overlaps(painted: &[Painted]) -> Vec<String> {
    let mut found = Vec::new();
    for (index, (first, here)) in painted.iter().enumerate() {
        for (second, there) in painted.iter().skip(index + 1) {
            let shared = here.intersect(*there);
            if shared.width() > 1.0 && shared.height() > 1.0 {
                found.push(format!("{first:?} {here:?} over {second:?} {there:?}"));
            }
        }
    }
    found
}

/// Render the whole window at `w` x `h` and report every collision on it, over
/// the one settled render every sized paint-layer property reads
/// ([`super::window`]).
fn collisions(w: f32, h: f32, trail: bool, tab: CenterTab) -> Vec<String> {
    overlaps(&visible(&super::window(
        w,
        h,
        trail,
        tab,
        InspectorTab::Transcript,
    )))
}

/// The property, at every size and with the conversation pane's accessory stack
/// both at rest and fully subscribed (the audit's step 8 — `a`, the activity
/// trail — is what over-subscribes it).
#[test]
fn no_two_runs_of_text_share_pixels_at_any_window_size() {
    let mut report = Vec::new();
    for (w, h) in super::SIZES {
        for trail in [false, true] {
            for tab in [CenterTab::Conversation, CenterTab::Config] {
                let found = collisions(w, h, trail, tab);
                if !found.is_empty() {
                    report.push(format!(
                        "{} runs share pixels at {w}x{h} (trail {trail}, {tab:?}):\n{}",
                        found.len(),
                        found.join("\n")
                    ));
                }
            }
        }
    }
    // Every size is reported at once: a resize defect is a shape across sizes,
    // and stopping at the first one hides which of them the fix actually moved.
    assert!(report.is_empty(), "{}", report.join("\n\n"));
}

/// The detector bites — the other direction of the same discipline `make
/// rules-audit` runs on its fixtures. A test that can only ever pass proves
/// nothing, and the walk above is the whole evidence for the one above it, so
/// it is shown a frame that really does paint two runs on one seat.
#[test]
fn the_walk_reports_a_frame_that_really_does_stack_two_runs() {
    let ctx = egui::Context::default();
    let out = ctx.run(input(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let seat =
                egui::Rect::from_min_size(egui::Pos2::new(40.0, 40.0), egui::vec2(200.0, 20.0));
            for text in ["under", "over"] {
                ui.put(seat, egui::Label::new(text));
            }
        });
    });
    let found = overlaps(&visible(&out));
    assert!(
        found
            .iter()
            .any(|pair| pair.contains("under") && pair.contains("over")),
        "two labels on one rect must be reported: {found:?}"
    );
}
