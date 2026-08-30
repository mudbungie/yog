//! The **returning exec**, and the two process-global effects it leaves
//! behind — `yog::git_env::exec`, driven for real (bl-3792, bl-419d).
//!
//! `CommandExt::exec` does not fork. std's `do_exec` runs in THIS process, so
//! an `execvp` that fails hands back a live process that std has already
//! altered twice on its way out:
//!
//! 1. **`SIGPIPE` is reset to `SIG_DFL`.** Every safe write in a Rust process
//!    is written against the runtime's `SIG_IGN`, so from here on a write to a
//!    reader that went away is a *death* instead of a `BrokenPipe` error.
//!    `git_env::exec` puts the disposition back; that is bl-3792's fix and the
//!    first leg below.
//! 2. **The process's `environ` is pointed at a copy std owns** (any command
//!    with an env delta — and `git_env::command` gives every command in the
//!    crate seven), restored on the way out, and then FREED as `exec` returns.
//!    std holds only the env *read* lock across that, so a peer thread reading
//!    the environment runs concurrently by design and can be walking the array
//!    as it is freed: it gets freed memory, which surfaces as
//!    `InvalidInput: "nul byte found in provided data"` at that peer's next
//!    spawn — a failing test in a module the exec is nowhere near. Nothing in
//!    yog can repair that, and no lock of yog's can exclude it (the linked
//!    balls forks `git` on its own account and takes none).
//!
//! **So this binary runs exactly one `#[test]`**, the `tests/multiplex_bl.rs`
//! and `tests/multiplex_litany.rs` precedent applied one hazard down: a
//! returning exec is lawful only where no peer thread exists. In production
//! that is `main.rs`, above clap and above eframe. The lib suite reaches
//! `execvp` nowhere — `multiplex::litany`'s unit test hands its arm a command
//! std refuses ABOVE `do_exec`, which is why the arm's mapping can still be
//! proved beside the code.

// clippy's allow-*-in-tests reaches `#[test]` fns, not an integration crate's
// free fixture helpers — the `tests/multiplex_litany.rs` precedent.
#![allow(clippy::unwrap_used)]

use std::io::Write as _;
use std::os::unix::process::ExitStatusExt as _;
use std::path::Path;
use std::process::{Command, Stdio};

/// Write into a pipe whose reader is gone, and answer what happened.
///
/// **A pipe, and not a `UnixStream`** — the mistake this replaces. std writes
/// a Unix socket with `MSG_NOSIGNAL`, so such a write can never raise
/// `SIGPIPE` at all and always answers `BrokenPipe`: the assertion passed
/// whether the disposition had been repaired or not, which is a beat that
/// proves nothing (the bl-70b8 family). A pipe has no such flag, so the answer
/// here is decided by the disposition and by nothing else. Ignored, this
/// returns `Err(BrokenPipe)`; defaulted, it does not return — the process dies
/// of signal 13 and takes the run with it, which is the loud shape the defect
/// had in the first place.
fn write_with_no_reader() -> std::io::Result<()> {
    let mut child = Command::new("cat").stdin(Stdio::piped()).spawn().unwrap();
    let mut sink = child.stdin.take().unwrap();
    // Kill the reader and reap it, so the pipe is provably readerless before
    // the write. Two writes: the first can still land in the pipe buffer.
    child.kill().unwrap();
    child.wait().unwrap();
    sink.write_all(b"x")?;
    sink.write_all(b"x")
}

#[test]
fn a_returning_exec_hands_back_a_process_whose_signals_and_environment_still_work() {
    // The probe is real, in the direction that matters: a child inherits
    // `SIG_DFL` for SIGPIPE (std resets it in every child it spawns), so a
    // child writing into a readerless pipe must DIE of signal 13. Without this
    // leg the two `BrokenPipe` assertions below could both be vacuous.
    let mut writer = Command::new("yes").stdout(Stdio::piped()).spawn().unwrap();
    drop(writer.stdout.take());
    assert_eq!(
        writer.wait().unwrap().signal(),
        Some(13),
        "a defaulted SIGPIPE must kill the writer — the probe below is blind otherwise"
    );

    // The baseline: before the exec, this process ignores SIGPIPE.
    assert_eq!(
        write_with_no_reader().unwrap_err().kind(),
        std::io::ErrorKind::BrokenPipe
    );
    let before: Vec<(std::ffi::OsString, std::ffi::OsString)> = std::env::vars_os().collect();

    // The act: a real `execvp` on a target that cannot be executed, through
    // the crate's one exec and its seven env removals — so std captures an
    // environment, lends it to the process, and frees it as the call returns.
    let mut cmd = yog::git_env::command(Path::new("/nonexistent/yog-successor"));
    let failure = yog::git_env::exec(&mut cmd);
    assert_eq!(failure.kind(), std::io::ErrorKind::NotFound);

    // bl-3792: the disposition is back, so the next write down a dead pipe is
    // an error again. A regression does not redden this line — it deletes the
    // run, which is precisely why the exec lives alone in this binary.
    assert_eq!(
        write_with_no_reader().unwrap_err().kind(),
        std::io::ErrorKind::BrokenPipe
    );

    // bl-419d: the environment this process reads is its own again, byte for
    // byte, and a spawn that captures it still builds an envp — the failure
    // the torn read produced was `InvalidInput` from exactly this step.
    assert_eq!(std::env::vars_os().collect::<Vec<_>>(), before);
    let status = yog::git_env::command(Path::new("true")).status().unwrap();
    assert!(
        status.success(),
        "a spawn after the exec still captures an env"
    );
}
