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
//! # The fork is the boundary too (bl-6397, amended bl-fd28)
//!
//! Building every child here and then letting each caller fork it by hand left
//! a second per-call contract open. [`spawn`], [`output`] and [`status`] are
//! therefore the crate's one fork — one place to reason about what a child
//! inherits, instead of a promise at every call site. [`exec`] joins them for a
//! different hazard: it forks nothing, it *replaces*, and the two
//! process-global effects a RETURNING one leaves are its own (bl-3792's
//! `SIGPIPE` reset, which it repairs, and bl-419d's freed `environ` copy, which
//! nothing can — read [`exec`] there). `rules/no-bare-fork.yml` holds the
//! chokepoint.
//!
//! ## ETXTBSY is closed on the WRITE side, and the lock is gone (bl-fd28)
//!
//! The fork carried a second job for two rounds and no longer does. `fs::write`
//! on a script holds a write fd; a `fork` in ANOTHER thread copies that fd into
//! a child that keeps it until its own `exec` completes; an `exec` of the script
//! inside that window is **ETXTBSY**. So this module took one process-wide lock
//! across the fork under `cfg(test)`: measured with 8 write-then-exec threads
//! against an 8-thread fork storm, ~9,600 pairs, unguarded forks cost 8.3%
//! ETXTBSY and every fork through one lock cost zero (bl-6397).
//!
//! **That zero was a CONDITION, not a property of the lock** (bl-6bf5). The
//! lock was a `cfg(test)` bracket around `Command::spawn` HERE, and a fork
//! performed by another crate in this same process never passed through it. yog
//! links its substrate — `balls`, `litany`, `brazen` — and each forks `git` on
//! its own account (this module says so below, at [`exec`]: *"the linked balls'
//! own `git` forks, which take no lock of yog's"*). A lib beat that drove one of
//! them in-process was an unlocked forker for as long as it ran. Measured over
//! one filter, 16 workers x 70 iterations on a 16-core box: the landing
//! repair's in-process `balls::substrate::found_landing` cost **8** ETXTBSY
//! failures, and `fan`'s attempt machinery another **2**. bl-6bf5 answered by
//! moving the beat out to a `tests/*.rs` of its own; `fan` could not follow,
//! its beats being unit tests of `pub(crate)` code no `tests/*.rs` can reach.
//!
//! bl-fd28 closed the hazard on the side that owns it instead, and bl-e6c9
//! finished the job. **Every executable file this crate writes is written by a
//! CHILD** — [`write_exec`], feeding the body to
//! `sh -c 'cat > "$1" && chmod 755 "$1"'` — so the write fd never exists in this
//! process and a peer fork, in ANY crate, has nothing of ours to copy.
//! `rules/no-hand-chmod.yml` makes it structural rather than a convention.
//!
//! **It is not a test discipline, and for one round this doc said it was.**
//! bl-fd28 converted the fixtures and wrote "every executable *fixture*" here,
//! which read as a settled boundary; the ENGINE still wrote its world shims with
//! `fs::write` + `set_permissions` and exec'd them (`world::tools::ensure_shim`),
//! which is the same window in the process that forks the most. Folding those
//! beats into bl-fd28's own filter reproduced it at **7** failures over 1,120
//! runs. The helper is production's now — [`write_exec`] is `pub(crate)` and
//! `crate::test_support::write_exec` is that function with the error turned into
//! a panic — so there is one home for "write an executable", not one per face.
//!
//! **This module's older claim that "the victim's own care cannot save it" is
//! retired with the lock.** It was true of a write-side *bracket* — a lock that
//! only excludes forks agreeing to take it — and false of a write-side
//! *relocation*, which leaves no descriptor for anyone to inherit.
//! `tests/integration/support/mod.rs` had already reached that answer for the
//! reason that proves it: yog linked as a library is not `cfg(test)`, so no
//! lock of this module's was ever available to that binary, and writing the
//! fixture from `sh` was the only move it had. It measured ~1 run in 8 before
//! and none after.
//!
//! **So the lock was measured out.** Same recipe as above — `fan::` plus the
//! five fixture-exec families, 16 workers x 70 iterations, three runs each,
//! 3,360 test-binary runs per side: **0 / 0 / 0** with the lock still standing,
//! **0 / 0 / 0** with the guard removed. It was closing nothing once the write
//! fd was gone, so `SPAWN_LOCK` and `spawn_guard` are gone with it. The one
//! fork above stays for its own reasons; the ETXTBSY discipline now lives
//! entirely on the write side, where the fd is.
//!
//! One consequence is subtraction, not vigilance: bl-6bf5's placement rule —
//! *the lib test binary drives no embedded substrate in-process* — is no longer
//! load-bearing for ETXTBSY. The `tests/multiplex_*.rs` split keeps its OTHER
//! reason, which is [`INHERITED`]: a binary that runs its subject in-process
//! must scrub its own process env, no spawn boundary existing to do it for a
//! fork it does not perform.

use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output, Stdio};

mod write_exec;

pub(crate) use write_exec::write_exec;

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
/// start a child here (`rules/no-bare-fork.yml`). It takes no lock: the ETXTBSY
/// window it used to close is closed on the write side now (module doc,
/// bl-fd28).
pub(crate) fn spawn(cmd: &mut Command) -> std::io::Result<Child> {
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
/// It is not an ETXTBSY party either way: that needs a fork to copy somebody's
/// write fd into a child, and an exec forks nothing. It could only ever be the
/// *victim*, and the write-side discipline above retired the fd that makes one.
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
