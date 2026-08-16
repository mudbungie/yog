//! The coverage gate's own regression suite (bl-673a). `scripts/check-coverage.sh`
//! answers with THREE outcomes, not two, and the third one is the whole point:
//! a run in which tarpaulin reports being SIGNALED judged nothing about the
//! tree, and must not be readable as a verdict about it.
//!
//! Why it needs a test rather than a comment: the outcome is consumed by exit
//! code, from a GitHub Actions `if:` expression
//! (`.github/workflows/speculate.yml`) that decides whether to write a FAIL
//! into the tree-keyed verdict cache. A stored FAIL is permanent — balls'
//! `speculate_run` stops the candidate chain at one on every later pass without
//! rebuilding, and re-running cannot dislodge it because the tree has not
//! changed. Five sightings became five poisoned trees that way.
//!
//! Each test drives the REAL script over a throwaway git repository with a fake
//! `make` first on `PATH`, so nothing here can pass by agreeing with a private
//! copy of the logic. The fake counts its own invocations, which is how "retry
//! once, and only for that class" is asserted in both directions.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

/// Tarpaulin's own words when a signal reached it, verbatim — the string the
/// script classifies on. Written once here so a drift in the script's pattern
/// shows up as a failing test rather than as a silent reclassification.
const SIGNALED: &str =
    "ERROR cargo_tarpaulin: Failed to run tests: Attempting to handle tarpaulin being signaled";

/// A throwaway repository whose `make` is a script we wrote.
struct Probe {
    dir: TempDir,
    bin: PathBuf,
    tally: PathBuf,
}

/// What one run of the gate answered.
struct Answer {
    code: i32,
    err: String,
    calls: usize,
}

impl Probe {
    /// `body` is the fake `make`'s payload, with `$n` bound to this
    /// invocation's 1-based count so a fixture can behave differently on the
    /// retry.
    fn new(body: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let status = yog::git_env::git()
            .current_dir(dir.path())
            .args(["init", "-q"])
            .status()
            .unwrap();
        assert!(status.success(), "git init");
        let bin = dir.path().join("bin");
        fs::create_dir(&bin).unwrap();
        let tally = dir.path().join("calls");
        fs::write(&tally, "").unwrap();
        let (t, make) = (tally.display(), bin.join("make"));
        let script =
            format!("#!/usr/bin/env bash\necho x >>\"{t}\"\nn=$(wc -l <\"{t}\")\n{body}\n");
        fs::write(&make, script).unwrap();
        fs::set_permissions(&make, fs::Permissions::from_mode(0o755)).unwrap();
        Self { dir, bin, tally }
    }

    fn run(&self) -> Answer {
        let path = format!(
            "{}:{}",
            self.bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/check-coverage.sh");
        let out = yog::git_env::command(&script)
            .current_dir(self.dir.path())
            .env("PATH", path)
            .output()
            .unwrap();
        Answer {
            code: out.status.code().unwrap(),
            err: String::from_utf8_lossy(&out.stderr).into_owned(),
            calls: fs::read_to_string(&self.tally).unwrap().lines().count(),
        }
    }
}

/// The defect: a signaled tarpaulin is retried once, and a second signal is
/// answered with 75 (EX_TEMPFAIL) — NOT the 1 the workflow would record a FAIL
/// from. The runner evidence rides along, because five sightings carried none.
#[test]
fn a_signaled_tarpaulin_is_retried_and_answered_with_no_verdict() {
    let a = Probe::new(&format!("echo '{SIGNALED}' >&2\nexit 1")).run();
    assert_eq!(a.code, 75, "exit 75 is 'no verdict': {}", a.err);
    assert_eq!(a.calls, 2, "signaled once is retried exactly once");
    assert!(a.err.contains("SIGNALED (attempt 1)"), "{}", a.err);
    assert!(a.err.contains("SIGNALED (attempt 2)"), "{}", a.err);
    assert!(a.err.contains("bl-673a"), "the diagnosis cites its ball");
}

/// The other half of "only that class": a run the retry rescues is a plain
/// pass, with nothing held back and nothing recorded about the interruption.
#[test]
fn a_signal_the_retry_survives_is_a_pass() {
    let body = format!("if [ \"$n\" -eq 1 ]; then echo '{SIGNALED}' >&2; exit 1; fi\nexit 0");
    let a = Probe::new(&body).run();
    assert_eq!(a.code, 0, "the retry earned the verdict: {}", a.err);
    assert_eq!(a.calls, 2);
}

/// A real failure is never retried — re-running it spends another tarpaulin to
/// learn the same thing — and it keeps the exit 1 the FAIL verdict is written
/// from, with the held stdout replayed so the gate names what failed (bl-0dff).
#[test]
fn a_real_failure_is_answered_once_and_replayed() {
    let a = Probe::new("echo 'test yog::sinks ... FAILED'\nexit 1").run();
    assert_eq!(a.code, 1, "a verdict about the tree: {}", a.err);
    assert_eq!(a.calls, 1, "a real failure is not retried");
    assert!(a.err.contains("test yog::sinks ... FAILED"), "{}", a.err);
}

/// A pass holds tarpaulin's stdout back, which is why the gate is quiet at
/// close — the behaviour the three-outcome rework had to preserve.
#[test]
fn a_pass_is_quiet() {
    let a = Probe::new("echo '|| yog/src/lib.rs | 100% |'\nexit 0").run();
    assert_eq!(a.code, 0, "{}", a.err);
    assert_eq!(a.calls, 1);
    assert!(
        !a.err.contains("100%"),
        "held stdout replayed on a pass: {}",
        a.err
    );
}
