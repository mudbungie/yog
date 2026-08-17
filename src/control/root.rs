//! The **writable root** (VISION §4.11 item 3, §4.10 item 1) and the path
//! algebra every containment question is asked through.
//!
//! The root is two directories: the agent's own worktree (`agents/<agent-id>/`
//! under the workspace, lernie ARCH §2.2) and the **bound attempt worktree** —
//! the balls `work/<id>` checkout the ball this workspace claimed materialized.
//! Inside either, writing is the job; outside, a write leaves the world.
//!
//! **The root is derived from facts yog owns, never from a fact the agent
//! controls.** That distinction is the whole security content of this module:
//!
//! - The **cwd** comes from lernie's own mark (`refs/lernie/cwd/<agent-id>`),
//!   because relative operands must resolve where the executor will run them.
//!   The mark is written by the agent's `cd` — so it is read to *interpret*
//!   operands and never to widen the root. An agent that could widen its root
//!   by `cd`-ing would have no root at all.
//! - The **bound worktree** is computed by the bl-delivery formula
//!   ([`work_worktree_path`]) over a claim yog itself made and logged. The
//!   claimant is the workspace name (§3.2), which is the workspace directory's
//!   own leaf — so the join needs no store read and no subprocess.
//!
//! A ball an agent claimed for itself mid-conversation leaves no yog-side row
//! (§3.2 states that limit), so its worktree is not in the root and writing
//! there classifies open-world — never a target write, so a workspace override
//! or a raised floor still catches it.
//!
//! **Containment is lexical, not canonical.** `..` is folded textually and `~`
//! expands against `$HOME`; symlinks are not resolved. A `canonicalize` would
//! touch disk on every operand of every tool call and would answer `None` for
//! the paths that matter most (files a patch is about to create). Symlink
//! escape is out of the threat model by construction — VISION §4.11 item 8:
//! rule classification bounds accident and drift, not adversarial evasion.

use std::path::{Component, Path, PathBuf};

use crate::binding::work_worktree_path;
use crate::opslog::OpEntry;

/// Workspace subdirectory holding the per-agent worktrees (lernie ARCH §2.2).
/// Mirrored here rather than imported, the convention every module that names
/// it already follows (`git_tree`, `files_view`, `transcript`).
const AGENTS_DIR: &str = "agents";

/// The logical binary an ops row's `argv[0]` names for a balls verb.
const BL: &str = "bl";
/// The `bl` verb whose logged row carries a claim yog made.
const CLAIM: &str = "claim";
/// The flag that stamps a claim with its claimant identity.
const AS_FLAG: &str = "--as";

/// Where an invocation may write, and where its relative operands resolve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Root {
    /// The directories writing is confined to: the agent worktree, plus the
    /// bound attempt worktree when the claim join yields one.
    pub writable: Vec<PathBuf>,
    /// The directory the executor will run this invocation in.
    pub cwd: PathBuf,
    /// `$HOME`, for `~` expansion in operands.
    pub home: PathBuf,
}

impl Root {
    /// Whether `p` — already absolute and normalized — lies inside the root.
    /// A root directory contains itself.
    pub fn holds(&self, p: &Path) -> bool {
        self.writable.iter().any(|w| p.starts_with(w))
    }

    /// Resolve one operand as written: `~`-expanded, made absolute against
    /// [`cwd`](Self::cwd), and lexically normalized.
    pub fn resolve(&self, operand: &str) -> PathBuf {
        let expanded = expand_home(operand, &self.home);
        let joined = if expanded.is_absolute() {
            expanded
        } else {
            self.cwd.join(expanded)
        };
        normalize(&joined)
    }

    /// Whether every operand in `operands` resolves inside the root. An empty
    /// list is inside it — the general path with no operands, not a special
    /// case: a command naming no path writes nowhere in particular, and its
    /// own rule already says what it does.
    pub fn holds_all(&self, operands: &[String]) -> bool {
        operands.iter().all(|o| self.holds(&self.resolve(o)))
    }
}

/// `~` / `~/…` expanded against `home`; anything else is taken as written.
fn expand_home(operand: &str, home: &Path) -> PathBuf {
    match operand.strip_prefix('~') {
        Some("") => home.to_path_buf(),
        Some(rest) => match rest.strip_prefix('/') {
            Some(tail) => home.join(tail),
            // `~user/…` names another account; not ours to expand, and it is
            // outside the root under any reading.
            None => PathBuf::from(operand),
        },
        None => PathBuf::from(operand),
    }
}

/// Fold `.` and `..` textually. A `..` above the root simply drops, exactly as
/// the kernel treats `/..`.
fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in p.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

/// The agent's own worktree under `workspace`.
pub fn agent_worktree(workspace: &Path, agent_id: &str) -> PathBuf {
    workspace.join(AGENTS_DIR).join(agent_id)
}

/// Workspace subdirectory holding the bare repository (lernie ARCH §2.2).
const REPO_DIR: &str = "repo.git";
/// The per-agent working-directory mark's ref namespace (lernie ARCH §3.3).
const CWD_REF: &str = "refs/lernie/cwd/";

