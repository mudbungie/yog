//! The **one** walk of a workspace's `steps/` tree (ARCH §2.2/§2.3), yielding
//! each step's Usage fold beside the model that billed it.
//!
//! Split out of [`super`] so both consumers share one walk rather than two:
//! the token figure ([`super::spend`], §5.1 #16) folds the bills and drops the
//! model; the §3.5 spend attribution ([`crate::spend`]) groups them by it. Yog
//! stores neither — the model is read back off the step's own `request.json`,
//! the same stateless re-read every other §5.1 fact is (§3.5).
//!
//! Forgiving by construction, exactly as the fold always was: an unreadable
//! `steps/` tree, step dir, `response.json` or `request.json` each contributes
//! a zero / `None`, never a panic and never a fabricated figure.

use std::fs;
use std::path::Path;

use super::{BudgetSpend, RESPONSE_FILE, STEP_SEQ_WIDTH, STEPS_DIR, last_usage, spend_from_bytes};

/// The wire-request snapshot of one step (ARCH §2.3) — where the model that
/// billed the step is named.
const REQUEST_FILE: &str = "request.json";
/// Per-step metadata (ARCH §2.3) — where the `started_at`/`ended_at` span is.
const META_FILE: &str = "meta.json";

/// Which conv-id dirs of a workspace's `steps/` tree a fold counts — the
/// three real granularities of the one id-namespaced tree, made mechanical:
/// one agent, one conversation (a root agent and its hyphenated descent), or
/// the whole workspace. The middle two are the §3.2 altitudes of ball
/// attribution; [`Scope::Agent`] is VISION V1.5's per-agent card figure,
/// *"the per-agent fold of `steps/<id>` — workspace-root and id-namespaced,
/// so nothing double-counts: a fork's shared prefix cost stays with the
/// ancestor"*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    /// Exactly one agent — `steps/<id>/` and none of its descent.
    Agent(String),
    /// One root agent and its descent — `steps/<root>/` plus every
    /// `steps/<root>-*/` (whole-tree, ARCH §6).
    Tree(String),
    /// Every conversation in the workspace — the §3.5 workspace-granularity
    /// attribution for a ball no conversation stamps.
    Workspace,
}

impl Scope {
    /// Does this scope count the conv-id dir `conv_id`? **Public because the
    /// walk and the selection are now two moments** (bl-9dd4): the worker walks
    /// a workspace once into `Snapshot::bills`, and every later figure — one
    /// conversation's, one ball's, an epic rollup's across workspaces — is this
    /// same predicate applied in memory. One rule, one home; a second copy of
    /// "is this conv in that tree" is how a rollup and a row would drift.
    pub fn wants(&self, conv_id: &str) -> bool {
        match self {
            Self::Workspace => true,
            Self::Agent(id) => conv_id == id,
            Self::Tree(root) => {
                conv_id == root
                    || (conv_id.starts_with(root.as_str())
                        && conv_id.as_bytes().get(root.len()) == Some(&b'-'))
            }
        }
    }
}

/// One step's spend and the model that billed it. `model` is `None` when the
/// step's `request.json` is missing, unparseable, or names no string `model` —
/// an honest unknown, never guessed from a sibling step.
///
/// `conv` is the step's conv-id dir — which agent billed it. It rides along so
/// a [`Scope`] can be applied *after* the walk (bl-9dd4): the tree is walked
/// once, on the worker, and every attribution question is then a filter over
/// the result rather than another disk pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepBill {
    pub conv: String,
    /// The step's own zero-padded sequence dir (`001`, `002`, … — ARCH §2.3).
    /// It rides along for the same reason `conv` does (bl-9dd4): with it, *which
    /// step is the latest* is a question answered in memory over the one walk,
    /// and the §5.1 #35 context figure needs no second disk pass to ask it.
    /// Zero-padded to a fixed width, so lexical order **is** step order.
    pub seq: String,
    pub model: Option<String>,
    pub spend: BudgetSpend,
    /// The **last** attempt segment's counters (§5.1 #35), beside the fold of
    /// all of them. Two different questions over one file: `spend` is what the
    /// step cost, `last_usage` is how big its final prompt was — and only the
    /// second describes the context as it now stands.
    pub last_usage: BudgetSpend,
    /// This step's `meta.json` span in seconds — `started_at` → `ended_at`
    /// (§3.9, bl-40ab), the wall half of the science projection's step-record
    /// columns. It rides the bill for bl-9dd4's own reason: the tree is walked
    /// once, so wall time is a fold over the result rather than a second disk
    /// pass over the same `steps/` tree.
    ///
    /// **Wall is wall** (lernie ARCH §6, whose `budget::derive::wall_seconds`
    /// this mirrors): the span covers the backoff sleeps between a step's
    /// attempts, so it counts waiting as well as streaming. Zero when
    /// `meta.json` is missing, unparseable, still unsettled, or reports an end
    /// before its start — an honest unknown, never a fabricated duration.
    pub wall_secs: u64,
}

