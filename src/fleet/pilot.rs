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

use super::Facts;
use crate::board::BoardRow;
use crate::boundary::dispatch::Deps;
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
}

/// **What one tick decides** — the level-triggered decision, split off at
/// §12's per-file budget (bl-b4b5) on the seam this module's own doc draws:
/// above is the thread and the acts it runs through the boundary, `plan` is the
/// pure function over a published snapshot that says which act, if any, and
/// `act` is what doing it costs.
mod act;
mod plan;

pub use plan::plan;

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
