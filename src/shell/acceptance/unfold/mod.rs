//! The §11 **unfold** at the paint layer (bl-fa82): the subagent field's arrow
//! and its two numbers, and the indented reply-elbow rows an expanded row
//! reveals.
//!
//! Asserted on **geometry as well as text**, in `name_column`'s discipline and
//! for its reason: the claim this ball makes is *where* a child row sits, and a
//! string assertion passes on a tree that paints the child flush against its
//! parent — which is the whole defect it would exist to catch. So the title's
//! left edge is measured per depth, and the field is measured against the title
//! it is pinned to the right of.
//!
//! Two directions everywhere, because a fold has two states and only asserting
//! the open one would pass on a list that could not close: the collapsed frame
//! must paint **no** elbow and **no** child name, the open one must paint both.
//!
//! **The gesture half is [`drive`], [`keys`] and [`column`]** (bl-89de), split
//! off here rather than driven from `super::walk`: those beats and these share
//! one three-generation fixture, and a fold has no meaning apart from the rows
//! it reveals. The seam between the children is the hand — [`drive`] is what
//! the *pointer* does to the fold (and the words the field says under it),
//! [`keys`] what the *keyboard* does (`→` unfolds, `↓` skips what is folded,
//! `←` pages up and then shuts), and [`column`] where the revealed rows land.

mod column;
mod drive;
mod keys;

use super::super::render;
use super::fixture::{World, world};
use super::input;
use crate::cli_outbound::Cli;
use crate::paint_probe::Painted;

/// A nameless descent child of `c-1` (the bl-63a1 chained-id shape), and its
/// own child — three generations, so `direct` and `total` differ on the root
/// and the middle row has a field of its own.
const CHILD: &str = "c-1-20260803T045643Z-1e5f99d4";
const GRANDCHILD: &str = "c-1-20260803T045643Z-1e5f99d4-20260803T045700Z-2a6b88c5";

/// A world holding one conversation three generations deep, converged.
///
/// **Nothing is focused.** Written against the altitude-1 descent tree, which
/// repainted every member name in the centre of any selected conversation and
/// so let a centre galley satisfy an assertion about the left column — the
/// "wrong source" vacuity. bl-8905 retired that tree, so the centre no longer
/// paints a *member* name at all; the fixture stays unfocused because the
/// header still paints the open conversation's own title, which is the root
/// row's needle, and because a beat about the list should not depend on which
/// conversation is open.
fn nested_world() -> World {
    let mut world = world();
    // Debounce off (a legal cadence — the bounds floor at zero) so the children
    // derive on the very next pass rather than on a wall clock to sleep against.
    std::fs::write(
        world.model.state_root().join("cadence.yaml"),
        "cadence:\n  watcher:\n    debounce_ms: 0\n",
    )
    .unwrap();
    world.model.after_lernie_verb();
    world.converge();
    world.add_child("c-1", CHILD);
    world.add_child(CHILD, GRANDCHILD);
    let ws = world.ws.clone();
    world.model.mark_dirty([ws]);
    world.converge();
    world
}

/// Render the whole window and hand back every galley with its rect. Three
/// frames for the same reason [`super::painted`] takes three: a panel is its
/// default size on the frame it first appears, so the settled one is what the
/// operator sees.
fn painted(world: &mut World) -> Vec<Painted> {
    let (lernie, bl, bz) = (Cli::new("lernie"), Cli::new("bl"), Cli::new("bz"));
    let ctx = egui::Context::default();
    let mut frame = || {
        ctx.run(input(), |ctx| {
            render(ctx, &mut world.model, &mut world.state, &lernie, &bl, &bz);
        })
    };
    let _ = frame();
    let _ = frame();
    let out = crate::paint_probe::painted_of(&frame());
    drive::one_title_each(&out, &drive::visible(world));
    out
}

/// The leftmost x of a galley reading as `needle` (`drive::reads_as` — the
/// column truncates, so the head is what reaches the screen), or `None` for one
/// this frame never painted.
fn left_of(painted: &[Painted], needle: &str) -> Option<f32> {
    painted
        .iter()
        .filter(|(text, _)| drive::reads_as(text, needle))
        .map(|(_, rect)| rect.min.x)
        .reduce(f32::min)
}

/// How many galleys carry the reply elbow — the two-direction counter: shut, it
/// must be zero.
fn elbows(painted: &[Painted]) -> usize {
    painted
        .iter()
        .filter(|(text, _)| text.ends_with(crate::theme::ELBOW))
        .count()
}

/// The names the list paints, per depth, for the current expanded set — read
/// off the derivation the paint reads, so the beat names no title of its own
/// and cannot drift from the §3.3 ladder.
fn rows(world: &World) -> Vec<(usize, String)> {
    world
        .model
        .visible_conversations(super::super::now_unix(), &world.state.expanded)
        .iter()
        .map(|r| (r.depth, r.display_name()))
        .collect()
}

