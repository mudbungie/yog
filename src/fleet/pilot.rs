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

use super::{Facts, row};
use crate::app::Snapshot;
use crate::board::BoardRow;
use crate::boundary::Action;
use crate::boundary::dispatch::{self, Deps};
use crate::opslog;
use crate::start::{BallSpec, Payload};
use crate::state::SnapshotCell;
use crate::ui_state::{Clock, UiState};

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
        let deps = self.deps(snapshot);
        let entry = match one {
            Move::Reap {
                row,
                claimant,
                since,
            } => {
                // The verb must actually have released it. A non-zero `bl`
                // is not an `Err` here (§8.2: its stderr is the product), so
                // the outcome is what decides — a row saying the loop reaped a
                // ball it still holds would be the trail lying.
                if !release(&deps, ui, ts, row, claimant) {
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
    ///
    /// **A birth is atomic against its own claim** (bl-ab13). [`dispatch::prepare`]
    /// runs the `bl claim`, and that is the flow's LAST mutating step — so a
    /// prepare that failed claimed nothing, while a prompt that failed has left
    /// the ball held by a workspace with no conversation on it. Nothing else
    /// would ever undo that: the §4.3 lease compares a *drone's* idleness and
    /// there is no drone to be idle, so the slot and the ball were consumed
    /// forever while the trail said the spawn had succeeded. The failing door
    /// therefore releases what the door before it took. No loop row either way —
    /// the birth did not land, and the `bl` claim/unclaim pair is the trail.
    fn birth(
        deps: &Deps,
        ui: &mut UiState,
        ts: &str,
        fleet: &Facts,
        row: &BoardRow,
    ) -> Option<String> {
        // The row names its project (bl-b4b5); the live cache is keyed by the
        // clone's path and the `prepare` door takes one, so the name resolves
        // here through the snapshot's own round trip.
        let project = deps.snapshot.project_path(&row.project).ok()?;
        let ball = deps
            .snapshot
            .balls_by_project
            .get(&project)?
            .iter()
            .find(|b| b.id == row.id)?;
        let payload = Payload::Ball {
            project: row.project.clone(),
            ball: BallSpec::Existing {
                id: ball.id.clone(),
                title: ball.title.clone(),
                body: ball.body.clone(),
                join: row.state,
                // §8.7: the loop reads the whole ball off the snapshot, so its
                // tags reach the start plan exactly as a clicked ▶ Start's do —
                // a fleet birth and a hand birth select one lineage (bl-380f).
                tags: ball.tags.clone(),
            },
        };
        let prepared = dispatch::prepare(deps, ts, &fleet.workspace, &project, &payload).ok()?;
        // The composed goal verbatim (§3.3, bl-6920): there is no operator at
        // the composer to edit it, and the loop must not become a second author.
        let goal = prepared.goal.clone();
        // No preview, so no seed (bl-1747): the mint draws off the stamp.
        let fired = dispatch::prompt(deps, ui, ts, &fleet.workspace, &prepared, &goal, None);
        if fired.is_err() {
            // The claim above landed and the fire did not: give it back. The
            // claimant is the workspace's own leaf, which is what the start
            // flow stamped `--as` a moment ago.
            release(deps, ui, ts, row, &crate::naming::leaf(&fleet.workspace));
        }
        fired.ok()
    }

    /// This pass's [`Deps`]: the template plus the snapshot it just read,
    /// exactly as the gestures consumer builds one.
    fn deps(&self, snapshot: &Arc<Snapshot>) -> Deps {
        Deps {
            snapshot: Arc::clone(snapshot),
            ..self.deps.clone()
        }
    }
}

/// **What one tick decides** — the level-triggered decision, split off at
/// §12's per-file budget (bl-b4b5) on the seam this module's own doc draws:
/// above is the thread and the acts it runs through the boundary, `plan` is the
/// pure function over a published snapshot that says which act, if any.
mod plan;

pub use plan::plan;

/// Give `row`'s claim back from `name`, through the boundary's own door, and
/// say whether it actually came back. **The loop's one spelling of a release**
/// — the lease reap spends it and so does a birth undoing its own claim
/// (bl-ab13), which is what keeps the two from drifting into two acts.
fn release(deps: &Deps, ui: &mut UiState, ts: &str, row: &BoardRow, name: &str) -> bool {
    released(dispatch::dispatch(
        deps,
        ui,
        ts,
        &Action::Release {
            project: row.project.clone(),
            id: row.id.clone(),
            name: name.to_owned(),
        },
    ))
}

/// Whether a released claim actually came back: the verb ran *and* exited
/// clean. Anything else leaves the ball where it was, and the next tick decides
/// again against the world as it then is.
fn released(reply: Result<crate::boundary::reply::Reply, String>) -> bool {
    matches!(reply, Ok(crate::boundary::reply::Reply::Outcome(o)) if o.ok())
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
