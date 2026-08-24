//! The §11 unfold under the **keyboard** (bl-89de) — the second half of the
//! ruling: up and down walk the list, left and right collapse and expand it
//! (including paging back up to the last level when left is pressed on a
//! child), and going down never expands anything on its own, skipping instead
//! to the next thing at the same level.
//!
//! Three claims, one per beat: `→` folds a row open (and the row below it, one
//! generation down), `↓` steps **over** what is folded and opens nothing, and
//! `←` pages up to the parent before it shuts anything.
//!
//! Driven on the **Command plane** (Ctrl+←/→/↓, ⌘ on macOS). Every selection
//! lands the composer (§11 focus discipline, bl-c21f), so after the first one
//! the bare plane is spent and the combo is the walk's continuation — the plane
//! an operator's second press really rides. That the bare plane reaches these
//! actions at all is `super::super::walk`'s claim, and the table binds both
//! (`keymap`'s bare and Command arms are the same four actions).
//!
//! Each beat asserts the **selection**, the **expanded set** and the **column**
//! together. Any one alone passes on a defect the other two catch: a walk that
//! moved the selection into a row the list is not painting, a fold that
//! repainted without moving anything, an expanded set the paint ignores.

use super::super::fixture::World;
use super::super::screen::{Screen, press};
use super::reads::{SECOND, at, column, driven_world, elbows, seat, visible};
use super::{CHILD, GRANDCHILD};

/// The selected agent's id, or `"-"` with nothing selected.
fn selected(world: &World) -> String {
    world
        .model
        .focused_agent()
        .map_or_else(|| "-".to_owned(), |a| a.agent_id.clone())
}

/// One press on the §11 combo plane.
fn key(screen: &Screen, world: &mut World, key: egui::Key) {
    screen.frame(world, vec![press(key, egui::Modifiers::COMMAND)]);
}

/// Select the root conversation the way an operator does — by clicking its
/// title — and hand back the world with that selection made and **nothing
/// unfolded**. The second half is a claim of its own: opening a conversation is
/// not opening its descent, so every beat below starts from a shut list it did
/// not have to shut.
fn selected_root() -> (World, Screen) {
    let (mut world, screen) = driven_world();
    let title = seat(&column(&screen, &mut world), "hello").center();
    super::super::screen::click(&screen, &mut world, title);
    assert_eq!(selected(&world), "c-1", "the click selects the root row");
    assert!(
        world.state.expanded.is_empty(),
        "and unfolds nothing by doing it: {:?}",
        world.state.expanded
    );
    (world, screen)
}

/// `→` unfolds **the selected row**, and one generation further down it does it
/// again — the recursion the ruling asks for (subagents indent recursively and
/// wear the little chat-reply line), reached by the keyboard alone.
///
/// The reveal is asserted positionally: each generation's title edge sits right
/// of the one above it, and the elbow count is the number of revealed children.
/// A string-only version of this beat passes on a list that paints every
/// generation flush at the left margin, which is the defect it exists for.
///
/// `→` is also asserted **not** to move the selection: it is a verb on the row
/// the selection already names (§11 rule 1), so a `→` that walked as it opened
/// would be a second walk with no key to stop it.
#[test]
fn the_right_arrow_unfolds_the_selected_row_and_then_the_generation_below_it() {
    let (mut world, screen) = selected_root();
    let (child, grandchild) = (
        super::name_of(&world, CHILD),
        super::name_of(&world, GRANDCHILD),
    );

    key(&screen, &mut world, egui::Key::ArrowRight);
    let open = column(&screen, &mut world);
    assert_eq!(selected(&world), "c-1", "→ opens the row, it does not walk");
    assert_eq!(
        world.state.expanded.len(),
        1,
        "and opens exactly the one row it names: {:?}",
        world.state.expanded
    );
    assert!(
        at(&open, &grandchild).is_none(),
        "one press, one generation — the grandchild is still folded under its own parent"
    );
    assert!(
        at(&open, "▼ 1/2").is_some() && at(&open, "▶ 1/1").is_some(),
        "the root's arrow points down and the revealed child wears a shut field of its own:\n{:?}",
        open.iter().map(|(t, _)| t).collect::<Vec<_>>()
    );

    // Down into the child (the walk's own claim is the beat below), then the
    // same key again — the same gesture at the next depth, no second control.
    key(&screen, &mut world, egui::Key::ArrowDown);
    assert_eq!(selected(&world), CHILD, "↓ enters the revealed child");
    key(&screen, &mut world, egui::Key::ArrowRight);
    let deep = column(&screen, &mut world);
    assert_eq!(selected(&world), CHILD, "and → still does not walk");

    let edges: Vec<f32> = [
        seat(&deep, "hello"),
        seat(&deep, &child),
        seat(&deep, &grandchild),
    ]
    .iter()
    .map(egui::Rect::left)
    .collect();
    for pair in edges.windows(2) {
        assert!(
            pair[1] > pair[0] + 1.0,
            "each generation indents past the one above it: {edges:?}"
        );
    }
    assert_eq!(elbows(&deep), 2, "one reply elbow per revealed child");
    assert!(
        at(&deep, "▼ 1/1").is_some(),
        "and the child's own field states its own subtree, open"
    );
}

