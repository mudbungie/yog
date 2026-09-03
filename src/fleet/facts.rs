//! **The loop renders as facts, not magic** (VISION §5 V4 item 2): the cap, the
//! current count, the tick, and where the spend ceiling will bind.
//!
//! Every one is derived on read and none is stored anywhere:
//!
//! | Fact | Derived from |
//! |---|---|
//! | cap | the `cadence.yaml` `fleet:` entry ([`super::arming`]) |
//! | project | the same entry |
//! | current count | the board's own **claimed** rows bound to this workspace |
//! | the last tick that acted | the newest `yog-fleet` row on the ops tail |
//! | the next tick | the clock's period — the loop looks again within it |
//! | the ceiling | §3.5's `Ceiling` over **every** workspace's already-walked bills |
//!
//! **Why the tick is a period and not a countdown.** A level-triggered loop's
//! tick is not an event: it converges from whatever state it finds, so a tick
//! that changed nothing is indistinguishable from one that never ran, and both
//! are correct. yog therefore states the two things that *are* facts — how long
//! ago the loop last changed the world, and the period inside which it will
//! look again — rather than storing a phase on disk to render a countdown from.
//! A stored phase would be a second home for a fact the loop does not need, and
//! it would be wrong the moment a tick was late (§4.3: *"a missed tick is
//! self-healing"*).
//!
//! **The ceiling renders where it will bind — on the next spawn** (V4 item 3).
//! It is the very policy object the spawn gate consults
//! ([`crate::boundary::ceiling`], bl-56d5), asked over the same figure; this
//! composes that gate and does not restate it, which is why raising the number
//! moves both at once. Since bl-a80a that figure is the **world's**, so the
//! verdict is folded once and every armed row shows it: one allowance, however
//! many projects are armed, is the whole of what that ball fixed.

use std::time::Duration;

use crate::app::Snapshot;
use crate::board::{BoardRow, Column};
use crate::spend::{Ceiling, Prices};

/// One armed workspace's loop, as the board states it. A value, never a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    /// The workspace's **§3.1 name**, never its path (REMOTE §8.1, bl-ef16).
    /// This row crosses on `Reply::Board` under the key every gesture takes, so
    /// a path here was the bl-22ab shape one reply over: a value a seat cannot
    /// feed back to the act its key names, and the engine's layout disclosed
    /// besides. The pilot resolves it at the one seam that owns the round trip
    /// ([`Snapshot::armed_path`](crate::app::Snapshot::armed_path)).
    pub workspace: String,
    /// Where this loop takes ready work from — the project's **§5.1 #1 wire
    /// name**, the word `BoardRow::project` and `--project` already take.
    pub project: String,
    /// The most balls this workspace may hold at once.
    pub cap: usize,
    /// How many it holds now — the board's claimed rows, counted.
    pub count: usize,
    /// The tick period: the loop looks again within this.
    pub tick: Duration,
    /// How long a claimed ball's conversations may be quiet before the loop
    /// releases the claim. `None` — the default — reaps nothing, ever, and
    /// renders as saying so.
    pub lease: Option<Duration>,
    /// How long ago the loop last spawned or reaped, in seconds. `None` is a
    /// loop that has never acted, which is not the same fact as "0 seconds
    /// ago" and must not render as one.
    pub since_act: Option<i64>,
    /// What the **next spawn** would be refused with, or `None` to let it fly.
    /// World-scoped since bl-a80a, so it reads the same on every armed row —
    /// the ceiling bounds the world, not this sphere.
    pub ceiling: Option<String>,
}

impl Facts {
    /// Whether the loop has room to spawn — the cap comparison, spelled once so
    /// the rendered fact and the pilot's own decision cannot disagree.
    pub fn has_room(&self) -> bool {
        self.count < self.cap && self.ceiling.is_none()
    }

    /// The one-line rendering both faces show. Facts only, in the order the
    /// operator asks them: how full, how often, how recently, and what a
    /// quiet drone costs its claim.
    pub fn label(&self) -> String {
        let last = match self.since_act {
            None => "nothing yet".to_owned(),
            Some(secs) => format!("last {} ago", secs_label(secs)),
        };
        let lease = match self.lease {
            None => "no lease".to_owned(),
            Some(lease) => format!("lease {}", period_label(lease)),
        };
        format!(
            "{}/{} drones · tick {} · {last} · {lease}",
            self.count,
            self.cap,
            period_label(self.tick),
        )
    }
}

/// A whole-second count as the §11 age vocabulary spells it (`42s`, `7m`,
/// `3h`), so a period, an age and a conversation row all read one way.
pub(crate) fn secs_label(secs: i64) -> String {
    crate::nav::convs::age_label(secs)
}

/// The same, for a [`Duration`] — saturating, because a config number is the
/// operator's and a bound is not a panic path.
fn period_label(period: Duration) -> String {
    secs_label(i64::try_from(period.as_secs()).unwrap_or(i64::MAX))
}

/// Every armed workspace's facts, in `cadence.yaml` order — empty when nothing
/// is armed, which is the burden check made structural: unarmed, the board has
/// no loop to render because there is no loop.
///
/// `rows` is the board's own rows, so the count is the very set the operator is
/// looking at rather than a second enumeration that could disagree.
pub fn of(
    snap: &Snapshot,
    prices: &Prices,
    ceiling: Ceiling,
    rows: &[BoardRow],
    now: i64,
) -> Vec<Facts> {
    let acts = super::row::of_rows(&snap.ops);
    // One world, one number, one verdict (bl-a80a): the ceiling bounds what the
    // whole world has spent, so it is folded once here and every armed row
    // carries the same answer — which is the fact, not a repetition of it.
    let verdict = ceiling.verdict(spent(snap, prices));
    snap.fleet
        .iter()
        .map(|(key, policy)| {
            // The arming table keys by directory and this row speaks names, so
            // the fold is where the two meet (bl-ef16) — once per armed loop,
            // in the vocabulary `held` below was already comparing in.
            let workspace = crate::naming::leaf(std::path::Path::new(key));
            Facts {
                count: held(rows, &workspace),
                cap: policy.cap,
                project: snap.project_name(&policy.project),
                tick: snap.cadence.full_sweep,
                lease: policy.lease,
                since_act: super::row::last_act(&acts, key).map(|ts| now.saturating_sub(ts)),
                ceiling: verdict.clone(),
                workspace,
            }
        })
        .collect()
}

/// The balls this workspace holds right now: the board's claimed rows bound to
/// it. One ball is one drone by construction — the loop spawns one conversation
/// per ball — so this is the drone count the cap governs.
pub fn held(rows: &[BoardRow], workspace: &str) -> usize {
    // A board row names its workspace since bl-b4b5, and since bl-ef16 so does
    // the row this count rides on — the comparison that used to convert here is
    // one vocabulary on both sides.
    rows.iter()
        .filter(|r| r.column == Column::Claimed && r.workspace.as_deref() == Some(workspace))
        .count()
}

/// The **world's** whole priced spend, off the worker's already-walked bills
/// (§7.2 — no frame reads disk): every workspace the derivation pass billed,
/// concatenated and folded once. The same scope the gate compares against
/// (bl-a80a), which is what keeps "the ceiling renders where it will bind" one
/// answer instead of two that could drift.
fn spent(snap: &Snapshot, prices: &Prices) -> Option<crate::spend::Cost> {
    let bills: Vec<_> = snap.bills.values().flatten().cloned().collect();
    crate::spend::priced(&bills, prices)
}

#[cfg(test)]
mod tests;
