//! The §11 fold affordance: an expanded row shows its whole body, a
//! contracted one shows exactly one line, a row with nothing to fold shows no
//! toggle at all, and the toggle's own click — the jsonview widget-split
//! exception (§11) — flips the caller's override set under a simulated pointer.

use super::render::{entry, input, painted_with, rendered_text, tx};
use crate::transcript::{AutoExpand, Block, EntryKind, Transcript, Usage};
use std::collections::HashSet;

/// A model reply whose payload is two lines — the auto-expanded case.
fn two_line_reply() -> Transcript {
    tx(vec![entry(EntryKind::Model {
        model_id: "opus".into(),
        usage: Usage::default(),
        blocks: vec![Block::Text("first line\nsecond line".into())],
    })])
}

#[test]
fn an_expanded_row_shows_its_whole_body_and_a_contracted_one_shows_one_line() {
    let t = two_line_reply();
    let open = rendered_text(&t, false);
    assert!(
        open.contains("▼"),
        "an expanded row shows the open fold:\n{open}"
    );
    assert!(open.contains("second line"), "body painted:\n{open}");
    // Contracting the class (the knob) leaves only the first line and ▶.
    let shut = painted_with(
        &t,
        false,
        AutoExpand {
            responses: false,
            others: false,
        },
        &mut HashSet::new(),
    );
    assert!(
        shut.contains("▶"),
        "a contracted row shows the shut fold:\n{shut}"
    );
    assert!(shut.contains("first line"));
    assert!(!shut.contains("second line"), "one line only:\n{shut}");
}

#[test]
fn a_row_with_nothing_to_fold_shows_the_alignment_mark_instead_of_a_toggle() {
    let t = tx(vec![entry(EntryKind::Model {
        model_id: "opus".into(),
        usage: Usage::default(),
        blocks: vec![Block::Text("all of it".into())],
    })]);
    let painted = rendered_text(&t, false);
    assert!(painted.contains('·'), "leaf mark painted:\n{painted}");
    assert!(!painted.contains('▼') && !painted.contains('▶'));
}

#[test]
fn clicking_a_rows_toggle_flips_it_and_clicking_again_restores_it() {
    let t = two_line_reply();
    let mut folds = HashSet::new();
    click_first_toggle(&t, &mut folds);
    assert!(
        folds.contains("tx/001-x.json#0"),
        "the click records the override: {folds:?}"
    );
    click_first_toggle(&t, &mut folds);
    assert!(folds.is_empty(), "the second click clears it: {folds:?}");
}

/// Two-frame pointer click on the first row's fold toggle (frame one lays the
/// widget out, frame two delivers the press/release) — the jsonview pattern.
fn click_first_toggle(t: &Transcript, folds: &mut HashSet<String>) {
    let ctx = egui::Context::default();
    let auto = AutoExpand::default();
    let run = |input: egui::RawInput, folds: &mut HashSet<String>| {
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                super::plain(ui, t, false, auto, folds);
            });
        });
    };
    // Two priming frames, so the §11 tail idiom's one-frame settle (the top
    // pad seating content on the bottom edge) has landed and the click frame
    // hit-tests the settled layout.
    run(input(), folds);
    run(input(), folds);
    // Aim at the fold glyph where the settled frame actually painted it — the
    // row's first widget is the §11 role stripe seat, so a hardcoded top-left
    // corner would press the stripe, not the toggle.
    let painted = crate::paint_probe::painted_settled(1024.0, 4096.0, |ui| {
        super::plain(ui, t, false, auto, &mut folds.clone());
    });
    let pos = painted
        .iter()
        .find(|(text, _)| text == "▼" || text == "▶")
        .map(|(_, rect)| rect.center())
        .expect("a foldable row paints its toggle");
    let button = |pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::default(),
    };
    let click = egui::RawInput {
        events: vec![egui::Event::PointerMoved(pos), button(true), button(false)],
        ..input()
    };
    run(click, folds);
}
