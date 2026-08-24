//! The **name column at depth** (bl-89de): §11's rule that a row's title edge
//! is its depth's indent and nothing else, held one generation in.
//!
//! `super::super::name_column` is this same discipline at depth 0 — the ball it
//! landed for (bl-b9e3) had the operator's complaint *"it makes the list not
//! align"* to answer, and answered it by moving the attention flag out of the
//! left prefix. The unfold gives every depth a left edge of its own, so the
//! rule now has to hold **within** each of them: this is that beat's fixture
//! one level down, on the child rows an unfolded conversation reveals.
//!
//! Two complementary predicates, because a frame can pass either alone: the
//! edges of one depth **agree**, and each title **shares no pixel** with the
//! metadata pinned right of it. A list that never indented satisfies the first
//! everywhere; a title truncated to nothing satisfies the second.

use super::super::screen::Screen;
use super::reads::{band, column, seat, visible};

/// Two more **direct children of `c-1`**, both leaves and siblings of each
/// other, differing in nothing but attention: the second is marked abandoned,
/// §6 rule 2's one suppressor — the same way `super::super::name_column` quiets
/// its second root, one altitude up.
const LEAF_FLAGGED: &str = "c-1-20260803T045801Z-3c7d55e6";
const LEAF_QUIET: &str = "c-1-20260803T045802Z-4d8e66f7";

/// **The name column, one depth in** (§11, bl-b9e3's rule under the unfold):
/// within a depth the title's left edge is that depth's indent and nothing
/// else, so two child rows differing **only in attention** start on the same x
/// — and each title is clear of the metadata pinned right of it.
///
/// The pair is asserted in the fixture first, both ways round: one leaf bears
/// §6 rule 2's rest evidence, the other is abandoned, which is the one gate
/// that suppresses it. Then the flag's own seat is pinned per row — painted on
/// the flagged leaf, right of every title, and absent from the quiet one —
/// because equal edges hold just as well on a tree that stopped painting the
/// flag at all, and the flag is the difference this equality is blind to.
///
/// Attention is the difference this beat varies because it is the one the
/// fixture can produce through the real derivation. The other conditional marks
/// — the flight chip, the §10 `?`, the verdict badge — no longer move the column
/// at any depth (bl-8257 seated the first and third in the trailing group and
/// gave the second a fixed-width slot), and their per-mark proof is
/// `super::super::name_column`'s second beat, which paints hand-built rows
/// because an uncertain state comes from an injected probe and a full-window
/// fixture cannot make one.
#[test]
fn one_depths_titles_share_an_edge_across_attention_and_clear_their_metadata() {
    let mut world = super::nested_world();
    world.add_child("c-1", LEAF_FLAGGED);
    world.add_child("c-1", LEAF_QUIET);
    world.quiet(LEAF_QUIET);
    let ws = world.ws.clone();
    world.model.mark_dirty([ws]);
    world.converge();
    world.state.expanded.insert("c-1".to_owned());

    let rows = visible(&world);
    // The row itself, so both the fixture's honesty and the needle below read
    // the one derivation the frame paints — never a name re-spelled here.
    let row = |id: &str| {
        rows.iter()
            .find(|r| r.root_id == id)
            .unwrap_or_else(|| panic!("{id} is a row of this list: {rows:?}"))
    };
    assert!(
        row(LEAF_FLAGGED).attention > 0 && row(LEAF_QUIET).attention == 0,
        "the two leaves must differ in attention, one each way: {rows:?}"
    );
    let screen = Screen::new();
    let painted = column(&screen, &mut world);

    // Every depth-1 title starts on one x, and it is not depth 0's — an
    // equality that held at every depth would be a list that never indented.
    let root = seat(&painted, "hello").left();
    let depth1: Vec<(String, egui::Rect)> = rows
        .iter()
        .filter(|r| r.depth == 1)
        .map(|r| (r.display_name(), seat(&painted, &r.display_name())))
        .collect();
    assert_eq!(
        depth1.len(),
        3,
        "three direct children reach the list: {rows:?}"
    );
    let edge = depth1.first().map_or(0.0, |(_, r)| r.left());
    assert!(
        edge > root + 1.0,
        "a child's title edge is its own, right of the root's: {edge} vs {root}"
    );
    for (name, rect) in &depth1 {
        assert!(
            (rect.left() - edge).abs() < 0.5,
            "the title edge is the depth's, not attention's: {name} sits at {} not {edge}",
            rect.left()
        );
        // Disjointness, the complementary predicate: nothing else on this row
        // shares a pixel of x with the title. Equal edges say where a title
        // starts; only this says the row it starts is still legible.
        for (other, r) in band(&painted, *rect) {
            assert!(
                r == *rect || r.right() <= rect.left() + 0.5 || r.left() >= rect.right() - 0.5,
                "{other:?} at {r:?} overlaps the title {name} at {rect:?}"
            );
        }
    }

    // The flag really is painted, on the flagged row and right of every title,
    // and the quiet row really has none.
    let flagged = seat(&painted, &row(LEAF_FLAGGED).display_name());
    let quiet = seat(&painted, &row(LEAF_QUIET).display_name());
    let flag = band(&painted, flagged)
        .into_iter()
        .find(|(text, _)| text == "⚑")
        .unwrap_or_else(|| panic!("the flagged leaf paints its ⚑ at {flagged:?}"));
    assert!(
        flag.1.left() > edge,
        "the flag rides the trailing group, right of every title: {:?}",
        flag.1
    );
    assert!(
        !band(&painted, quiet).iter().any(|(text, _)| text == "⚑"),
        "and the abandoned leaf paints none at all"
    );
}
