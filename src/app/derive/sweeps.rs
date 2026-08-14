//! The work one derivation pass does: the two sweeps, the watch-set reconcile,
//! the live `bl` projection and the ops tail (DESIGN §7.2, §5.1 #2/#4, §4.2).
//!
//! Split from [`super::derive`] per §12's budget — that file is the *pass*
//! (what is dirty, what is due, what gets published), this one is the *work*:
//! the sweeps, the fetches, and — since bl-4b28 — re-deriving one root, which
//! is the work every one of them ends in.
//! All of it runs on the derivation worker, never on the frame: the ball fetch
//! is a directory walk per project and the full sweep re-derives every
//! workspace, and those are exactly the costs that used to stall the window
//! (bl-ee0a).

use super::super::drift::{self, Drift};
use super::super::snapshot::growth_between;
use super::super::{desired_watches, needs_liveness_reprobe};
use super::Deriver;
use crate::budgets::Scope;
use crate::fs_watcher::RootKind;
use crate::git_tree::AgentState;
use crate::opslog::{self, OpRow};
use crate::projects;
use crate::projects::join;
use crate::watch::Mark;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// How many `ops.jsonl` lines the ops pane tails (§4.2, §11 accessory).
const OPS_TAIL: usize = 256;

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
        self.workspaces = crate::binding::workspaces(&self.roots.yog_data, &self.roots.lernie_data);
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
        // The workspace set is a join axis (§3.5): a fresh minted workspace (the
        // start flow's `lernie new`) that lands via a NamesRoot event must re-bind
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
        let mut live: Vec<(PathBuf, Vec<String>)> = Vec::new();
        for (path, tree) in &self.trees {
            if needs_liveness_reprobe(tree) {
                let ids = tree.agents.iter().map(|a| a.agent_id.clone()).collect();
                live.push((path.clone(), ids));
            }
        }
        for (path, ids) in live {
            self.probes.invalidate_liveness(&path, &ids);
            self.schedule.mark([(path, Mark::Poll)]);
        }
        found
    }

    /// §10's **eager liveness refresh**, the half the cheap sweep cannot do: a
    /// filesystem change under an agent at rest may be a driver *arriving*, and
    /// the macOS probe cache would answer from store for up to its 2 s TTL — so
    /// the row stays at rest, and the §11 flight strip stays down, while a model
    /// call is already running.
    ///
    /// **Exactly the agents the sweep does not take.** [`cheap_sweep`](sweeps)
    /// re-probes the Live/InFlight ones, because only those can die *silently* —
    /// a released flock emits no event for any watcher to carry. So this takes
    /// the rest, on the signal that *is* an event. Between them every transition
    /// is observed, and neither pays for the other's case: a streaming
    /// `response.json` append storm is an agent already known live, so it evicts
    /// nothing here and stays collapsed on the cache, which is the whole reason
    /// the cache exists.
    ///
    /// A root with no tree is not a workspace (a root deriving for the first
    /// time, or one whose read failed) — nothing observed yet, nothing to
    /// forget. On Linux the eviction is a no-op: the `/proc` probes are
    /// stateless and always definite (§10).
    pub(super) fn refresh_liveness(&self, root: &Path) {
        let Some(tree) = self.trees.get(root) else {
            return;
        };
        let resting: Vec<String> = tree
            .agents
            .iter()
            .filter(|a| !matches!(a.state, AgentState::Live | AgentState::InFlight))
            .map(|a| a.agent_id.clone())
            .collect();
        self.probes.invalidate_liveness(root, &resting);
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

    /// Re-fetch every *visible* project's live ball list (§5.1 #2) and rebuild
    /// the §3.5 join. The fetch-cadence entry point: run on the clones-root
    /// dirtiness or the 15 s full sweep (§7.2), never per frame. Nested-delivery
    /// clones are never walked (§5.1 #1, bl-e3e7).
    pub(super) fn refresh_balls(&mut self) {
        let all = projects::enumerate(&self.roots.balls_clones);
        self.projects = all.iter().map(|p| p.path.clone()).collect();
        let visible: Vec<PathBuf> = projects::visible(&all)
            .into_iter()
            .map(|p| p.path.clone())
            .collect();
        // Key only the projects that list cleanly; a cloned project absent from
        // the map is unlistable → an orphaned row in the join (§3.5).
        self.balls_by_project = visible
            .iter()
            .filter_map(|p| Some((p.clone(), self.balls.live(p).ok()?)))
            .collect();
        self.rebuild_join(&visible);
    }

    /// Re-fetch **one** project after a dispatched `bl` verb (§15 Y16): its live
    /// *and* closed balls (the delivered-row source, §5.1 #4 — closed is fetched
    /// only here, never on the cadence), then rebuild the join and re-read the
    /// ops tail. The frame reaches this by marking the project's clone dir
    /// dirty; it is the same routing every other root gets, not a second
    /// channel.
    pub(super) fn refetch_project(&mut self, project: &Path) {
        // Forgiving here (unlike the cadence fetch): a verb just ran against this
        // project, so an unreadable listing is transient noise, not the §3.5
        // orphan signal — show no balls and let the next sweep re-derive.
        let live = self.balls.live(project).unwrap_or_default();
        let closed = self.balls.closed(project).unwrap_or_default();
        self.balls_by_project.insert(project.to_path_buf(), live);
        self.closed_by_project.insert(project.to_path_buf(), closed);
        let cloned: Vec<PathBuf> = self.balls_by_project.keys().cloned().collect();
        self.rebuild_join(&cloned);
        self.refresh_ops();
    }

    /// Rebuild the join rows from the cached live + closed balls and the
    /// enumerated workspaces (§3.5). The binding is claimant = workspace name
    /// (§3.2); no operator identity enters. Pure over the pre-fetched caches.
    fn rebuild_join(&mut self, cloned: &[PathBuf]) {
        let rows = join::join(
            cloned,
            &self.balls_by_project,
            &self.closed_by_project,
            &self.workspaces,
        );
        if rows != self.join_rows {
            self.join_rows = rows;
            self.changed = true;
        }
    }

    /// Re-read the `ops.jsonl` tail (§4.2). Run on the yog-state dirtiness, the
    /// full sweep, and after any dispatched verb.
    ///
    /// Each line is projected through [`opslog::detached::fold`] first, so a
    /// detached driver's captured stderr — the only evidence a fired prompt died
    /// after launch (§8.1, §13.3) — reaches the row from its sink file on *this*
    /// pass. The fold is read-time by construction: the sink stays the
    /// authority, `ops.jsonl` is never rewritten, and a driver still writing
    /// surfaces more on the next pass.
    pub(super) fn refresh_ops(&mut self) {
        let root = self.roots.yog_state.clone();
        let rows: Vec<OpRow> = opslog::tail(&root, OPS_TAIL)
            .iter()
            .map(|entry| OpRow::from(&opslog::detached::fold(&root, entry)))
            .collect();
        if rows != self.ops {
            self.ops = rows;
            self.changed = true;
        }
    }
}