/// The agent's working directory as lernie's own mark records it — the ref names
/// a blob whose bytes are the absolute path. `None` when the mark is unset
/// (the ordinary state of an agent that never called `cd`), when the repo is not
/// there, or when git will not run: in every one of those the caller's default
/// applies, which is the agent worktree the executor would have used anyway.
///
/// **Read to interpret operands, never to widen the root.** The mark is written
/// by the agent's own `cd` — and a `cd` out of the writable root is itself an
/// open-world invocation this control holds, so the mark can only ever name a
/// directory the operator let it name.
pub fn agent_cwd(workspace: &Path, agent_id: &str) -> Option<PathBuf> {
    let repo = workspace.join(REPO_DIR);
    // A git that will not spawn and a git that exits non-zero are one answer —
    // the mark is not readable, so the caller's default applies — and they fold
    // into one `?` rather than two branches that mean the same thing.
    let out = crate::git_env::git()
        .arg("--git-dir")
        .arg(&repo)
        .args(["cat-file", "blob", &format!("{CWD_REF}{agent_id}")])
        .output()
        .ok()
        .filter(|out| out.status.success())?;
    // Lossy rather than strict: lernie's own writer declines a directory that
    // does not survive a trimmed-UTF-8 round trip, so a mark that is not UTF-8
    // is unrepresentable upstream — and a lossy path that names nothing simply
    // resolves outside the root, which holds.
    let path = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    (!path.is_empty()).then(|| PathBuf::from(path))
}

/// The bound attempt worktree candidates for `claimant`, computed from the `bl
/// claim` rows yog's own trail carries (§3.2 join, §8.1 formula). The **last**
/// matching row wins — a re-claim supersedes — and that row's `cwd` is the
/// project the formula mirrors. Both leaf spellings are returned: the canonical
/// `<id>` and the `<id>-<claimant>` disambiguation balls mints when the
/// canonical leaf is already taken. Which one exists is a disk question the
/// containment check does not need to ask — a candidate that was never minted
/// contains nothing. Empty when this workspace never claimed through yog.
pub fn bound_worktrees(
    entries: &[OpEntry],
    balls_state_root: &Path,
    claimant: &str,
) -> Vec<PathBuf> {
    claimed(entries, claimant)
        .map(|(project, id)| {
            vec![
                work_worktree_path(balls_state_root, &project, &id, None),
                work_worktree_path(balls_state_root, &project, &id, Some(claimant)),
            ]
        })
        .unwrap_or_default()
}

/// The §4.10 **fan candidates'** worktrees for `claimant` — the other half of
/// the writable root once a delivery obligation is fanned. With N > 1 no work
/// happens in `work/<id>`: each candidate writes in its own attempt worktree,
/// so a root that named the claim alone would classify every candidate's edit
/// as open-world. Derived from the same two yog-owned facts the claim join
/// uses — the trail's own claim row for the project, and its own fire rows for
/// the bindings ([`crate::fan::cohort`]) — never from the agent's `cd` mark.
/// Empty for an ordinary N = 1 start, which binds no attempt at all.
pub fn candidate_worktrees(
    entries: &[OpEntry],
    balls: &balls::layout::Xdg,
    workspace: &Path,
    claimant: &str,
) -> Vec<PathBuf> {
    claimed(entries, claimant)
        .map(|(project, _)| crate::fan::cohort::worktrees(entries, balls, &project, workspace))
        .unwrap_or_default()
}

/// The claim this workspace last made: the project the row ran in and the ball
/// id it named. The **last** matching row wins — a re-claim supersedes — and a
/// workspace that never claimed through yog has none. `pub(crate)` since
/// bl-c2bd: the work-diff's candidate rows read the fan's obligation from this
/// same rule rather than keeping a second copy of it, and since bl-34b1 so does
/// the §8.6 confinement backend's writable set — the project half of this pair
/// is the one member of that set outside the world, and a revived driver has no
/// other way to know it.
pub(crate) fn claimed(entries: &[OpEntry], claimant: &str) -> Option<(PathBuf, String)> {
    entries
        .iter()
        .rev()
        .find_map(|e| Some((PathBuf::from(&e.cwd), claim_of(e, claimant)?)))
}

/// The ball id a `bl claim <id> --as <claimant>` row names, when its claimant
/// is ours. Any other argv shape contributes nothing.
fn claim_of(entry: &OpEntry, claimant: &str) -> Option<String> {
    let argv: Vec<&str> = entry.argv.iter().map(String::as_str).collect();
    let [bin, verb, id, rest @ ..] = argv.as_slice() else {
        return None;
    };
    if *bin != BL || *verb != CLAIM {
        return None;
    }
    let stamped = rest
        .windows(2)
        .any(|w| w.first() == Some(&AS_FLAG) && w.get(1) == Some(&claimant));
    stamped.then(|| (*id).to_owned())
}

#[cfg(test)]
mod tests;
