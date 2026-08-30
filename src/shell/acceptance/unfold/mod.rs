//! The §11 **unfold** (bl-fa82): the subagent field's arrow and its two
//! numbers, and the indented reply-elbow rows an expanded row reveals — the one
//! three-generation fixture every beat below runs on, and the window-wide reads
//! over it.
//!
//! **One claim per child.** [`paint`] is what a fold must state and reveal
//! however it got there; the gesture halves are [`drive`], [`keys`] and
//! [`column`] (bl-89de), split off here rather than driven from `super::walk`
//! because those beats and these share one three-generation fixture, and a fold
//! has no meaning apart from the rows it reveals. The seam between the gesture
//! files is the hand — [`drive`] is what the *pointer* does to the fold (and
//! the words the field says under it), [`keys`] what the *keyboard* does (`→`
//! unfolds, `↓` skips what is folded, `←` pages up and then shuts), and
//! [`column`] where the revealed rows land. The column-scoped reads all three
//! spend are [`reads`]; the fixture and the window-wide reads are here.

mod column;
mod drive;
mod keys;
mod paint;
mod reads;

use super::super::render;
use super::fixture::{World, world};
use super::input;
use crate::cli_outbound::Cli;
use crate::paint_probe::Painted;

/// A nameless descent child of `c-1` (the bl-63a1 chained-id shape), and its
/// own child — three generations, so `direct` and `total` differ on the root
/// and the middle row has a field of its own.
pub(super) const CHILD: &str = "c-1-20260803T045643Z-1e5f99d4";
pub(super) const GRANDCHILD: &str = "c-1-20260803T045643Z-1e5f99d4-20260803T045700Z-2a6b88c5";

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
pub(super) fn nested_world() -> World {
    let mut world = world();
    // Debounce off (a legal cadence — the bounds floor at zero) so the children
    // derive on the very next pass rather than on a wall clock to sleep against.
    std::fs::write(
        world.model.state_root().join("cadence.yaml"),
        "cadence:\n  watcher:\n    debounce_ms: 0\n",
    )
    .unwrap();
    world.model.after_litany_verb();
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
pub(super) fn painted(world: &mut World) -> Vec<Painted> {
    let (litany, bl, bz) = (Cli::new("litany"), Cli::new("bl"), Cli::new("bz"));
    let ctx = egui::Context::default();
    let frame = |world: &mut World| {
        ctx.run(input(), |ctx| {
            render(ctx, &mut world.model, &mut world.state, &litany, &bl, &bz);
        })
    };
    // The wire settled between frames (bl-44e9): the list is a `Reply` now, so
    // the frame that declares the question and the frame that paints its answer
    // are two, and this harness renders the settled one.
    let _ = frame(world);
    world.settle();
    let _ = frame(world);
    world.settle();
    let out = crate::paint_probe::painted_of(&frame(world));
    reads::one_title_each(&out, &reads::visible(world));
    out
}

/// The leftmost x of a galley reading as `needle` (`reads::reads_as` — the
/// column truncates, so the head is what reaches the screen), or `None` for one
/// this frame never painted.
pub(super) fn left_of(painted: &[Painted], needle: &str) -> Option<f32> {
    painted
        .iter()
        .filter(|(text, _)| reads::reads_as(text, needle))
        .map(|(_, rect)| rect.min.x)
        .reduce(f32::min)
}

/// How many galleys carry the reply elbow — the two-direction counter: shut, it
/// must be zero.
pub(super) fn elbows(painted: &[Painted]) -> usize {
    painted
        .iter()
        .filter(|(text, _)| text.ends_with(crate::theme::ELBOW))
        .count()
}

/// The names the list paints, per depth, for the current expanded set — read
/// off the derivation the paint reads, so the beat names no title of its own
/// and cannot drift from the §3.3 ladder.
pub(super) fn rows(world: &World) -> Vec<(usize, String)> {
    crate::test_support::convs::visible(
        &world.model,
        super::super::now_unix(),
        &world.state.expanded,
    )
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
    crate::test_support::convs::visible(&world.model, super::super::now_unix(), &open)
        .iter()
        .find(|r| r.root_id == id)
        .map_or_else(
            || panic!("the wide-open list must hold a row for {id}"),
            crate::nav::convs::ConvRow::display_name,
        )
}
