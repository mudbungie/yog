//! STORIES **S6-T1** attention-predicates: one fixture per §6 rule, each
//! asserted true, then false once the matching watermark is written — except
//! the two that no acknowledgement may answer.
//!
//! **The row's premise drifted: §6 has SIX rules, not five.** Rule 6 (`held`)
//! landed with the capability boundary (§8.6, S15–S17): a tool invocation
//! parked at the control. It joins rule 5 (mail) as a signal a watermark cannot
//! clear — "each self-clears when the world moves (a driver drains the inbox;
//! litany lifts the hold mark on the answer's re-adjudication), and no watermark
//! may pretend to answer them" (`attention::evidence`).
//!
//! Rule 2 carries **two** fixtures since bl-2194 widened it to *rest*: the
//! wounded rest (a failed latest step ⇒ `Stopped`) and the clean one (a complete
//! `response.json` ⇒ `Quiescent`). Both stir, both clear on the same tip
//! watermark, and only the state badge tells them apart.

#![allow(clippy::unwrap_used)]
// The fixture lookup below is a free helper, not a `#[test]` fn, so clippy's
// allow-*-in-tests does not reach it; it panics like any test would.
#![allow(clippy::panic)]

use crate::support::{AgentFixture, build_agents, write_deposit};
use tempfile::tempdir;
use yog::attention::{self, Attention};
use yog::git_tree::{Agent, AgentState, GitTree};

/// Marks that suppress every §6 rule but the one under test: `abandoned` is
/// rule 2's own suppressor (the will-not-retry assertion), so an agent carrying
/// it stirs only for the signal its fixture adds.
const QUIET: &str = "abandoned";

/// The workspace seen-key (§4.1) — any stable string; the predicate only keys on it.
const WS: &str = "/ws/cobalt";

fn agent_named(tree: &GitTree, id: &str) -> Agent {
    tree.agents
        .iter()
        .find(|a| a.agent_id == id)
        .unwrap_or_else(|| panic!("fixture agent {id} missing"))
        .clone()
}

/// The predicate with nothing acknowledged.
fn unacked(agent: &Agent) -> Attention {
    attention::attention(agent, WS, &|_, _, _, _| false)
}

/// The predicate with **every** watermark this agent could ever carry written —
/// `attention::evidence` is the authority on what those are, so a new
/// watermarkable rule is covered here by construction.
fn fully_acked(agent: &Agent) -> Attention {
    let acked = attention::evidence(agent);
    attention::attention(agent, WS, &|kind, _, _, oid| {
        acked.iter().any(|(k, o)| *k == kind && o == oid)
    })
}

