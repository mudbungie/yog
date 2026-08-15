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
//! | the ceiling | §3.5's `Ceiling` over the workspace's already-walked bills |
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
//! moves both at once.

use std::path::PathBuf;
use std::time::Duration;

use crate::app::Snapshot;
use crate::board::{BoardRow, Column};
use crate::spend::{Attribution, Ceiling, Prices};

/// One armed workspace's loop, as the board states it. A value, never a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub workspace: PathBuf,
    /// Where this loop takes ready work from.
    pub project: PathBuf,
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
    snap.fleet
        .iter()
        .map(|(key, policy)| {
            let workspace = PathBuf::from(key);
            Facts {
                count: held(rows, &workspace),
                cap: policy.cap,
                project: policy.project.clone(),
                tick: snap.cadence.full_sweep,
                lease: policy.lease,
                since_act: super::row::last_act(&acts, key).map(|ts| now.saturating_sub(ts)),
                ceiling: ceiling.verdict(&spent(snap, &workspace, prices)),
                workspace,
            }
        })
        .collect()
}

/// The balls this workspace holds right now: the board's claimed rows bound to
/// it. One ball is one drone by construction — the loop spawns one conversation
/// per ball — so this is the drone count the cap governs.
pub fn held(rows: &[BoardRow], workspace: &std::path::Path) -> usize {
    // A board row names its workspace since bl-b4b5; the `cadence.yaml` entry
    // this loop was armed from names a directory, and §3.1 makes the leaf the
    // name, so the comparison is made in the vocabulary the answer speaks.
    let name = crate::naming::leaf(workspace);
    rows.iter()
        .filter(|r| r.column == Column::Claimed && r.workspace.as_deref() == Some(name.as_str()))
        .count()
}

/// The workspace's whole priced spend, off the worker's already-walked bills
/// (§7.2 — no frame reads disk). The same scope the gate compares against.
fn spent(snap: &Snapshot, workspace: &std::path::Path, prices: &Prices) -> crate::spend::Figure {
    let bills = snap.bills.get(workspace).cloned().unwrap_or_default();
    crate::spend::figure(&bills, prices, Attribution::Workspace)
}

#[cfg(test)]
mod tests;
