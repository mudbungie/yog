//! **What a detached launch produced** (DESIGN §8.1, §13.3, bl-b95e) — the
//! state a `-2` row's failure is derived from, in place of what its sink said.
//!
//! §13.3's `driver.log` ruling is the general one: *"The log is append-only
//! across launches, so its **content** is never the trigger — a stale line from
//! a healed crash must not alarm — only the diagnosis."* The §8.1 stderr sink
//! was the one capture file yog still read the other way round: a byte in it
//! made the row a rendered failure, and the marker table (`opslog::notice`,
//! bl-1296) existed only to hold back the benign lines that reading swept up.
//! A phrase table over sentences lernie is free to reword is not a classifier,
//! and it could never reach the defect underneath it — the sink is append-only
//! for the driver's whole life, so **one** unrecognized line held that row red
//! for every later sweep, however many turns the driver went on to run.
//!
//! The **state** is the orphan's own pair ([`crate::steps_view::orphan`], the
//! template), asked of the thing the launch was fired to produce:
//!
//! - **Nobody is driving it** — the target agent's §3.5 lock is free
//!   ([`AgentState::driven`]). A driver at work is the answer to "did it
//!   survive", so nothing else is asked.
//! - **It has not acted since the launch** — every matching agent's
//!   `last_action_unix` predates the row's own `ts`. A conversation that was
//!   born, or a step that was written, is the launch's product; whatever went
//!   wrong *after* that belongs to the wound and the orphan, which have a
//!   surface to say it on.
//!
//! Both hold **vacuously when the target does not exist at all**, which is the
//! class the sink was added for (bl-4895): a `lernie prompt` whose driver died
//! before writing a branch leaves no conversation, no step and no transcript —
//! nothing but its ops row and its sink.
//!
//! Nothing is stored and no new signal is introduced. The verdict is re-derived
//! per sweep from the already-derived §3.5 trees, and the sink is read **only**
//! when it holds — a healthy launch pays no syscall for it, exactly as a
//! healthy conversation pays none for `driver.log`.
//!
//! **It waits, and it never answers over a world it cannot read.** The §7.3
//! grace window (bl-90bf, `Cadence::wound_grace`) is the same catch-up bound
//! the wound banner rides: a launch younger than it has not had time to produce
//! anything, and the rising edge of a healthy start is indistinguishable from a
//! death until yog has looked again (bl-18e8). A workspace with no derived tree
//! is **no verdict** rather than a death — §10's rule against a false definite,
//! and the reason a start into a wall yog has not enumerated yet is silent.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use super::{DETACHED_EXIT, OpEntry};
use crate::git_tree::{Agent, GitTree};

/// lernie's start verb, and the flag carrying the §3.3 name the fire minted.
/// Here rather than beside the spawn for [`super::detached::sink`]'s own
/// reason: the launch writes these tokens into `ops.jsonl` and this reads them
/// back, so the two sides of that join have one home and cannot drift.
pub(crate) const PROMPT: &str = "prompt";
/// See [`PROMPT`].
pub(crate) const NAME_FLAG: &str = "--name";
/// lernie's resume verb (§8.2): `advance <workspace> <agent>`, the driver
/// launch behind a capability release and the operator's own nudge.
pub(crate) const ADVANCE: &str = "advance";

/// Which agents a detached launch was fired to move — the row's own argv, read
/// back. `Conversation` is a start, named by the §3.3 name the fire minted and
/// matched on [`Agent::name_fact`]; `Agent` is a resume, named by the id it was
/// handed.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Target {
    Conversation(String),
    Agent(String),
}

impl Target {
    /// The launch's **workspace and target**, or `None` for an argv this cannot
    /// read — a verb yog does not fire detached, and a pre-bl-08f2 `prompt`
    /// line with no `--name`. No target is **no verdict**: a launch whose
    /// product cannot be named cannot be found missing.
    ///
    /// The workspace rides second-from-last in every detached argv yog writes —
    /// [`super::detached::fold`]'s own rule, which holds across flag growth and
    /// across both verbs (`prompt … <ws> <goal>`, `advance <ws> <agent>`) — and
    /// is read first, so an argv too short to hold one leaves before the verb is
    /// consulted rather than after.
    fn of(argv: &[String]) -> Option<(PathBuf, Self)> {
        let workspace = PathBuf::from(argv.iter().rev().nth(1)?);
        // The verb is matched rather than `?`-ed: an argv too short to hold one
        // has already left above, so a second early return on the same length
        // would be a line nothing can reach.
        let target = match argv.get(1).map(String::as_str) {
            Some(PROMPT) => {
                let at = argv.iter().position(|a| a == NAME_FLAG)?;
                Self::Conversation(argv.get(at + 1)?.clone())
            }
            // `[<binary>, advance, <workspace>, <agent>]`, built literally by
            // `boundary::control::advance` and never with a flag.
            Some(ADVANCE) => Self::Agent(argv.get(3)?.clone()),
            _ => return None,
        };
        Some((workspace, target))
    }

    /// Whether this derived agent is the one the launch named.
    fn names(&self, agent: &Agent) -> bool {
        match self {
            Self::Conversation(name) => agent.name_fact().as_deref() == Some(name.as_str()),
            Self::Agent(id) => agent.agent_id == *id,
        }
    }
}

/// Whether this ops entry is a detached launch that **produced nothing** —
/// the module's whole question, and the one input to a `-2` row's failure.
///
/// `now` is the derivation's own clock reading and `grace` the §7.3 window
/// ([`Cadence::wound_grace`](crate::app::Cadence::wound_grace)); `trees` is
/// the pass's already-derived §3.5 forest, keyed by workspace path.
pub(crate) fn stillborn(
    trees: &HashMap<PathBuf, GitTree>,
    entry: &OpEntry,
    now: i64,
    grace: Duration,
) -> bool {
    if entry.exit != DETACHED_EXIT {
        return false;
    }
    let Ok(ts) = entry.ts.parse::<i64>() else {
        return false;
    };
    if now.saturating_sub(ts) < i64::try_from(grace.as_secs()).unwrap_or(i64::MAX) {
        return false;
    }
    let Some((workspace, target)) = Target::of(&entry.argv) else {
        return false;
    };
    let Some(tree) = trees.get(&workspace) else {
        return false;
    };
    tree.agents
        .iter()
        .filter(|agent| target.names(agent))
        .all(|agent| !agent.state.driven() && agent.last_action_unix < ts)
}

#[cfg(test)]
mod tests;
