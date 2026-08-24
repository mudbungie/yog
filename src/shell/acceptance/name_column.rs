//! Row geometry (§11, bl-b9e3): inside the conversation panel the name column
//! is a **column** — the title's left edge is the row's fixed prefix and
//! nothing else. Split from `geometry`, which is the same discipline one
//! altitude out (the panel's width against its content).
//!
//! **The conditional-mark half is [`marks`]** (bl-8257), split off at §12's
//! budget on the seam the two beats already had: this file holds the attention
//! flag to the rule over a derived world through the whole window, that one
//! holds every conditional mark to it over rows built by hand.

/// Every conditional prefix element, one per row, painted through the real
/// `conv_row` — the completion of the rule below.
mod marks;

use super::super::{now_unix, render};
use super::fixture::world_titled;
use super::input;
use crate::cli_outbound::Cli;

/// The name column is a column (§11, bl-b9e3). The operator's complaint about
/// the row's old `⚑N` was **alignment**, not the glyph — *"it makes the list
/// not align"* — because the flag was painted in the row's left prefix, and
/// every conditional element there moves the title's left edge on exactly the
/// rows that have it.
///
/// Asserted on **geometry**, not on the painted string: the glyph could be
/// re-spelled and the column still break, and it could be deleted outright and
/// a string test still pass. Two roots in one list, differing only in
/// attention — the fixture's `c-1` bears undismissable mail (§6 rule 5), the
/// second is marked `abandoned`, which is the one gate that suppresses rule 2 —
/// and their titles must land on the same x. Both halves of the fixture are
/// asserted first, so a world where neither row (or both) bears attention
/// fails here rather than passing vacuously.
///
/// This is the two-fixture form. A beat that fails the *day a new conditional
/// prefix element appears* would have to enumerate the prefix, which is a knob
/// or a source scan. So the flag's own seat is pinned here, and the third
/// assertion — that the flag paints to the RIGHT of every title — is what keeps
/// the alignment claim honest. The three elements that outlived bl-b9e3 in the
/// prefix are ruled on by bl-8257 and pinned in the beat below.
#[test]
fn the_titles_left_edge_is_the_same_on_a_flagged_row_and_a_quiet_one() {
    let (lernie, bl, bz) = (Cli::new("lernie"), Cli::new("bl"), Cli::new("bz"));
    let mut world = world_titled("hello");
    // Zero the watcher debounce so the second root derives on the very next
    // pass instead of on a wall clock this test would have to sleep against
    // (the same seam `super::walk` opens for its child).
    std::fs::write(
        world.model.state_root().join("cadence.yaml"),
        "cadence:\n  watcher:\n    debounce_ms: 0\n",
    )
    .unwrap();
    world.model.after_lernie_verb();
    world.converge();
    world.add_root("c-2", "quiet-root");
    world.quiet("c-2");
    let ws = world.ws.clone();
    world.model.mark_dirty([ws]);
    world.converge();
    let ctx = egui::Context::default();
    // Three frames: a panel is its default size on the frame it first appears,
    // so the settled third is the one the operator sees (as `super::painted`).
    let painted = {
        let frame = |world: &mut super::fixture::World| {
            ctx.run(input(), |ctx| {
                render(ctx, &mut world.model, &mut world.state, &lernie, &bl, &bz);
            })
        };
        // The wire settled between frames (bl-44e9): the column is painted off
        // a `Reply::Conversations`, which lands a round trip later.
        let _ = frame(&mut world);
        world.settle();
        let _ = frame(&mut world);
        world.settle();
        crate::paint_probe::painted_of(&frame(&mut world))
    };
    let rows = crate::test_support::convs::conversations(&world.model, now_unix());

    // The fixture is honest: two rows, exactly one of them flagged.
    assert_eq!(rows.len(), 2, "two roots must reach the list: {rows:?}");
    let flagged = rows.iter().filter(|r| r.attention > 0).count();
    assert_eq!(flagged, 1, "exactly one row must bear attention: {rows:?}");

    // The leftmost galley of each title is the conversation column's copy —
    // the panel is the window's left column, and the centre repaints the open
    // conversation's name further right.
    let leftmost = |needle: &str| {
        painted
            .iter()
            .filter(|(text, _)| text == needle)
            .map(|(_, rect)| rect.min.x)
            .fold(f32::INFINITY, f32::min)
    };
    let mut edges = Vec::new();
    for row in &rows {
        let name = row.display_name();
        let left = leftmost(&name);
        assert!(left.is_finite(), "row {name:?} must paint its title");
        edges.push((name, row.attention, left));
    }
    let (_, _, first) = edges[0];
    for (name, attention, left) in &edges {
        assert!(
            (left - first).abs() < 0.5,
            "the title's left edge is the prefix's, not attention's: \
             {edges:?} — {name:?} (attention {attention}) sits at {left}, not {first}"
        );
    }

    // And the flag really is painted, to the right of every title — without
    // this the equality above would also hold on a tree that simply deleted it.
    let flag = leftmost("⚑");
    assert!(flag.is_finite(), "the flagged row must paint its ⚑");
    assert!(
        flag > first,
        "the flag rides the trailing group, right of the title: {flag} <= {first}"
    );
}