/// The title the list paints for `id` — read off the same derivation with the
/// **whole forest open**, so a needle is always a galley some frame really
/// paints.
///
/// Spelled out because guessing it is a live vacuity: the §3.3 floor of a
/// chained id is its terminal *generation* (`20260803T045643Z-1e5f99d4`), not
/// the hash alone, so a beat that split the id on its last `-` asserted the
/// absence of a string this tree never paints in either direction — green on a
/// list that could not fold at all.
pub(super) fn name_of(world: &World, id: &str) -> String {
    let open: std::collections::HashSet<String> = world
        .model
        .tree(&world.ws)
        .map(|t| t.agents.iter().map(|a| a.agent_id.clone()).collect())
        .unwrap_or_default();
    world
        .model
        .visible_conversations(super::super::now_unix(), &open)
        .iter()
        .find(|r| r.root_id == id)
        .map_or_else(
            || panic!("the wide-open list must hold a row for {id}"),
            crate::nav::convs::ConvRow::display_name,
        )
}

/// Shut, the row states the ruling's two numbers — `direct` then
/// `total`, which the three-generation fixture makes different — behind the
/// collapsed arrow, and its descent is not in the list: no child name, no
/// elbow. The field is **right** of the title, which is the seat bl-b9e3's
/// name-column rule requires of every conditional mark.
#[test]
fn a_collapsed_row_states_direct_and_total_right_of_its_title_and_hides_the_descent() {
    let mut world = nested_world();
    let list = rows(&world);
    assert_eq!(
        list.len(),
        1,
        "collapsed, the three-generation conversation is one row: {list:?}"
    );
    let shut = painted(&mut world);

    let title = left_of(&shut, &list[0].1).expect("the root row paints its title");
    let field = left_of(&shut, "▶ 1/2").unwrap_or_else(|| {
        panic!(
            "the subagent field states ▶ direct/total — 1 dispatched here, 2 under it \
             altogether:\n{:?}",
            shut.iter().map(|(t, _)| t).collect::<Vec<_>>()
        )
    });
    assert!(
        field > title,
        "the field rides the trailing right-pinned group: {field} <= {title}"
    );
    assert_eq!(elbows(&shut), 0, "a shut list draws no reply elbow");
    // The needles are [`name_of`]'s, and the same two are asserted **present**
    // below on the frame that opens the fold: two directions inside the one
    // beat, which is what makes an absence claim evidence of anything.
    let hidden: Vec<String> = [CHILD, GRANDCHILD]
        .iter()
        .map(|id| name_of(&world, id))
        .collect();
    for name in &hidden {
        assert!(
            left_of(&shut, name).is_none(),
            "{name} is folded away, so nothing paints it"
        );
    }
    world.state.expanded.insert("c-1".to_owned());
    world.state.expanded.insert(CHILD.to_owned());
    let open = painted(&mut world);
    for name in &hidden {
        assert!(
            left_of(&open, name).is_some(),
            "and the very same needle lands once the fold opens: {name}"
        );
    }
}

/// Open, each generation is a row of the same anatomy indented past the one
/// above it — the per-depth title edge §11 promises — wearing the reply elbow,
/// and each row's field states **its own** subtree: the middle row says 1/1
/// where the root says 1/2, which is the whole of "a row is the subtree rooted
/// at its agent".
#[test]
fn unfolding_indents_each_generation_past_the_one_above_it() {
    let mut world = nested_world();
    world.state.expanded.insert("c-1".to_owned());
    world.state.expanded.insert(CHILD.to_owned());
    let list = rows(&world);
    assert_eq!(
        list.iter().map(|(d, _)| *d).collect::<Vec<_>>(),
        vec![0, 1, 2],
        "three generations, one row each: {list:?}"
    );
    let painted = painted(&mut world);

    // The title edges: strictly increasing, depth by depth. Equal edges is the
    // defect this beat exists for — a child painted flush under its parent
    // reads as a sibling, and every string assertion in the file would pass.
    let edges: Vec<f32> = list
        .iter()
        .map(|(_, name)| left_of(&painted, name).unwrap_or_else(|| panic!("{name} paints")))
        .collect();
    for pair in edges.windows(2) {
        assert!(
            pair[1] > pair[0] + 1.0,
            "each depth's title edge sits right of the one above it: {edges:?}"
        );
    }
    // Two children, two elbows — and the elbow is ahead of the title it belongs
    // to, not somewhere in the trailing group.
    assert_eq!(elbows(&painted), 2, "one reply elbow per revealed child");
    let elbow_left = painted
        .iter()
        .filter(|(text, _)| text.ends_with(crate::theme::ELBOW))
        .map(|(_, rect)| rect.min.x)
        .reduce(f32::min)
        .unwrap();
    assert!(
        elbow_left < edges[1],
        "the elbow leads the child's row: {elbow_left} >= {}",
        edges[1]
    );
    // Each row folds its own subtree: the arrow flips where it was clicked, and
    // the numbers are that row's, not the conversation's.
    assert!(
        left_of(&painted, "▼ 1/2").is_some() && left_of(&painted, "▼ 1/1").is_some(),
        "the root says 1/2 open, the middle row 1/1 open:\n{:?}",
        painted.iter().map(|(t, _)| t).collect::<Vec<_>>()
    );
    assert!(
        left_of(&painted, "▶ 1/2").is_none(),
        "and no row still wears the shut arrow it was opened from"
    );
}
