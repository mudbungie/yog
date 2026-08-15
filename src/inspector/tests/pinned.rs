//! **S10-T6 pinned-inspector**: the pin reaches every pinnable tab, says what
//! it is showing above whichever one is open, and — since bl-1802 — carries
//! the way back on the banner itself.
//!
//! The pin's *effects* are folded before [`crate::inspector::render`] ever
//! sees them (`shell::inspector::rail`), so what this half proves is the seam:
//! the banner states the commit and the budget as of it on every pinnable tab,
//! and clicking it releases the pin. The gesture that *raises* a pin lives in
//! the chat and is proved there (`transcript::tests::spine`); this file proves
//! the operator who pinned and then opened Files is not stranded.

use super::{empty_tab_data, paint};
use crate::files_view::FilesView;
use crate::inspector::{Ephemera, render};
use crate::keymap::InspectorTab;
use crate::nav::convs::Titles;
use crate::rail::{Notch, Place, Rail, pin};
use crate::steps_view::StepsView;
use crate::transcript::Transcript;

/// A two-notch spine — the shape `rail::build` yields for two settled steps,
/// stated directly here because what this half tests is the seam, not the
/// build.
fn rail() -> Rail {
    let notch = |seq: &str, oid: &str, budget, cut| Notch {
        seq: seq.to_owned(),
        commit: Some(oid.to_owned()),
        budget,
        place: Some(Place {
            row: format!("tx/{seq}-user.md#0"),
            cut,
        }),
    };
    Rail {
        notches: vec![
            // The notch's own figure is the ROLLUP as of it (bl-44e9): 5 spent
            // by the first, 12 by the second — so a pin selects rather than
            // sums.
            notch("001", "aaaa1111", 5, 1),
            notch("002", "bbbb2222", 12, 3),
        ],
        cards: vec![],
    }
}

fn data(rail: Rail, notch: Option<usize>) -> crate::inspector::TabData {
    let pin = pin(&rail, notch);
    let mut data = empty_tab_data(
        Transcript::default(),
        StepsView::default(),
        Vec::new(),
        FilesView::default(),
        None,
    );
    data.rail = rail;
    data.pin = pin;
    data
}

/// A pin says what it is showing and what had been spent by then, above every
/// tab the pin reaches — and stays silent on the one it does not (the Work tab
/// reads the project repo, which no conversation commit indexes). No pin says
/// nothing at all.
#[test]
fn a_pin_states_the_commit_and_the_budget_as_of_it() {
    let pinned = data(rail(), Some(1));
    for tab in InspectorTab::all() {
        let painted = paint(tab, &pinned);
        assert_eq!(
            painted.contains("as of bbbb222"),
            tab.pinnable(),
            "{tab:?}:\n{painted}"
        );
        assert_eq!(painted.contains("12 tokens"), tab.pinnable(), "{tab:?}");
    }
    let unpinned = paint(InspectorTab::Files, &data(rail(), None));
    assert!(!unpinned.contains("as of"), "unpinned banner:\n{unpinned}");
}

/// **The release lives on the banner.** The rule that raised the pin is in the
/// Transcript tab, so an operator who pinned and then opened Files would have
/// no mark to click again — the banner they are already reading is the way
/// back, and it says so.
#[test]
fn clicking_the_banner_releases_the_pin_from_any_pinnable_tab() {
    let built = data(rail(), Some(1));
    assert!(
        paint(InspectorTab::Files, &built).contains("Click here to come back"),
        "the banner must offer the release it is"
    );
    let ctx = egui::Context::default();
    let mut eph = Ephemera {
        notch_sel: Some(1),
        ..Ephemera::default()
    };
    let run = |input: egui::RawInput, eph: &mut Ephemera| {
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let _ = render(ui, InspectorTab::Files, &built, &Titles::default(), eph);
            });
        });
    };
    run(crate::paint_probe::screen(), &mut eph);
    run(crate::paint_probe::screen(), &mut eph);
    let mut probe = Ephemera {
        notch_sel: Some(1),
        ..Ephemera::default()
    };
    let painted = crate::paint_probe::painted_settled(1024.0, 4096.0, |ui| {
        let _ = render(
            ui,
            InspectorTab::Files,
            &built,
            &Titles::default(),
            &mut probe,
        );
    });
    let pos = painted
        .iter()
        .find(|(text, _)| text.contains("as of bbbb222"))
        .map(|(_, rect)| rect.center())
        .expect("the banner is on screen");
    let button = |pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::default(),
    };
    run(
        egui::RawInput {
            events: vec![egui::Event::PointerMoved(pos), button(true), button(false)],
            ..crate::paint_probe::screen()
        },
        &mut eph,
    );
    assert_eq!(eph.notch_sel, None, "the banner is the release");
}
