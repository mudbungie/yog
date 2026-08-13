//! The scrub is asserted two ways: on the command's env delta (exact, and it
//! holds even on a machine where no `GIT_*` is set), and end-to-end against a
//! throwaway repo — the latter is the bl-0dff regression itself, so a suite run
//! from inside a git hook fails HERE, naming the invariant, instead of three
//! rows down in `fs_watcher::drift_tests` with no clue why.

use super::{INHERITED, git};
use std::process::Stdio;
use tempfile::tempdir;

/// Run a scrubbed `git` in `dir` and hand back its trimmed stdout. Forks under
/// the binary-wide `SPAWN_LOCK` (`crate::test_support`) like every other fork
/// in this binary.
fn run(dir: &std::path::Path, args: &[&str]) -> String {
    let mut cmd = git();
    cmd.args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let out = crate::test_support::spawn_locked(&mut cmd)
        .unwrap()
        .wait_with_output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {out:?}");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn every_inherited_var_is_removed_from_the_child_and_nothing_else_is_touched() {
    let cmd = git();
    let delta: Vec<(String, Option<String>)> = cmd
        .get_envs()
        .map(|(k, v)| {
            (
                k.to_string_lossy().into_owned(),
                v.map(|v| v.to_string_lossy().into_owned()),
            )
        })
        .collect();
    for var in INHERITED {
        assert!(
            delta.contains(&((*var).to_string(), None)),
            "{var} is not scrubbed: {delta:?}"
        );
    }
    assert_eq!(delta.len(), INHERITED.len(), "only the scrub, nothing else");
    assert_eq!(cmd.get_program(), "git", "it is still a git command");
}

#[test]
fn a_scrubbed_git_answers_about_the_directory_it_was_pointed_at() {
    // The bl-0dff invariant, end to end: an ambient `GIT_DIR` (which `git`
    // exports into every hook it runs) outranks `current_dir`, so an unscrubbed
    // fork here would answer with the OUTER repo's git dir instead of this
    // fixture's. Fails only where the bug can actually bite — under a hook.
    let dir = tempdir().unwrap();
    let repo = dir.path().canonicalize().unwrap().join("repo.git");
    std::fs::create_dir_all(&repo).unwrap();
    run(&repo, &["init", "-q", "--bare", "."]);
    assert_eq!(
        std::path::PathBuf::from(run(&repo, &["rev-parse", "--absolute-git-dir"])),
        repo,
        "the fixture, not whatever repo the ambient git env names"
    );
}
