//! Ref-derived agent marks (§2.6, §6, ARCH §8, §3.3 — the `refs/lernie/*` set).
//!
//! Five orthogonal marks, each a git ref keyed by agent id, read once per
//! `from_repo` tick. Four are carried to the view-model as **ref oids** (not
//! mere membership booleans): the attention model (DESIGN §6) seen-gates each
//! signal on its evidence oid — a moved ref re-notifies (§4.1). The fifth,
//! `held`, carries a **value** instead, for the reason §6 rule 6 gives: a park
//! is not something an acknowledgement may quiet, so there is no watermark to
//! compare and the useful fact is the blob's content.
//!
//! - **conflicted** — `refs/lernie/conflicted/<id>`: a child's work-product
//!   transfer failed to apply and was declined loudly (§2.6).
//! - **budget-exhausted** — `refs/lernie/budget-exhausted/<id>`: the agent
//!   tree hit a spend ceiling (§6).
//! - **abandoned** — `refs/lernie/abandoned/<id>`: a policy assertion that a
//!   stopped branch will not be retried (ARCH §8 `mark_abandoned`); it
//!   suppresses the stop-attention signal (§6 rule 2).
//! - **notify** — `refs/lernie/notify/<id>`: the branch asked the UI to
//!   raise a user-facing notification (ARCH §8 `notify_ui`, §6 rule 1).
//! - **held** — `refs/lernie/held/<id>` (ARCH §3.3, DESIGN §8.6): the
//!   capability control parked a tool invocation before it executed. The ref
//!   names a blob saying which invocation and why ([`crate::control::hold`]).
//!
//! Derived from `git for-each-ref`, never stored (PRINCIPLES "Single
//! source of truth"). Every namespace keys off the raw agent id (no
//! `agents/` prefix), matching the harness's ref writers.

use super::cmd::{blob, for_each_ref_under};
use super::{Agent, GitTreeError};
use crate::control::hold::{self, HELD_PREFIX, Held};
use std::collections::HashMap;
use std::path::Path;

const CONFLICTED_PREFIX: &str = "refs/lernie/conflicted/";
const BUDGET_PREFIX: &str = "refs/lernie/budget-exhausted/";
const ABANDONED_PREFIX: &str = "refs/lernie/abandoned/";
const NOTIFY_PREFIX: &str = "refs/lernie/notify/";

/// The mark maps for a workspace: four `agent-id -> ref oid` (a membership
/// query is `oid.is_some()`; the oid itself is the §6 watermark evidence), and
/// the parked invocations keyed the same way.
#[derive(Debug, Default)]
pub(super) struct Marks {
    conflicted: HashMap<String, String>,
    budget: HashMap<String, String>,
    abandoned: HashMap<String, String>,
    notify: HashMap<String, String>,
    held: HashMap<String, Held>,
}

impl Marks {
    pub(super) fn from_repo(git_dir: &Path) -> Result<Self, GitTreeError> {
        Ok(Self {
            conflicted: oids_under(git_dir, CONFLICTED_PREFIX)?,
            budget: oids_under(git_dir, BUDGET_PREFIX)?,
            abandoned: oids_under(git_dir, ABANDONED_PREFIX)?,
            notify: oids_under(git_dir, NOTIFY_PREFIX)?,
            held: held_under(git_dir)?,
        })
    }

    pub(super) fn held(&self, agent_id: &str) -> Option<Held> {
        self.held.get(agent_id).cloned()
    }

    pub(super) fn conflicted_oid(&self, agent_id: &str) -> Option<String> {
        self.conflicted.get(agent_id).cloned()
    }

    pub(super) fn budget_oid(&self, agent_id: &str) -> Option<String> {
        self.budget.get(agent_id).cloned()
    }

    pub(super) fn abandoned_oid(&self, agent_id: &str) -> Option<String> {
        self.abandoned.get(agent_id).cloned()
    }

    pub(super) fn notify_oid(&self, agent_id: &str) -> Option<String> {
        self.notify.get(agent_id).cloned()
    }
}

/// A `refs/lernie/*` mark an agent **wears** — the durable fact, as opposed to
/// the [attention signal](crate::attention) it may be gating. One variant per
/// namespace this module reads, so the set is closed and total: a mark cannot be
/// added to [`Marks`] without a variant, and a variant cannot exist without the
/// words [`crate::theme::mark_badge`] gives it. Ordered as §6 orders its rules
/// (notify, budget, conflict) with the non-attention `abandoned` last, so every
/// seat paints marks in one order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMark {
    Notified,
    BudgetExhausted,
    Conflicted,
    /// The capability control parked an invocation before it executed (§8.6).
    /// Ordered with the attention-bearing marks because it is one — §6 rule 6.
    Held,
    Abandoned,
}

impl Agent {
    /// Every mark this agent wears, in §6 badge order (DESIGN §6, §11).
    ///
    /// The marks are read from refs, never stored, and are **not** seen-gated:
    /// acknowledging an attention signal moves a `ui.json` watermark and leaves
    /// the ref exactly where it was, which is what makes this the carrier §6
    /// promises — *"acknowledging it clears the signal, not the fact"*. Derived
    /// on demand from the oids already on the view-model, so there is nothing to
    /// keep in sync.
    pub fn marks(&self) -> Vec<AgentMark> {
        [
            (self.notify_oid.is_some(), AgentMark::Notified),
            (self.budget_oid.is_some(), AgentMark::BudgetExhausted),
            (self.conflicted_oid.is_some(), AgentMark::Conflicted),
            (self.held.is_some(), AgentMark::Held),
            (self.abandoned_oid.is_some(), AgentMark::Abandoned),
        ]
        .into_iter()
        .filter_map(|(worn, mark)| worn.then_some(mark))
        .collect()
    }
}

fn oids_under(git_dir: &Path, prefix: &str) -> Result<HashMap<String, String>, GitTreeError> {
    let out = for_each_ref_under(git_dir, prefix)?;
    Ok(parse_oids(&out, prefix))
}

/// Every parked invocation in the workspace: the `held` namespace enumerated,
/// then each oid's blob read for its value. A workspace with nothing parked —
/// the ordinary one — makes the enumeration and no `cat-file` at all, so the
/// cost tracks the number of holds rather than the number of agents. A blob
/// that will not read, or does not parse, contributes nothing (the mark's own
/// discipline: never a forged park).
fn held_under(git_dir: &Path) -> Result<HashMap<String, Held>, GitTreeError> {
    let out = for_each_ref_under(git_dir, HELD_PREFIX)?;
    Ok(parse_oids(&out, HELD_PREFIX)
        .into_iter()
        .filter_map(|(agent, oid)| {
            let bytes = blob(git_dir, &oid).ok()?;
            Some((agent, hold::parse(&String::from_utf8_lossy(&bytes))?))
        })
        .collect())
}

/// Strip `prefix` off each `<refname> <oid>` line, yielding the
/// `agent-id -> oid` map. Lines that don't match the namespace, carry no
/// oid, or strip to an empty id contribute nothing.
fn parse_oids(stdout: &[u8], prefix: &str) -> HashMap<String, String> {
    String::from_utf8_lossy(stdout)
        .lines()
        .filter_map(|line| {
            let (refname, oid) = line.trim().split_once(' ')?;
            let id = refname.strip_prefix(prefix)?;
            (!id.is_empty()).then(|| (id.to_string(), oid.to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests;
