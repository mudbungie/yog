//! Tables for the census folds ([`super`], bl-b4b5): the §3.6 gate and the §3.3
//! occupied set, read off an answered forest.
//!
//! The *parity* half — that these say what the engine's own
//! [`liveness`](crate::nav::convs::liveness) and
//! [`names_in`](crate::boundary::answer::names_in) say — is asserted here for
//! liveness (both take the same agent set, one through `forest_rows`) and in
//! `app::balls::tests::convball` for the names, where a real tree exists.

use super::*;
use crate::git_tree::AgentState;
use crate::nav::convs::expand::forest_rows;
use crate::nav::convs::tests::{agent, plain, unseen};

/// Two conversations: one root with two members — the deeper of which is the
/// only thing running — and a second, quiet, root that carries a stored name.
fn family() -> [crate::git_tree::Agent; 4] {
    let mut named = agent("s-0", AgentState::Quiescent, 90);
    named.name = Some("pennant".to_owned());
    let mut uncertain = agent("r-0-b-1", AgentState::Quiescent, 12);
    uncertain.state_uncertain = true;
    [
        agent("r-0", AgentState::Quiescent, 10),
        agent("r-0-a-1", AgentState::Quiescent, 11),
        uncertain,
        named,
    ]
}

fn forest(agents: &[crate::git_tree::Agent]) -> Vec<ConvRow> {
    forest_rows(agents, "/ws", &unseen, 100, &plain, &[])
}

/// One entry per **root**, and a conversation is live when anything in its own
/// subtree is — the §10 uncertainty counting as live, so the gate fails closed.
/// A member's liveness must not leak into the conversation beside it.
#[test]
fn the_gate_reads_each_conversations_own_subtree_and_fails_closed() {
    let agents = family();
    let rows = forest(&agents);
    let seat = liveness_of_rows(&rows);
    assert_eq!(seat.len(), 2, "one entry per root, never per member");
    // Two projections of one derivation: the engine folds the agent set, the
    // seat folds the rows answered from it, and they may not disagree about
    // *what* is live. They do differ in order, and lawfully: the answer is
    // sorted by recency (§11) while the agent set is in §2.3 descent order, so
    // the comparison is over the set rather than the sequence.
    let sorted = |mut v: Vec<crate::nav::convs::Conversation>| {
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    };
    assert_eq!(
        sorted(seat.clone()),
        sorted(crate::nav::convs::liveness(&agents))
    );
    let by_name = |name: &str| seat.iter().find(|c| c.name == name).cloned();
    assert!(
        by_name("r-0").expect("the first root").live,
        "an unobservable member counts as live"
    );
    assert!(
        !by_name("pennant").expect("the named root").live,
        "the quiet conversation beside it is not"
    );
}

/// The mint's occupied set is every **member's** own stored name, not the
/// roots' — lernie refuses a name any living agent wears, so a named child must
/// count. A row with no stored name contributes nothing.
#[test]
fn the_occupied_set_is_every_members_stored_name() {
    let rows = forest(&family());
    assert_eq!(names_in_rows(&rows), ["pennant".to_owned()]);
    assert!(
        names_in_rows(&[]).is_empty(),
        "an unanswered forest occupies nothing"
    );
}

/// The subtree run is the row itself plus everything deeper below it, and it
/// stops at the next row of its own depth — which is what keeps one
/// conversation's members out of the next one's census.
#[test]
fn a_subtree_is_the_row_and_the_deeper_run_under_it() {
    let rows = forest(&family());
    // The answer is sorted by recency, so the row a subtree starts at is found
    // by id rather than assumed — which is the seat's own reading of it.
    let at = |id: &str| {
        rows.iter()
            .position(|r| r.root_id == id)
            .unwrap_or(rows.len())
    };
    let ids =
        |id: &str| -> Vec<String> { subtree(&rows, at(id)).map(|r| r.root_id.clone()).collect() };
    assert_eq!(
        ids("r-0"),
        ["r-0", "r-0-a-1", "r-0-b-1"].map(str::to_owned),
        "the root takes its whole descent and stops at the next root"
    );
    assert_eq!(
        ids("r-0-a-1"),
        ["r-0-a-1".to_owned()],
        "a leaf is its own subtree"
    );
    assert!(ids("nobody").is_empty(), "a row nothing answers has none");
}
