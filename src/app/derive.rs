//! The derivation worker's state and its one pass (DESIGN §7.2, §7.3).
//!
//! **Everything yog derives from disk happens here, and here is never the frame
//! thread** (bl-ee0a). One pass: drain the watchers and the frame's dirty marks
//! and route each root by kind — a workspace root opens a debounce window, an
//! enumeration root re-enumerates + reconciles the watch set, the yog-state root
//! re-reads `ui.json` and the ops tail, a balls-clone root re-fetches that
//! project. Then the periodic sweeps ([`sweeps`](super::sweeps)) and,
//! finally, re-derivation of every workspace whose 100 ms window has elapsed.
//! A pass that changed anything **publishes** a new [`Snapshot`] and the frame
//! renders it; a pass that changed nothing publishes nothing.
//!
//! Cost scales with the workspace's branch count, which is why it cannot ride
//! the frame: 227 branches appearing under one conversation in 90 s used to
//! mean 227 branches walked inside `App::update`, and a frame that does not
//! return is a window the desktop calls unresponsive (bl-ee0a).
//!
//! **Every dirty root carries why it is dirty** ([`Mark`]), and the sweeps
//! report what they FOUND, not merely that they ran (§7.2, [`super::drift`]): a
//! re-derivation that changes a snapshot nobody announced is a dropped event,
//! and so is a pass that took longer than the cadence it promised.

/// Which root means what — the §7.1 dirty-root routing table.
mod route;
/// The work one pass does — the sweeps, the reconcile, the fetch cadence.
mod sweeps;
/// The thread the pass runs on.
pub mod worker;

use super::Roots;
use super::drift::{self, Drift};
use super::snapshot::{Growth, Snapshot};
use crate::binding::Workspace;
use crate::budgets::StepBill;
use crate::git_tree::{GitTree, ProbeStack};
use crate::opslog::OpRow;
use crate::projects::balls::Ball;
use crate::projects::join::JoinRow;
use crate::projects::runner::BlRunner;
use crate::state::{
    DirtySet, SnapshotCell, WatchSetHandle, lock_watchset, new_watchset, publish_snapshot,
};
use crate::ui_state::Clock;
use crate::watch::Mark;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// The derivation half of yog, owned by one worker thread (§7.2).
///
/// It holds every effect the derivation needs — the platform probe stack, the
/// watch set, the `bl` read runner, the injected clock — and every cache it
/// keeps warm. Nothing here is reachable from the frame except through the two
/// hand-offs in [`crate::state`]: the frame marks roots dirty, the worker
/// publishes snapshots.
pub struct Deriver {
    roots: Roots,
    clock: Arc<dyn Clock>,
    probes: ProbeStack,
    /// The §7.1 watchers, shared with the [`Bridge`](crate::watch::Bridge): the
    /// worker reconciles them, the bridge drains them into `dirty`. Ingest stays
    /// its own thread so an *announcement* and a *derivation* remain two
    /// separately observable facts (§7.2 provenance).
    pub(super) watches: WatchSetHandle,
    dirty: DirtySet,
    schedule: super::dirty::Schedule,
    /// The clock's live periods (bl-3381): the last adopted `cadence.yaml`
    /// read, mirrored into [`schedule`](Self::schedule) and ridden out on every
    /// snapshot. Re-read only when the yog-state root announces a change —
    /// never per tick.
    pub(super) cadence: super::Cadence,
    /// The armed §4.3 fleet loops (bl-66fb), read from the same file and on the
    /// same announcement as [`cadence`](Self::cadence) — one read of one file,
    /// two policies out of it — and ridden out on every snapshot so the board
    /// can render a cap without a frame touching disk.
    pub(super) fleet: std::collections::BTreeMap<String, crate::fleet::Policy>,
    cell: SnapshotCell,
    balls: Box<dyn BlRunner>,
    /// Every enumerated project's decoded invocation path (§5.1 #1) — the
    /// routing key a frame names when a `bl` verb landed there. Refreshed by
    /// the ball fetch, never stored beyond it.
    pub(super) projects: Vec<PathBuf>,
    pub(super) workspaces: Vec<Workspace>,
    pub(super) trees: HashMap<PathBuf, GitTree>,
    /// The per-workspace `steps/` fold ridden out on every snapshot (§3.5,
    /// bl-9dd4) — the one walk every spend figure is then a filter over.
    pub(super) bills: HashMap<PathBuf, Vec<StepBill>>,
    /// The §9.2 `models.yaml` context-window declarations (§5.1 #35), ridden
    /// out on every snapshot for the same reason the cadence periods are: the
    /// frame must not read disk to say a number.
    pub(super) windows: std::collections::BTreeMap<String, u64>,
    pub(super) balls_by_project: HashMap<PathBuf, Vec<Ball>>,
    pub(super) closed_by_project: HashMap<PathBuf, Vec<Ball>>,
    pub(super) join_rows: Vec<JoinRow>,
    pub(super) ops: Vec<OpRow>,
    /// Set by every mutation a pass makes; consumed by [`Deriver::step`] to
    /// decide whether there is a new snapshot to publish at all.
    pub(super) changed: bool,
    /// Whether the **last** pass missed the period it promised (bl-4b28): the
    /// one bit that makes [`Drift::Late`] an edge rather than a level. This
    /// worker's own observation about its own passes; a second instance holds
    /// and reports its own.
    late: bool,
    growth: Vec<Growth>,
    ui_bytes: Option<Vec<u8>>,
}

