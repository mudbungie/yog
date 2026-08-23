//! The pending echo's own contract (§7.2): what the fold writes, what retires
//! it, and what it refuses to touch. The paint-layer proof that an operator
//! actually sees it is `shell::acceptance::echo`; these pin the decisions that
//! beat rides on.

use super::{Echo, Target, compose};
use crate::app::Snapshot;
use crate::git_tree::{Agent, AgentState, GitTree};
use std::path::{Path, PathBuf};
use std::sync::Arc;

const WS: &str = "/ws";

fn ws() -> PathBuf {
    PathBuf::from(WS)
}

/// A derived agent: one off `for-each-ref`, so it has a tip and a count.
fn agent(id: &str, name: Option<&str>, messages: usize) -> Agent {
    Agent {
        branch_name: format!("agents/{id}"),
        agent_id: id.to_owned(),
        tip_oid: "a".repeat(40),
        tip_short_oid: "aaaaaaaa".to_owned(),
        tip_timestamp_unix: 1,
        last_action_unix: 1,
        messages,
        call_start_unix: None,
        steps: Vec::new(),
        preview: None,
        stream: crate::git_tree::Stream::default(),
        tool_calls: Vec::new(),
        state: AgentState::Quiescent,
        state_uncertain: false,
        pending: Vec::new(),
        conflicted_oid: None,
        budget_oid: None,
        abandoned_oid: None,
        notify_oid: None,
        held: None,
        goal_ball: None,
        name: name.map(str::to_owned),
        goal_name: None,
    }
}

/// A published derivation carrying `agents` under [`WS`].
fn snap(agents: Vec<Agent>) -> Arc<Snapshot> {
    let mut s = Snapshot::empty(0);
    s.trees.insert(
        ws(),
        GitTree {
            commits: Vec::new(),
            agents,
        },
    );
    Arc::new(s)
}

fn folded(derived: &Arc<Snapshot>, echo: &Echo) -> Vec<Agent> {
    compose(derived, Some(echo), None).trees[&ws()]
        .agents
        .clone()
}

#[test]
fn a_start_with_no_branch_yet_folds_in_as_a_pending_conversation() {
    let derived = snap(vec![agent("c-1", Some("other"), 2)]);
    let echo = Echo::started(Path::new(WS), "stench-pug", "open the gate", 99);
    let agents = folded(&derived, &echo);

    assert_eq!(
        agents.len(),
        2,
        "the roster gained the started conversation"
    );
    let pending = agents.last().expect("the folded row");
    assert_eq!(pending.agent_id, "stench-pug", "keyed by the minted name");
    assert_eq!(pending.name_fact().as_deref(), Some("stench-pug"));
    assert_eq!(pending.preview.as_deref(), Some("open the gate"));
    assert_eq!(pending.last_action_unix, 99, "a send is an action");
    assert!(
        pending.in_memory(),
        "no branch, no tip — which is what paints it faded (§11)"
    );
    let deposit = pending.pending.first().expect("the echo's deposit");
    assert!(deposit.in_memory(), "no file, no name");
    assert_eq!(deposit.deposit.body, "open the gate");
    assert_eq!(deposit.deposit.sender.as_deref(), Some("user"));
    // The derivation itself is untouched: the fold is the frame's copy.
    assert_eq!(derived.trees[&ws()].agents.len(), 1);
}

#[test]
fn a_follow_up_folds_onto_the_agent_it_was_aimed_at() {
    let derived = snap(vec![agent("c-1", None, 2)]);
    let echo = Echo::messaged(&derived, Path::new(WS), "c-1", "and again", 0, 42);
    assert_eq!(echo.baseline, 2, "the baseline is what has already landed");
    let agents = folded(&derived, &echo);

    assert_eq!(
        agents.len(),
        1,
        "no row is invented for an agent that exists"
    );
    assert_eq!(agents[0].pending.len(), 1, "one more undelivered deposit");
    assert!(agents[0].pending[0].in_memory());
    assert_eq!(agents[0].last_action_unix, 42, "and the row rises");
    assert!(!agents[0].in_memory(), "the conversation itself is real");
}

