//! `spawn_detached`: the fire-and-forget launch into a new process group
//! (§8.1). Each test observes the child's fifo effect (and, for the reaping
//! test, the process table) and NEVER waits on or signals its process
//! itself — a `waitpid`/`kill` from a test fights both the coverage
//! harness's ptrace reaping and `spawn_detached`'s own reaper thread, whose
//! wait is the one that must be seen to work (and the child, in its own
//! group, would ignore a terminal signal anyway). Under Linux coverage
//! (tarpaulin's `--engine llvm`, which follows the detached child under
//! ptrace) the child scripts fork no sub-process that would race that same
//! ptrace machine: the Linux drivers use only shell builtins (`read`,
//! `printf`, `pwd`) and a `/proc` redirect. macOS CI runs plain `cargo
//! test` (no tarpaulin, no ptrace), so its process-group driver may fork
//! `ps` — see [`GROUP_DRIVER`]. This mirrors lernie's fire-and-forget
//! launcher.
//!
//! `spawn_detached` forks directly like `run`, so — as with the streaming
//! and actions fixtures — each test HOLDS `SPAWN_LOCK` across the call:
//! `write_script` returns the guard (kept as `_guard` through the fifo
//! read), and the missing-binary test takes the lock explicitly.

use super::*;
use std::time::Duration;
use tempfile::tempdir;

/// Deadline for the reaping observation below: 200 × 25 ms. Generous by
/// design — the good path leaves the table on the first pass, and the bad
/// path (a zombie nobody waits on) never leaves it at all, so a long
/// deadline buys robustness and costs nothing.
const REAP_TRIES: u32 = 200;
const REAP_STEP: Duration = Duration::from_millis(25);

/// A sink path under `dir` for a test that does not inspect the capture.
fn sink_in(dir: &Path) -> PathBuf {
    dir.join("sink").join("child.err")
}

/// Create a fifo at `path` via the `mkfifo(1)` command (no `libc` FFI, so
/// no `unsafe` in the test tree). A detached child writes its one report
/// there and closes; reading the fifo blocks until the child has run, so
/// the read completing is itself proof the child survived our dropping its
/// handle — no `waitpid` required to synchronize.
fn make_fifo(path: &Path) {
    let status = crate::git_env::status(
        crate::git_env::command(Path::new("mkfifo"))
            .args(["-m", "600"])
            .arg(path),
    )
    .expect("spawn mkfifo");
    assert!(status.success(), "mkfifo");
}

/// Per-OS driver that reports `"<pid> <pgid>"` down the fifo, where
/// `<pgid>` is the numeric process-group handle each platform exposes.
/// `process_group(0)` (set on the `Command` in `spawn_detached`) makes the
/// child the leader of a brand-new process group, so `pgid == pid`; the
/// test asserts that identity. (Unlike the prior `setsid`, this does NOT
/// create a new session — the group is what escapes yog's terminal signal
/// group, per the module doc's semantic-delta note.)
///
/// - **Linux** reads field 5 of `/proc/self/stat` — the numeric process
///   group id (`pgrp`) — via a shell redirect (a builtin, no fork). The
///   no-fork rule is mandatory here, not stylistic: Linux CI runs under
///   tarpaulin's ptrace engine, which follows the detached child; a forked
///   helper would race that ptrace teardown (see `tarpaulin.toml`).
/// - **macOS** has no `/proc`; the portable numeric handle is `pgid`, a
///   POSIX `-o` format keyword defined as "the process group ID ... as a
///   decimal integer" on both procps and BSD ps. After `process_group(0)`
///   it equals the pid; a plain fork/exec would instead inherit the
///   harness's process group (leader != child), so `pgid == pid` faithfully
///   proves the new group took effect. `ps` forks a sub-process, which is
///   safe because macOS CI runs `make test` (plain `cargo test`) — no
///   tarpaulin, no ptrace machine to race.
#[cfg(target_os = "linux")]
const GROUP_DRIVER: &str = "#!/bin/sh\nread _ _ _ _ pgid _ < /proc/self/stat\nprintf '%s %s\\n' \"$$\" \"$pgid\" > \"$1\"\n";
#[cfg(not(target_os = "linux"))]
const GROUP_DRIVER: &str =
    "#!/bin/sh\nprintf '%s %s\\n' \"$$\" \"$(ps -o pgid= -p $$)\" > \"$1\"\n";

#[test]
fn detached_child_survives_and_leads_own_group() {
    let dir = tempdir().unwrap();
    let fifo = dir.path().join("report");
    make_fifo(&fifo);
    // The driver ([`GROUP_DRIVER`], per-OS) reports "pid pgid" down the
    // fifo. The write happens AFTER spawn_detached returned and dropped the
    // internal Child — had that drop killed it (as Stream's drop does), the
    // fifo read below would block until timeout. process_group(0) makes the
    // child its own group leader, so the second field equals the pid.
    let bin = write_script(dir.path(), "driver", GROUP_DRIVER);
    let fifo_str = fifo.to_str().unwrap();
    let pid = Cli::new(bin)
        .spawn_detached(None, &sink_in(dir.path()), &[fifo_str])
        .unwrap();
    let recorded = std::fs::read_to_string(&fifo).unwrap();
    let mut fields = recorded.split_whitespace();
    let child_pid: u32 = fields.next().unwrap().parse().unwrap();
    let child_leader: u32 = fields.next().unwrap().parse().unwrap();
    assert_eq!(
        child_pid, pid,
        "returned pid is the detached child's own pid"
    );
    assert_eq!(
        child_leader, pid,
        "process_group(0): detached child leads its own group"
    );
}

