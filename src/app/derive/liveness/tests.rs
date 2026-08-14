//! The re-probe predicate: which trees the §7.2 sweep must look at again.

use super::needs_liveness_reprobe;
use crate::app::tests::harness::agent;
use crate::git_tree::{AgentState, GitTree};

/// Only a live agent can die *silently*, so only a live agent's workspace is
/// worth a poll. Asserted in both directions and on the empty tree, because
/// "re-probe everything" and "re-probe nothing" are the two ways this goes
/// wrong and each is invisible to the other's test.
#[test]
fn it_fires_only_on_live_or_in_flight() {
    let live = GitTree {
        commits: vec![],
        agents: vec![agent("x", AgentState::Live)],
    };
    let inflight = GitTree {
        commits: vec![],
        agents: vec![agent("y", AgentState::InFlight)],
    };
    let quiescent = GitTree {
        commits: vec![],
        agents: vec![agent("z", AgentState::Quiescent)],
    };
    assert!(needs_liveness_reprobe(&live));
    assert!(needs_liveness_reprobe(&inflight));
    assert!(!needs_liveness_reprobe(&quiescent));
    assert!(!needs_liveness_reprobe(&GitTree::default()));
}
