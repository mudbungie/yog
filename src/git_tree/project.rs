//! Reads of a **project** repo — the balls invocation path's own git repo,
//! not a lernie workspace's `repo.git` (DESIGN §5.1 #32, VISION §4.10).
//!
//! Every other module here reads the workspace repo; this one reads the repo
//! the agent's work lands in. It lives beside them anyway because [`cmd`] is
//! the crate's one `git` doorway (its `base_cmd` is what scrubs the inherited
//! `GIT_DIR`/`GIT_INDEX_FILE`, §16), and a second fork site would be a second
//! place for that scrub to be forgotten.
//!
//! Four reads, all pure: name the integration branch, resolve a ref to a
//! commit, count the churn between two commits, and read one file's patch.
//! Nothing here writes, and nothing here spends a balls or lernie verb — the
//! project diff is a pure git read (VISION §4.10 item 4).

use super::GitTreeError;
use super::cmd::{git, git_optional};
use std::path::Path;

/// The branch this repo's `HEAD` points at — balls' own integration-branch
/// derivation (`git symbolic-ref --short HEAD`), spelled the way balls spells
/// it so yog and `bl close` can never name two different targets.
///
/// `None` covers both ways there is no answer: the path is not a readable git
/// repo, and a detached `HEAD` that names no branch at all. Both are "this
/// project can state no target", which is one fact for the caller to report,
/// not two.
pub(crate) fn head_branch(repo: &Path) -> Result<Option<String>, GitTreeError> {
    Ok(git_optional(repo, &["symbolic-ref", "--short", "HEAD"])?
        .map(|out| String::from_utf8_lossy(&out).trim().to_owned())
        .filter(|name| !name.is_empty()))
}

/// The commit `spec` resolves to, or `None` when this repo does not have it —
/// a work branch never minted, a target branch that is gone. Absence is the
/// answer, never an error: a ref that is not there is a fact about the repo.
pub(crate) fn rev_parse(repo: &Path, spec: &str) -> Result<Option<String>, GitTreeError> {
    let arg = format!("{spec}^{{commit}}");
    Ok(
        git_optional(repo, &["rev-parse", "--verify", "--quiet", &arg])?
            .map(|out| String::from_utf8_lossy(&out).trim().to_owned())
            .filter(|oid| !oid.is_empty()),
    )
}

/// `git diff --numstat <range>` — one `<added>\t<removed>\t<path>` row per
/// changed file, with `-` for both counts of a binary file. Counts, not bytes:
/// the listing of what an attempt changed must not cost the whole patch.
pub(crate) fn numstat(repo: &Path, range: &str) -> Result<Vec<u8>, GitTreeError> {
    git(repo, &["diff", "--numstat", range])
}

/// `git diff <range> -- <path>` — one file's patch text, read only when the
/// operator asks for that file.
pub(crate) fn file_patch(repo: &Path, range: &str, path: &str) -> Result<Vec<u8>, GitTreeError> {
    git(repo, &["diff", range, "--", path])
}
