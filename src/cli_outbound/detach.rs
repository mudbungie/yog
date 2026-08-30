//! The fire-and-forget detached spawn (DESIGN §8.1): a long-lived driver yog's
//! own exit can never kill. Split from [`super`] so that shape — no pipe, no
//! signal, and a status nobody reads, in contrast to the piped
//! [`run`](super::Cli::run) family whose [`Stream`](super::Stream) drop SIGTERMs
//! — keeps [`super`] under the 300-line cap.
//!
//! **Fire-and-forget is not the same as parentless (bl-3016).** Dropping a
//! `std::process::Child` neither signals nor reparents: yog stays the parent for
//! as long as it lives, so a driver that exits first sits in the process table
//! as a zombie until yog itself dies. Reaping is an obligation separate from
//! detachment, and it is discharged here by a [`reap`] thread per spawn.
//!
//! **stdin/stdout are null; stderr is not.** A detached child has no waitable
//! status, so if its stderr went to `/dev/null` too, a driver that dies right
//! after launch would be indistinguishable from a clean launch (§13.3 as
//! amended). The caller names a per-spawn **sink file** instead; the ops-row
//! projection folds its tail back in at read time
//! ([`opslog::detached`](crate::opslog::detached)). This crate stays generic —
//! it opens whatever path it is handed and knows nothing of the naming.

use std::fs;
use std::path::Path;
use std::process::{Child, Stdio};
use std::thread;

use super::{Cli, CliError};

impl Cli {
    /// Spawn `<binary> <args...>` fully detached and return only the child pid.
    /// The child gets its own process group (`process_group(0)`), stdin/stdout
    /// bound to null, stderr bound to the `stderr` sink file, and no pipe and no
    /// signal from us — only the [`reap`] thread's blocking wait, which observes
    /// the child without touching it. This launches long-lived drivers (§8.1:
    /// `litany prompt` detached) so yog's exit can never kill a running loop:
    /// yog's death takes the reaper thread with it and init adopts the still-live
    /// driver, exactly as before. `cwd`, when set,
    /// is the child's working directory (a bound workspace's work-worktree,
    /// §3.1). The new group (not a new session — see [`super`]'s semantic-delta
    /// note) is what keeps yog's terminal signals from reaching the child.
    pub fn spawn_detached(
        &self,
        cwd: Option<&Path>,
        stderr: &Path,
        args: &[&str],
    ) -> Result<u32, CliError> {
        use std::os::unix::process::CommandExt;
        // Physical spawn (§16.7 W12): the wrapper when one stands (§8.6), then
        // `program` + the namespace `prefix` — the shared spawn base, scrubbed
        // of the ambient git env like every child (`crate::git_env`).
        let mut cmd = self.spawn_base();
        cmd.args(args)
            // The detached driver nests too (§16.6 W2): the standing world env
            // rides the long-lived `litany prompt` so its agents' own tool
            // subprocesses inherit the nested `$XDG_STATE_HOME` (§16.4).
            .envs(self.standing_env())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(sink(stderr))
            // Own process group (`0` = a new group led by the child):
            // escapes yog's terminal signal group with safe std, no FFI.
            .process_group(0);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        // Through the crate's one fork like `run`, so under `cargo test` it
        // takes the spawn lock and never lands while a peer holds a
        // not-yet-closed recorder write fd (ETXTBSY, `crate::git_env`).
        let child = crate::git_env::spawn(&mut cmd)
            .map_err(|e| CliError::spawn(self.exec_target(), cwd, e))?;
        let pid = child.id();
        reap(child);
        Ok(pid)
    }
}

/// Hand `child` to a thread that blocks in `wait` until it exits, then drops the
/// status on the floor (bl-3016). This is the *only* thing yog does with a
/// detached child after launch, and it is not a leash: `wait` posts no signal
/// and holds no pipe, so the driver runs exactly as long as it wants, and yog's
/// own exit destroys the thread rather than the child (init adopts it). Without
/// it yog leaks one zombie per fire — a `Child` drop does not reparent, so the
/// kernel holds every exited driver's status for a parent that never asks.
///
/// A thread rather than the two alternatives, both worse: `SA_NOCLDWAIT` is
/// process-global and makes `waitpid` fail `ECHILD`, which would destroy the
/// exit statuses [`run`](Cli::run) and [`Streamed`](super::Streamed) read; a
/// double fork buys a truly parentless driver at the price of an `unsafe`
/// `pre_exec` whose body — running only between fork and exec, in a process that
/// execs away before writing coverage — can never be tested (AGENTS.md: "if it
/// can't be tested, it mustn't be built"). One parked thread per live driver is
/// the cheap, testable, entirely-safe discharge; the same shape [`Stream`]'s
/// drain threads already use.
///
/// [`Stream`]: super::Stream
fn reap(mut child: Child) {
    thread::spawn(move || {
        let _ = child.wait();
    });
}

/// Open `path` (creating its parent chain) as the child's stderr, **degrading to
/// `/dev/null`** when it cannot be created: an unwritable sink loses the capture,
/// but it must never block the launch — the driver is the point, the log is the
/// diagnosis.
fn sink(path: &Path) -> Stdio {
    let reachable = path.parent().is_none_or(|p| fs::create_dir_all(p).is_ok());
    match reachable
        .then(|| fs::File::create(path))
        .and_then(Result::ok)
    {
        Some(file) => Stdio::from(file),
        None => Stdio::null(),
    }
}
