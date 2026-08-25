//! **The read itself** — `target..source` against a real project repo, and the
//! one file patch a seat asks for. Split from the vocabulary at §12's budget on
//! the seam the module's own doc draws: [`super`] is *what a work-diff says*
//! (the three distinct states an answer can take, never one silent empty
//! listing), this is the pure git read that says it.
//!
//! Nothing here stores or spawns: both ends are already-derived facts, and
//! which ref plays which part is entirely the caller's arithmetic ([`super`]'s
//! `plan`).

use std::path::Path;

use balls::delivery_path::work_branch;

use super::{Attempt, Change, WorkFile, candidates, plan};
use crate::app::Snapshot;
use crate::binding::named_of;
use crate::files_view::{MAX_ENTRIES, Preview, classify};
use crate::git_tree::{file_patch, head_branch, numstat, rev_parse};

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
pub(super) fn diff_change(project: &Path, target: String, source: String) -> Change {
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
