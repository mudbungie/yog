//! The level trigger and the thread it runs on (VISION §4.3).
//!
//! **Level-triggered, and at most one move per tick.** A tick reads the board
//! the worker last published, decides one thing — reap this ball, or spawn on
//! that one — does it, and stops. It keeps no memory of the tick before,
//! because it does not need one: the next tick reads the world the last one
//! left, so a missed tick, a crashed yog and a second instance all converge
//! (§4.3, verbatim: *"a missed tick is self-healing because the loop converges
//! from whatever state it finds"*). One move per tick is what bounds it: no
//! tick can ever storm, whatever the board looks like.
//!
//! **Reaps go before spawns.** Freeing a slot before filling one is what makes
//! a cap-full workspace with a dead drone recover in two ticks rather than
//! never.
//!
//! **Never the frame, and never the derivation worker either.** A spawn runs
//! `bl` and forks a driver; the worker's pass is a correctness floor measured
//! against the cheap-sweep cadence (§7.2), and a fork inside it would read as
//! yog being late. So this is its own thread, in the shape the worker, the
//! consumer and the sentry already use: a stop flag, a park loop, a `Drop` that
//! joins. All the logic is [`PilotCtx::pass`], which a test drives directly.
//!
//! Its period is the clock's **full-sweep** cadence off the published snapshot:
//! the loop ticks with the slowest thing yog does, because a spawn is a
//! decision about a whole ball and not a poll.
//!
//! **Unarmed, a tick costs one read of an already-published snapshot** — it
//! returns before it builds a board, opens `ui.json` or looks at the trail. The
//! burden check is that early return, and it is tested, not promised.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use super::{Facts, facts, row};
use crate::app::Snapshot;
use crate::board::{BoardRow, Column};
use crate::boundary::Action;
use crate::boundary::dispatch::{self, Deps};
use crate::git_tree::{AgentState, GitTree};
use crate::opslog;
use crate::start::{BallSpec, Payload};
use crate::state::SnapshotCell;
use crate::ui_state::{Clock, UiState, content_hash};

/// What the pilot thread needs: a [`Deps`] template (everything but the
/// per-pass snapshot and mint seed), the cell the worker publishes to, the
/// clock, and the durable `ui.json` the ceiling and the price table live in.
pub struct PilotCtx {
    pub deps: Deps,
    pub cell: SnapshotCell,
    pub clock: Arc<dyn Clock>,
    pub ui_path: PathBuf,
}

/// The one move a tick may make. Not a plan and not a queue — a tick that finds
/// two things to do does the first and lets the next tick find the second.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Move {
    /// Release this claimed ball from `claimant`, with the comparison that
    /// decided it. The claimant rides the move rather than being re-read at
    /// fire time: a ball nobody is named on cannot be released, so that is a
    /// condition of *planning* a reap, not a guard inside doing one.
    Reap {
        row: BoardRow,
        claimant: String,
        since: String,
    },
    /// Start a drone on this ready ball.
    Spawn { row: BoardRow },
}

impl PilotCtx {
    /// The tick period — the clock's full-sweep cadence, off the same published
    /// snapshot everything else here reads, so tuning the clock tunes the loop
    /// with it and no thread re-reads the file.
    pub fn period(&self) -> Duration {
        crate::state::latest_snapshot(&self.cell).cadence.full_sweep
    }

    /// One tick: at most one move, and none at all unless something is armed.
    /// Returns whether it acted — a test's assertion, nothing the thread needs.
    pub fn pass(&self) -> bool {
        let snapshot = crate::state::latest_snapshot(&self.cell);
        // The burden check, structural: unarmed, a tick reads nothing further
        // and does nothing at all.
        if snapshot.fleet.is_empty() {
            return false;
        }
        let ts = self.clock.stamp();
        let now: i64 = ts.parse().unwrap_or(0);
        let mut ui = UiState::open(self.ui_path.clone());
        let board = crate::board::build(&snapshot, &ui, now);
        for fleet in &board.fleet {
            if let Some(one) = plan(&snapshot, fleet, &board.rows, now) {
                return self.fire(&snapshot, &mut ui, &ts, fleet, &one);
            }
        }
        false
    }

    /// Make the move and leave one row behind — only if it landed. A refused or
    /// failed move already has its own §4.2 row from the executor that refused
    /// it (the ceiling's, `bl`'s, the start flow's), and the level trigger is
    /// the whole retry: the next tick re-reads the world and decides again.
    fn fire(
        &self,
        snapshot: &Arc<Snapshot>,
        ui: &mut UiState,
        ts: &str,
        fleet: &Facts,
        one: &Move,
    ) -> bool {
        let deps = self.deps(snapshot, ts);
        let entry = match one {
            Move::Reap {
                row,
                claimant,
                since,
            } => {
                let release = Action::Release {
                    project: snapshot.project_name(&row.project),
                    id: row.id.clone(),
                    name: claimant.clone(),
                };
                // The verb must actually have released it. A non-zero `bl`
                // is not an `Err` here (§8.2: its stderr is the product), so
                // the outcome is what decides — a row saying the loop reaped a
                // ball it still holds would be the trail lying.
                if !released(dispatch::dispatch(&deps, ui, ts, &release)) {
                    return false;
                }
                row::reaped(ts.to_owned(), &fleet.workspace, &row.id, claimant, since)
            }
            Move::Spawn { row } => {
                let Some(conversation) = Self::birth(&deps, ui, ts, fleet, row) else {
                    return false;
                };
                row::spawned(ts.to_owned(), &fleet.workspace, &row.id, &conversation)
            }
        };
        let _ = opslog::append(&deps.state_root, &entry);
        true
    }

