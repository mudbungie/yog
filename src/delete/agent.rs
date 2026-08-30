//! Agent deletion — the §3.6 class, one conversation deep (bl-f17a).
//!
//! Unlike the workspace unmaking ([`super`]), the removal here is **not** yog's
//! own write: litany ships `litany delete <ws> <agent> [--children] [--dry-run]`
//! (0.0.4), and I2's litany-verbs-only rule makes that verb the only lawful
//! remover of agent state. yog's part is the §3.6 confirm discipline around it:
//!
//! - **The gate** ([`AgentConfirmation`]): refuse while the conversation's root
//!   or any member probes Live/InFlight — the §10 "?" uncertainty counting as
//!   live, fail closed. Stop keeps its own semantics (§3.6 rejected (c)); the
//!   refusal names the live members so the operator stops them first. litany's
//!   own `Driven` decline is the substrate's independent fail-closed under the
//!   race; yog gates first.
//! - **The arming** ([`AgentConfirmation::subtree_armed`], §3.6's amended
//!   doctrine): typed-name confirm **iff the verb destroys objects beyond the
//!   one named on screen**. A leaf agent is the row under the pointer — a plain
//!   explicit confirm suffices; retyping its name proves nothing the dialog
//!   does not already show. A subtree delete destroys conversations that are
//!   *not* that row, so it takes the typed name — and the typed name is the
//!   only thing that unlocks `--children`. An unarmed fire is the **bare**
//!   verb, and litany's own `HasDescendants` decline (naming the descendants)
//!   rides back: the census that gates the subtree is computed by the substrate
//!   that performs the act, at the moment it acts — never a yog re-derivation,
//!   never a stale dialog's.
//! - **The census** ([`census`]): the dialog enumerates from `litany delete
//!   --children --dry-run` — the descendants by name and the pending-deposit
//!   count come straight off litany's `DeleteReport` line ([`parse_report`]).
//!   A dry run mutates nothing, so it is the unlogged `collect` read (the
//!   `bl conf` seam's idiom), not an ops row per dialog open.

use crate::actions::verbs::{Outcome, collect, run_logged};
use crate::cli_outbound::Cli;
use crate::git_tree::{Agent, AgentState};
use crate::opslog::Origin;
use std::io;
use std::path::Path;

/// litany's verb and flags, pinned to the locked crate
/// (`src/cmd/delete.rs`).
const DELETE: &str = "delete";
const CHILDREN: &str = "--children";
const DRY_RUN: &str = "--dry-run";

/// What the §3.6 dialog and the dispatch gate read of one conversation —
/// derived, never stored: its display name (the §3.3 ladder) and the members
/// that hold (or may hold) a driver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentConfirmation {
    pub name: String,
    /// The member agent ids probing Live/InFlight — or unobservable (§10),
    /// which counts as live. Empty ⇒ the verb may proceed.
    pub live: Vec<String>,
}

impl AgentConfirmation {
    /// The gate: refused while anything in the conversation runs (fail closed).
    pub fn refused(&self) -> bool {
        !self.live.is_empty()
    }

    /// The subtree arming (§3.6 as amended): the operator retyped the
    /// conversation's own name — surrounding whitespace forgiven, nothing
    /// else — which is what unlocks `--children`. Never armed while refused.
    pub fn subtree_armed(&self, typed: &str) -> bool {
        !self.refused() && typed.trim() == self.name
    }
}

/// Derive the confirmation for the conversation rooted at `root` from the
/// agent snapshot. A root the snapshot does not carry yields the general path
/// with empty inputs: its own id for a name, nothing live — and litany's
/// delete of an absent agent is already its postcondition (convergent).
pub fn confirmation(root: &str, agents: &[Agent]) -> AgentConfirmation {
    let live = crate::nav::convs::members(agents, root)
        .iter()
        .filter_map(|r| agents.get(r.index))
        .filter(|a| matches!(a.state, AgentState::Live | AgentState::InFlight) || a.state_uncertain)
        .map(|a| a.agent_id.clone())
        .collect();
    AgentConfirmation {
        name: crate::nav::convs::display_name_of(agents, root),
        live,
    }
}