impl Deriver {
    /// Build the worker's state. Nothing is read yet — [`boot`](Self::boot)
    /// takes the first derivation, so construction stays cheap and the caller
    /// decides when the first disk pass happens.
    pub(super) fn new(
        roots: Roots,
        clock: Arc<dyn Clock>,
        balls: Box<dyn BlRunner>,
        dirty: DirtySet,
        cell: SnapshotCell,
    ) -> Self {
        let cadence = super::Cadence::default();
        let schedule = super::dirty::Schedule::new(Arc::clone(&clock), cadence);
        Self {
            roots,
            clock,
            probes: ProbeStack::platform(),
            watches: new_watchset(),
            dirty,
            schedule,
            cadence,
            fleet: std::collections::BTreeMap::new(),
            cell,
            balls,
            projects: Vec::new(),
            workspaces: Vec::new(),
            trees: HashMap::new(),
            bills: HashMap::new(),
            windows: std::collections::BTreeMap::new(),
            balls_by_project: HashMap::new(),
            closed_by_project: HashMap::new(),
            join_rows: Vec::new(),
            ops: Vec::new(),
            changed: false,
            late: false,
            growth: Vec::new(),
            ui_bytes: None,
        }
    }

    /// The shared watch set, for the [`Bridge`](crate::watch::Bridge) that
    /// drains it (§7.2 — `main.rs` wires the two together).
    pub fn watchset_handle(&self) -> WatchSetHandle {
        Arc::clone(&self.watches)
    }

    /// The frame→worker dirty hand-off, for the same wiring.
    pub fn dirty_handle(&self) -> DirtySet {
        self.dirty.clone()
    }

    /// The first derivation: arm the watches, snapshot every workspace, take the
    /// first ball/ops fetch, publish.
    ///
    /// Arm BEFORE the first read, never after. A watch armed after the snapshot
    /// is blind to everything that landed in between, and that gap is a dropped
    /// event by construction — silently repaired 15 s later by the full sweep,
    /// which is exactly the self-healing this design is trying to stop needing
    /// (§7.2). Arm-then-read has no window at all: a change during the read is
    /// announced and re-derived; a change before the read is already in it.
    /// Enumerating and deriving *directly* rather than through
    /// [`reconcile`](Self::reconcile) is deliberate: reconcile opens a debounce
    /// window for every workspace it finds un-snapshotted, and a window opened
    /// here would still be pending on the first real pass — where it would
    /// outrank that pass's own [`Mark`] and make a genuine dropped event read as
    /// the watcher working (§7.2 provenance).
    pub(super) fn boot(&mut self) {
        // The clock's periods first (bl-3381): the first schedule decision
        // already runs at the operator's tuning, not one default tick of it.
        self.adopt_cadence();
        self.adopt_windows();
        self.workspaces = crate::binding::workspaces(&self.roots.yog_data, &self.roots.lernie_data);
        lock_watchset(&self.watches)
            .reconcile(&super::desired_watches(&self.roots, &self.workspaces));
        let paths: Vec<PathBuf> = self.workspaces.iter().map(|w| w.path.clone()).collect();
        for path in paths {
            self.rederive(&path);
        }
        self.refresh_balls();
        self.refresh_ops();
        self.publish();
    }

