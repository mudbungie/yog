//! The ambient-git-environment scrub: one list, one constructor, every child.
//!
//! `git` exports `GIT_DIR`, `GIT_INDEX_FILE` and friends into every process it
//! starts — hooks above all. Those variables OUTRANK `-C <repo>` and
//! `current_dir`, so any `git` yog forks while such a variable is set is
//! silently re-aimed at the *outer* repo: a fixture the caller just built is
//! read straight past, and a production read of a workspace answers about
//! whatever repo the hook was committing to.
//!
//! **The scrub belongs to every child, not just to `git`** (bl-916a). Scrubbing
//! only yog's own `git` forks left the larger half open: `bl`, `lernie`, `bz`,
//! an `$EDITOR` shim and the fake substrate scripts the suite drives all fork
//! `git` *of their own accord*, and they inherit whatever yog handed them. A
//! hook-run suite therefore still committed onto the branch being committed —
//! reproduced: the fake `lernie new` arm's `git commit -m 'config: init
//! [config/default]'` landed on the outer work branch and replaced its tree.
//! Scrubbing at yog's spawn boundary clears the variables from the whole
//! descendant process tree at once, so no descendant needs to remember.
//!
//! The cure is not per-call vigilance — a spawn site that forgets is a defect
//! nobody sees until a hook runs it. It is that **no child is spawned by
//! hand**: [`command`] is the crate's one [`Command`] constructor ([`git`] is
//! its `git` spelling), it scrubs, and a caller cannot opt out because there is
//! nothing else to call. Enforced by `rules/no-bare-command.yml`.

use std::path::Path;
use std::process::Command;

/// The variables `git` exports into a hook's (or any child's) environment that
/// re-aim a child `git` at another repository. Public because a few callers
/// scrub the *process* environment rather than one child's — the multiplex
/// integration tests run their subject in-process — and that list must be this
/// list.
pub const INHERITED: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_PREFIX",
    "GIT_COMMON_DIR",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

/// A command running `program` with every [`INHERITED`] variable removed from
/// the child's environment. The only lawful way to build a [`Command`] in this
/// crate — see the module doc for why there is no unscrubbed alternative.
pub fn command(program: &Path) -> Command {
    let mut cmd = Command::new(program);
    for var in INHERITED {
        cmd.env_remove(var);
    }
    cmd
}

/// [`command`] for `git` itself — the crate's git constructor.
pub fn git() -> Command {
    command(Path::new("git"))
}

#[cfg(test)]
mod tests;
