//! **Which rung of the §3.3 ladder a seat is entitled to** — the paint half,
//! split from [`super`]'s value scan at §12's budget on the seam the two
//! already had. That scan forbids spelling more of an agent id than the
//! ladder's floor and says nothing about which rung a seat may take; these hold
//! the seats whose defect was taking the wrong one while spelling nothing
//! unlawful, so the scan was green over both of them.

use super::{PEER, painted_over_ids};
use crate::cli_outbound::Cli;

/// **bl-b6d0, the paint half: a named sender's deposit paints the NAME.** The
/// scan above forbids spelling more of an id than the ladder's floor and says
/// nothing about which *rung* a seat is entitled to — and `header_line` sat on
/// the floor unconditionally, so a deposit from a peer that HAS a name painted
/// its raw id in a frame whose conversation list painted 'peregrine'. Ruled in
/// the ball: the sender is an agent, §3.3's ladder is the one answer to what an
/// agent is called, so this seat takes rung one and falls through like the rest.
/// Both directions on the real window — the name is where the id was, and the
/// id is nowhere, [`PEER`] being one generation and so unable to hide behind its
/// own floor. The Inbox tab's frame also carries the §11 inbox-composer's
/// pending row for the same deposit, `header_line`'s other painted seat.
#[test]
fn a_named_senders_deposit_paints_the_name_the_rest_of_the_frame_paints() {
    let painted = painted_over_ids(crate::keymap::InspectorTab::Inbox);
    assert!(
        painted.contains("✉ peregrine · t0"),
        "the header names its sender — what this very frame's roster calls \
         it:\n{painted}"
    );
    assert!(
        !painted.contains(PEER),
        "and the id it used to spell is nowhere: a named agent's id has two \
         seats, the ladder's floor and the hover, and this is neither:\n{painted}"
    );
}

/// bl-63a1, the paint half. The operator's screenshot: agent skimmer's descent
/// tree, dozens of nameless fan-out children each titled with the FULL
/// ancestry-chain id — "These are insane. Unparseable." The seat rode the
/// ladder (the scan above held), but the ladder's floor itself spelled the
/// whole chain; the floor now spells the terminal generation only, and this
/// proves it at the paint layer: the row shows the child's own
/// `<stamp>-<hash>`, never the chain, and the full id still rides the hover
/// (the bl-2d87 forced-tooltip idiom).
///
/// **Re-pointed by bl-8905**: the row this was written against was the
/// altitude-1 descent tree's, which that ball retired as a second rendering of
/// the conversation list's own unfolded rows. The child is read where it now
/// paints — as a list row, its parent unfolded — and the claim is unchanged,
/// including the hover, whose id seat moved onto the list row with it.
#[test]
fn a_nameless_chained_child_row_shows_its_terminal_segment_and_hovers_the_full_id() {
    let (litany, bl) = (Cli::new("litany"), Cli::new("bl"));
    let mut world = super::super::inbox_composer::quick(super::super::fixture::world());
    let ws = world.ws.clone();
    // A descent child whose id embeds the ancestry chain, with no name blob,
    // no goal and no step record — nothing above the ladder's floor.
    world.add_child("c-1", "c-1-20260803T045643Z-1e5f99d4");
    world.model.mark_dirty([ws.clone()]);
    world.converge();
    world.model.focus_agent(&ws, "c-1");
    // Unfold the parent: since bl-fa82 a member is a row of the conversation
    // list, and since bl-8905 that is the only place it paints.
    world.state.expanded.insert("c-1".to_owned());
    let out = super::super::painted(&mut world, &litany, &bl);
    assert!(
        out.contains("20260803T045643Z-1e5f99d4"),
        "the child's row titles it by its terminal generation:\n{out}"
    );
    assert!(
        !out.contains("c-1-20260803T045643Z-1e5f99d4"),
        "the row never re-spells the lineage — the indent and its elbow state it:\n{out}"
    );
    // The full id's display seat is the hover: force every tooltip visible and
    // it reaches the galleys.
    let bz = Cli::new("bz");
    let ctx = egui::Context::default();
    ctx.memory_mut(|m| m.set_everything_is_visible(true));
    let mut frame = || {
        let full = ctx.run(super::super::input(), |ctx| {
            super::super::super::render(ctx, &mut world.model, &mut world.state, &litany, &bl, &bz);
        });
        crate::paint_probe::text_of(&full)
    };
    frame();
    let hovered = frame();
    assert!(
        hovered.contains("c-1-20260803T045643Z-1e5f99d4"),
        "the full chained id rides the hover:\n{hovered}"
    );
}