/// Reports down the fifo and exits immediately after — so the read below
/// rendezvous with the driver's last act. Builtins only (module doc).
const EXITING_DRIVER: &str = "#!/bin/sh\nprintf done > \"$1\"\n";

#[test]
fn the_detached_child_is_reaped_and_leaves_no_zombie() {
    let dir = tempdir().unwrap();
    let fifo = dir.path().join("report");
    make_fifo(&fifo);
    let bin = write_script(dir.path(), "driver", EXITING_DRIVER);
    let fifo_str = fifo.to_str().unwrap();
    let pid = Cli::new(bin)
        .spawn_detached(None, &sink_in(dir.path()), &[fifo_str])
        .unwrap();
    assert_eq!(std::fs::read_to_string(&fifo).unwrap(), "done");
    // An exited child leaves the process table only when its parent waits;
    // unreaped it stays there as a zombie indefinitely (bl-3016), so this is
    // a race-free assertion, not a timing guess: `true` is reached in one
    // pass when the reaper thread works and NEVER when it does not.
    // Sleep-before-check (not check-then-sleep, the drop tests' shape) so
    // every line of the closure runs on that first pass — under coverage a
    // conditional sleep would be a line whose hit depends on scheduling.
    let reaped = (0..REAP_TRIES).any(|_| {
        std::thread::sleep(REAP_STEP);
        !process_exists(pid)
    });
    assert!(
        reaped,
        "detached child {pid} never left the process table: yog is still its \
         parent and nothing waited on it — one zombie per fire"
    );
}

#[test]
fn spawn_detached_propagates_cwd() {
    let dir = tempdir().unwrap();
    let fifo = dir.path().join("report");
    make_fifo(&fifo);
    // Report the physical cwd (set via `current_dir`) down the fifo —
    // builtins only. (Env inheritance is a plain `Command::spawn` property
    // shared with `run`; it is proven, without mutating yog's own env, by
    // `run_env_sets_child_environment_variables`, which passes vars via
    // `Command::env` explicitly.)
    let bin = write_script(dir.path(), "driver", "#!/bin/sh\npwd -P > \"$1\"\n");
    let fifo_str = fifo.to_str().unwrap();
    Cli::new(bin)
        .spawn_detached(Some(dir.path()), &sink_in(dir.path()), &[fifo_str])
        .unwrap();
    let recorded = std::fs::read_to_string(&fifo).unwrap();
    let cwd = recorded.trim();
    assert_eq!(
        std::fs::canonicalize(cwd).unwrap(),
        std::fs::canonicalize(dir.path()).unwrap(),
    );
}

#[test]
fn spawn_detached_errors_on_missing_binary() {
    // A failing spawn still forks before exec reports ENOENT; hold
    // SPAWN_LOCK so that transient child can't inherit a peer's recorder
    // write fd.
    let dir = tempdir().unwrap();
    let cli = Cli::new("/definitely/not/a/real/binary/lernie-detach");
    let err = cli
        .spawn_detached(None, &sink_in(dir.path()), &[])
        .unwrap_err();
    assert!(err.to_string().contains("failed to spawn"), "{err}");
}

/// A driver that dies talking: one line to stderr, then the fifo write that
/// rendezvous with the reader below. Builtins only (see the module doc's
/// no-fork rule under tarpaulin's ptrace engine).
const DYING_DRIVER: &str =
    "#!/bin/sh\nprintf 'version skew: refusing\\n' >&2\nprintf done > \"$1\"\n";

#[test]
fn detached_child_stderr_lands_in_the_sink_file() {
    let dir = tempdir().unwrap();
    let fifo = dir.path().join("report");
    make_fifo(&fifo);
    let bin = write_script(dir.path(), "driver", DYING_DRIVER);
    let sink = sink_in(dir.path());
    let fifo_str = fifo.to_str().unwrap();
    // The sink's parent does not exist yet: the spawn creates the chain.
    assert!(!sink.parent().unwrap().exists());
    Cli::new(bin)
        .spawn_detached(None, &sink, &[fifo_str])
        .unwrap();
    // The fifo read rendezvous *after* the child's stderr write, so the sink
    // is complete by the time we read it — no sleep, no waitpid.
    assert_eq!(std::fs::read_to_string(&fifo).unwrap(), "done");
    assert_eq!(
        std::fs::read_to_string(&sink).unwrap(),
        "version skew: refusing\n",
        "the detached child's stderr is captured, not dropped (§8.1)"
    );
}

#[test]
fn an_unopenable_sink_degrades_to_null_and_still_launches() {
    let dir = tempdir().unwrap();
    let fifo = dir.path().join("report");
    make_fifo(&fifo);
    let bin = write_script(dir.path(), "driver", DYING_DRIVER);
    // A sink whose parent chain cannot be created (the "dir" is a file): the
    // capture is lost, but the driver is the point — the launch must proceed.
    let blocker = dir.path().join("blocker");
    fs::write(&blocker, "not a directory").unwrap();
    let fifo_str = fifo.to_str().unwrap();
    Cli::new(bin)
        .spawn_detached(None, &blocker.join("child.err"), &[fifo_str])
        .unwrap();
    assert_eq!(std::fs::read_to_string(&fifo).unwrap(), "done");
}