/// **The same census off an answered forest** (REMOTE §9.7, bl-b4b5) —
/// [`confirmation`]'s seat-side twin, over `Query::Conversations`' rows.
///
/// The answer is pre-order and carries every member's own §3.5 state and §10
/// uncertainty, so the conversation's members are the run of deeper rows below
/// its root and the gate is that run's disjunction — the same fail-closed
/// reading, from what a seat holds. A root the forest does not carry answers
/// its own id for a name and nothing live, which is [`confirmation`]'s own
/// empty-input arm rather than a case of its own.
pub fn confirmation_of_rows(rows: &[crate::nav::convs::ConvRow], root: &str) -> AgentConfirmation {
    let at = rows.iter().position(|r| r.root_id == root);
    AgentConfirmation {
        name: crate::nav::convs::selection(rows, root).name,
        live: at
            .map(|at| {
                crate::nav::convs::census::subtree(rows, at)
                    .filter(|r| {
                        matches!(r.state, AgentState::Live | AgentState::InFlight) || r.uncertain
                    })
                    .map(|r| r.root_id.clone())
                    .collect()
            })
            .unwrap_or_default(),
    }
}

/// The gate's refusal wording — the live members named so the operator can
/// stop them first (Stop stays its own verb).
pub fn live_refusal(live: &[String]) -> String {
    format!(
        "refused \u{2014} live: {} \u{2014} stop them first",
        live.join(", ")
    )
}

/// The census a confirmation dialog enumerates: what a subtree delete would
/// take, per litany's own `DeleteReport`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Census {
    /// The hyphen-descendants that die with the root, by id (litany ARCH §2.3).
    pub descendants: Vec<String>,
    /// Undelivered deposits across the subtree's inboxes — mail addressed *to*
    /// these agents, which dies with them.
    pub pending_deposits: usize,
}

/// Parse litany's one-line `DeleteReport` (either mood): `would delete <id>;
/// descendants: N (a, b); pending deposits: M` — the names ride in the
/// parenthesis, absent when N is 0. `None` for anything else.
pub fn parse_report(line: &str) -> Option<Census> {
    let (_, rest) = line.split_once("; descendants: ")?;
    let (desc, pending) = rest.split_once("; pending deposits: ")?;
    let pending_deposits = pending.trim().parse().ok()?;
    let descendants = match desc.split_once(" (") {
        Some((_, named)) => named
            .strip_suffix(')')?
            .split(", ")
            .map(str::to_owned)
            .collect(),
        None => Vec::new(),
    };
    Some(Census {
        descendants,
        pending_deposits,
    })
}

/// Run `litany delete <ws> <root> --children --dry-run` and read the census
/// off its report. Unlogged (a dry run mutates nothing — the `collect` read
/// seam, like the marks knob's `bl conf`); a decline rides back as its stderr.
pub fn census(litany: &Cli, ws: &Path, root: &str) -> Result<Census, String> {
    let ws_s = ws.to_string_lossy().into_owned();
    let outcome = collect(litany.run_in(ws, &[DELETE, &ws_s, root, CHILDREN, DRY_RUN]))
        .map_err(|e| e.to_string())?;
    if !outcome.ok() {
        return Err(outcome.stderr.trim().to_owned());
    }
    let line = outcome.stdout.trim();
    parse_report(line).ok_or_else(|| format!("unrecognized delete report: {line}"))
}

/// The removal itself: `litany delete <ws> <root> [--children]`, short, piped
/// and logged like every litany verb (§8.2, Origin::Conversation). The bare
/// form is deliberate when unarmed — litany declines a subtree nobody
/// confirmed, naming it.
pub fn spawn(
    litany: &Cli,
    state_root: &Path,
    ts: &str,
    ws: &Path,
    root: &str,
    children: bool,
) -> io::Result<Outcome> {
    let ws_s = ws.to_string_lossy();
    let mut args = vec![DELETE, ws_s.as_ref(), root];
    if children {
        args.push(CHILDREN);
    }
    run_logged(litany, state_root, ts, ws, &args, Origin::Conversation)
}

#[cfg(test)]
mod tests;
