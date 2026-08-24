//! QUALITY §1 **G1** (*"Nothing clipped … deliberate elision shows an ellipsis
//! and the full value is reachable"*), pinned on the paint layer at the same
//! window sizes its sibling [`super::overlap`] pins G4 at (bl-36c3).
//!
//! The two are complementary and neither subsumes the other: a row can keep
//! every label whole and still stack two of them on one seat (disjointness), and
//! it can keep them apart and still slice one in half (this file). bl-bc06's
//! first shape passed the first and failed the second the other way round —
//! pinning a control right while the text beside it was free to extend turned an
//! elision defect into an overlap — which is why both walk the same frame.
//!
//! **The predicate.** [`crate::paint_probe::seen_of`] hands back, per galley,
//! the glyphs, the rect they were laid into and the part of that rect the clip
//! let through. A run whose shown width is narrower than its laid width is a run
//! the operator is reading the front of and nothing else — and because the
//! galley itself was never truncated, egui never added the `…` that would say
//! so. That is G1's defect exactly: not that text was cut, but that it was cut
//! **silently**. A galley egui truncates on purpose is not reported here at all;
//! it lays out to fit its box, so its shown width equals its laid width and the
//! ellipsis it ends in is on the glass.
//!
//! **The known list is empty, and that is the claim** (bl-5410). It was seven
//! entries when the predicate was first run — the audit's own G1 findings, all
//! filed on one ball — and the assertion was written one-directional so it
//! could redden on a *new* surface while the filed ones were repaired. They
//! have been: rule 1 is now stated at every bounded panel rather than at the
//! side panel alone, and the two rows that carried prose in a trailing slot put
//! it on a wrapped line of its own. So this reads as an absolute invariant
//! today — **no run in the whole window is cut without saying so, at any of the
//! four sizes** — and the list stays only as the mechanism a future finding is
//! filed through, never as an excuse standing on its own. An entry whose defect
//! has been fixed is stale, not wrong: delete it when its ball closes, which is
//! what emptied it.
//!
//! **§11 rule 8's all-or-none is [`strip`]**, split off at §12's budget: this
//! file asks whether a run reached the glass whole, that one whether a set of
//! controls reached it together.

use super::window;
use crate::keymap::{CenterTab, InspectorTab};
use crate::paint_probe::seen_of;

/// The silent cuts the shipped frame paints today, each with the ball that owns
/// it. A needle is matched as a substring of the run's glyphs, not as an
/// equality: the point of the entry is *which surface*, and pinning the whole
/// string would make the list rot on a re-wording rather than on a repair.
///
/// **Empty since bl-5410.** What stood here was, in order: `Stop` (a control,
/// 16 of its 25 points, 420x320), `Declare` (a control, at 800x500), the
/// altitude-1 headline's two chips, the §8.3 `auth …` fact in both of its
/// seats, the Login pane's blocked reason, and the brazen pane's built-in-rows
/// note. Every one of them is now whole or marked, so the sweeps below assert
/// the property outright.
const KNOWN: [(&str, &str); 0] = [];

/// Every run one settled frame cut without saying so, each already stamped with
/// the seat it was found on.
fn silent_cuts(
    w: f32,
    h: f32,
    trail: bool,
    tab: CenterTab,
    inspector: InspectorTab,
) -> Vec<String> {
    seen_of(&window(w, h, trail, tab, inspector))
        .into_iter()
        .filter(|seen| seen.shown.width() < seen.laid.width() - 1.0)
        .map(|seen| {
            format!(
                "{:?} laid {:.0} pt, {:.0} pt shown — at {w}x{h} \
                 (trail {trail}, {tab:?}, {inspector:?})",
                seen.text,
                seen.laid.width(),
                seen.shown.width()
            )
        })
        .filter(|cut| !KNOWN.iter().any(|(needle, _)| cut.contains(needle)))
        .collect()
}

/// Fail on anything the sweep found that no ball owns, and say what IS owned —
/// a red gate here is most often someone else's defect drifting into a new
/// seat, and the list is the difference between reading that and re-finding it.
/// With [`KNOWN`] empty the second half reads "nothing", which is the state the
/// gate is meant to be kept in.
fn assert_all_filed(report: &[String]) {
    let excused: Vec<String> = KNOWN
        .iter()
        .map(|(needle, ball)| format!("  {needle:?} — {ball}"))
        .collect();
    assert!(
        report.is_empty(),
        "runs cut with no ellipsis, on no filed ball:\n{}\n\nalready filed:\n{}",
        report.join("\n"),
        if excused.is_empty() {
            "  (nothing — the list is empty and must stay that way)".to_owned()
        } else {
            excused.join("\n")
        }
    );
}

/// **No run is cut without saying so** (G1), over the whole shipped frame at
/// every size, with the activity trail both at rest and open, and on every
/// centre tab — the four surfaces an operator can put in the middle of the
/// window, since a defect that lives on one of them is invisible from the
/// others.
#[test]
fn nothing_is_cut_without_an_ellipsis_at_any_window_size() {
    let mut report = Vec::new();
    for (w, h) in super::SIZES {
        for trail in [false, true] {
            for tab in [
                CenterTab::Conversation,
                CenterTab::Config,
                CenterTab::Login,
                CenterTab::Search,
            ] {
                report.extend(silent_cuts(w, h, trail, tab, InspectorTab::Transcript));
            }
        }
    }
    assert_all_filed(&report);
}

/// The same property over the altitude-2 inspector's own tabs, which the sweep
/// above holds at `Transcript` — one render each rather than the full cross
/// product, because the inspector's content is what changes and the window
/// around it is not.
#[test]
fn nothing_is_cut_without_an_ellipsis_on_any_inspector_tab() {
    let mut report = Vec::new();
    for (w, h) in super::SIZES {
        for inspector in InspectorTab::all() {
            report.extend(silent_cuts(w, h, false, CenterTab::Conversation, inspector));
        }
    }
    assert_all_filed(&report);
}

/// The detector bites. A run laid wider than the box it sits in is exactly what
/// the sweeps above are the whole evidence for, so it is shown one: a label with
/// egui's default `Extend` wrap mode in a panel too narrow for it — which is not
/// a contrivance but the shipped centre's own condition (§11 rule 1 sets
/// `Truncate` at the *side* panel's root and nowhere else).
#[test]
fn the_walk_reports_a_run_its_seat_really_does_slice() {
    let ctx = egui::Context::default();
    let out = ctx.run(crate::paint_probe::screen_sized(120.0, 200.0), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Laid horizontally: egui wraps a label in a vertical layout, and a
            // wrapped run is never cut. The shipped rows this stands for — a
            // headline's chips, a provider row's fact — are horizontal for the
            // same reason, which is why they are the ones that get sliced.
            ui.horizontal(|ui| {
                ui.label("a run far wider than the hundred points it was given");
            });
        });
    });
    let cut: Vec<_> = seen_of(&out)
        .into_iter()
        .filter(|seen| seen.shown.width() < seen.laid.width() - 1.0)
        .collect();
    assert!(
        cut.iter().any(|seen| seen.text.starts_with("a run far")
            && !seen.text.ends_with('…')
            && seen.shown.width() < 130.0),
        "a run sliced by its seat, with no ellipsis to say so, must be reported: {:?}",
        cut.iter().map(|s| &s.text).collect::<Vec<_>>()
    );
}

mod floor;
/// The peer-strip half of QUALITY §1 on the same frame — §11 rule 8's
/// all-or-none, which no predicate over painted runs can see.
mod strip;
