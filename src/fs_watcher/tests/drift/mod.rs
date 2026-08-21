//! The drift scenario suite (bl-49f4, DESIGN §7.2/§7.3): the real systems and
//! processes that make a filesystem change reach yog late or not at all, each
//! built deliberately and the watcher's behaviour asserted.
//!
//! Every scenario here was a *silent* loss before this suite existed — repaired
//! 15 s later by the full sweep, with nothing to say it had happened. Two are
//! now caught at the watcher layer (the backend's own loss announcements), one
//! was a plain allowlist hole (`packed-refs`), and one was already correct and
//! is pinned so it stays that way (atomic renames).
//!
//! Split at the observation seam: [`loss`] holds the announcements that arrive
//! on the event channel itself (a kernel overflow, a backend error, a dead
//! inode), [`refs`] the git ref writes that must reach a live watcher. The
//! fixtures both halves share live here.

mod loss;
mod refs;

use super::super::*;
use std::fs;
use std::time::Duration;
use tempfile::tempdir;

/// Detection budget, mirroring `tests.rs` — generous for FSEvents, first-sample
/// on Linux inotify.
const DETECT: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(25);

/// Poll `watcher.tick()` until some change satisfies `pred`; the full tick's
/// changes come back so siblings can be asserted on too. Empty on timeout.
/// Built on `tests::poll_until`, whose found and timed-out arms are unit-tested
/// there — no second polling loop to leave half-exercised.
fn wait_for(watcher: &Watcher, pred: impl Fn(&Change) -> bool) -> Vec<Change> {
    let probe = || {
        let changes = watcher.tick();
        changes.iter().any(&pred).then_some(changes)
    };
    super::poll_until(probe, DETECT, POLL).unwrap_or_default()
}

/// Run `git` in `dir`, asserting it succeeded. Hermetic against the machine's
/// own git config (identity, `core.hooksPath`) AND against the ambient git env
/// (`crate::git_env` — an inherited `GIT_DIR` outranks `current_dir` and aimed
/// these forks at the real repo whenever a hook ran the suite, bl-0dff).
fn git(dir: &Path, args: &[&str]) -> String {
    let out = crate::git_env::output(
        crate::git_env::git()
            .args(args)
            .current_dir(dir)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .env("GIT_AUTHOR_NAME", "Tester")
            .env("GIT_AUTHOR_EMAIL", "t@t.local")
            .env("GIT_COMMITTER_NAME", "Tester")
            .env("GIT_COMMITTER_EMAIL", "t@t.local"),
    )
    .expect("git runs");
    assert!(out.status.success(), "git {args:?}: {out:?}");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A workspace holding a bare `repo.git` with two agent refs — the shape
/// `git_tree` reads through `git for-each-ref` (ARCH §2.2/§2.3).
fn workspace_with_refs() -> (tempfile::TempDir, PathBuf) {
    let dir = tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let repo = root.join("repo.git");
    fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q", "--bare", "."]);
    let tree = git(&repo, &["hash-object", "-w", "-t", "tree", "/dev/null"]);
    let commit = git(&repo, &["commit-tree", &tree, "-m", "x"]);
    git(&repo, &["update-ref", "refs/heads/agents/aa-bb", &commit]);
    git(&repo, &["update-ref", "refs/heads/agents/cc-dd", &commit]);
    (dir, root)
}
