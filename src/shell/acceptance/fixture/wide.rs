//! **The wide roster's own bytes** (bl-0424): conversations whose *content* is
//! wider than the column, laid on the shipped world. Sibling of [`crowd`],
//! which varies the same fixture along the other axis — that one is a list too
//! TALL for the column, this one is rows too WIDE for it — and split from
//! [`super`] at §12's budget on the same seam.
//!
//! Until this existed the §11 panel-geometry beats asked their question of a
//! column whose widest row was short. `world_titled` gives one row a long
//! title, and a title truncates correctly and always did; nothing in the suite
//! gave a row a long title **and** a preview beside it, which is the pair that
//! reproduces the operator's report — the preview is laid after the greedy
//! title, so a title that fills the row leaves it zero width, and a run laid at
//! zero width still allocates its own ellipsis past the panel's edge. That
//! allocation is what egui stores as next frame's panel width.
//!
//! [`crowd`]: super::crowd

use super::World;

/// The rows the wide roster seats: the §3.3 name each wears and the first
/// payload line that rides weak beside it. Both long on purpose and both
/// distinct, so a beat can tell a title that survived from a preview that did.
///
/// **The name and the row's trailing metadata fill the row between them**,
/// which is what makes the pair bite: measured at every size in
/// [`SIZES`](crate::shell::acceptance::SIZES), the preview after them is laid
/// into nothing and lands as a bare `…`.
pub(in crate::shell::acceptance) const ROWS: [(&str, &str, &str); 2] = [
    (
        "w-1",
        "WidenedCourtyardRooftopEscarpment",
        "the first payload line of a conversation whose name already fills the \
         column, so this preview is laid into what is left of it",
    ),
    (
        "w-2",
        "WidenedAxolotlHeadlandPromontory",
        "a second such row, because one row cannot show that the walk compounds \
         frame after frame the way the operator described it",
    ),
];

/// **A world whose conversation rows are wider than the column** — the fixture
/// every §11 rule 1/1b/2 panel-width beat is driven over. Identical to
/// [`world`](super::world) in every other respect, so a beat that reddens here
/// and passes there has found a defect of *width*, which is the only axis these
/// two fixtures differ on.
pub(in crate::shell::acceptance) fn world_wide() -> World {
    let mut world = super::build::build_world("hello", &super::build::Roster::One);
    // The §7.2 watcher debounce, spent rather than waited on: a fixture that
    // changes disk and converges in the same breath is inside the window
    // otherwise, and the rows below would never reach the derivation
    // (`inbox_composer::quick`'s recipe, one door over).
    std::fs::write(
        world.model.state_root().join("cadence.yaml"),
        "cadence:\n  watcher:\n    debounce_ms: 0\n",
    )
    .unwrap();
    world.model.after_lernie_verb();
    world.converge();
    for (id, name, preview) in ROWS {
        // The payload line first: `build_agent` writes it as the conversation's
        // goal, which is the preview the §3.3 ladder reads. Then the name fact
        // on top, so the ladder lands on the name and the preview becomes the
        // weak subtitle beside it rather than the title itself.
        world.fx.build_agent(id, preview);
        world.fx.name_agent(id, name);
    }
    let ws = world.ws.clone();
    world.model.mark_dirty([ws]);
    world.converge();
    world
}
