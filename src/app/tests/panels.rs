//! The §4.1 `panels` sizes on the model: the opening fold (stored, else the
//! default, never below the floor), the one-write-per-gesture settle rule, and
//! the regression bl-9ad4 was filed for — a dragged boundary is still where the
//! operator left it at the next launch.

use super::Harness;
use crate::ui_state::Panel;

/// `f32` equality without the float-compare trap: sizes are snapped to a whole
/// point on the way to disk, so anything closer than half a point is the same
/// boundary.
fn is(size: f32, want: f32) -> bool {
    (size - want).abs() < 0.5
}

/// The window extent every size here is folded against — wide enough that the
/// §11 ceiling (half the window) is not what these tests are measuring, except
/// where one says so.
const WINDOW: f32 = 1600.0;

#[test]
fn an_undragged_panel_opens_at_its_default() {
    let h = Harness::new();
    let (_c, model) = h.model();
    for panel in [Panel::Conversations, Panel::ActivityTrail, Panel::StartGoal] {
        assert!(
            is(model.panel_size(panel, WINDOW), panel.default_size()),
            "{panel:?}"
        );
    }
}

/// The regression: drag a boundary, quit, relaunch — the panel is where it was
/// left. Two `Harness::model()` builds over one XDG root are two launches over
/// one `ui.json`.
#[test]
fn a_dragged_boundary_survives_a_relaunch() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    model.settle_panel_size(Panel::Conversations, 412.0, WINDOW, true);
    model.settle_panel_size(Panel::ActivityTrail, 333.0, WINDOW, true);
    drop(model);

    let (_c2, relaunched) = h.model();
    assert!(is(
        relaunched.panel_size(Panel::Conversations, WINDOW),
        412.0
    ));
    assert!(is(
        relaunched.panel_size(Panel::ActivityTrail, WINDOW),
        333.0
    ));
    assert!(
        is(
            relaunched.panel_size(Panel::StartGoal, WINDOW),
            Panel::StartGoal.default_size()
        ),
        "an untouched boundary keeps its default"
    );
}

/// One drag is one write: nothing lands while the pointer is still down, and a
/// boundary that has not moved lands nothing at all — the frame loop reports
/// the same size 60 times a second, and none of those are gestures.
#[test]
fn only_a_settled_move_writes() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    let opened = model.panel_size(Panel::Conversations, WINDOW);

    model.settle_panel_size(Panel::Conversations, 500.0, WINDOW, false);
    assert!(
        is(model.panel_size(Panel::Conversations, WINDOW), opened),
        "mid-drag"
    );

    // A sub-point wobble at rest is not a move either.
    model.settle_panel_size(Panel::Conversations, opened + 0.4, WINDOW, true);
    assert!(
        is(model.panel_size(Panel::Conversations, WINDOW), opened),
        "wobble"
    );

    model.settle_panel_size(Panel::Conversations, 500.0, WINDOW, true);
    assert!(
        is(model.panel_size(Panel::Conversations, WINDOW), 500.0),
        "released"
    );
}

/// The ceiling holds on both sides of the same door (§11, bl-ac3d). A width
/// that reached the document while the window was wide — or that a runaway row
/// ratcheted there — opens folded into half of *this* window, so the next
/// launch on a small screen is never handed an unusable centre; and a settle
/// above the ceiling can never store more than the ceiling.
#[test]
fn a_too_wide_stored_width_recovers_at_the_next_launch() {
    let h = Harness::new();
    let ui = h.roots.ui_json();
    // A panel size is a PANE fact (REMOTE §7, bl-8bbc): the local window's own
    // document, not the shared `ui.json`.
    let pane = crate::registry::pane(ui.parent().unwrap(), &crate::registry::window());
    std::fs::create_dir_all(pane.parent().unwrap()).unwrap();
    std::fs::write(&pane, br#"{"panels":{"conversations":690}}"#).unwrap();
    let (_c, mut model) = h.model();
    // An 800 pt window: 690 would leave ~110 pt of centre — the filed defect.
    assert!(
        is(model.panel_size(Panel::Conversations, 800.0), 400.0),
        "clamped on read"
    );
    // Wide again, and the operator's own width is untouched — the clamp is a
    // ceiling on what the window can show, not an edit of what was stored.
    assert!(is(model.panel_size(Panel::Conversations, 1600.0), 690.0));

    // What the small window shows is not a new fact about the boundary, so
    // nothing is written: the operator's 690 survives the visit.
    model.settle_panel_size(Panel::Conversations, 3000.0, 800.0, true);
    let stored = crate::ui_state::UiState::open(ui).panel_size(Panel::Conversations);
    assert_eq!(stored, Some(690.0), "the stored width stays the operator's");
}

/// The settle reports the panel's *content* rect, so a row that overflows
/// reports a width nobody dragged (the bl-9669 mechanism, and the row that
/// escaped it in bl-ac3d). The ceiling bounds what that can cost: half the
/// window, never the runaway itself.
#[test]
fn a_runaway_row_can_store_no_more_than_the_ceiling() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    model.settle_panel_size(Panel::Conversations, 3000.0, 800.0, true);
    assert!(is(model.panel_size(Panel::Conversations, 800.0), 400.0));
}

/// The floor holds on both sides of the door: a drag past it, and a `ui.json`
/// hand-edited below it, both open a panel that still has a boundary to grab.
#[test]
fn the_floor_holds_against_a_drag_and_a_hand_edit() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    let floor = Panel::ActivityTrail.min_size();
    model.settle_panel_size(Panel::ActivityTrail, 0.0, WINDOW, true);
    assert!(
        is(model.panel_size(Panel::ActivityTrail, WINDOW), floor),
        "dragged"
    );

    let ui = h.roots.ui_json();
    std::fs::write(&ui, br#"{"panels":{"activity_trail":-40}}"#).unwrap();
    let (_c2, reopened) = h.model();
    assert!(
        is(reopened.panel_size(Panel::ActivityTrail, WINDOW), floor),
        "edited"
    );
}
