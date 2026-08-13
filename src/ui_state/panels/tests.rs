//! The `panels` object's table: the forgiving read (absent, wrong-typed value,
//! wrong-typed container), the point snap, and that the three panels are three
//! independent slots which round-trip through the file.

use super::super::tests::{load, mk};
use super::*;
use tempfile::tempdir;

/// Every panel, so a new variant cannot be added without a test seeing it.
const ALL: [Panel; 3] = [Panel::Conversations, Panel::ActivityTrail, Panel::StartGoal];

#[test]
fn an_undragged_boundary_has_no_stored_size() {
    let d = tempdir().unwrap();
    let ui = mk(d.path());
    for panel in ALL {
        assert!(ui.panel_size(panel).is_none(), "{panel:?}");
        assert!(panel.min_size() < panel.default_size(), "{panel:?} floor");
    }
}

#[test]
fn a_dragged_size_snaps_to_a_whole_point_and_round_trips_through_the_file() {
    let d = tempdir().unwrap();
    let mut ui = mk(d.path());
    ui.set_panel_size(Panel::Conversations, 317.4);
    assert_eq!(ui.panel_size(Panel::Conversations), Some(317.0));
    // A second launch over the same file reads the same size back.
    let reopened = UiState::open(d.path().join("ui.json"));
    assert_eq!(reopened.panel_size(Panel::Conversations), Some(317.0));
    assert!(
        reopened.panel_size(Panel::ActivityTrail).is_none(),
        "only its own slot"
    );
}

#[test]
fn each_panel_is_its_own_slot() {
    let d = tempdir().unwrap();
    let mut ui = mk(d.path());
    for (panel, size) in ALL.into_iter().zip([100.0, 200.0, 300.0]) {
        ui.set_panel_size(panel, size);
    }
    for (panel, size) in ALL.into_iter().zip([100.0, 200.0, 300.0]) {
        assert_eq!(ui.panel_size(panel), Some(size), "{panel:?}");
    }
}

/// The ceiling is a share of the window, the floor is points, and every panel
/// obeys both (§11, bl-ac3d). A size above the ceiling folds down to it, one
/// below the floor folds up — and at a window too small to hold twice a floor,
/// the floor still wins, so a boundary is always grabbable.
#[test]
fn the_clamp_holds_a_panel_between_its_floor_and_half_the_window() {
    // Sizes are points, so anything closer than half of one is the same size.
    let is = |size: f32, want: f32| (size - want).abs() < 0.5;
    for panel in ALL {
        assert!(is(panel.max_size(1000.0), 500.0), "{panel:?} ceiling");
        assert!(is(panel.clamp(900.0, 1000.0), 500.0), "{panel:?} over");
        assert!(
            is(panel.clamp(-40.0, 1000.0), panel.min_size()),
            "{panel:?} under"
        );
        assert!(is(panel.clamp(300.0, 1000.0), 300.0), "{panel:?} within");
        // A window narrower than two floors: the floor outranks the share.
        let tiny = panel.min_size();
        assert!(is(panel.clamp(tiny, tiny), tiny), "{panel:?} tiny window");
    }
}

#[test]
fn a_hand_edited_document_never_errors() {
    // A `panels` that is not an object, and a member that is not a number:
    // both read as never-dragged (the forgiving read), and a write coerces the
    // container rather than refusing.
    for doc in [
        br#"{"panels":7}"#.as_slice(),
        br#"{"panels":{"conversations":"wide"}}"#,
    ] {
        let mut ui = load(doc);
        assert!(ui.panel_size(Panel::Conversations).is_none());
        ui.set_panel_size(Panel::Conversations, 300.0);
        assert_eq!(ui.panel_size(Panel::Conversations), Some(300.0));
    }
}
