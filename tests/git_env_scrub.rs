//! The spawn-boundary regression for bl-916a: **a child yog spawns inherits no
//! ambient git environment**, so the `git` that child forks of its own accord
//! can never be re-aimed at the repo a hook is committing to.
//!
//! bl-0dff scrubbed yog's *own* `git` forks. That left the larger half open:
//! `bl`, `litany`, an `$EDITOR` shim and the suite's fake substrate scripts all
//! fork `git` themselves, and they inherit whatever yog hands them. Running the
//! suite from `.githooks/pre-commit` (where `git` exports `GIT_DIR` and
//! `GIT_INDEX_FILE`) therefore let the fake `litany new` arm's
//! `git commit -m 'config: init [config/default]'` land on the **outer work
//! branch**, replacing its tree — observed, not theorized.
//!
//! The invariant now lives at the one place children are built,
//! [`yog::git_env::command`], so one `env_remove` clears the whole descendant
//! process tree. This test proves it end to end: it sets the hook's variables in
//! its OWN environment, spawns a probe through [`Cli::run`], and checks both
//! that the probe saw none of them and that the `git` the probe forked answered
//! about the probe's fixture rather than the decoy repo `GIT_DIR` names.
//!
//! **One `#[test]` per binary, deliberately.** The setup mutates process-global
//! env, `unsafe` in edition 2024 and sound only with no peer thread reading it —
//! the same discipline (and the same reason) as `tests/multiplex_bl.rs`.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use yog::cli_outbound::Cli;
use yog::git_env::INHERITED;

/// The probe: a fake substrate binary in the shape of the one that caused the
/// corruption. It records every `GIT_*` variable it can see, then forks `git`
/// itself — `init` plus a `rev-parse`, whose answer names whichever repo that
/// child `git` actually resolved.
fn probe(dir: &Path) -> std::path::PathBuf {
    let path = dir.join("probe");
    fs::write(
        &path,
        "#!/bin/sh\nenv | sed -n 's/^\\(GIT_[A-Z_]*\\)=.*/\\1/p' | sort > \"$1/leaked\"\n\
         git init -q --bare \"$1/probe.git\"\n\
         git -C \"$1/probe.git\" rev-parse --absolute-git-dir > \"$1/aimed\"\n",
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

#[test]
fn a_spawned_child_inherits_none_of_the_ambient_git_environment() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path().canonicalize().unwrap();

    // The decoy: exactly what `git` exports into a hook, naming a repo that is
    // NOT the one the child means to touch. An unscrubbed spawn aims the
    // child's own `git init`/`rev-parse` here.
    let decoy = dir.join("decoy.git");
    let init = yog::git_env::git()
        .args(["init", "-q", "--bare"])
        .arg(&decoy)
        .status()
        .unwrap();
    assert!(init.success(), "decoy repo");
    for (var, value) in [
        ("GIT_DIR", decoy.clone()),
        ("GIT_INDEX_FILE", decoy.join("index")),
    ] {
        // SAFETY: single-threaded — this binary runs exactly one #[test], and
        // no other thread exists to read the env concurrently (module doc).
        unsafe { std::env::set_var(var, value) };
    }

    let work = dir.join("work");
    fs::create_dir(&work).unwrap();
    let stream = Cli::new(probe(&dir))
        .run(&[work.to_str().unwrap()])
        .unwrap();
    stream.count(); // drain to the child's exit

    // Only the repo-aiming set is scrubbed: identity/editor vars (`GIT_AUTHOR_*`,
    // `GIT_EDITOR`) name no repository and ride through untouched by design, so
    // the check is the intersection with `INHERITED`, not "no `GIT_*` at all".
    let leaked = fs::read_to_string(work.join("leaked")).unwrap();
    let aimers: Vec<&str> = leaked.lines().filter(|v| INHERITED.contains(v)).collect();
    assert!(
        aimers.is_empty(),
        "the child inherited repo-aiming git vars: {aimers:?}"
    );
    assert_eq!(
        fs::read_to_string(work.join("aimed")).unwrap().trim(),
        work.join("probe.git").to_str().unwrap(),
        "the git the child forked answered about the decoy, not its own fixture"
    );
}
