//! **What the agent actually changed** — the project work-diff (DESIGN §5.1
//! #32, §11 Altitude-2 Work tab; VISION §4.10, bl-2b8c).
//!
//! The ruling this rung is built on, verbatim (VISION §4.10 item 4): *"the
//! project diff is a pure git read (`target..source`); a missing project repo
//! renders as a named absence, never a guess"*. Both ends are already derived
//! facts, so nothing new is stored and no verb is spent:
//!
//! - the **source** is the ball's claim — `work/<id>`, spelled by balls' own
//!   [`work_branch`](balls::delivery_path::work_branch), never by a literal
//!   here;
//! - the **target** is what `bl close` already derives — the parent's work
//!   branch for a close-gating subtask, else the project's integration branch
//!   (`git symbolic-ref --short HEAD`, balls' own spelling). The graph
//!   arithmetic is [`plan`]'s; the repo names the integration branch.
//!
//! The binding from the seat is the §3.2 claimant equality: the focused
//! conversation's workspace holds the balls whose claimant is its name. A
//! workspace holding two is two attempts, both shown — the layer that binds a
//! *conversation* to a project directory is lernie's working-directory mark,
//! which yog passes typed only once bl-6654 lands, and until then a
//! per-conversation answer would be a guess.
//!
//! **Everything unreadable is said, not swallowed** (the §4.10 mandate): a
//! project repo that cannot name a branch reads as [`Change::Unreadable`], a
//! ref that does not resolve as [`Change::Absent`] naming exactly which end is
//! missing, and a resolved pair with no changes as an empty [`Change::Diff`] —
//! three distinct states, never one silent empty listing.
//!
//! The listing is churn counts (`--numstat`), bounded at the Files tab's own
//! [`MAX_ENTRIES`]; one file's patch is read only when asked for
//! ([`patch`]) and comes back through the same
//! [`Preview`] vocabulary the Files tab already uses. Both reads are memoized
//! per snapshot by the seat that asks (§7.2), never per frame.

use std::path::{Path, PathBuf};

use balls::delivery_path::work_branch;

use crate::app::Snapshot;
use crate::binding::named_of;
use crate::files_view::{MAX_ENTRIES, Preview, classify};
use crate::git_tree::{file_patch, head_branch, numstat, rev_parse};

mod plan;
mod render;
pub(crate) mod wire;

pub use render::render;

/// How much one file changed. `Binary` is git's own `-`/`-` numstat row said
/// as itself: a file that changed by an amount lines cannot express.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Churn {
    Text { added: u64, removed: u64 },
    Binary,
}

/// One changed file in an attempt's diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChurn {
    /// The path as git names it — a rename's `{old => new}` composite included.
    pub path: String,
    pub churn: Churn,
}

/// What the project repo could say about one attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// The project path is not a readable git repo, or its `HEAD` names no
    /// branch — either way it can state no target, so there is no comparison
    /// to make and the surface says so.
    Unreadable,
    /// One or both ends of `target..source` do not resolve here; `missing`
    /// names them.
    Absent {
        target: String,
        source: String,
        missing: Vec<String>,
    },
    /// The read: the two refs, the two commits they resolved to, and the
    /// per-file churn between them. `files` empty means the attempt has
    /// changed nothing yet — a fact, and a different one from every arm above.
    Diff {
        target: String,
        source: String,
        target_oid: String,
        source_oid: String,
        files: Vec<FileChurn>,
        truncated: bool,
    },
}

/// One delivery attempt as this seat can read it (VISION §4.10 item 1): the
/// project it lands in, the ball whose claim materialized it, and what git
/// says about the two ends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    pub project: PathBuf,
    pub ball_id: String,
    pub change: Change,
}

/// One file of one attempt — what a patch read is asked for. The ball id
/// rides along because a workspace may hold more than one attempt and a bare
/// path would not say which diff it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkFile {
    pub ball: String,
    pub path: String,
}

impl Attempt {
    /// The exact range this attempt is read at, `target..source` — the literal
    /// spelling of the ruling, and what the seat shows so the operator can run
    /// the same read in a shell. `None` for an unreadable project.
    pub fn range(&self) -> Option<String> {
        match &self.change {
            Change::Unreadable => None,
            Change::Absent { target, source, .. } | Change::Diff { target, source, .. } => {
                Some(format!("{target}..{source}"))
            }
        }
    }
}

/// Every attempt the workspace at `workspace` holds, read against its project
/// repo. An empty vec is the honest answer for a workspace that claims no ball
/// — a bare or path start has no delivery obligation (VISION §4.10 item 8),
/// which is the general path with no inputs rather than an arm of its own.
pub fn read(snap: &Snapshot, workspace: &Path) -> Vec<Attempt> {
    let Some(name) = named_of(&snap.workspaces, workspace) else {
        return Vec::new();
    };
    plan::plans(&snap.balls_by_project, &name)
        .into_iter()
        .map(resolve)
        .collect()
}

/// Resolve one plan against its project repo: name the target, resolve both
/// ends, then count the churn between them.
fn resolve(plan: plan::Plan) -> Attempt {
    let source = work_branch(&plan.ball_id);
    let attempt = |change| Attempt {
        project: plan.project.clone(),
        ball_id: plan.ball_id.clone(),
        change,
    };
    let Ok(Some(head)) = head_branch(&plan.project) else {
        return attempt(Change::Unreadable);
    };
    let target = plan.target_ball.as_deref().map_or(head, work_branch);
    let resolved = |spec: &str| rev_parse(&plan.project, spec).ok().flatten();
    let (Some(target_oid), Some(source_oid)) = (resolved(&target), resolved(&source)) else {
        let missing = [&target, &source]
            .into_iter()
            .filter(|spec| resolved(spec).is_none())
            .cloned()
            .collect();
        return attempt(Change::Absent {
            target,
            source,
            missing,
        });
    };
    let Ok(out) = numstat(&plan.project, &format!("{target}..{source}")) else {
        return attempt(Change::Unreadable);
    };
    let mut files = plan::parse_numstat(&out);
    let truncated = files.len() > MAX_ENTRIES;
    files.truncate(MAX_ENTRIES);
    attempt(Change::Diff {
        target,
        source,
        target_oid,
        source_oid,
        files,
        truncated,
    })
}

/// One file's patch, out of the attempt `file.ball` names — the bounded bytes
/// of `git diff <target>..<source> -- <path>`, classified by the same
/// `Text`/`Truncated`/`Binary` vocabulary the Files tab's preview uses.
/// `None` when no attempt here carries that ball, or its ends never resolved:
/// there is no range to read at, and an empty patch would read as "unchanged".
pub fn patch(attempts: &[Attempt], file: &WorkFile) -> Option<Preview> {
    let attempt = attempts.iter().find(|a| a.ball_id == file.ball)?;
    let range = attempt.range()?;
    if matches!(attempt.change, Change::Absent { .. }) {
        return None;
    }
    let bytes = file_patch(&attempt.project, &range, &file.path).unwrap_or_default();
    let size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    Some(classify(&bytes, size))
}

#[cfg(test)]
mod tests;
