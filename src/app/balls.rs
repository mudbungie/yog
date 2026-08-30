//! What a frame asks of the live `bl` projection — the §3.5 join, the ops tail,
//! and the two verb hooks that tell the worker a `bl`/`litany` action landed
//! (DESIGN §5.1 #2/#7, §7.2, §4.2, §15 Y16).
//!
//! Y14 built the pure projection ([`crate::projects`]); Y16 held it live. Since
//! bl-ee0a the *holding* is the worker's — the [`BlRunner`](crate::projects::runner::BlRunner)
//! fetch cadence, the join rebuild and the ops re-read all run in
//! [`Deriver`](super::Deriver) — and what is left here is reading the result and
//! naming the root a dispatched verb changed, so the worker re-fetches ahead of
//! the watch. The convergence the operator sees is unchanged; it arrives on the
//! next pass instead of inside the click's own frame, which is what stops a
//! `bl` listing from happening on the paint thread at all.

use super::AppModel;
use crate::cli_outbound::Cli;
use crate::opslog::SurfaceFailure;
use crate::projects::join;
use crate::projects::runner;
use std::path::{Path, PathBuf};

/// The empty-project hint's two lines (STORIES S3-T5, bl-b491): elidable prose
/// then the verbatim command. The split is the fix — see
/// [`AppModel::empty_project_hint`].
#[derive(Debug, PartialEq, Eq)]
pub struct EmptyHint {
    /// The prose that introduces the command; may elide harmlessly.
    pub lead: String,
    /// The command to type, verbatim — rendered alone so it never elides.
    pub command: String,
}

impl AppModel {
    /// The operator identity (§4.1): recorded `identity_last_used` else `$USER`
    /// else empty. **Not** a claim stamp — Z3's start flow and Z4's close/release/
    /// assign/move all stamp `--as <workspace name>` (§3.2 ownership line), never
    /// the operator. Retained as the *author* identity for the standalone `bl
    /// create`/`bl update` verbs (§8.2 New ball / Update ball), where the operator
    /// — not a workspace — is the reporter.
    pub fn identity(&self) -> String {
        runner::identity(self.ui.identity_last_used(), self.identity_user.clone())
    }

    /// **One surface's** last-failure view-model (§5.3, §7.3): the most recent
    /// ops row `origin` attributed to that surface, *iff* it is a rendered
    /// failure ([`OpRow::failed`]), projected to its argv and stderr tail.
    /// `None` when that surface's last attempted action succeeded, so the
    /// surface clears its ichor-red banner. Reads the already-derived tail, so
    /// the banner and the ops pane never diverge (both project the same durable
    /// ops line, §4.2). The shell paints it; nothing is held.
    ///
    /// **The banner's lifetime now ends at an ack as well** (bl-c417): the query
    /// runs over [`since_ack`](crate::opslog::since_ack)'s rows, so a dismissal
    /// quiets it even though nothing was retried. It is still not a stored flag —
    /// the watermark is the newest ack *line*, and a NEW failure of this origin
    /// lands after it and banners again.
    ///
    /// The origin parameter is the whole fix for bl-48f8. Un-parameterised this
    /// asked one global question — "did the *last* op fail?" — which three
    /// surfaces then answered identically, so a failed ▶ Start painted itself on
    /// the balls fold, the composer and the bootstrap box at once, and any one
    /// surface's clean run wiped the other two's live banners. Per-origin it is
    /// the same rule with the general input: the last row **of this surface**,
    /// iff it failed. §6's retirement therefore stays per-surface too — a clean
    /// re-run retires the banner it superseded and no one else's.
    pub fn last_failure(&self, origin: crate::opslog::Origin) -> Option<SurfaceFailure> {
        crate::opslog::since_ack(&self.snap.ops)
            .iter()
            .rev()
            .find(|r| r.origin == origin)
            .filter(|r| r.failed())
            .map(SurfaceFailure::from)
    }

    /// The yog state root — where `ops.jsonl` lives, the verb-log target the
    /// shell passes to [`crate::actions::verbs`] (§4.2).
    pub(crate) fn state_root(&self) -> &Path {
        &self.roots.yog_state
    }

