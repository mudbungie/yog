//! What `tests/multiplex_bl.rs` plants on disk before it drives the arm: the
//! plugin wrappers, the git identity, the project repository, and the two
//! readers the drive checks them with. Split from the drive at the 300-line cap
//! (bl-ff85) along the seam that was already there — the parent binary owns the
//! process environment and the rung, this file owns the scaffolding.

use std::fs;
use std::path::{Path, PathBuf};

/// A wrapper program answering a sibling plugin's contract by exec'ing the
/// built yog under that namespace (the parent's module doc).
pub(crate) fn plugin_wrapper(dir: &Path, namespace: &str) -> PathBuf {
    let path = dir.join(namespace);
    crate::write_exec::write_exec(
        &path,
        &format!(
            "#!/bin/sh\nexec '{}' '{namespace}' \"$@\"\n",
            env!("CARGO_BIN_EXE_yog")
        ),
    );
    path
}

/// The synthetic committer this binary's repositories carry — one pair, read by
/// both places it must be planted (the global fixture and the project's own).
pub(crate) const IDENT: (&str, &str) = ("Tester", "t@test.invalid");

/// The scratch global gitconfig: identity for the store commits balls seals,
/// and the wall against every other ambient global setting.
pub(crate) fn fixture_gitconfig(dir: &Path) -> PathBuf {
    let path = dir.join("gitconfig");
    let (name, email) = IDENT;
    fs::write(
        &path,
        format!("[user]\n\tname = {name}\n\temail = {email}\n[commit]\n\tgpgsign = false\n"),
    )
    .unwrap();
    path
}

pub(crate) fn git(dir: &Path, args: &[&str]) -> String {
    let out = yog::git_env::git()
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// The one dir entry of `dir` (the percent-encoded clone key, the worktree id
/// dir, …) — asserting there is exactly one.
pub(crate) fn sole_child(dir: &Path) -> PathBuf {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "one entry under {}: {entries:?}",
        dir.display()
    );
    entries.remove(0)
}

/// The project repo the store binds to, founded and seeded, left as the process
/// cwd (balls' invocation path is where `bl` runs) and returned.
///
/// **A repository this test founds carries its own identity** (bl-ff85): balls
/// rebuilds `bl-delivery`'s git environment (`safegit::delivery_env` nulls
/// global AND system config), so repository-local config is the delivery
/// authority and the scratch global fixture cannot reach it. Absent it git
/// guesses `<user>@<host>`, whose name half is empty on a runner — `close` died
/// `empty ident name` on every remote build while dev boxes passed.
///
/// The cwd is re-read from the source balls reads it from. balls keys its store
/// — and bl-delivery mirrors its territory — on the directory the process runs
/// in, which the kernel (`getcwd`) hands back fully *resolved*: on macOS the
/// tempdir's `/var/folders/…` is a symlink to `/private/var/folders/…`, so
/// bl-delivery mirrors the `/private/…` spelling while `tmp` still says
/// `/var/…`. Deriving the expected paths from the same source keeps both sides
/// in one spelling on every platform (a no-op on Linux).
pub(crate) fn found_project(tmp: &Path) -> PathBuf {
    let proj = tmp.join("proj");
    fs::create_dir(&proj).unwrap();
    git(&proj, &["init", "-q", "-b", "main"]);
    let (name, email) = IDENT;
    git(&proj, &["config", "user.name", name]);
    git(&proj, &["config", "user.email", email]);
    fs::write(proj.join("README.md"), "seed\n").unwrap();
    git(&proj, &["add", "-A"]);
    git(&proj, &["commit", "-qm", "seed"]);
    std::env::set_current_dir(&proj).unwrap();
    std::env::current_dir().unwrap()
}
