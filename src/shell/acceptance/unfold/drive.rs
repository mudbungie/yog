//! The §11 unfold under the **pointer** (bl-89de): the subagent field's click,
//! and the words it says while the mouse rests on it. The keyboard half is
//! [`super::keys`], the geometry of the rows they reveal [`super::column`], and
//! the driven fixture plus the column reads all three share [`super::reads`].

use super::super::fixture::World;
use super::super::screen::click;
use super::CHILD;
use super::reads::{at, column, driven_world, elbows, seat};

/// **The pointer half of the operator's ruling** — *"a field on the right of it
/// that indicates the number of subagents, which if clicked, expands the list;
/// arrow pointing right normally, click on it to expand down"* — in both
/// directions, because a control that only opened would leave the list no way
/// back.
///
/// What the click may **not** do is asserted beside what it must: the row stays
/// unselected. The field rides the trailing group, outside the selectable
/// title, so folding is not the §6 acknowledgement a click on the row is — and
/// without that assertion this beat would pass equally on a tree whose whole
/// row toggled, which is a different control answering a different question.
#[test]
fn clicking_the_field_unfolds_the_row_and_folds_it_again_without_selecting_it() {
    let (mut world, screen) = driven_world();
    let child = super::name_of(&world, CHILD);
    let shut = column(&screen, &mut world);
    let arrow = seat(&shut, "▶ 1/2").center();
    assert!(
        at(&shut, &child).is_none(),
        "the descent starts folded away, so the column paints no {child}"
    );

    click(&screen, &mut world, arrow);
    let open = column(&screen, &mut world);
    assert!(
        world.state.expanded.contains("c-1"),
        "the click flips the row's id in the expanded set: {:?}",
        world.state.expanded
    );
    assert_eq!(
        world.model.focused_agent().map(|a| a.agent_id.clone()),
        None,
        "and selects nothing — the field is its own control, not a strip of the row"
    );
    let title = seat(&open, "hello");
    let revealed = seat(&open, &child);
    assert!(
        revealed.left() > title.left() + 1.0,
        "the revealed child hangs indented under its parent: {revealed:?} vs {title:?}"
    );
    assert_eq!(elbows(&open), 1, "wearing the one reply elbow it earned");
    assert!(
        at(&open, "▼ 1/2").is_some() && at(&open, "▶ 1/2").is_none(),
        "and the arrow the click was aimed at now points down"
    );

    // Back the other way, on the arrow where it now is: the same gesture shuts
    // what it opened, and the child leaves the column with it.
    click(&screen, &mut world, seat(&open, "▼ 1/2").center());
    let shut = column(&screen, &mut world);
    assert!(
        !world.state.expanded.contains("c-1"),
        "the second click folds it back: {:?}",
        world.state.expanded
    );
    assert!(
        at(&shut, &child).is_none() && elbows(&shut) == 0,
        "so the column is the one-row list it started as:\n{:?}",
        shut.iter().map(|(t, _)| t).collect::<Vec<_>>()
    );
}

/// The operator's own requirement of the field — *"mouseover should indicate
/// what the numbers mean"* — driven to the paint layer, which is where the
/// §11 hover scan cannot reach it: that scan's fixture has no descent at all,
/// so the field it holds to the spelling rule is one it never paints.
///
/// The needles carry **this row's own numbers**, which is what makes them a
/// claim about the field rather than about some sentence on the surface: `1` is
/// the direct count, `2` the total, and a hover that stated one of them, or
/// stated them the other way round, fails here.
#[test]
fn the_fields_hover_states_both_numbers_and_the_keys_that_press_it() {
    let mut world = super::nested_world();
    let (lernie, bl, bz) = (
        crate::cli_outbound::Cli::new("/yog-absent-lernie"),
        crate::cli_outbound::Cli::new("/yog-absent-bl"),
        crate::cli_outbound::Cli::new("/yog-absent-bz"),
    );
    let ctx = egui::Context::default();
    // egui's own "show every tooltip" — the bl-2d87 idiom `super::super::hover`
    // proves its own wiring with: a hover hung on the neighbouring widget
    // rather than the control never reaches the galleys.
    ctx.memory_mut(|m| m.set_everything_is_visible(true));
    let frame = |world: &mut World| {
        let out = ctx.run(super::super::input(), |ctx| {
            super::super::super::render(ctx, &mut world.model, &mut world.state, &lernie, &bl, &bz);
        });
        crate::paint_probe::text_of(&out)
    };
    // The wire settled between frames (bl-44e9): the field this beat hovers is
    // painted off a `Reply::Conversations`, which lands a round trip later.
    frame(&mut world);
    world.settle();
    frame(&mut world);
    world.settle();
    frame(&mut world);
    world.settle();
    let painted = frame(&mut world);
    for phrase in [
        "1 dispatched by this agent itself",
        "2 under it altogether at any depth",
        "→ unfolds the selected row",
        "← folds it shut",
        "← on a child pages the selection up to its parent",
    ] {
        assert!(
            painted.contains(phrase),
            "the subagent field's hover must say {phrase:?}:\n{painted}"
        );
    }
}
