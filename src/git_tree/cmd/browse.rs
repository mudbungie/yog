//! Config-branch browse plumbing (DESIGN §9.3 / §5.1 #17–#18) and the plain
//! reachability reads beside it.
//!
//! The read-only config surface reads config branches, their trees, and their
//! file contents, and derives an agent's governing config by folding
//! `merge-base` over the config refs. Split off [`super`] at §12's pre-split
//! band on the seam that file already banner-marked: the parent is the git
//! doorway itself (the scrubbed `Command`, the log/step-commit parsing the
//! §7.1 walk consumes), and this is the §9.3 surface's own vocabulary over it.
//! Every call still routes through the parent's runners, so the env scrub is
//! never bypassed by a second doorway.
//!
//! `merge_base`/`is_ancestor` are **not only that fold's** (bl-40ab): they are
//! plain reachability reads over any repo, and §3.9's science projection asks
//! them of a *project* repo. One spelling of one git command.

use super::{git, git_optional};
use crate::git_tree::GitTreeError;
use std::path::Path;

/// Config branches: every `refs/heads/config/*` ref as
/// `<short-name> <oid> <committer-unix>` lines (§5.1 #18). `%(refname:short)`
/// yields `config/<name>`; the caller strips the `config/` prefix.
pub(crate) fn for_each_ref_config(repo: &Path) -> Result<Vec<u8>, GitTreeError> {
    git(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname:short) %(objectname) %(committerdate:unix)",
            "refs/heads/config/",
        ],
    )
}

/// Every file path in a commit's tree (`git ls-tree -r --name-only`, §5.1 #18)
/// — the config commit's control files (`souls/**`, `workflow.yaml`, …).
pub(crate) fn ls_tree(repo: &Path, refspec: &str) -> Result<Vec<u8>, GitTreeError> {
    git(repo, &["ls-tree", "-r", "--name-only", refspec])
}

/// Every **blob** in a commit's tree with its byte size
/// (`git ls-tree -r -l <refspec>`, VISION V1.2's pinned tree): lines of
/// `<mode> blob <oid> <size>\t<path>`. The long form exists because a listing
/// that shows a size must not show a guessed one — `--name-only` cannot say
/// how big a file was at that commit, and a zero would be a lie.
pub(crate) fn ls_tree_long(repo: &Path, refspec: &str) -> Result<Vec<u8>, GitTreeError> {
    git(repo, &["ls-tree", "-r", "-l", refspec])
}

/// One file's raw bytes from a commit's tree (`git show <refspec>:<path>`,
/// §9.3). YAML is returned as text — yog adds no YAML dependency.
pub(crate) fn show_file(repo: &Path, refspec: &str, path: &str) -> Result<Vec<u8>, GitTreeError> {
    git(repo, &["show", &format!("{refspec}:{path}")])
}

/// The paths added or modified between two commits under `prefix`
/// (`git diff --name-only --diff-filter=AM`, VISION §4.9): the transcript delta
/// a monitor check reads, derived from the sha its ops row names rather than
/// remembered. Renames and deletions are excluded because the transcript is
/// append-only — a file that vanished between two shas is not new work.
pub(crate) fn diff_names(
    repo: &Path,
    from: &str,
    to: &str,
    prefix: &str,
) -> Result<Vec<String>, GitTreeError> {
    let out = git(
        repo,
        &[
            "diff",
            "--name-only",
            "--diff-filter=AM",
            from,
            to,
            "--",
            prefix,
        ],
    )?;
    Ok(String::from_utf8_lossy(&out)
        .lines()
        .map(str::to_owned)
        .filter(|line| !line.is_empty())
        .collect())
}

/// The best common ancestor of two commits (`git merge-base`), or `None` when
/// they share no history (an unrelated orphan lineage — exit 1, §9.3 fold).
pub(crate) fn merge_base(repo: &Path, a: &str, b: &str) -> Result<Option<String>, GitTreeError> {
    Ok(git_optional(repo, &["merge-base", a, b])?
        .map(|out| String::from_utf8_lossy(&out).trim().to_string()))
}

/// Is `a` an ancestor of `b`? (`git merge-base --is-ancestor`, §9.3 fold
/// tie-break — exit 0 = yes, any non-zero = no.)
pub(crate) fn is_ancestor(repo: &Path, a: &str, b: &str) -> Result<bool, GitTreeError> {
    Ok(git_optional(repo, &["merge-base", "--is-ancestor", a, b])?.is_some())
}
