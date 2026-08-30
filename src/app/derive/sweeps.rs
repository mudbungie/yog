//! The work one derivation pass does: the two sweeps, the watch-set reconcile,
//! and re-deriving one root (DESIGN §7.2, §7.3). The live `bl` projection and
//! the ops tail are [`super::fetch`], split off at §12's budget.
//!
//! Split from [`super::pass`] per §12's budget — that file is the *pass*
//! (what is dirty, what is due, what gets published), this one is the *work*:
//! the sweeps, the fetches, and — since bl-4b28 — re-deriving one root, which
//! is the work every one of them ends in.
//! All of it runs on the derivation worker, never on the frame: the ball fetch
//! is a directory walk per project and the full sweep re-derives every
//! workspace, and those are exactly the costs that used to stall the window
//! (bl-ee0a).

use super::super::desired_watches;
use super::super::drift::{self, Drift};
use super::super::snapshot::growth_between;
use super::Deriver;
use crate::budgets::Scope;
use crate::fs_watcher::RootKind;
use crate::opslog;
use crate::watch::Mark;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

impl Deriver {
    /// Re-enumerate the workspace set, reconcile the watch set to it, prune
    /// snapshots for vanished workspaces, and mark newly-appeared workspaces for
    /// an initial derive (§7.2 cheap enumeration; §7.3 re-primed-clone rebuild).
    ///
    /// `mark` is why this reconcile is running. Under [`Mark::Sweep`] a
    /// workspace whose *membership changed here* appeared or vanished with no
    /// enumeration-root event announcing it — drift, returned as
    /// [`Drift::Unenumerated`]. Keying on the membership delta (not on "has no
    /// snapshot yet") is what keeps this a one-shot finding: a workspace that
    /// simply fails to derive stays un-snapshotted forever and must not
    /// re-accuse the watcher every 2 s.
    ///
    /// …and only where an announcement was *possible*: the delta is filtered to
    /// the enumeration roots [`announceable`](Self::announceable) says were armed
    /// before this pass re-armed anything. On a pristine world the names root
    /// does not exist until the start flow's own `create_dir_all` founds it
    /// (§8.1 `EnsureWorkspace`), so the first workspace is born under a directory
    /// nothing was watching — the watcher dropped no event, it was never given
    /// one, and accusing it painted a healthy first run red forever (bl-f726).
    pub(super) fn reconcile(&mut self, mark: Mark) -> Vec<Drift> {
        let before: BTreeSet<PathBuf> = self.workspaces.iter().map(|w| w.path.clone()).collect();
        let announceable = self.announceable();
        self.workspaces = crate::binding::workspaces(&self.roots.yog_data, &self.roots.litany_data);
        let desired = desired_watches(&self.roots, &self.workspaces);
        crate::state::lock_watchset(&self.watches).reconcile(&desired);
        let known: BTreeSet<PathBuf> = self.workspaces.iter().map(|w| w.path.clone()).collect();
        if before != known {
            self.changed = true;
        }
        self.trees.retain(|path, _| known.contains(path));
        let mut missing = Vec::new();
        for w in &self.workspaces {
            if !self.trees.contains_key(&w.path) {
                missing.push((w.path.clone(), mark));
            }
        }
        self.schedule.mark(missing);
        // The workspace set is a join axis (§3.5): a freshly named workspace (the
        // start flow's `litany new`) that lands via a NamesRoot event must re-bind
        // the balls at once, else the just-claimed ball renders claimed-elsewhere
        // until the 15 s sweep. Rebuild the join over the already-fetched balls.
        let cloned: Vec<PathBuf> = self.balls_by_project.keys().cloned().collect();
        self.rebuild_join(&cloned);
        if mark != Mark::Sweep {
            return Vec::new();
        }
        before
            .symmetric_difference(&known)
            // The three roots are flat (§3.1), so a workspace's parent *is* the
            // enumeration root that would have announced it.
            .filter(|ws| ws.parent().is_some_and(|root| announceable.contains(root)))
            .cloned()
            .map(Drift::Unenumerated)
            .collect()
    }

