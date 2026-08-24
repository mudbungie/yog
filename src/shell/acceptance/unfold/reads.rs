//! **How an unfold beat reads the conversation column** — the driven fixture
//! all three gesture files share, and the column-scoped reads over it.
//!
//! Split from [`super::drive`] at §12's budget on the seam that file's own doc
//! already named: what the *pointer* does to a fold is one subject, how any of
//! the three hands then reads the list another. [`super::keys`] and
//! [`super::column`] were already importing this half through a file about the
//! mouse.
//!
//! Everything here is scoped to the **conversation column's** galleys, not the
//! window's. Written when the altitude-1 descent tree repainted every member
//! name in the centre of any selected conversation, which made a window-wide
//! "nothing paints this child" false on a correct tree and a window-wide
//! "something does" true of a tree that never unfolded anything. bl-8905
//! retired that tree; the filter stays because the centre's header still paints
//! the open conversation's title — the root row's own needle — and because the
//! predicate that distinguishes a fold is *where* a name landed, which is a
//! claim only a column-scoped read can make.
//!
//! And on **rects** wherever the claim is positional: a revealed child's title
//! edge is a number, and asserting the number is what separates a fold from a
//! list that paints its children flush under their parent.

use super::super::fixture::World;
use super::super::screen::Screen;
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
    crate::test_support::convs::visible(
        &world.model,
        crate::shell::now_unix(),
        &world.state.expanded,
    )
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
