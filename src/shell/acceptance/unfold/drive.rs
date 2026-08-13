//! The §11 unfold under the **pointer** (bl-89de): the subagent field's click,
//! and the words it says while the mouse rests on it. The keyboard half is
//! [`super::keys`], the geometry of the rows they reveal [`super::column`];
//! the driven fixture and the column reads all three share start here.
//!
//! Everything here is asserted on the **conversation column's** galleys, not
//! the window's ([`column`]). Written when the altitude-1 descent tree repainted
//! every member name in the centre of any selected conversation, which made a
//! window-wide "nothing paints this child" false on a correct tree and a
//! window-wide "something does" true of a tree that never unfolded anything.
//! bl-8905 retired that tree; the filter stays because the centre's header still
//! paints the open conversation's title — the root row's own needle — and
//! because the predicate that distinguishes a fold is *where* a name landed,
//! which is a claim only a column-scoped read can make.
//!
//! And on **rects** wherever the claim is positional: a revealed child's title
//! edge is a number, and asserting the number is what separates a fold from a
//! list that paints its children flush under their parent.

use super::super::fixture::World;
use super::super::screen::{Screen, click};
use super::CHILD;
use crate::nav::convs::ConvRow;
use crate::paint_probe::Painted;

/// A second **root** beside the three-generation conversation: the row `↓` must
/// land on while the descent under the selection is folded away
/// ([`super::keys`]).
pub(super) const SECOND: &str = "c-2";

/// [`super::nested_world`]'s three generations with [`SECOND`] beside them, on
/// a persistent screen — one window, so a selection and a fold carry frame to
/// frame exactly as they do under the operator's hand.
pub(super) fn driven_world() -> (World, Screen) {
    let mut world = super::nested_world();
    world.add_root(SECOND, "second-root");
    let ws = world.ws.clone();
    world.model.mark_dirty([ws]);
    world.converge();
    let screen = Screen::new();
    screen.idle(&mut world);
    (world, screen)
}

/// Every galley the settled frame painted **inside the conversation column**,
/// with its rect. One frame to settle the panel rect this reads, then the frame
/// whose galleys it hands back.
pub(super) fn column(screen: &Screen, world: &mut World) -> Vec<Painted> {
    screen.idle(world);
    let shapes = screen.shapes(world, Vec::new());
    let edge = screen.column();
    let mut out = Vec::new();
    for clipped in &shapes {
        crate::paint_probe::collect(&clipped.shape, &mut out);
    }
    let out: Vec<Painted> = out.into_iter().filter(|(_, r)| r.left() < edge).collect();
    one_title_each(&out, &visible(world));
    out
}

/// Does a painted galley read as the title `name`?
///
/// Exactly it, **or the head egui left of it**. The conversation column is
/// width-bound (§11 rule 1: nothing in this panel extends past it), so a title
/// wider than the room its row's trailing metadata leaves is truncated, and
/// what reaches the screen is `20260803T0456…` — a 25-character chained-id
/// floor does not fit beside a subagent field. The probe reads *glyphs* rather
/// than the galley's input (bl-bc06), so that head is all a beat can see, and
/// matching it is what keeps these beats claims about the **fold** rather than
/// accidental claims about the column's width.
///
/// A head names one row only because [`one_title_each`] says so on the very
/// frame being read; on its own an ellipsis is evidence of nothing, which is
/// why the empty head matches nothing here.
pub(super) fn reads_as(text: &str, name: &str) -> bool {
    text == name
        || text
            .strip_suffix('…')
            .is_some_and(|head| !head.is_empty() && name.starts_with(head))
}

/// Every galley of this frame reads as **at most one** row's title.
///
/// The guard [`reads_as`] leans on, checked once on the frame itself rather
/// than carried by each needle. Truncation is not injective: the two sibling
/// leaves `…T045801Z-3c7d55e6` and `…T045802Z-4d8e66f7` share a 13-character
/// head, so a column one glyph narrower would make every assertion below match
/// whichever row painted first — green, and about nothing. That fails here
/// instead, naming both rows.
pub(super) fn one_title_each(painted: &[Painted], rows: &[ConvRow]) {
    let names: std::collections::BTreeSet<String> =
        rows.iter().map(ConvRow::display_name).collect();
    for (text, _) in painted {
        let reads: Vec<&String> = names.iter().filter(|n| reads_as(text, n)).collect();
        assert!(
            reads.len() <= 1,
            "the frame paints {text:?}, which reads as more than one row's title: {reads:?}"
        );
    }
}

/// The rect of the column's leftmost galley reading as `needle` — leftmost
/// because a row paints its name twice when the §3.3 ladder leaves the payload
/// line weak beside it, and the title is the left one.
pub(super) fn at(painted: &[Painted], needle: &str) -> Option<egui::Rect> {
    painted
        .iter()
        .filter(|(text, _)| reads_as(text, needle))
        .map(|(_, rect)| *rect)
        .reduce(|a, b| if a.left() <= b.left() { a } else { b })
}

/// That rect, or a panic naming what the column did paint.
pub(super) fn seat(painted: &[Painted], needle: &str) -> egui::Rect {
    at(painted, needle).unwrap_or_else(|| {
        panic!(
            "the conversation column paints no {needle:?}:\n{:?}",
            painted.iter().map(|(t, _)| t).collect::<Vec<_>>()
        )
    })
}

/// How many reply elbows the column paints — one per revealed child, zero when
/// everything is shut. The elbow is the list's alone (the altitude-1 tree
/// indents with blanks), so it counts a fold with no id to match on.
pub(super) fn elbows(painted: &[Painted]) -> usize {
    painted
        .iter()
        .filter(|(text, _)| text.ends_with(crate::theme::ELBOW))
        .count()
}

/// The rows the frame is painting, from the derivation the frame itself reads.
pub(super) fn visible(world: &World) -> Vec<ConvRow> {
    world
        .model
        .visible_conversations(crate::shell::now_unix(), &world.state.expanded)
}

/// The galleys sharing one row with `rect` — everything whose vertical centre
/// falls inside it.
pub(super) fn band(painted: &[Painted], rect: egui::Rect) -> Vec<Painted> {
    painted
        .iter()
        .filter(|(_, r)| r.center().y > rect.top() && r.center().y < rect.bottom())
        .cloned()
        .collect()
}

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
        crate::cli_outbound::Cli::new("yog-absent-lernie"),
        crate::cli_outbound::Cli::new("yog-absent-bl"),
        crate::cli_outbound::Cli::new("yog-absent-bz"),
    );
    let ctx = egui::Context::default();
    // egui's own "show every tooltip" — the bl-2d87 idiom `super::super::hover`
    // proves its own wiring with: a hover hung on the neighbouring widget
    // rather than the control never reaches the galleys.
    ctx.memory_mut(|m| m.set_everything_is_visible(true));
    let mut frame = || {
        let out = ctx.run(super::super::input(), |ctx| {
            super::super::super::render(ctx, &mut world.model, &mut world.state, &lernie, &bl, &bz);
        });
        crate::paint_probe::text_of(&out)
    };
    frame();
    let painted = frame();
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
