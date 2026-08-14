//! The §9.4 **drift clause and its two exits** (bl-9786, bl-2d19), driven on
//! the real window.
//!
//! The claim is a *seat*: the way out of the config freeze belongs beside the
//! sentence that states the freeze — the operator reads *this conversation is
//! frozen on …* and the verbs that answer it are the next thing on that row,
//! never a fourth control somewhere else. Only a laid-out frame can say that,
//! so these read the painted galleys and the settings panel's own rect.
//!
//! They are asserted on **painted glyphs**, not on the strings handed to the
//! widgets (bl-bc06): `Galley::text()` is the text that went *in*, so a button
//! elided to a bare `…` would pass an assertion made against it. The whole
//! value of this beat is that the exit is readable where it is offered.

use super::super::render;
use super::fixture::world;
use super::input;
use crate::cli_outbound::Cli;
use crate::model_pick::tests::TEMPLATE_PROVIDERS;
use crate::model_pick::{NEW_CONVERSATION_EXIT, RETARGET_EXIT};
use crate::paint_probe::Painted;

/// The frozen sentence's own opening — composed and asserted in
/// `model_pick::header`; what this beat owns is that it reaches the glass.
const FROZEN: &str = "this conversation is frozen on";

/// A settled window over the fixture world with the conversation selected, and
/// — when `drifted` — the workspace's config lineage advanced past it. Returns
/// the frame's galleys and the settings panel's stored rect.
fn painted_seat(drifted: bool) -> (Vec<Painted>, egui::Rect) {
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    if drifted {
        // Zero the watcher debounce so the edit below is folded in on the very
        // next pass instead of on a wall clock this test would have to sleep
        // against (`acceptance::walk`'s idiom).
        std::fs::write(
            world.model.state_root().join("cadence.yaml"),
            "cadence:\n  watcher:\n    debounce_ms: 0\n",
        )
        .unwrap();
        world.model.after_lernie_verb();
        world.converge();
        // One ordinary config edit, made after every agent forked: the
        // governing commit and the lineage tip part, which is the entire
        // condition the clause and its exits are offered under.
        world.advance_config(TEMPLATE_PROVIDERS);
        world.model.mark_dirty([ws]);
        world.converge();
    }
    let ctx = egui::Context::default();
    let (lernie, bl, bz) = (
        Cli::new("yog-absent-lernie"),
        Cli::new("yog-absent-bl"),
        Cli::new("yog-absent-bz"),
    );
    let mut out = None;
    // Four frames: panels adopt their content height a frame late, and the
    // composer's queue region settles one after that.
    for _ in 0..4 {
        out = Some(ctx.run(input(), |ctx| {
            render(ctx, &mut world.model, &mut world.state, &lernie, &bl, &bz);
        }));
    }
    let painted = crate::paint_probe::painted_of(&out.expect("four frames ran"));
    let seat =
        egui::containers::panel::PanelState::load(&ctx, egui::Id::new("conversation-settings"))
            .expect("the settings panel stores its rect")
            .rect;
    (painted, seat)
}

/// The rect of the first painted galley whose **glyphs** contain `needle`, or a
/// panic naming it.
fn one(painted: &[Painted], needle: &str) -> egui::Rect {
    painted
        .iter()
        .find(|(text, _)| text.contains(needle))
        .map_or_else(
            || panic!("{needle:?} is not on the glass"),
            |(_, rect)| *rect,
        )
}

/// Whether anything painted carries `needle`.
fn any(painted: &[Painted], needle: &str) -> bool {
    painted.iter().any(|(text, _)| text.contains(needle))
}

/// **The ruling** (bl-2d19): a drifted conversation states its freeze, and both
/// ways out are offered on that sentence's own row — the one that keeps the
/// conversation and the one that starts over — inside the settings seat, in
/// glyphs an operator can actually read.
#[test]
fn a_drifted_conversation_offers_both_exits_beside_the_frozen_sentence() {
    let (painted, seat) = painted_seat(true);

    let clause = one(&painted, FROZEN);
    let retarget = one(&painted, RETARGET_EXIT);
    let fresh = one(&painted, NEW_CONVERSATION_EXIT);

    for (what, rect) in [
        ("the frozen sentence", clause),
        ("the retarget exit", retarget),
        ("the new-conversation exit", fresh),
    ] {
        assert!(
            rect.top() >= seat.top() - 1.0,
            "{what} belongs to the settings seat: {rect:?} vs {seat:?}"
        );
    }
    // Beside the sentence, not elsewhere on the surface: with room for the
    // strip, the exit shares the clause's own row.
    assert!(
        retarget.top() < clause.bottom() && clause.top() < retarget.bottom(),
        "the retarget exit sits on the frozen sentence's own row: \
         {retarget:?} vs {clause:?}"
    );
    // And the exit that keeps the conversation leads the one that discards it.
    assert!(
        retarget.left() < fresh.left(),
        "the keeping exit comes first: {retarget:?} vs {fresh:?}"
    );
}

/// The other direction, which is what makes the beat above evidence: an
/// undrifted conversation — the ordinary case — is the bare pair and nothing
/// else. It already runs the current config, so there is nothing to escape and
/// no verb is offered for escaping it.
#[test]
fn an_undrifted_conversation_is_offered_no_exit_at_all() {
    let (painted, _) = painted_seat(false);
    assert!(!any(&painted, FROZEN), "no freeze is named");
    assert!(!any(&painted, RETARGET_EXIT), "and no way out is offered");
    assert!(!any(&painted, NEW_CONVERSATION_EXIT));
}
