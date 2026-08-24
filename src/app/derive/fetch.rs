//! **The live `bl` projection and the ops tail** (DESIGN §5.1 #2/#4, §4.2) —
//! the fetches, split off [`super::sweeps`] at §12's budget on the seam that
//! file's own doc lists: there are the sweeps and the reconcile (what the world
//! *is*), here is what yog re-reads out of `bl` and out of `ops.jsonl`.
//!
//! Two entry points and one cadence rule between them: the whole-world fetch
//! runs on the clones-root dirtiness or the 15 s full sweep, and the one-project
//! fetch runs after a dispatched verb. Never per frame — a ball fetch is a
//! directory walk per project, which is exactly the cost that used to stall the
//! window (bl-ee0a).

use super::Deriver;
use crate::opslog::{self, OpRow};
use crate::projects;
use crate::projects::join;
use std::path::{Path, PathBuf};

impl Deriver {
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
    pub(super) fn rebuild_join(&mut self, cloned: &[PathBuf]) {
        let rows = join::join(
            &self.projects,
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
        let rows: Vec<OpRow> = opslog::tail(&root, opslog::OPS_TAIL)
            .iter()
            .map(|entry| OpRow::from(&opslog::detached::fold(&root, entry)))
            .collect();
        if rows != self.ops {
            self.ops = rows;
            self.changed = true;
        }
    }
}
