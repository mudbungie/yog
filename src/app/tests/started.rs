//! The §3.4 **start claim**: a fired start focuses the conversation it started.
//!
//! The regression these pin (bl-49cb): Enter in the composer minted, stamped and
//! fired a conversation, and the center pane stayed on the new-conversation
//! placeholder — `prepare_start` adopted the *workspace* and nothing adopted the
//! **conversation**, so the reply streamed unwatched in the roster and the
//! operator had to press ↓ onto their own start. The claim cannot be spent at
//! the fire: the root has no `agents/<id>` ref until the detached driver writes
//! one, so it is held by the minted §3.3 name and spent by the frame's
//! [`AppModel::refresh`] on the first roster whose root wears the name fact —
//! the lernie-stored `name` blob `--name` committed (bl-6920: the goal itself
//! carries no stamp).

use super::Harness;
use crate::nav::ws_key;
use crate::ui_state::SeenKind;
use crate::watch::Mark;
use std::time::Duration;

/// Land a root wearing `name` the way a modern fire does (bl-6920): the goal
/// verbatim on the branch, the name as the lernie-committed `name` blob the
/// derivation reads back as its `name_fact`.
fn build_named(h: &Harness, id: &str, name: &str) {
    h.build_more(id, "fix the gate");
    h.fx.name_agent(id, name);
}

#[test]
fn a_fired_start_focuses_the_conversation_it_started() {
    let h = Harness::new();
    let (clock, mut model) = h.model();
    // Where the operator stands the instant Enter is pressed: the workspace is
    // focused (`prepare_start`'s adoption), nothing is selected.
    model.focus_workspace(&crate::naming::leaf(&h.ws));
    assert!(model.focus().agent.is_none(), "the placeholder's state");

    model.await_conversation(&h.ws, "stench-pug", "fix the gate");
    // The driver has not written the branch yet, and the claim focuses anyway
    // (bl-2e8f): the echo's own row wears the minted name, so there IS something
    // to select the instant Enter lands.
    assert_eq!(
        model.focus().agent.as_deref(),
        Some("stench-pug"),
        "the start selects what it started, by the name it minted"
    );

    // The detached driver writes the root, wearing the minted name blob.
    build_named(&h, "c-2", "stench-pug");
    model.dirty_handle().mark_all([(h.ws.clone(), Mark::Watch)]);
    model.tick(); // the mark enters the worker's coalescing window
    clock.advance(Duration::from_millis(150));
    assert!(model.tick(), "the roster now carries the started root");

    assert_eq!(
        model.focus().agent.as_deref(),
        Some("c-2"),
        "the start's own conversation is what renders — no ↓, no click"
    );
    // Landing is the ordinary `focus_agent` path the ↓ key takes, so it
    // acknowledges (§6) exactly as an arrival by any other hand does.
    let tip = model
        .tree(&h.ws)
        .and_then(|t| t.agents.iter().find(|a| a.agent_id == "c-2").cloned())
        .map(|a| a.tip_oid)
        .expect("the started root is in the tree");
    assert!(
        model.is_seen(SeenKind::Stopped, &ws_key(&h.ws), "c-2", &tip),
        "arriving on it recorded its watermark"
    );
}

#[test]
fn the_claim_is_spent_once_and_leaves_later_focus_alone() {
    let h = Harness::new();
    let (clock, mut model) = h.model();
    build_named(&h, "c-2", "stench-pug");
    model.dirty_handle().mark_all([(h.ws.clone(), Mark::Watch)]);
    model.tick();
    clock.advance(Duration::from_millis(150));
    model.tick();

    model.await_conversation(&h.ws, "stench-pug", "fix the gate");
    model.refresh();
    assert_eq!(model.focus().agent.as_deref(), Some("c-2"), "claimed");

    // Spent: the operator walks off it and no later frame drags them back.
    model.focus_agent(&h.ws, "c-1");
    model.refresh();
    assert_eq!(
        model.focus().agent.as_deref(),
        Some("c-1"),
        "the claim was spent on its arrival, not held"
    );
}

/// **The claim is spent where it was made, and nowhere else** (bl-56c6): a
/// start can take a minute to write its branch, and an operator who reads
/// something else in that minute is not dragged back by their own conversation
/// arriving. §3.4's rule is *a start focuses what it started* — the selection
/// it makes at the fire — not *a start outranks whatever you did next*.
///
/// The claim still **spends**: the echo takes the id either way, so nothing is
/// left holding a name that will never match again.
#[test]
fn navigation_during_the_window_outranks_the_arriving_claim() {
    let h = Harness::new();
    let (clock, mut model) = h.model();
    model.await_conversation(&h.ws, "stench-pug", "fix the gate");
    assert_eq!(
        model.focus().agent.as_deref(),
        Some("stench-pug"),
        "the fire's own selection, made at once (bl-2e8f)"
    );

    // Mid-window, the operator goes to read something else.
    model.focus_agent(&h.ws, "c-1");
    build_named(&h, "c-2", "stench-pug");
    model.dirty_handle().mark_all([(h.ws.clone(), Mark::Watch)]);
    model.tick();
    clock.advance(Duration::from_millis(150));
    assert!(model.tick(), "the roster now carries the started root");

    assert_eq!(
        model.focus().agent.as_deref(),
        Some("c-1"),
        "where they went is where they are"
    );
    // And it is spent, not merely deferred: the operator's own arrival on the
    // started conversation later is theirs, and no frame re-aims anything.
    model.focus_workspace(&crate::naming::leaf(&h.ws));
    model.refresh();
    assert!(
        model.focus().agent.is_none(),
        "a spent claim has no name left to match"
    );
}

#[test]
fn a_claim_whose_root_never_appears_stays_on_the_name_it_minted() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    model.focus_agent(&h.ws, "c-1");
    // A start whose driver died before writing its branch, and a workspace with
    // no tree at all — both are the general path with the name fact absent.
    model.await_conversation(&h.ws, "never-written", "fix the gate");
    model.refresh();
    assert_eq!(
        model.focus().agent.as_deref(),
        Some("never-written"),
        "the claim's own selection stands: a root that never lands is a faded \
         row (§7.2), not a focus that snaps back to the conversation before it"
    );
    model.await_conversation(
        std::path::Path::new("/no/such/ws"),
        "stench-pug",
        "fix the gate",
    );
    model.refresh();
    assert_eq!(
        model.focus().agent.as_deref(),
        Some("stench-pug"),
        "an unresolvable claim is inert — nothing spends it, and nothing \
         re-aims the focus it made"
    );
}