    /// The §8.1 start flow, through the boundary's own two typed doors — the
    /// same bodies a click, a line and a deposit run. The §3.5 spend ceiling
    /// and the §4.11 confinement refusal are seated inside
    /// [`dispatch::prompt`], so a loop spawn is gated by construction rather
    /// than by this module remembering to ask.
    fn birth(deps: &Deps, ui: &UiState, ts: &str, fleet: &Facts, row: &BoardRow) -> Option<String> {
        let ball = deps
            .snapshot
            .balls_by_project
            .get(&row.project)?
            .iter()
            .find(|b| b.id == row.id)?;
        let payload = Payload::Ball {
            project: deps.snapshot.project_name(&row.project),
            ball: BallSpec::Existing {
                id: ball.id.clone(),
                title: ball.title.clone(),
                body: ball.body.clone(),
                join: row.state,
            },
        };
        let prepared =
            dispatch::prepare(deps, ts, &fleet.workspace, &row.project, &payload).ok()?;
        // The composed goal verbatim (§3.3, bl-6920): there is no operator at
        // the composer to edit it, and the loop must not become a second author.
        let goal = prepared.goal.clone();
        dispatch::prompt(deps, ui, ts, &fleet.workspace, &prepared, &goal).ok()
    }

    /// This pass's [`Deps`]: the template plus the snapshot it just read and a
    /// seed from its own stamp, exactly as the gestures consumer builds one.
    fn deps(&self, snapshot: &Arc<Snapshot>, ts: &str) -> Deps {
        Deps {
            snapshot: Arc::clone(snapshot),
            mint_seed: content_hash(ts.as_bytes()),
            ..self.deps.clone()
        }
    }
}

/// Whether a released claim actually came back: the verb ran *and* exited
/// clean. Anything else leaves the ball where it was, and the next tick decides
/// again against the world as it then is.
fn released(reply: Result<crate::boundary::reply::Reply, String>) -> bool {
    matches!(reply, Ok(crate::boundary::reply::Reply::Outcome(o)) if o.ok())
}

/// One workspace's move, or `None` when its loop has nothing to do. Pure over
/// the published snapshot and the board built from it — which is what lets the
/// whole decision be a table test.
pub fn plan(snap: &Snapshot, fleet: &Facts, rows: &[BoardRow], now: i64) -> Option<Move> {
    reap(snap, fleet, rows, now).or_else(|| spawn(fleet, rows))
}

/// The reap: the first claimed ball of this workspace whose conversations have
/// all been quiet past the lease. No lease, no reap — releasing a claim is not
/// something yog does on a default (see [`arming`]).
fn reap(snap: &Snapshot, fleet: &Facts, rows: &[BoardRow], now: i64) -> Option<Move> {
    let lease = i64::try_from(fleet.lease?.as_secs()).ok()?;
    let tree = snap.trees.get(&fleet.workspace)?;
    rows.iter().filter(|r| held_here(r, fleet)).find_map(|row| {
        // A row that names nobody cannot be released from anyone.
        let claimant = row.claimant.clone()?;
        let idle = quiet_for(tree, row, now)?;
        (idle >= lease).then(move || Move::Reap {
            row: row.clone(),
            claimant,
            // The comparison itself, never a diagnosis (§4.3): how far past
            // the operator's own number this ball has gone, and nothing
            // about why.
            since: format!("lease expired {} ago", facts::secs_label(idle - lease)),
        })
    })
}

/// The spawn: the board's top ready ball in this loop's project, when the
/// workspace has room under its cap and the ceiling would not refuse the birth
/// anyway. **Ready only** — a gated ball can be started but cannot be
/// delivered, and a loop that fills its cap with undeliverable work has stopped
/// being a fleet.
fn spawn(fleet: &Facts, rows: &[BoardRow]) -> Option<Move> {
    if !fleet.has_room() {
        return None;
    }
    rows.iter()
        .find(|r| r.column == Column::Ready && r.project == fleet.project)
        .map(|row| Move::Spawn { row: row.clone() })
}

/// Whether this row is a ball the armed workspace holds right now.
fn held_here(row: &BoardRow, fleet: &Facts) -> bool {
    row.column == Column::Claimed && row.workspace.as_deref() == Some(&fleet.workspace)
}

/// How long every conversation on this ball has been quiet, or `None` when one
/// of them is still running (or the ball has no conversation at all).
///
/// **Nothing running is ever reaped**, whatever its age: the ceiling's own
/// ruling — killing mid-ball destroys uncommitted work — applies to a claim as
/// much as to a spend, and a live drone's claim is not idle by any reading.
fn quiet_for(tree: &GitTree, row: &BoardRow, now: i64) -> Option<i64> {
    let mut newest: Option<i64> = None;
    for agent in &tree.agents {
        let root = crate::nav::convs::root_of(&tree.agents, &agent.agent_id)?;
        if !row.drones.iter().any(|d| d.root_id == root) {
            continue;
        }
        if matches!(agent.state, AgentState::Live | AgentState::InFlight) {
            return None;
        }
        newest = Some(newest.unwrap_or(i64::MIN).max(agent.last_action_unix));
    }
    newest.map(|last| now.saturating_sub(last))
}

/// The pilot thread. The worker's shutdown shape (§7.2): stop flag, unpark,
/// join.
pub struct Pilot {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Pilot {
    /// Run [`PilotCtx::pass`] forever, parked for the clock's full-sweep period
    /// between ticks.
    pub fn spawn(ctx: PilotCtx) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                ctx.pass();
                std::thread::park_timeout(ctx.period());
            }
        });
        Self {
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for Pilot {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests;
