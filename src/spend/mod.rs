//! Spend attribution: the yog-side join (DESIGN §3.5; VISION spend attribution).
//!
//! Cost per ball is a **query**, and single-source-of-truth is what makes it
//! one. Every fact it needs already has an owner: brazen counts tokens and
//! never learns a price; litany commits that count into the step record; balls
//! tags every delivery `[bl-id]` and stays metric-free. Yog adds exactly two
//! things nobody below it may hold — **the price table** ([`prices`]) and
//! **the join** (this module) — and stores neither result (§3.5: a figure is
//! re-derived from disk, never written down).
//!
//! The join is `Σ(step usage over the agents tied to the ball) × prices`, and
//! *tied to* is the honest part. §3.2 enumerates two altitudes of ball
//! attribution and this module renders both rather than papering over the
//! gap:
//!
//! - a ball a conversation's `goal.md` **stamps** (`Ball <id>:`, §3.3) is
//!   attributed at conversation granularity — the exact agents, their whole
//!   descent included;
//! - a ball an agent claimed **mid-conversation** stamps only the *workspace*
//!   name, and no fact anywhere records which conversation picked it up.
//!   The ruling: accept **workspace-granularity**
//!   attribution for such a ball and say so on the figure. No linkage fact is
//!   invented — a yog-side conversation↔ball registry would be a second home
//!   for someone else's fact, which is the thing §3.2 already refused.
//!
//! Unpriced tokens are reported, never rounded to free: a step whose model the
//! table does not price contributes to [`Cost::unpriced_tokens`], so a partial
//! table reads as "at least this much", which is true, instead of a number
//! that is quietly wrong.
//!
//! **The walk is the worker's, the join is anyone's** (bl-9dd4). Every function
//! here is pure over a workspace's already-walked [`StepBill`]s, which the
//! derivation worker folds once per pass onto `Snapshot::bills`. That is what
//! lets a whole *board* carry a spend column: a figure per row is a filter over
//! memory, not a `steps/` walk per row on the frame thread (§7.2 — the frame
//! renders snapshots and reads no disk).

use crate::budgets::{self, BudgetSpend, Scope, StepBill};
use std::path::PathBuf;

mod ceiling;
mod prices;
pub use ceiling::Ceiling;
pub use prices::{MICRO_PER_CENT, MICRO_PER_USD, Price, Prices};

/// What a figure cost, in micro-USD, plus what it could not price.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Cost {
    /// Micro-USD (1e-6 USD) billed by every step whose model the table prices.
    pub micro_usd: u64,
    /// Tokens spent by steps the table has no rate for — either the step named
    /// no model or the table names no such model. Non-zero means the figure is
    /// a floor, and the render says so.
    pub unpriced_tokens: u64,
}

impl Cost {
    /// The figure as dollars and cents. A non-zero cost below a cent renders
    /// `<$0.01` rather than `$0.00`: "too small to show" and "nothing" are
    /// different facts and a spend column must not conflate them.
    pub fn usd(&self) -> String {
        if self.micro_usd > 0 && self.micro_usd < MICRO_PER_CENT {
            return "<$0.01".to_owned();
        }
        format!(
            "${}.{:02}",
            self.micro_usd / MICRO_PER_USD,
            (self.micro_usd % MICRO_PER_USD) / MICRO_PER_CENT
        )
    }

    fn add(&mut self, micro_usd: u64, unpriced: u64) {
        self.micro_usd = self.micro_usd.saturating_add(micro_usd);
        self.unpriced_tokens = self.unpriced_tokens.saturating_add(unpriced);
    }
}

/// The granularity a figure is honest at (§3.2's two altitudes, §3.5's
/// ruling). Not a quality grade — both arms are exact sums; they differ in
/// *what* they sum over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attribution {
    /// The N goal-stamped conversations that name the ball, their descents
    /// included. `1` is the ordinary case and the conversation figure's own.
    Conversations(usize),
    /// Every conversation in the workspace — the ruling's accepted limit for a
    /// ball claimed mid-conversation, which records no conversation link.
    Workspace,
}

/// The clause an attribution says out loud, with the explanation behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub label: String,
    pub hover: String,
}

impl Attribution {
    /// What the figure must disclose, or `None` when the attribution is
    /// exactly what its seat already claims (one stamped conversation).
    pub fn note(&self) -> Option<Note> {
        match self {
            Self::Conversations(1) => None,
            Self::Conversations(n) => Some(Note {
                label: format!("over {n} conversations"),
                hover: format!(
                    "{n} conversations name this ball in their goal. The figure \
                     sums all of them, sub-agents included."
                ),
            }),
            Self::Workspace => Some(Note {
                label: "workspace-wide".to_owned(),
                hover: "No conversation names this ball in its goal — it was picked up \
                        mid-conversation, and a pickup records only the workspace, never \
                        which conversation did it. So this is the whole workspace's spend: \
                        an upper bound on the ball, not the ball alone."
                    .to_owned(),
            }),
        }
    }
}

