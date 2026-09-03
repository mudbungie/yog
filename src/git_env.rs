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
//! only yog's own `git` forks left the larger half open: `bl`, `litany`, `bz`,
//! an `$EDITOR` shim and the fake substrate scripts the suite drives all fork
//! `git` *of their own accord*, and they inherit whatever yog handed them. A
//! hook-run suite therefore still committed onto the branch being committed —
//! reproduced: the fake `litany new` arm's `git commit -m 'config: init
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
//! and the two process-global effects a RETURNING one leaves are its own
//! (bl-3792's `SIGPIPE` reset, which it repairs, and bl-419d's freed `environ`
//! copy, which nothing can — read [`exec`] there). Measured on this box
//! with 8 write-then-exec threads against an 8-thread fork storm, ~9,600 pairs
//! each: unguarded forks, 8.3% ETXTBSY; every fork through one lock, **zero** —
//! and zero *with the writes left entirely unguarded*, which is why the write
//! side needs no contract at all. Releasing the lock the instant the fork
//! returns is enough (a child's inherited fds are gone by then), so a child is
//! never waited on under it and the suite's subprocesses still run concurrently.
//!
//! ## The lock covers yog's forks, and the binary must contain no others
//!
//! **That "zero" was measured with every fork in the process going through the
//! lock — a CONDITION, not a property of the lock** (bl-6bf5). The lock is a
//! `cfg(test)` bracket around `Command::spawn` here; a fork performed by
//! another crate in this same process never passes through it. yog links its
//! substrate — `balls`, `litany`, `brazen` — and each forks on its own account
//! (this module says so below, at [`exec`]: *"the linked balls' own `git`
//! forks, which take no lock of yog's"*). A lib test that drives one of them
//! **in-process** puts an unlocked forker back in the binary, and every
//! write-then-exec fixture in it is a victim again.
//!
//! So the condition to keep is: **the lib test binary drives no embedded
//! substrate in-process.** A beat that must belongs in a `tests/*.rs` process
//! of its own — `tests/multiplex_bl.rs`, `tests/multiplex_litany.rs`,
//! `tests/multiplex_landing.rs` — each carrying the note that says why it may
//! not come back. That placement is also what lets those files scrub their own
//! process env of [`INHERITED`]: no spawn boundary exists to do it for a fork
//! they do not perform.
//!
//! Measured both ways, one filter over the lib test binary (`multiplex` plus
//! the five fixture-exec families), 16 workers x 70 iterations on a 16-core
//! box: with the landing repair's in-process `balls::substrate::found_landing`
//! still in the lib binary, **8 ETXTBSY failures**; with it moved out, **0** —
//! and 0 for the same victims with no substrate beat in the filter at all.
//!
//! **One unlocked forker remains and is not a test's to move** (bl-6bf5, filed
//! on as bl-fd28): `fan`'s production path opens balls' attempts, which forks `git` inside
//! balls, and `fan`'s beats are unit tests of `pub(crate)` code no `tests/*.rs`
//! can reach. Same filter with `fan::` in place of `multiplex`, same volume:
//! 2 ETXTBSY failures. Adding an in-process substrate drive here is therefore not "one
//! more like fan" — it re-opens a hole that is already costing the suite.

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
///
/// # A returning exec has a SECOND global effect, and it cannot be repaired
///
/// `SIGPIPE` above is repairable because the damage outlives the call. The
/// other one does not (bl-419d). A `Command` that carries any env delta — and
/// [`command`] gives every command in this crate seven — makes std capture the
/// environment into a `CStringArray`, and `do_exec` points the process's own
/// `environ` at it on the way to `execvp`. A failed `execvp` restores the old
/// pointer, and then FREES that array as `exec` returns. std holds only the env
/// **read** lock across all of it, so a peer thread's env read runs
/// concurrently by design: it can be walking that array at the moment it is
/// freed, and what it hands back is freed memory — entries with interior NUL
/// bytes, which surface as `InvalidInput: "nul byte found in provided data"` at
/// that peer's next spawn, in a module the exec is nowhere near. Measured on
/// this box: 6 reader threads against a failing-exec loop, 73,435 torn entries
/// in 21.2M reads; zero with the env delta removed, which is the leg that
/// proves the swap is the party.
///
/// **The torn read has two faces, and the quiet one buys a wrong diagnosis**
/// (bl-2f8b). The NUL above is the loud one. The other is a peer whose spawn
/// cannot find its *program*: the `PATH` it read out of that array was gone or
/// garbage, `execvp` answers ENOENT, and the spawn fails
/// `NotFound: "No such file or directory"` — which reads as a missing
/// DIRECTORY, so the reader goes hunting a path bug. It was seen as a `git
/// status` refusing a landing checkout that existed and had just been
/// committed to, and it was filed against a `/var` vs `/private/var`
/// canonicalization split that the failing log itself refutes: both halves of
/// that path carry one spelling, and a split needs two. Both sightings landed
/// in `multiplex::landing`, which is simply the densest git-forker in the lib
/// binary and therefore the likeliest victim, never the author. That it showed
/// on macOS and not on Linux is the allocator: glibc's `free` leaves the bytes
/// readable, so the same race there usually reads intact.
///
/// Nothing here can fix it — the window is inside std, and the victim is any
/// env reader in the process, including the linked balls' own `git` forks,
/// which take no lock of yog's. So the discipline is placement, not repair:
/// **a returning exec belongs only in a process with no peer threads.** In
/// production that is where it already stands (`main.rs`, above eframe). Its
/// proof is `tests/exec_return.rs`, an integration binary with exactly one
/// `#[test]` — the `tests/multiplex_bl.rs` precedent — and that is why this
/// verb is `pub` while [`spawn`]/[`output`]/[`status`] are not. The lib suite
/// must never reach `execvp`: `multiplex::litany`'s unit test hands the arm a
/// command std refuses ABOVE `do_exec`, so it proves the arm without spending
/// either effect.
pub fn exec(cmd: &mut Command) -> std::io::Error {
    use std::os::unix::process::CommandExt as _;
    let failure = cmd.exec();
    crate::cli_outbound::sys::ignore_sigpipe();
    failure
}

#[cfg(test)]
mod tests;
