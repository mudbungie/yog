//! The fixtures' one `git` fork site. Split out of `fixture` so both the
//! helper and the workspace-builder stay under the 300-line source cap.
//!
//! Every fork here routes through `crate::git_env`, the binary-wide
//! spawn discipline: a fork must not land while another test holds a
//! not-yet-closed write fd to a recorder script it is about to exec
//! (ETXTBSY — see `crate::test_support`).

use std::path::Path;

/// A fork whose **stdout is the product** — `hash-object`, which is the only
/// fixture git call whose answer is needed rather than merely its success.
pub(crate) fn git_out(repo: &Path, args: &[&str]) -> String {
    let out = crate::git_env::output(
        crate::git_env::git()
            .arg("-C")
            .arg(repo)
            .args(args)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null"),
    )
    .unwrap();
    assert!(out.status.success(), "git {args:?} failed");
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

pub(crate) fn run_git(repo: &Path, args: &[&str]) {
    let mut cmd = crate::git_env::git();
    cmd.arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_AUTHOR_DATE", "2026-04-22T12:00:00Z")
        .env("GIT_COMMITTER_DATE", "2026-04-22T12:00:00Z")
        // Fixture git reads no machine config: the ambient global config can
        // carry a `core.hooksPath` whose commit-msg hook refuses the fixture
        // identity (the multiplex tests scrub the same way).
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null");
    let status = crate::git_env::status(&mut cmd).unwrap();
    assert!(status.success(), "git {args:?} failed");
}