/// **The walk skips the hidden** — the ruling's *"don't automatically expand
/// just from going down; skip to the next thing at the same level"*.
///
/// With the root's two generations folded away, `↓` lands on the next **root**,
/// not on the child directly beneath it in the tree; and the expanded set is
/// untouched, which is the half a selection assertion alone would miss (a walk
/// that revealed a subtree in order to step into it would still end on a
/// different row and still read green).
///
/// The landing is order-independent by construction: the shut list is exactly
/// two rows, so `↓` from one of them is the other whichever way the recency
/// sort put them.
#[test]
fn the_walk_steps_over_a_folded_subtree_and_opens_nothing() {
    let (mut world, screen) = selected_root();
    let child = super::name_of(&world, CHILD);
    assert_eq!(
        visible(&world).len(),
        2,
        "shut, the three-generation conversation and its neighbour are two rows"
    );

    key(&screen, &mut world, egui::Key::ArrowDown);
    let after = column(&screen, &mut world);
    assert_eq!(
        selected(&world),
        SECOND,
        "↓ lands on the next row at the same level, not inside the fold"
    );
    assert!(
        world.state.expanded.is_empty(),
        "and the walk expanded nothing on its way: {:?}",
        world.state.expanded
    );
    assert!(
        at(&after, &child).is_none() && elbows(&after) == 0,
        "so the column is still the shut list:\n{:?}",
        after.iter().map(|(t, _)| t).collect::<Vec<_>>()
    );

    // The other direction of the same rule: what is unfolded IS walked into.
    // Without this the beat above would also pass on a walk that could never
    // reach a child at all.
    key(&screen, &mut world, egui::Key::ArrowUp);
    assert_eq!(selected(&world), "c-1", "↑ comes back to the root");
    key(&screen, &mut world, egui::Key::ArrowRight);
    key(&screen, &mut world, egui::Key::ArrowDown);
    assert_eq!(
        selected(&world),
        CHILD,
        "and once it is open, ↓ steps into the child rather than over it"
    );
}

/// `←` **pages up before it folds** (the ruling's *"including paging back up to
/// the last level, if you hit left while on a child"*): on a child it moves the
/// selection to the parent, and only then does a second press shut the parent.
///
/// Two presses, two different effects from one key, so each is asserted against
/// *both* pieces of state: the first must move the selection and leave the
/// expanded set alone, the second must leave the selection alone and empty the
/// expanded set. A beat that watched only one of them would pass on a `←` that
/// collapsed the parent out from under the child on the first press — the
/// gesture the ruling exists to rule out.
///
/// The third press is the floor: at a root with nothing left to fold, `←` is a
/// no-op rather than a wrap onto some other conversation.
#[test]
fn the_left_arrow_pages_up_to_the_parent_before_it_folds_the_parent_shut() {
    let (mut world, screen) = selected_root();
    let child = super::name_of(&world, CHILD);
    key(&screen, &mut world, egui::Key::ArrowRight);
    key(&screen, &mut world, egui::Key::ArrowDown);
    assert_eq!(selected(&world), CHILD, "standing on the revealed child");

    key(&screen, &mut world, egui::Key::ArrowLeft);
    let paged = column(&screen, &mut world);
    assert_eq!(
        selected(&world),
        "c-1",
        "← on a child pages the selection up to its parent"
    );
    assert!(
        world.state.expanded.contains("c-1"),
        "without folding what it paged out of: {:?}",
        world.state.expanded
    );
    assert!(
        at(&paged, &child).is_some() && elbows(&paged) == 1,
        "so the child is still on screen, where the operator left it"
    );

    key(&screen, &mut world, egui::Key::ArrowLeft);
    let shut = column(&screen, &mut world);
    assert!(
        world.state.expanded.is_empty(),
        "the second ← shuts the row the first one landed on: {:?}",
        world.state.expanded
    );
    assert_eq!(selected(&world), "c-1", "and stays where it is");
    assert!(
        at(&shut, &child).is_none() && elbows(&shut) == 0,
        "the descent leaves the column with the fold:\n{:?}",
        shut.iter().map(|(t, _)| t).collect::<Vec<_>>()
    );
    assert!(
        at(&shut, "▶ 1/2").is_some(),
        "and the arrow points right again"
    );

    key(&screen, &mut world, egui::Key::ArrowLeft);
    assert_eq!(
        (selected(&world), world.state.expanded.len()),
        ("c-1".to_owned(), 0),
        "a shut root has no parent to page to and nothing to fold: ← rests"
    );
}
