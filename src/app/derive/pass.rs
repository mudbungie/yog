//! **One derivation pass** (DESIGN §7.2, §7.3) — the first one and every one
//! after it, split off [`super`] at §12's budget on the seam that module's own
//! doc names: there is the worker's *state* (every effect it holds and every
//! cache it keeps warm), here is what one pass DOES with it.
//!
//! Drain the watchers and the frame's dirty marks and route each root by kind,
//! run the periodic sweeps ([`super::sweeps`]), then re-derive every workspace
//! whose debounce window has elapsed. A pass that changed anything **publishes**
//! a new [`Snapshot`] and the frame renders it; a pass that changed nothing
//! publishes nothing.
//!
//! **Every dirty root carries why it is dirty** ([`Mark`]), and the sweeps
//! report what they FOUND, not merely that they ran (§7.2, [`super::super::drift`]):
//! a re-derivation that changes a snapshot nobody announced is a dropped event,
//! and so is a pass that took longer than the cadence it promised.

use super::super::desired_watches;
use super::super::dirty::Sweep;
use super::super::drift::{self, Drift};
use super::super::snapshot::Snapshot;
use super::Deriver;
use crate::state::{lock_watchset, publish_snapshot};
use crate::watch::Mark;
use std::path::PathBuf;
use std::sync::Arc;

impl Deriver {
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
    pub(in crate::app) fn boot(&mut self) {
        // The clock's periods first (bl-3381): the first schedule decision
        // already runs at the operator's tuning, not one default tick of it.
        self.adopt_cadence();
        self.adopt_windows();
        self.workspaces = crate::binding::workspaces(&self.roots.yog_data, &self.roots.litany_data);
        lock_watchset(&self.watches).reconcile(&desired_watches(&self.roots, &self.workspaces));
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
            Sweep::Full => found.extend(self.full_sweep()),
            Sweep::Cheap => found.extend(self.cheap_sweep()),
            Sweep::None => {}
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
        let publish = self.changed || sweep == Sweep::Full;
        if publish {
            self.publish();
        }
        publish
    }

    /// Freeze the derived state into a new [`Snapshot`] and hand it to the
    /// frame. The clone is the price of immutability, and it is paid here — on
    /// the worker, once per changed pass — rather than by a frame that would
    /// otherwise have to hold a lock across a whole render.
    pub(in crate::app) fn publish(&mut self) {
        publish_snapshot(
            &self.cell,
            Arc::new(Snapshot {
                workspaces: self.workspaces.clone(),
                projects: self.projects.clone(),
                trees: self.trees.clone(),
                bills: self.bills.clone(),
                windows: self.windows.clone(),
                balls_by_project: self.balls_by_project.clone(),
                closed_by_project: self.closed_by_project.clone(),
                join_rows: self.join_rows.clone(),
                ops: self.ops.clone(),
                growth: std::mem::take(&mut self.growth),
                ui_bytes: self.ui_bytes.take(),
                derived_at_unix: self.clock.unix(),
                cadence: self.cadence,
                fleet: self.fleet.clone(),
            }),
        );
    }
}