    /// One derivation pass (§7.2). Returns whether it published a new snapshot
    /// — the worker's repaint trigger, and, for a test driving the pass by
    /// hand, "something happened". Public because the derivation is drivable
    /// without its thread on purpose: that is what keeps it testable.
    pub fn step(&mut self) -> bool {
        let started = self.clock.now();
        self.changed = false;
        self.growth.clear();
        self.ui_bytes = None;
        let delivered = self.dirty.drain();
        // The backend's own announcement of a loss: the one drop class the
        // kernel tells us about, so it is recorded even though yog handles it.
        let mut found: Vec<Drift> = delivered
            .iter()
            .filter(|(_, mark)| **mark == Mark::Desync)
            .map(|(root, _)| Drift::Desync(root.clone()))
            .collect();
        self.dispatch_dirty(delivered);
        let sweep = self.schedule.sweep();
        match sweep {
            super::dirty::Sweep::Full => found.extend(self.full_sweep()),
            super::dirty::Sweep::Cheap => found.extend(self.cheap_sweep()),
            super::dirty::Sweep::None => {}
        }
        for (root, mark) in self.schedule.due() {
            // Drift is *divergence*, and divergence needs a baseline. A root with
            // no snapshot yet is taking its first one — there is nothing it could
            // have diverged from, and no watch was armed over it to have dropped
            // an event either (watches are armed from the enumerated set, so a
            // root yog has never derived is a root yog has never watched). Its
            // appearance is the enumeration side's question, asked once there
            // (`Unenumerated`); accusing it here as well made every newborn
            // workspace two findings for one event (bl-f726).
            let baseline = self.trees.contains_key(&root);
            if mark == Mark::Watch || mark == Mark::Desync {
                self.refresh_liveness(&root);
            }
            if self.rederive(&root) && mark == Mark::Sweep && baseline {
                found.push(Drift::Unannounced(root));
            }
        }
        // Judged against the promise of the sweep this pass ran, and written on
        // the EDGE into lateness (bl-4b28): a permanently late derivation is one
        // event, not a row every sweep. Whether it is late *now* is the §11
        // staleness line's question, derived from the snapshot's own age.
        let late = drift::lateness(started, self.clock.now(), self.cadence.late_pass(sweep));
        if let Some(secs) = drift::late_edge(late, self.late) {
            found.push(Drift::Late(self.roots.yog_state.clone(), secs));
        }
        self.late = late.is_some();
        self.report_drift(&found);
        // A full sweep publishes even when it found nothing: it re-stamps the
        // snapshot's age, which is what makes the §11 staleness line mean
        // "passes are not completing" rather than "the world is quiet".
        let publish = self.changed || sweep == super::dirty::Sweep::Full;
        if publish {
            self.publish();
        }
        publish
    }

    /// Freeze the derived state into a new [`Snapshot`] and hand it to the
    /// frame. The clone is the price of immutability, and it is paid here — on
    /// the worker, once per changed pass — rather than by a frame that would
    /// otherwise have to hold a lock across a whole render.
    pub(super) fn publish(&mut self) {
        publish_snapshot(
            &self.cell,
            Arc::new(Snapshot {
                workspaces: self.workspaces.clone(),
                trees: self.trees.clone(),
                bills: self.bills.clone(),
                windows: self.windows.clone(),
                balls_by_project: self.balls_by_project.clone(),
                closed_by_project: self.closed_by_project.clone(),
                join_rows: self.join_rows.clone(),
                ops: self.ops.clone(),
                growth: std::mem::take(&mut self.growth),
                ui_bytes: self.ui_bytes.take(),
                derived_at: self.clock.now(),
                cadence: self.cadence,
                fleet: self.fleet.clone(),
            }),
        );
    }
}