    /// The enumeration roots a watcher is armed on **right now** — read before
    /// [`reconcile`](Self::reconcile) re-arms, so it answers "was an announcement
    /// possible over the interval this delta happened in?".
    ///
    /// Derived from `desired_watches` rather than restating the root list: that
    /// function is the single home of which root is which kind (§7.1), and a
    /// second copy here would be the fourth place the names root is spelled.
    fn announceable(&self) -> BTreeSet<PathBuf> {
        let set = crate::state::lock_watchset(&self.watches);
        desired_watches(&self.roots, &[])
            .into_iter()
            .filter(|(root, kind)| {
                matches!(kind, RootKind::NamesRoot | RootKind::WorkspacesRoot)
                    && set.watches(root, *kind)
            })
            .map(|(root, _)| root)
            .collect()
    }

    /// The 2 s cheap sweep (§7.2): reconcile, then the *targeted* liveness
    /// re-probe — for each workspace holding a Live/InFlight agent, evict its
    /// agents' cached lock observations (§10 eager refresh, so a silently
    /// released flock is caught) and mark it for re-derivation.
    ///
    /// The liveness half is a poll of **process state**, not of the filesystem:
    /// a released flock emits no event for any watcher to drop, so its roots are
    /// marked [`Mark::Poll`] and a change under them is the poll working, never
    /// drift. That separation is why the two sweeps are justified separately
    /// (§7.2).
    pub(super) fn cheap_sweep(&mut self) -> Vec<Drift> {
        let found = self.reconcile(Mark::Sweep);
        self.reprobe_live();
        found
    }

    /// The 15 s full sweep (§7.2): reconcile, re-fetch every project's balls and
    /// the ops tail (the fetch cadence's floor), and mark every workspace
    /// [`Mark::Sweep`] — so a dropped inotify event costs ≤15 s of latency,
    /// never divergence, **and is named** when the re-derivation proves one
    /// happened.
    pub(super) fn full_sweep(&mut self) -> Vec<Drift> {
        let found = self.reconcile(Mark::Sweep);
        self.refresh_balls();
        self.refresh_ops();
        // The §5.1 #35 windows ride the fetch cadence's floor for the same
        // reason the balls do: one hand-edited world-global file, re-read on
        // the sweep rather than watched (`adopt_windows`).
        self.adopt_windows();
        let all: Vec<(PathBuf, Mark)> = self
            .workspaces
            .iter()
            .map(|w| (w.path.clone(), Mark::Sweep))
            .collect();
        self.schedule.mark(all);
        found
    }

    /// Re-derive one workspace through the held probe stack, replacing its
    /// snapshot iff it actually changed (`GitTree: PartialEq` suppresses no-op
    /// repaints, §7.2) and recording what grew (§7.2 growth, bl-ee0a). A read
    /// failure keeps the last good snapshot.
    pub(super) fn rederive(&mut self, workspace: &Path) -> bool {
        // The `steps/` fold first, and **outside the tree's equality gate**
        // (bl-9dd4): a step's `response.json` growing is spend that changed
        // while every git ref stood still, so a fold behind `old == tree` would
        // freeze the spend column at whatever it read when the refs last moved.
        let billed = self.refold_bills(workspace);
        let Ok(tree) = self.probes.derive(workspace) else {
            return billed;
        };
        let old = self.trees.get(workspace);
        if old == Some(&tree) {
            return billed;
        }
        self.growth.extend(growth_between(workspace, old, &tree));
        self.trees.insert(workspace.to_path_buf(), tree);
        self.changed = true;
        true
    }

    /// Re-walk one workspace's `steps/` tree, replacing its bills iff they
    /// actually changed — the same no-op-repaint discipline the tree read
    /// keeps, so a quiet workspace publishes nothing.
    fn refold_bills(&mut self, workspace: &Path) -> bool {
        let bills = crate::budgets::bills(workspace, &Scope::Workspace);
        if self.bills.get(workspace) == Some(&bills) {
            return false;
        }
        self.bills.insert(workspace.to_path_buf(), bills);
        self.changed = true;
        true
    }

    /// Append this pass's findings to `ops.jsonl` and re-read the tail, so the
    /// §11 activity chip carries the drift count on the very snapshot it was
    /// found in. Silent when nothing was found — the normal case, and the one
    /// the design is aiming at. The `ts` comes from the injected clock (§4.2),
    /// which is why this crate still reads no wall clock of its own.
    pub(super) fn report_drift(&mut self, found: &[Drift]) {
        if found.is_empty() {
            return;
        }
        let root = self.roots.yog_state.clone();
        let cwd = root.to_string_lossy().into_owned();
        for entry in drift::entries(&self.clock.stamp(), &cwd, found) {
            let _ = opslog::append(&root, &entry);
        }
        self.refresh_ops();
    }
}