    /// The empty-project roster hint (STORIES S3-T5) as its two rendered lines.
    /// With **zero projects** in the
    /// world — no clone lists cleanly and none is orphaned — the roster shows the
    /// paved way to enter one, `yog exec bl prime` in a repo (v1 keeps `bl prime`
    /// out of the UI, §8.3). Since bl-44a5/bl-2930 that gesture works with only
    /// yog on `PATH`: the hatch seeds the world's shims and the embedded `bl`
    /// runs `prime` with a plugin chain that is yog (§16.4). `None` once any
    /// project is present (clean or orphaned) — a project surface then exists to
    /// work with.
    ///
    /// Two lines, not one sentence (bl-b491): the roster truncates every row
    /// rather than widening the panel (§11, bl-9669), and a single
    /// "No projects yet — add one with: yog exec bl prime" lost the command to
    /// the ellipsis at the default width — the one part of the hint that is the
    /// hint. The prose leads on its own elidable line; the command follows
    /// alone, so the width it must survive is its own.
    pub fn empty_project_hint(&self) -> Option<EmptyHint> {
        let has_project = !self.snap.balls_by_project.is_empty()
            || self
                .snap
                .join_rows
                .iter()
                .any(|r| r.state == join::JoinState::OrphanedProject);
        (!has_project).then(|| EmptyHint {
            lead: "No projects yet — add one with:".to_owned(),
            command: "yog exec bl prime".to_owned(),
        })
    }

    /// A dispatched `bl` verb landed against `project` (§15 Y16): mark the
    /// **project** dirty so the worker re-fetches its live *and* closed balls
    /// (the delivered-row source, §5.1 #4), rebuilds the join and re-reads the
    /// ops tail on its next pass — the immediate convergence the operator sees,
    /// ahead of the watch.
    ///
    /// The root named is the project's own identity — its decoded invocation
    /// path (§5.1 #1) — which is the vocabulary every other project surface
    /// already speaks. yog never spells the percent-encoded clone dir: that
    /// encoding is balls', and one fact has one owner.
    pub fn after_bl_verb(&mut self, project: &Path) {
        self.mark_dirty([project.to_path_buf()]);
    }

    /// A dispatched `litany` verb landed (message/stop/scan): it touches no
    /// ball, so only the ops tail changes — the yog-state root's ordinary
    /// routing (§7.1).
    pub fn after_litany_verb(&mut self) {
        self.mark_dirty([self.roots.yog_state.clone()]);
    }

    /// The yog data root — where bound workspaces live (§3.1) and the anchor
    /// every world path derives from (§16.2). The start flow reads
    /// `roots.yog_data` through `PlanInputs`; the V2 fork composer reads it
    /// here, to name the world's skills pool without minting a whole
    /// [`boundary_deps`](Self::boundary_deps) per frame.
    pub(crate) fn yog_data_root(&self) -> &Path {
        &self.roots.yog_data
    }

    /// The balls state root — the parent of the per-project clones dir (balls
    /// arch §1: `clones/` always lives under it); the start flow's
    /// `work_worktree_path` derives from it (§3.3).
    pub fn balls_state_root(&self) -> PathBuf {
        let clones = &self.roots.balls_clones;
        // `clones` is always nested under the state root, so it has a parent;
        // the fallback (the clones dir itself) keeps this panic-free.
        clones.parent().unwrap_or(clones).to_path_buf()
    }

    /// The boundary [`Deps`](crate::boundary::dispatch::Deps) this instance
    /// answers with (§8.5): its roots, its composed world, its published
    /// snapshot, its verb binaries.
    ///
    /// **No gesture executes through it any more** (REMOTE §9.8, bl-1747): the
    /// window posts every act over the wire and the engine builds the `Deps`
    /// the act runs in. What is left here is the §8.5 line's *query* arm, which
    /// is answered in place, and the acceptance world's stand-in for the
    /// transport — both reads.
    ///
    /// The world it carries is **lensed on the focused workspace's wall**
    /// (§16.2 as amended): a §9 config gesture folds brazen's destinations out
    /// of `deps.world`, and those live inside the sphere the operator is
    /// looking at. No focus is no wall — the gesture refuses rather than
    /// reaching the machine's own brazen state.
    pub fn boundary_deps(&self, litany: &Cli, bl: &Cli) -> crate::boundary::dispatch::Deps {
        crate::boundary::dispatch::Deps {
            litany: litany.clone(),
            bl: bl.clone(),
            state_root: self.state_root().to_path_buf(),
            yog_binary: crate::cli_outbound::self_exe().unwrap_or_default(),
            world: crate::world::wall::env_opt(
                &self.roots.world,
                self.focused_workspace().as_deref(),
            ),
            home: self.roots.home.clone(),
            yog_data_root: self.roots.yog_data.clone(),
            balls_state_root: self.balls_state_root(),
            // The **derivation**, never the §7.2 fold: a gesture and a
            // machine-facing query may not be decided by a fact that is only
            // optimistic.
            snapshot: std::sync::Arc::clone(&self.derived),
            caller: crate::boundary::dispatch::Caller::default(),
        }
    }
}

/// The start-flow's hand-off (`startable`/`resumable`/`start_bare_inputs`/…) —
/// split out per §12's 300-line budget.
mod starts;

/// `pub(crate)` so a sibling test corpus shares this one `FakeBl` rather than
/// standing up a second fake of the same runner.
#[cfg(test)]
pub(crate) mod tests;
