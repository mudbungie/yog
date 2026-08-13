//! The §11 transcript-density knobs on the model: the operator's defaults, and
//! that each setter moves exactly its own knob through `ui.json` (§4.1) — plus
//! the whole-UI zoom, whose whole point is surviving the process.

use super::Harness;
use crate::keymap::ZoomStep;

/// `f32` equality without the float-compare trap: every value here is snapped
/// to a hundredth on the way to disk, so anything closer than half a step is
/// the same size.
fn is(zoom: f32, want: f32) -> bool {
    (zoom - want).abs() < 0.001
}

#[test]
fn transcript_auto_expand_defaults_to_replies_open_and_the_rest_folded() {
    let h = Harness::new();
    let (_c, model) = h.model();
    let auto = model.transcript_auto_expand();
    assert!(auto.responses, "actual responses to the user auto-expand");
    assert!(!auto.others, "everything else auto-contracts");
}

#[test]
fn each_knob_moves_only_itself() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    model.set_transcript_expand_responses(false);
    assert!(!model.transcript_auto_expand().responses);
    assert!(!model.transcript_auto_expand().others, "untouched");
    model.set_transcript_expand_others(true);
    assert!(model.transcript_auto_expand().others);
    assert!(!model.transcript_auto_expand().responses, "untouched");
}

/// The regression bl-42e7 was filed for: the text size an operator sets is
/// still the text size when yog is launched again. Two `Harness::model()`
/// builds over one XDG root are two launches over one `ui.json`.
#[test]
fn text_size_survives_a_relaunch() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    assert!(is(model.zoom(), 1.0), "an unwritten ui.json opens at 1.0");
    model.zoom_nudge(ZoomStep::In);
    model.zoom_nudge(ZoomStep::In);
    assert!(is(model.zoom(), 1.2), "two steps in");
    drop(model);

    let (_c2, relaunched) = h.model();
    assert!(is(relaunched.zoom(), 1.2), "reopened at the same size");
}

#[test]
fn zoom_steps_stay_inside_the_egui_domain_and_reset_to_one() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    for _ in 0..60 {
        model.zoom_nudge(ZoomStep::In);
    }
    assert!(is(model.zoom(), 5.0), "clamped at the ceiling");
    for _ in 0..60 {
        model.zoom_nudge(ZoomStep::Out);
    }
    assert!(is(model.zoom(), 0.2), "clamped at the floor");
    model.zoom_nudge(ZoomStep::Reset);
    assert!(is(model.zoom(), 1.0), "reset is 1.0, whatever it was");
}
