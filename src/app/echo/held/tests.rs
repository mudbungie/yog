//! The §3.4 start window's own contract (bl-56c6): what a send aimed at an
//! unresolved mint does, when it leaves, and what the swap hands the seats.
//! The paint-layer proof that an operator sees it is
//! `shell::acceptance::echo::window`; these pin the decisions that beat rides
//! on.

use crate::app::tests::Harness;
use crate::boundary::{Action, Gesture, codec};
use crate::watch::Mark;
use std::time::Duration;

/// Every act the frame posted, decoded — the acts path stood in for exactly as
/// `app::acts::tests` does it.
fn posted(outbox: &crate::wire::post::Outbox) -> Vec<Action> {
    let mut out = Vec::new();
    while let Some((_, envelope)) = outbox.try_next() {
        if let Ok(Gesture::Act(action)) = codec::decode(&envelope) {
            out.push(action);
        }
    }
    out
}

/// **A send at the pending mint is held, and only there.** The name resolves
/// nowhere until the branch exists, so nothing may be aimed at it — and every
/// other target is untouched, which is the whole of what keeps this from being
/// a mode.
#[test]
fn a_send_at_the_unresolved_mint_is_taken_and_every_other_send_is_not() {
    let h = Harness::new();
    let (_c, mut rig) = h.model();
    assert!(
        !rig.model.hold_send(&h.ws, "stench-pug", "too early"),
        "with nothing started there is no mint to aim at"
    );

    rig.model
        .await_conversation(&h.ws, "stench-pug", "open the gate");
    assert!(
        !rig.model.hold_send(&h.ws, "c-1", "hello"),
        "a conversation the world carries is addressed the ordinary way"
    );
    assert!(
        !rig.model
            .hold_send(std::path::Path::new("/elsewhere"), "stench-pug", "hello"),
        "an echo belongs to the workspace it was fired in"
    );
    assert!(
        rig.model
            .hold_send(&h.ws, "stench-pug", "and another thing")
    );

    // What it took is one more deposit on the same echo — no second concept,
    // and the §11 queue seat carries it in the order it was said.
    let queued = rig.model.echoed_pending("stench-pug", Vec::new());
    let said: Vec<String> = queued.iter().map(|e| e.deposit.body.clone()).collect();
    assert_eq!(said, ["open the gate", "and another thing"]);
    assert!(
        queued.iter().all(crate::inboxview::InboxEntry::in_memory),
        "both are yog's own word, not disk's"
    );
}

/// **The claim resolving is what posts them**, in the order they were said and
/// addressed by the id the branch brought — never by the name, which is the
/// whole point. And the hold is spent: a second resolution posts nothing again.
#[test]
fn the_held_sends_go_out_when_the_start_resolves_and_in_order() {
    let h = Harness::new();
    let (clock, mut rig) = h.model();
    let (post, outbox) = crate::wire::post::pair();
    rig.model.adopt_post(post);

    rig.model
        .await_conversation(&h.ws, "stench-pug", "open the gate");
    assert!(rig.model.hold_send(&h.ws, "stench-pug", "first"));
    assert!(rig.model.hold_send(&h.ws, "stench-pug", "second"));
    rig.model.refresh();
    assert!(
        posted(&outbox).is_empty(),
        "nothing is fired at a name that resolves nowhere"
    );

    // The detached driver writes the root, wearing the minted name blob.
    h.build_more("c-2", "open the gate");
    h.fx.name_agent("c-2", "stench-pug");
    rig.model
        .dirty_handle()
        .mark_all([(h.ws.clone(), Mark::Watch)]);
    rig.tick();
    clock.advance(Duration::from_millis(150));
    assert!(rig.tick(), "the roster carries the started root");

    let acts = posted(&outbox);
    assert_eq!(acts.len(), 2, "both, and neither twice: {acts:?}");
    for (action, said) in acts.iter().zip(["first", "second"]) {
        match action {
            Action::Message { agent, content, .. } => {
                assert_eq!(agent, "c-2", "addressed by the id, never the minted name");
                assert_eq!(content, said, "and in the order the operator said them");
            }
            other => panic!("a held send is a message: {other:?}"),
        }
    }
    rig.model.refresh();
    assert!(
        posted(&outbox).is_empty(),
        "the hold is spent: a later frame posts nothing again"
    );
}

/// **The swap the composer's draft key has to follow.** `None` until the claim
/// resolves and `None` again once the echo retires, so the seat that acts on it
/// may ask every frame.
#[test]
fn the_name_to_id_swap_is_readable_for_exactly_the_echos_remaining_life() {
    let h = Harness::new();
    let (clock, mut rig) = h.model();
    assert_eq!(rig.model.adopted_names(), None, "nothing started");

    rig.model
        .await_conversation(&h.ws, "stench-pug", "open the gate");
    rig.model.refresh();
    assert_eq!(
        rig.model.adopted_names(),
        None,
        "unresolved: the box is still keyed by the name and nothing has moved"
    );

    h.build_more("c-2", "open the gate");
    h.fx.name_agent("c-2", "stench-pug");
    rig.model
        .dirty_handle()
        .mark_all([(h.ws.clone(), Mark::Watch)]);
    rig.tick();
    clock.advance(Duration::from_millis(150));
    rig.tick();
    assert_eq!(
        rig.model.adopted_names(),
        Some(("stench-pug".to_owned(), "c-2".to_owned())),
        "the swap, for as long as the echo it belongs to is alive"
    );

    // A follow-up's echo was born addressing an id, so it never swapped.
    rig.model.await_message(&h.ws, "c-1", "and again", 0);
    assert_eq!(rig.model.adopted_names(), None);
}
