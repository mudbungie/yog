//! **Which conversation an attempt is bound to, and what it can be asked**
//! (§3.9, bl-40ab) — the projection's agent-side half.
//!
//! **One rule for the binding, at every N.** The conversation bound to an
//! attempt is the last fire whose `--cwd` names *that attempt's worktree*, and
//! both worktree formulas are balls' own: [`attempt_path`] for a fan candidate,
//! [`work_worktree_path`] for the ordinary claim. So N = 1 is not a case here
//! either — the pointer is the same pointer (§4.10 item 4, and the reproduction
//! discipline [`crate::fan::cohort`] states: the path is re-derived and
//! compared, never parsed).
//!
//! **Two leaf spellings for a claim, for [`crate::control::root`]'s reason
//! exactly**: balls mints `<id>` or, when that leaf is taken, `<id>-<claimant>`,
//! and which one exists is a disk question this join does not need to ask — a
//! fire matched one of them or it matched neither.
//!
//! **What the bound conversation can then be asked is [`super::observed`]'s** —
//! the seam being that this module answers *which* conversation and that one
//! answers *what about it*, and the two share nothing but the agent id.

use std::path::{Path, PathBuf};

use balls::delivery_path::attempt_path;
use balls::layout::Xdg;

use crate::app::Snapshot;
use crate::binding::work_worktree_path;
use crate::fan::Fire;

/// Where balls puts things — the three values every worktree formula needs,
/// owned rather than borrowed (no named lifetimes, rule 1). It is one value
/// because it is one question ("where would this attempt's worktree be"), asked
/// once per row against facts that do not change between rows.
#[derive(Debug, Clone)]
pub(super) struct Layout {
    xdg: Xdg,
    balls_state_root: PathBuf,
    /// The §3.2 claimant — the workspace's own name, which is what balls
    /// disambiguates a taken work-worktree leaf with.
    claimant: String,
}

impl Layout {
    pub(super) fn of(xdg: &Xdg, balls_state_root: &Path, claimant: &str) -> Layout {
        Layout {
            xdg: xdg.clone(),
            balls_state_root: balls_state_root.to_path_buf(),
            claimant: claimant.to_owned(),
        }
    }

    /// Every worktree path this attempt could be bound to, in `repo` — one for
    /// a candidate, both leaf spellings for a claim.
    fn worktrees(&self, attempt: &crate::workdiff::Attempt, repo: &Path) -> Vec<PathBuf> {
        match &attempt.handle {
            Some(handle) => vec![attempt_path(&self.xdg, &repo.to_string_lossy(), handle)],
            None => [None, Some(self.claimant.as_str())]
                .into_iter()
                .map(|c| work_worktree_path(&self.balls_state_root, repo, &attempt.ball_id, c))
                .collect(),
        }
    }
}

/// The fire that bound `attempt`, or `None` when none did. The **last** match
/// wins — a re-fire onto one attempt is one attempt, the rule the cohort fold
/// and the writable root's claim join both keep. `None` for a project whose
/// path did not resolve: with no repo there is no formula to reproduce, and a
/// guess would attribute another attempt's conversation to this row.
pub(super) fn fire_for(
    fires: &[Fire],
    attempt: &crate::workdiff::Attempt,
    layout: &Layout,
    repo: Option<&Path>,
) -> Option<Fire> {
    let candidates = layout.worktrees(attempt, repo?);
    fires
        .iter()
        .rev()
        .find(|fire| candidates.contains(&fire.worktree))
        .cloned()
}

/// The **agent id** of the conversation a fire minted, resolved through the
/// §3.3 name ladder over the published tree — the same fold every seat that
/// names an agent reads ([`Agent::name_fact`](crate::git_tree::Agent::name_fact)).
///
/// `None` while the detached driver has not written its branch yet: the fire
/// happened and the conversation has no id, which is exactly what §7.2's
/// pending row means and not an error.
pub(super) fn agent_of(snap: &Snapshot, workspace: &Path, conversation: &str) -> Option<String> {
    snap.trees
        .get(workspace)?
        .agents
        .iter()
        .find(|a| a.name_fact().as_deref() == Some(conversation))
        .map(|a| a.agent_id.clone())
}
