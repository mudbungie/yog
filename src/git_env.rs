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
//!
//! # The fork is the boundary too (bl-6397)
//!
//! Building every child here and then letting each caller fork it by hand left
//! a second per-call contract open, and it cost the suite a recurring flake.
//! `fs::write` on a fixture script holds a write fd; a `fork` in ANOTHER thread
//! copies that fd into a child that keeps it until its own `exec` completes; an
//! `exec` of the script inside that window is **ETXTBSY**. So a test that never
//! spawns anything can still redden a test three modules away, and the victim's
//! own care cannot save it.
//!
//! [`spawn`], [`output`] and [`status`] are therefore the crate's one fork, and
//! in `cfg(test)` they take one process-wide lock across the fork — the whole of
//! the discipline, in one place nobody has to remember. [`exec`] joins them for
//! the same reason and not the same hazard: it forks nothing, it *replaces*,
//! and what it owes the process is the `SIGPIPE` disposition std clobbers on
//! the way out (bl-3792, read it there). Measured on this box
//! with 8 write-then-exec threads against an 8-thread fork storm, ~9,600 pairs
//! each: unguarded forks, 8.3% ETXTBSY; every fork through one lock, **zero** —
//! and zero *with the writes left entirely unguarded*, which is why the write
//! side needs no contract at all. Releasing the lock the instant the fork
//! returns is enough (a child's inherited fds are gone by then), so a child is
//! never waited on under it and the suite's subprocesses still run concurrently.

use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output, Stdio};

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

/// Fork + exec `cmd` — **the crate's one fork**, and the only lawful way to
/// start a child here (`rules/no-bare-fork.yml`). Under `cfg(test)` the fork
/// happens under the binary-wide spawn lock; see the module doc for the race
/// that buys.
pub(crate) fn spawn(cmd: &mut Command) -> std::io::Result<Child> {
    #[cfg(test)]
    let _guard = crate::test_support::spawn_guard();
    cmd.spawn()
}

/// [`spawn`] then read the child to EOF — [`Command::output`]'s behavior, with
/// its default stdio spelled out, so the lock covers the fork alone and never
/// the child's whole life.
pub(crate) fn output(cmd: &mut Command) -> std::io::Result<Output> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    spawn(cmd)?.wait_with_output()
}

/// [`spawn`] then wait — [`Command::status`]'s behavior (stdio inherited unless
/// the caller said otherwise), with the lock over the fork alone.
pub(crate) fn status(cmd: &mut Command) -> std::io::Result<ExitStatus> {
    spawn(cmd)?.wait()
}

/// Replace this process image with `cmd` — **the crate's one `exec`**, and the
/// only lawful way to spend an `execve` baton here (`rules/no-bare-fork.yml`).
/// It returns only on failure, carrying that failure, because an `execvp` that
/// works never comes back.
///
/// **The return is the whole reason it lives here** (bl-3792).
/// `CommandExt::exec` does not fork: std's `do_exec` runs in THIS process and
/// resets `SIGPIPE` to `SIG_DFL` on its way to `execvp`, so what a failed exec
/// hands back is a live process in which the next write to a reader that went
/// away is a *death* and no longer a `BrokenPipe` error. Under `cargo test`
/// that process is the whole test binary and the writer is any peer thread, so
/// the death lands nowhere near the exec and reports no failing test — a
/// `signal: 13, SIGPIPE` on roughly one parallel run in four. Putting the
/// disposition back is therefore not the caller's errand any more than the git
/// scrub is: a contract a caller has to remember is a defect nobody sees until
/// it fires.
///
/// No spawn lock, and the asymmetry is the point: ETXTBSY needs a fork to copy
/// somebody's write fd into a child, and an exec forks nothing. It can only be
/// ETXTBSY's *victim*, and the discipline above already retired the party that
/// makes one.
pub(crate) fn exec(cmd: &mut Command) -> std::io::Error {
    use std::os::unix::process::CommandExt as _;
    let failure = cmd.exec();
    crate::cli_outbound::sys::ignore_sigpipe();
    failure
}

#[cfg(test)]
mod tests;