#[test]
fn an_echo_retires_exactly_when_its_message_lands_and_not_before() {
    let before = snap(vec![agent("c-1", None, 2)]);
    let echo = Echo::messaged(&before, Path::new(WS), "c-1", "and again", 0, 42);
    assert!(!echo.landed(&before), "nothing has landed yet");
    // Anything short of a new message file holds it — a moved tip or a
    // streaming token is not the fact the echo is waiting for.
    let mut busy = agent("c-1", None, 2);
    busy.last_action_unix = 9_999;
    busy.tip_oid = "b".repeat(40);
    assert!(
        !echo.landed(&snap(vec![busy])),
        "a commit and a fresh mtime are not the message"
    );
    assert!(
        echo.landed(&snap(vec![agent("c-1", None, 3)])),
        "the count passing the baseline is"
    );
}

#[test]
fn a_starts_target_takes_the_id_its_branch_brings_and_only_once() {
    let echo = Echo::started(Path::new(WS), "stench-pug", "open the gate", 1);
    assert_eq!(
        echo.resolved(&snap(vec![agent("c-1", Some("other"), 1)])),
        None,
        "a roster without the name resolves nothing"
    );
    let landed = snap(vec![agent("c-2", Some("stench-pug"), 0)]);
    assert_eq!(echo.resolved(&landed).as_deref(), Some("c-2"));
    assert!(
        !echo.landed(&landed),
        "and resolving is not landing: the branch is there, the message is not"
    );
    let resolved = Echo {
        target: Target::Agent("c-2".to_owned()),
        ..echo
    };
    assert_eq!(
        resolved.resolved(&landed),
        None,
        "a resolved target has no name left to match, so the claim spends once"
    );
}

#[test]
fn nothing_pending_folds_nothing_and_an_absent_target_invents_nothing() {
    let derived = snap(vec![agent("c-1", None, 2)]);
    assert!(
        Arc::ptr_eq(&compose(&derived, None, None), &derived),
        "with nothing pending the rendered snapshot IS the derivation"
    );
    // A follow-up whose agent is gone (deleted under it) folds nothing: the
    // start arm is the only one that mints a row, because it is the only one
    // whose subject legitimately has no branch yet.
    let orphan = Echo::messaged(&derived, Path::new(WS), "gone", "hello?", 0, 5);
    assert_eq!(folded(&derived, &orphan), derived.trees[&ws()].agents);
}

#[test]
fn a_start_into_a_workspace_with_no_tree_still_paints_its_row() {
    // The §3.4 bootstrap: the first conversation of a workspace the worker has
    // never derived. The general path with an empty tree, not a special case.
    let derived = Arc::new(Snapshot::empty(0));
    let echo = Echo::started(Path::new(WS), "stench-pug", "found the world", 7);
    let rendered = compose(&derived, Some(&echo), None);
    assert_eq!(rendered.trees[&ws()].agents.len(), 1);
    assert_eq!(rendered.trees[&ws()].agents[0].agent_id, "stench-pug");
}

/// **The fold stops pushing once the derivation carries the deposit** (§7.2,
/// bl-78d8). This is the same yielding the queue seat does, at the snapshot
/// altitude — and the seat that pays for it here is the `✉n` badge, which
/// counts `pending` and so counted the operator's one message twice for as long
/// as it sat undelivered. `landed` cannot cover it: the badge's own listing
/// carries the deposit an entire step boundary before `messages/` does.
#[test]
fn the_snapshot_fold_yields_its_deposit_rather_than_doubling_it() {
    let derived = snap(vec![agent("c-1", None, 2)]);
    // Queued against a roster showing no pending mail at all.
    let echo = Echo::messaged(&derived, Path::new(WS), "c-1", "and again", 0, 42);
    assert!(!echo.deposited(0), "nothing has arrived to stand for it");
    assert!(
        echo.deposited(1),
        "a longer listing is the deposit, counted"
    );

    let mut carried = agent("c-1", None, 2);
    carried.pending = vec![echo.deposit()];
    let folded = folded(&snap(vec![carried]), &echo);
    assert_eq!(
        folded[0].pending.len(),
        1,
        "the derivation's deposit stands alone: no second copy of one message"
    );
    assert_eq!(
        folded[0].last_action_unix, 42,
        "and the row still rises — the send is an action either way"
    );
    // The message has not landed, so the echo itself is still held: the two
    // predicates answer two questions and neither stands in for the other.
    assert!(!echo.landed(&derived));
}