/// Every step under the conv-id dirs `scope` selects. A missing `steps/` tree
/// yields no bills.
pub fn bills(workspace: &Path, scope: &Scope) -> Vec<StepBill> {
    let Ok(entries) = fs::read_dir(workspace.join(STEPS_DIR)) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let raw = entry.file_name();
        let conv = raw.to_string_lossy();
        if scope.wants(&conv) {
            conv_bills(&entry.path(), &conv, &mut out);
        }
    }
    out
}

/// Sum of every bill's spend — the token figure, model dropped (§5.1 #16).
pub fn total(bills: &[StepBill]) -> BudgetSpend {
    let mut total = BudgetSpend::default();
    for bill in bills {
        total.add(bill.spend);
    }
    total
}

/// Sum of every bill's `meta.json` span — the wall figure (§3.9, bl-40ab),
/// beside [`total`] and over the same walk. Summed per step rather than taken
/// as the first-to-last span, exactly as lernie's own `wall_seconds` is: a
/// conversation that sat idle for an hour between two calls spent no wall time
/// on them, and it is the calls a budget bounds.
pub fn wall(bills: &[StepBill]) -> u64 {
    bills.iter().map(|b| b.wall_secs).sum()
}

/// Append every 3-digit step subdir of one conv-id dir. A conv-id entry that
/// is not a readable directory contributes nothing.
fn conv_bills(conv_dir: &Path, conv: &str, out: &mut Vec<StepBill>) {
    let Ok(entries) = fs::read_dir(conv_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let raw = entry.file_name();
        let name = raw.to_string_lossy();
        if name.len() == STEP_SEQ_WIDTH && name.bytes().all(|b| b.is_ascii_digit()) {
            out.push(step_bill(&entry.path(), conv, &name));
        }
    }
}

/// One step dir's bill: every Usage segment of `response.json` (ARCH §6, a
/// billed retry included) priced against `request.json`'s model, stamped with
/// the conv-id dir it was found under.
fn step_bill(step_dir: &Path, conv: &str, seq: &str) -> StepBill {
    let bytes = fs::read(step_dir.join(RESPONSE_FILE)).unwrap_or_default();
    StepBill {
        conv: conv.to_owned(),
        seq: seq.to_owned(),
        model: step_model(step_dir),
        spend: spend_from_bytes(&bytes),
        last_usage: last_usage(&bytes),
        wall_secs: step_wall(step_dir),
    }
}

/// One step's `started_at` → `ended_at` span in seconds (§2.3 `meta.json`).
/// Zero on any read, parse or ordering failure — the same forgiving reading
/// every other field here takes, and the reason a still-running step
/// contributes nothing rather than a negative.
fn step_wall(step_dir: &Path) -> u64 {
    let bytes = fs::read(step_dir.join(META_FILE)).unwrap_or_default();
    let Ok(meta) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return 0;
    };
    let at = |key: &str| {
        crate::ui_state::epoch_from_iso8601(meta.get(key).and_then(serde_json::Value::as_str)?)
    };
    let (Some(start), Some(end)) = (at("started_at"), at("ended_at")) else {
        return 0;
    };
    u64::try_from(end - start).unwrap_or(0)
}

/// The model named in a step's `request.json` — the wire request's own
/// `"model"` string, i.e. the id `models.yaml` declares and the §3.5 price
/// table is keyed by. `None` on any read/parse/shape failure.
fn step_model(step_dir: &Path) -> Option<String> {
    let bytes = fs::read(step_dir.join(REQUEST_FILE)).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    Some(value.get("model")?.as_str()?.to_owned())
}

#[cfg(test)]
mod tests;
