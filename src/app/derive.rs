//! **The derivation worker's state** (DESIGN §7.2, §7.3): every effect the
//! derivation needs and every cache it keeps warm, in one value owned by one
//! thread.
//!
//! **Everything yog derives from disk happens here, and here is never the frame
//! thread** (bl-ee0a). Cost scales with the workspace's branch count, which is
//! why it cannot ride the frame: 227 branches appearing under one conversation
//! in 90 s used to mean 227 branches walked inside `App::update`, and a frame
//! that does not return is a window the desktop calls unresponsive.
//!
//! What one pass *does* with all of it is [`pass`], split off at §12's budget
//! on the seam this file's own doc drew — drain the marks, route each root by
//! kind, sweep, re-derive, publish — and the work each of those steps ends in
//! is [`sweeps`] beside it.

/// The live `bl` projection and the ops tail — the fetches, split off
/// [`sweeps`] at §12's budget on the seam that file's doc lists.
mod fetch;
/// Which cached liveness observations are thrown away, and on which signal.
mod liveness;
/// One pass: what is dirty, what is due, what gets published — split off this
/// file at §12's budget on the seam its own doc draws (state ↔ pass).
mod pass;
/// Which root means what — the §7.1 dirty-root routing table.
mod route;
/// The work one pass does — the sweeps, the reconcile, the fetch cadence.
mod sweeps;
/// The thread the pass runs on.
pub mod worker;

use super::Roots;
use super::snapshot::Growth;
use crate::binding::Workspace;
use crate::budgets::StepBill;
use crate::git_tree::{GitTree, ProbeStack};
use crate::opslog::OpRow;
use crate::projects::balls::Ball;
use crate::projects::join::JoinRow;
use crate::projects::runner::BlRunner;
use crate::state::{DirtySet, SnapshotCell, WatchSetHandle, new_watchset};
use crate::ui_state::Clock;
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
}
