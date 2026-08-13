//! Spawn failure and teardown: a missing binary surfaces a spawn error,
//! and dropping a live `Stream` terminates the child — SIGTERM first,
//! escalating to SIGKILL when SIGTERM is trapped.

use super::*;
use std::time::{Duration, Instant};
use tempfile::tempdir;

#[test]
fn run_errors_on_missing_binary() {
    // A failing spawn still forks before exec reports ENOENT; hold the lock
    // so that transient child can't inherit a peer's recorder write fd.
    let _g = spawn_guard();
    let cli = Cli::new("/definitely/not/a/real/binary/lernie-xyz");
    let Err(err) = cli.run(&[]) else {
        panic!("expected spawn failure");
    };
    let msg = err.to_string();
    assert!(msg.contains("failed to spawn"), "{msg}");
}

/// bl-6191: `std::process` reports a bad `current_dir` as ENOENT **against the
/// program path**, so the raw error reads "failed to spawn <yog binary>: No such
/// file or directory" — the operator typed a bad directory and is told their
/// binary is missing. The spawn boundary asks the cwd its own question first.
#[test]
fn a_missing_work_directory_is_named_instead_of_the_binary() {
    let dir = tempdir().unwrap();
    let (bin, _guard) = write_script(dir.path(), "fake_lernie", "#!/bin/sh\nexit 0\n");
    let missing = dir.path().join("nonexistent-uat-dir");
    let cli = Cli::new(bin);
    let Err(err) = cli.run_in(&missing, &[]) else {
        panic!("expected spawn failure");
    };
    assert_eq!(
        err.to_string(),
        format!("work directory does not exist: {}", missing.display())
    );
}

#[test]
fn drop_terminates_long_running_child() {
    let dir = tempdir().unwrap();
    let (bin, _spawn_guard) = write_script(dir.path(), "fake_lernie", "#!/bin/sh\nsleep 30\n");
    let cli = Cli::new(bin);
    let stream = cli.run(&[]).unwrap();
    let pid = stream.pid().unwrap();
    drop(stream);
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if !process_exists(pid) {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("child pid {pid} still alive after drop");
}

#[test]
fn drop_escalates_to_sigkill_if_sigterm_ignored() {
    let dir = tempdir().unwrap();
    // Trap SIGTERM to ignore it, THEN emit a readiness marker. Without the
    // marker, the parent's `drop(stream)` can race the shell's startup and
    // send SIGTERM before `trap '' TERM` is installed — which quietly kills
    // the shell and leaves the SIGKILL-escalation branch in
    // `Stream::drop` uncovered. Waiting for "ready\n" on stdout pins the
    // ordering.
    let (bin, _spawn_guard) = write_script(
        dir.path(),
        "fake_lernie",
        "#!/bin/sh\ntrap '' TERM\nprintf 'ready\\n'\nwhile :; do sleep 1; done\n",
    );
    let cli = Cli::new(bin);
    let mut stream = cli.run(&[]).unwrap();
    let pid = stream.pid().unwrap();
    let ready_deadline = Instant::now() + Duration::from_secs(3);
    loop {
        assert!(
            Instant::now() < ready_deadline,
            "child pid {pid} did not signal ready before drop"
        );
        match stream.next() {
            Some(Chunk::Stdout(b)) if b.starts_with(b"ready") => break,
            Some(_) => {}
            None => panic!("child pid {pid} exited before signaling ready"),
        }
    }
    drop(stream);
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if !process_exists(pid) {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("child pid {pid} survived SIGKILL escalation");
}