/// One attributed spend figure: the tokens, what they cost when the table
/// prices them, and the granularity the sum is honest at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Figure {
    pub tokens: BudgetSpend,
    /// `None` when the price table is empty — the §3.5 severability gate.
    /// Deleting `ui.json.prices` deletes the column, not this code.
    pub cost: Option<Cost>,
    pub attribution: Attribution,
}

/// The bills `roots` claims out of one workspace's walk: every bill under any
/// named root's tree, or — when `roots` is empty — the whole workspace, which
/// is §3.5's workspace-granularity arm. One bill is taken at most once however
/// many roots would want it, so a caller cannot double-bill by listing a root
/// twice.
pub fn select(bills: &[StepBill], roots: &[String]) -> Vec<StepBill> {
    if roots.is_empty() {
        return bills.to_vec();
    }
    bills
        .iter()
        .filter(|b| roots.iter().any(|r| Scope::Tree(r.clone()).wants(&b.conv)))
        .cloned()
        .collect()
}

/// The granularity a set of roots is honest at — empty is the workspace arm.
pub fn attribution(roots: &[String]) -> Attribution {
    match roots.len() {
        0 => Attribution::Workspace,
        n => Attribution::Conversations(n),
    }
}

/// **The whole world's priced spend** — every workspace in `workspaces`, folded
/// into one number, which is the scope the §3.5
/// [`Ceiling`](crate::boundary::ceiling) gate compares against since bl-a80a.
/// `None` is the empty price table, the §3.5 severability gate.
///
/// A *cost* rather than a [`Figure`], because a bound is arithmetic and never a
/// rendering: an [`Attribution`] label answers "what is this figure honest
/// about" for a figure somebody reads, and nothing reads this one — the gate
/// compares it and the board asks the gate. Inventing a world label for a value
/// no surface shows would be a fact with no reader.
///
/// **The one fold that still walks disk itself** (bl-56d5 × bl-9dd4): every
/// other figure here reads the worker's `Snapshot::bills`, but a *gate* must
/// compare against the world as it is at the instant it refuses, not against a
/// snapshot that may be a debounce window old. It runs once per spawn, at a
/// chokepoint, so it costs one walk on a gesture rather than a walk per row per
/// frame — and the spawn rate is already bounded at one per full sweep (§4.3),
/// however many workspaces the roster holds.
pub fn of_world(workspaces: &[PathBuf], prices: &Prices) -> Option<Cost> {
    let mut bills = Vec::new();
    for workspace in workspaces {
        bills.extend(budgets::bills(workspace, &Scope::Workspace));
    }
    priced(&bills, prices)
}

/// One conversation's whole-tree figure (§5.1 #16 priced): the root agent and
/// its hyphenated descent, attributed to itself. `bills` is its workspace's
/// walk.
pub fn of_conversation(bills: &[StepBill], root_id: &str, prices: &Prices) -> Figure {
    let roots = [root_id.to_owned()];
    figure(
        &select(bills, &roots),
        prices,
        Attribution::Conversations(1),
    )
}

/// A ball's figure, attributed as honestly as the facts allow (§3.5's ruling).
/// `stamped_roots` is every root in the workspace whose goal stamps the ball;
/// empty falls back to the whole workspace and labels itself so.
pub fn of_ball(bills: &[StepBill], stamped_roots: &[String], prices: &Prices) -> Figure {
    figure(
        &select(bills, stamped_roots),
        prices,
        attribution(stamped_roots),
    )
}

/// Fold a bill set into a figure, pricing it only when a table exists. **Public
/// because a rollup crosses workspaces** (§3.5, bl-9dd4): the board selects one
/// slice per workspace, concatenates them, and folds the whole here — one fold,
/// whatever enumerated it.
pub fn figure(bills: &[StepBill], prices: &Prices, attribution: Attribution) -> Figure {
    Figure {
        tokens: budgets::total(bills),
        cost: priced(bills, prices),
        attribution,
    }
}

/// The money half of [`figure`] alone: what `bills` cost, or `None` when the
/// table is empty. **The severability gate lives here and only here** — an
/// unpriced yog bounds nothing rather than inventing a token proxy for dollars
/// (§3.5) — so the ceiling's world fold and every rendered figure ask one
/// function and cannot disagree about what "unpriced" means.
pub fn priced(bills: &[StepBill], prices: &Prices) -> Option<Cost> {
    (!prices.is_empty()).then(|| cost(bills, prices))
}

/// Price every bill by its own step's model — the join proper. A bill whose
/// model the table does not carry lands in `unpriced_tokens`.
fn cost(bills: &[StepBill], prices: &Prices) -> Cost {
    let mut cost = Cost::default();
    for bill in bills {
        match prices.of(bill.model.as_deref()) {
            Some(price) => cost.add(price.cost(bill.spend), 0),
            None => cost.add(0, bill.spend.total_tokens()),
        }
    }
    cost
}

#[cfg(test)]
mod tests;
