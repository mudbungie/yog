//! Tables for the selection's fold ([`super::super::select`], bl-48ae): what a
//! seat reads about the row it has picked, out of the forest the boundary
//! answered. The *parity* half — that these are the same facts
//! `boundary::answer::agent` derives — is pinned where a snapshot exists, in
//! `boundary::answer::agent::tests`; what is pinned here is the fold itself and
//! the answer it gives for a selection the forest does not carry.

use super::*;
use crate::nav::convs::expand::forest_rows;
use crate::nav::convs::{ConvRow, selection};

/// Three generations under one root, plus a second root that leads on recency —
/// so the chain a jump unfolds has to be read past an intervening conversation.
fn family() -> [Agent; 5] {
    [
        agent("r-0", AgentState::Quiescent, 10),
        agent("r-0-a-1", AgentState::Quiescent, 11),
        agent("r-0-b-1", AgentState::Live, 12),
        agent("r-0-b-1-x-2", AgentState::Quiescent, 13),
        agent("s-0", AgentState::Quiescent, 90),
    ]
}

fn forest(agents: &[Agent]) -> Vec<ConvRow> {
    forest_rows(agents, "/ws", &unseen, 100, &plain, &[])
}

/// The chain §11's visible-selection invariant unfolds is the shallower rows
/// above the selection, outermost first — the depth rule
/// [`parent_of`](crate::nav::convs::parent_of) reads one generation at a time,
/// iterated to the root.
#[test]
fn the_chain_above_a_member_is_every_shallower_row_over_it() {
    let rows = forest(&family());
    let deep = selection(&rows, "r-0-b-1-x-2");
    assert_eq!(deep.ancestors, ["r-0".to_owned(), "r-0-b-1".to_owned()]);
    assert_eq!(deep.root, "r-0");
    // A sibling at depth 1 opens one generation, not its cousin's.
    assert_eq!(selection(&rows, "r-0-a-1").ancestors, ["r-0".to_owned()]);
    // A root opens nothing, and is its own conversation.
    let root = selection(&rows, "s-0");
    assert!(root.ancestors.is_empty(), "a root opens nothing");
    assert_eq!(root.root, "s-0");
}

/// The name and the flight class are the **conversation's** — the root row's —
/// while the §8.2 gates are the selection's own. That split is what the §11
/// centre paints: one heading per conversation, one menu per member.
#[test]
fn the_name_is_the_conversations_and_the_gates_are_the_members_own() {
    let mut agents = family();
    if let Some(root) = agents.first_mut() {
        root.name = Some("pennant".to_owned());
    }
    let rows = forest(&agents);
    let member = selection(&rows, "r-0-b-1");
    assert_eq!(
        member.name, "pennant",
        "the conversation's name, not its own"
    );
    assert!(!member.display_only);
    assert!(member.present);
    // `r-0-b-1` is the one holding a driver, so Stop is offered on it and on
    // nothing else in the conversation.
    assert!(member.stoppable);
    assert!(
        !selection(&rows, "r-0").stoppable,
        "a quiet root kills nothing"
    );
    // The cascade is the looser prefix test: the root and the middle member
    // both have something under them, the leaf does not.
    assert!(selection(&rows, "r-0").stop_children);
    assert!(member.stop_children);
    assert!(!selection(&rows, "r-0-b-1-x-2").stop_children);
    // The legacy §3.3 rung is stated as the root's, so no seat reads an
    // unaddressable name as an addressable one.
    let mut legacy = family();
    if let Some(root) = legacy.first_mut() {
        root.goal_name = Some("relic".to_owned());
    }
    let legacy = selection(&forest(&legacy), "r-0-a-1");
    assert_eq!(legacy.name, "relic");
    assert!(legacy.display_only);
}

/// **Absence is a value, not a refusal** — `answer::agent`'s own ruling, kept at
/// this altitude: a selection the answer does not carry reads as its own root,
/// named by the ladder's floor, present nowhere and gating nothing. An
/// unanswered wire is the same shape at zero rows.
#[test]
fn a_selection_the_forest_does_not_carry_is_answered_as_itself() {
    for rows in [forest(&family()), Vec::new()] {
        let ghost = selection(&rows, "20260801T000000Z-gh0");
        assert_eq!(ghost.agent_id, "20260801T000000Z-gh0");
        assert_eq!(ghost.root, ghost.agent_id, "it roots itself");
        assert!(ghost.ancestors.is_empty());
        assert_eq!(ghost.name, "20260801T000000Z-gh0", "the ladder's floor");
        assert!(!ghost.display_only);
        assert_eq!(ghost.flight, None);
        assert!(!ghost.present, "nothing to message");
        assert!(!ghost.stoppable && !ghost.stop_children);
    }
}

/// The live class is the **conversation's** rollup, so every member of one
/// conversation reads the same class — the §5.1 #28 fact the header states once.
#[test]
fn the_flight_class_is_the_conversations_and_every_member_reads_it() {
    let mut agents = family();
    if let Some(kid) = agents.get_mut(3) {
        kid.state = AgentState::InFlight;
    }
    let rows = forest(&agents);
    let class = selection(&rows, "r-0").flight;
    assert_eq!(class, Some(crate::nav::convs::Flight::Inference));
    for id in ["r-0-a-1", "r-0-b-1", "r-0-b-1-x-2"] {
        assert_eq!(
            selection(&rows, id).flight,
            class,
            "one conversation, one class"
        );
    }
    assert_eq!(
        selection(&rows, "s-0").flight,
        None,
        "the other one is at rest"
    );
}