/// STORIES **S6-T1** attention-predicates.
#[test]
fn s6_t1_every_rule_fires_and_only_the_answerable_ones_clear() {
    let root = tempdir().unwrap();
    let ws = root.path().join("cobalt");
    std::fs::create_dir_all(&ws).unwrap();
    build_agents(
        &ws,
        &[
            // Rule 1 — an unacknowledged notify.
            AgentFixture::new("n-001", "notify\n")
                .settled(true)
                .mark(QUIET)
                .mark("notify"),
            // Rule 2a — the WOUNDED rest: a failed latest step.
            AgentFixture::new("w-001", "wounded\n").settled(false),
            // Rule 2b — the CLEAN rest: a complete latest step.
            AgentFixture::new("q-001", "quiescent\n").settled(true),
            // Rule 3 — the spend ceiling.
            AgentFixture::new("b-001", "budget\n")
                .settled(true)
                .mark(QUIET)
                .mark("budget-exhausted"),
            // Rule 4 — a declined work-product transfer.
            AgentFixture::new("x-001", "conflicted\n")
                .settled(true)
                .mark(QUIET)
                .mark("conflicted"),
            // Rule 5 — pending mail nobody is driving (its deposit lands below).
            AgentFixture::new("m-001", "mail\n")
                .settled(true)
                .mark(QUIET),
            // Rule 6 — a tool invocation parked at the capability boundary.
            AgentFixture::new("h-001", "held\n")
                .settled(true)
                .mark(QUIET)
                .held("tu_1", "bash", "loss-shaped: rm -rf"),
        ],
    );
    write_deposit(&ws, "m-001", "0001", "from: peer\n\nhave a look\n");

    let tree = GitTree::from_repo(&ws).unwrap();

    // --- Rule 1: notify. Fires unseen, clears on its own oid.
    let notified = agent_named(&tree, "n-001");
    assert!(unacked(&notified).notify, "an unseen notify stirs");
    assert!(!fully_acked(&notified).notify, "the watermark clears it");

    // --- Rule 2a: the wounded rest. The state badge says Stopped …
    let wounded = agent_named(&tree, "w-001");
    assert_eq!(wounded.state, AgentState::Stopped);
    assert!(unacked(&wounded).stopped, "a wounded rest stirs");
    assert!(
        !fully_acked(&wounded).stopped,
        "the tip watermark clears it"
    );

    // --- Rule 2b: the clean rest. … and here Quiescent — but BOTH stir, on the
    // same evidence and the same watermark. A running agent is the one that is
    // not in the queue; how it came to rest is the badge's business, not §6's.
    let clean = agent_named(&tree, "q-001");
    assert_eq!(clean.state, AgentState::Quiescent);
    assert!(unacked(&clean).stopped, "a clean rest stirs too (bl-2194)");
    assert!(
        !fully_acked(&clean).stopped,
        "and clears on the same watermark"
    );
    assert_eq!(
        attention::rest_evidence(&wounded).is_some(),
        attention::rest_evidence(&clean).is_some(),
        "one evidence rule for both kinds of rest"
    );

    // --- Rule 3: budget.
    let budget = agent_named(&tree, "b-001");
    assert!(unacked(&budget).budget);
    assert!(!fully_acked(&budget).budget);

    // --- Rule 4: conflicted.
    let conflicted = agent_named(&tree, "x-001");
    assert!(unacked(&conflicted).conflicted);
    assert!(!fully_acked(&conflicted).conflicted);

    // --- Rule 5: pending mail nobody is driving. Deliberately NOT silenceable —
    // a stall you can dismiss is a stall you will miss.
    let mail = agent_named(&tree, "m-001");
    assert_eq!(mail.pending.len(), 1, "the deposit is on disk");
    assert!(unacked(&mail).mail);
    assert!(
        fully_acked(&mail).mail,
        "no watermark answers a stall; only a driver does"
    );
    // Its evidence set is EMPTY — the mail contributes nothing to acknowledge
    // (its rest is abandoned, and mail carries no oid of its own). That is the
    // structural reason no watermark can pretend to answer a stall.
    assert!(
        attention::evidence(&mail).is_empty(),
        "mail is not watermarkable: {:?}",
        attention::evidence(&mail)
    );
    // It self-clears when a driver takes the executor lock (§2.11) — which is an
    // open fd on the agent's inbox directory, so holding one here makes THIS
    // process the driver the probe finds in /proc.
    let driver = std::fs::File::open(ws.join("inbox").join("m-001")).unwrap();
    let driven = agent_named(&GitTree::from_repo(&ws).unwrap(), "m-001");
    assert!(
        !unacked(&driven).mail,
        "someone is driving it, so the mail is not stalled"
    );
    drop(driver);

    // --- Rule 6: the park. Answered, never acknowledged (§8.6).
    let held_agent = agent_named(&tree, "h-001");
    let held = held_agent
        .held
        .clone()
        .expect("the hold mark parsed off its blob");
    assert_eq!(held.tool, "bash");
    assert_eq!(held.tool_use_id, "tu_1");
    assert!(unacked(&held_agent).held);
    assert!(
        fully_acked(&held_agent).held,
        "an answer lifts a hold; an acknowledgement never does"
    );

    // Its evidence set is EMPTY: a park offers a watermark nothing to bite on,
    // so "an acknowledgement cannot clear it" is a property of the data, not a
    // branch someone remembered to write.
    assert!(
        attention::evidence(&held_agent).is_empty(),
        "a park is not watermarkable: {:?}",
        attention::evidence(&held_agent)
    );
}
