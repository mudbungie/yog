//! **Where the §11 fold line sits** (bl-929d) — the geometry half of
//! [`super`]'s drive, split off at §12's budget. The line's position IS the
//! queue's content height: floored at the bare input row (the general path with
//! zero items, never a case), capped at half the pane, and eased back down by
//! the structural snap a delivery fires. Driven on the real window with an
//! explicit clock, because the snap is time-eased render ephemera.

use super::super::fixture::world;
use super::super::input;
use super::{Frames, converge_ws, deposit, drain, quick};

/// The fold line's position IS the content height: the empty inbox is the
/// bare input row (the general path, zero items), each landing item pushes the
/// line up, and past half the pane the line stops — more items scroll instead
/// of climbing.
#[test]
fn the_fold_line_is_the_content_height_with_floor_and_cap() {
    let mut world = quick(world());
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    let frames = Frames::new();

    // Zero items: the bare input row.
    drain(&world);
    converge_ws(&mut world);
    frames.settle(&mut world, 0.0);
    let bare = frames.panel("composer").height();

    // One item raises the line; a second raises it again.
    deposit(&world, "user-001.md", "t0", "one");
    converge_ws(&mut world);
    frames.settle(&mut world, 1.0);
    let one = frames.panel("composer").height();
    deposit(&world, "user-002.md", "t1", "two");
    converge_ws(&mut world);
    frames.settle(&mut world, 2.0);
    let two = frames.panel("composer").height();
    assert!(one > bare + 4.0, "an item raises the line: {bare} → {one}");
    assert!(two > one + 4.0, "and the next again: {one} → {two}");

    // The cap: a flood of items stops the line at half the pane — the
    // sixty-deposit queue and the eighty-deposit queue sit at the same
    // boundary, and the extra rows scroll behind it.
    for i in 3..=60 {
        deposit(&world, &format!("user-{i:03}.md"), "t", "pile");
    }
    converge_ws(&mut world);
    frames.settle(&mut world, 3.0);
    let sixty = frames.panel("composer").height();
    for i in 61..=80 {
        deposit(&world, &format!("user-{i:03}.md"), "t", "pile");
    }
    converge_ws(&mut world);
    frames.settle(&mut world, 4.0);
    let eighty = frames.panel("composer").height();
    assert!(
        sixty > two,
        "the line kept climbing to the cap: {two} → {sixty}"
    );
    assert!(
        (eighty - sixty).abs() < 2.0,
        "past the cap the line holds still: {sixty} → {eighty}"
    );
    let window = input().screen_rect.expect("the probe sizes the screen");
    assert!(
        sixty < window.height() / 2.0 + 60.0,
        "the cap is half the pane: {sixty}"
    );
}

/// The snap-down is triggered structurally — the pending count dropping on
/// delivery — and eases the line from its pre-drain height to the bare row;
/// no gesture is consulted, so a drain by driver, scan or another instance
/// snaps identically.
#[test]
fn a_delivery_drain_snaps_the_line_down_to_its_floor() {
    let mut world = quick(world());
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    deposit(&world, "user-002.md", "t1", "two");
    deposit(&world, "user-003.md", "t2", "three");
    converge_ws(&mut world);
    let frames = Frames::new();
    frames.settle(&mut world, 0.0);
    let full = frames.panel("composer").height();

    // The delivery commit lands: the inbox empties with no yog gesture.
    drain(&world);
    converge_ws(&mut world);
    // The drop is observed at t=5.0; mid-ease the line sits between the two
    // heights, and past the ease it settles on the bare row.
    frames.run(&mut world, 5.0);
    frames.run(&mut world, 5.0 + crate::composer::SNAP_SECS * 0.3);
    let mid = frames.panel("composer").height();
    frames.settle(&mut world, 6.0);
    let bare = frames.panel("composer").height();
    assert!(bare < full - 8.0, "the queue emptied: {full} → {bare}");
    assert!(
        mid > bare + 2.0,
        "mid-ease the line is still descending: {mid} !> {bare}"
    );
    assert!(mid < full + 2.0, "and never above where it started: {mid}");
}
