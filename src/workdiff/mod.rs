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

use std::path::Path;

use balls::delivery_path::work_branch;

use crate::app::Snapshot;
use crate::binding::named_of;
use crate::files_view::{MAX_ENTRIES, Preview, classify};
use crate::git_tree::{file_patch, head_branch, numstat, rev_parse};

mod candidates;
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
    /// The project's **wire name** (REMOTE §8, bl-ccf7) — the §5.1 #1 identity
    /// [`Snapshot::project_name`](crate::app::Snapshot::project_name) derives,
    /// which is the same word the §11 roster labels it with and the same word
    /// `--project` takes. It carried the absolute repository path until this
    /// ball: the last row of §8's residual list that identifies rather than
    /// locates, and the one a client on another machine could neither resolve
    /// nor unsee.
    pub project: String,
    pub ball_id: String,
    /// balls' opaque attempt handle when this row is a §3.8 fan candidate
    /// (bl-c2bd); `None` is the ordinary claim attempt, whose source is
    /// `work/<id>` rather than `attempt/<handle>`.
    pub handle: Option<String>,
    /// The delivery commit the target's history records for this attempt — the
    /// **derived acceptance mark** (VISION V3.2), never a stored winner. `None`
    /// is one fact covering pending and rejected alike, because rejection is
    /// the absence of a delivery (§4.10 item 6).
    pub delivered: Option<String>,
    pub change: Change,
}

/// One file of one attempt — what a patch read is asked for. The ball id
/// rides along because a workspace may hold more than one attempt and a bare
/// path would not say which diff it belongs to; the handle rides for the same
/// reason one row deeper — a fan's candidates all carry the obligation's ball,
/// and only the handle says which candidate's diff the path belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkFile {
    pub ball: String,
    pub handle: Option<String>,
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
/// repo: the claim attempts (the §3.2 claimant join), then the §3.8 fan
/// candidates the workspace's own trail binds (bl-c2bd) — claims first, so a
/// [`WorkFile`] naming no handle finds the claim row it always found. An empty
/// vec is the honest answer for a workspace that claims no ball — a bare or
/// path start has no delivery obligation (VISION §4.10 item 8), which is the
/// general path with no inputs rather than an arm of its own.
pub fn read(
    snap: &Snapshot,
    workspace: &Path,
    entries: &[crate::opslog::OpEntry],
    xdg: &balls::layout::Xdg,
) -> Vec<Attempt> {
    let Some(name) = named_of(&snap.workspaces, workspace) else {
        return Vec::new();
    };
    let mut out: Vec<Attempt> = plan::plans(&snap.balls_by_project, &name)
        .into_iter()
        .map(|plan| {
            let named = snap.project_name(&plan.project);
            resolve(plan, named)
        })
        .collect();
    out.extend(candidates::candidates(snap, workspace, entries, xdg, &name));
    out
}

/// Resolve one plan against its project repo: name the target, then read the
/// churn between the two ends ([`diff_change`]).
///
/// **The claim attempt wears the derived acceptance mark too** (bl-40ab). The
/// scan is `fan::delivered_commit`'s, unchanged — its own doc says the tag it
/// reads is *"an attempt handle or a ball id: both deliver under the same
/// `[<id>]` subject tag, so one scan reads both"* — and for a claim the id is
/// the ball, so what the mark answers here is the ball's own delivery onto the
/// branch it closes into. bl-c2bd filled this for candidates only because V3's
/// surface was about candidates; leaving it `None` at N = 1 would have made the
/// ordinary single start the one attempt whose acceptance could not be read,
/// and N = 1 is not a case (VISION §4.10 item 8).
fn resolve(plan: plan::Plan, project: String) -> Attempt {
    let source = work_branch(&plan.ball_id);
    let attempt = |change, delivered| Attempt {
        project: project.clone(),
        ball_id: plan.ball_id.clone(),
        handle: None,
        delivered,
        change,
    };
    let Ok(Some(head)) = head_branch(&plan.project) else {
        return attempt(Change::Unreadable, None);
    };
    let target = plan.target_ball.as_deref().map_or(head, work_branch);
    let mark = crate::fan::delivered_commit(&plan.project, &target, &plan.ball_id);
    attempt(diff_change(&plan.project, target, source), mark)
}

/// The git half of one attempt's read, shared by the claim rows and the fan
/// candidates (bl-c2bd): resolve both ends of `target..source`, then count the
/// churn between them. Pure over the repo — which ref plays which part is
/// entirely the caller's derivation.
fn diff_change(project: &Path, target: String, source: String) -> Change {
    let resolved = |spec: &str| rev_parse(project, spec).ok().flatten();
    let (Some(target_oid), Some(source_oid)) = (resolved(&target), resolved(&source)) else {
        let missing = [&target, &source]
            .into_iter()
            .filter(|spec| resolved(spec).is_none())
            .cloned()
            .collect();
        return Change::Absent {
            target,
            source,
            missing,
        };
    };
    let Ok(out) = numstat(project, &format!("{target}..{source}")) else {
        return Change::Unreadable;
    };
    let mut files = plan::parse_numstat(&out);
    let truncated = files.len() > MAX_ENTRIES;
    files.truncate(MAX_ENTRIES);
    Change::Diff {
        target,
        source,
        target_oid,
        source_oid,
        files,
        truncated,
    }
}

/// One file's patch, out of the attempt `file.ball` names — the bounded bytes
/// of `git diff <target>..<source> -- <path>`, classified by the same
/// `Text`/`Truncated`/`Binary` vocabulary the Files tab's preview uses.
/// `None` when no attempt here carries that ball, or its ends never resolved:
/// there is no range to read at, and an empty patch would read as "unchanged".
///
/// It takes the snapshot because an [`Attempt`] names its project rather than
/// locating it (REMOTE §8, bl-ccf7), and the enumeration is where that name
/// becomes the repository a `git` read can run in — the same resolution
/// [`read`] spells in the other direction, at the same one place.
pub fn patch(snap: &Snapshot, attempts: &[Attempt], file: &WorkFile) -> Option<Preview> {
    let attempt = attempts
        .iter()
        .find(|a| a.ball_id == file.ball && a.handle == file.handle)?;
    let range = attempt.range()?;
    if matches!(attempt.change, Change::Absent { .. }) {
        return None;
    }
    let project = snap.project_path(&attempt.project).ok()?;
    let bytes = file_patch(&project, &range, &file.path).unwrap_or_default();
    let size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    Some(classify(&bytes, size))
}

#[cfg(test)]
pub(crate) mod tests;
