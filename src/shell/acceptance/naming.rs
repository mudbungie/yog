//! The §3.3 naming invariant, machine-held (bl-df72): **no seat formats an
//! agent id as a display name**. The operator met the violation as "the agent
//! list at the top is named just some incoherent timestamp" — the descent-tree
//! member row painting `agent_id` raw. The one naming rule is the display
//! ladder (`crate::nav::convs::display_name` and its seats); its floor may be
//! the id — an id is a fact — but only the ladder spells it, and the id's
//! other seat is the hover. A per-seat fix would leave the next seat equally
//! free to leak, so the rule is held here in the [`super::hover`] idiom: read
//! the tree's own source and fail the seat, reachable by a fixture or not.

use super::hover::lex::skeleton;
use super::hover::scan::{args_of, rust_files, sites};
use crate::cli_outbound::Cli;
use std::path::Path;

/// Constructors whose argument the operator reads as a title or label — the
/// text-painting counterpart of the hover scan's interactive `CONTROLS`.
const PAINTS: &[&str] = &[
    "ui.label(",
    "ui.heading(",
    "ui.weak(",
    "ui.strong(",
    "ui.small(",
    "ui.monospace(",
    "ui.colored_label(",
    "ui.selectable_label(",
    "ui.selectable_value(",
    "ui.button(",
    "ui.small_button(",
    "RichText::new(",
];

/// The identifiers an id travels under in this tree — `agent_id` on the
/// [`crate::git_tree::Agent`], `root_id` on the row and the ladder's own
/// seats. bl-63a1's lesson: the first scan forbade only `agent_id`, so a seat
/// painting the same fact under its other name would have passed; the set now
/// names both spellings of the one fact.
const ID_IDENTS: &[&str] = &["agent_id", "root_id"];

/// **The invariant.** No paint call's argument span spells an id identifier: a
/// seat that wants a name calls the ladder before it paints, so an id can only
/// reach the screen as the ladder's own floor — which since bl-63a1 spells the
/// terminal generation only (`nav::convs`'s `id_floor`; the floor rule itself
/// is held by the ladder's unit tests). Enumerating nothing is itself a
/// failure — the same two-direction discipline as the hover scan, so a rotted
/// pattern list cannot pass by matching zero call sites.
#[test]
fn no_paint_seat_spells_an_agent_id_as_its_text() {
    let mut leaks = Vec::new();
    let mut seen = 0usize;
    for file in rust_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src")) {
        let source = std::fs::read_to_string(&file).unwrap();
        let skeleton = skeleton(&source);
        for (at, paint) in sites(&skeleton, PAINTS) {
            seen += 1;
            if ID_IDENTS
                .iter()
                .any(|ident| args_of(&skeleton, at).contains(ident))
            {
                leaks.push(format!("{}: {paint}", file.display()));
            }
        }
    }
    assert!(
        seen > 50,
        "the scan matched {seen} paint calls — the pattern list has rotted"
    );
    assert!(
        leaks.is_empty(),
        "these seats paint an agent id as display text — §3.3: every title rides \
         the display ladder, and the id's seats are the ladder's floor and the \
         hover:\n{}",
        leaks.join("\n")
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
    let (lernie, bl) = (Cli::new("lernie"), Cli::new("bl"));
    let mut world = super::fixture::world();
    let ws = world.ws.clone();
    // Debounce off (a legal cadence — the bounds floor at zero), so the marked
    // workspace derives on the very next pass instead of a wall-clock sleep.
    std::fs::write(
        world.model.state_root().join("cadence.yaml"),
        "cadence:\n  watcher:\n    debounce_ms: 0\n",
    )
    .unwrap();
    world.model.after_lernie_verb();
    world.converge();
    // A descent child whose id embeds the ancestry chain, with no name blob,
    // no goal and no step record — nothing above the ladder's floor.
    world.add_child("c-1", "c-1-20260803T045643Z-1e5f99d4");
    world.model.mark_dirty([ws.clone()]);
    world.converge();
    world.model.focus_agent(&ws, "c-1");
    // Unfold the parent: since bl-fa82 a member is a row of the conversation
    // list, and since bl-8905 that is the only place it paints.
    world.state.expanded.insert("c-1".to_owned());
    let out = super::painted(&mut world, &lernie, &bl);
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
        let full = ctx.run(super::input(), |ctx| {
            super::super::render(ctx, &mut world.model, &mut world.state, &lernie, &bl, &bz);
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
