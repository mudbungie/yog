//! Tests for the short verbs and their dispatch/logging core.
//!
//! This module holds the shared [`World`] recorder fixture and the per-verb
//! argv/cwd tests; [`dispatch`] exercises the [`super::dispatch`] primitives
//! (run_logged, the synthetic spawn-failure line, the yog-step encoding, the
//! failure view-models) over the same fixture — split per §12's line budget.

use super::*;
use crate::actions::verbs::edit::{Create as BallCreate, Update as BallUpdate};
use crate::cli_outbound::Cli;
use crate::opslog;
use crate::xdg::Env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

mod bound;
mod dispatch;
mod edit;

/// A fake `bl`/`lernie`. Nothing brackets the write: the crate's one fork
/// (`crate::git_env`) owns the ETXTBSY exclusion, so a fixture just writes.
fn recorder(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, body).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

/// A recorder that prints `OUT` to stdout and `ERR` to stderr, exit 0.
const OK_BODY: &str = "#!/bin/sh\nprintf 'OUT\\n'\nprintf 'ERR\\n' 1>&2\nexit 0\n";

/// A hermetic verb world: a dir holding the fake binary, plus a state root for
/// `ops.jsonl`. `ws`/`project` reuse the (existing) dir so `run_in`'s chdir
/// succeeds.
struct World {
    _dir: TempDir,
    state: TempDir,
    cli: Cli,
    cwd: PathBuf,
}

impl World {
    fn new(name: &str, body: &str) -> Self {
        let dir = tempdir().unwrap();
        let bin = recorder(dir.path(), name, body);
        let cwd = dir.path().to_path_buf();
        Self {
            _dir: dir,
            state: tempdir().unwrap(),
            cli: Cli::new(bin),
            cwd,
        }
    }

    /// This world's `lernie` bound to its workspace (`cwd`) inside a world
    /// anchored on this world's own scratch — the handle every §8.2 lernie verb
    /// now takes, and the only way to reach one.
    fn bound(&self) -> Bound {
        Bound::at(
            &self.cli,
            &Env::from_pairs([("XDG_DATA_HOME", self.state.path().display().to_string())]),
            &self.cwd,
        )
    }

    /// The single logged entry (each test issues exactly one verb).
    fn logged(&self) -> crate::opslog::OpEntry {
        let mut entries = opslog::tail(self.state.path(), 8);
        assert_eq!(entries.len(), 1, "exactly one op logged");
        entries.pop().unwrap()
    }
}

/// The verb args each dispatcher builds (argv past the binary), asserted via the
/// logged entry so the exact argv AND cwd are proven at once.
fn args_of(e: &crate::opslog::OpEntry) -> Vec<String> {
    e.argv[1..].to_vec()
}

#[test]
fn message_builds_argv_and_runs_in_the_workspace() {
    let w = World::new("lernie", OK_BODY);
    let ws = &w.cwd;
    message(&w.bound(), w.state.path(), "TS", "a-1", "hi there").unwrap();
    let e = w.logged();
    assert_eq!(
        args_of(&e),
        vec!["message", &ws.display().to_string(), "a-1", "hi there"]
    );
    assert_eq!(e.cwd, ws.display().to_string());
}

#[test]
fn stop_omits_and_includes_the_children_flag() {
    let w = World::new("lernie", OK_BODY);
    let ws = w.cwd.clone();
    stop(&w.bound(), w.state.path(), "TS", "a-1", false).unwrap();
    assert_eq!(
        args_of(&w.logged()),
        vec!["stop", &ws.display().to_string(), "a-1"]
    );
    stop(&w.bound(), w.state.path(), "TS", "a-1", true).unwrap();
    let e = opslog::tail(w.state.path(), 8).pop().unwrap();
    assert_eq!(
        args_of(&e),
        vec!["stop", &ws.display().to_string(), "a-1", "--stop-children"]
    );
}

#[test]
fn scan_builds_argv() {
    let w = World::new("lernie", OK_BODY);
    let ws = &w.cwd;
    scan(&w.bound(), w.state.path(), "TS").unwrap();
    assert_eq!(
        args_of(&w.logged()),
        vec!["scan", &ws.display().to_string()]
    );
}

/// The §9.4 exit (bl-2d19): the workspace, the conversation, and **no config
/// flag** — lernie's own default lineage is the one yog's picker writes, so
/// naming it would be a knob with one lawful value.
#[test]
fn retarget_builds_argv_for_one_conversation_with_no_config_flag() {
    let w = World::new("lernie", OK_BODY);
    let ws = &w.cwd;
    retarget(&w.bound(), w.state.path(), "TS", "a-1").unwrap();
    let e = w.logged();
    assert_eq!(
        args_of(&e),
        vec!["retarget", &ws.display().to_string(), "a-1"]
    );
    assert_eq!(e.cwd, ws.display().to_string());
}

#[test]
fn close_and_unclaim_build_argv_in_the_project() {
    let w = World::new("bl", OK_BODY);
    let proj = &w.cwd;
    close(&w.cli, w.state.path(), "TS", proj, "bl-7", "filtered").unwrap();
    let e = w.logged();
    assert_eq!(args_of(&e), vec!["close", "bl-7", "--as", "filtered"]);
    assert_eq!(e.cwd, proj.display().to_string());

    unclaim(&w.cli, w.state.path(), "TS", proj, "bl-7", "filtered").unwrap();
    let e = opslog::tail(w.state.path(), 8).pop().unwrap();
    assert_eq!(args_of(&e), vec!["unclaim", "bl-7", "--as", "filtered"]);
}

#[test]
fn assign_claims_the_ball_for_the_target_workspace() {
    let w = World::new("bl", OK_BODY);
    let proj = &w.cwd;
    assign(&w.cli, w.state.path(), "TS", proj, "bl-7", "cobalt-gecko").unwrap();
    let e = w.logged();
    assert_eq!(args_of(&e), vec!["claim", "bl-7", "--as", "cobalt-gecko"]);
    assert_eq!(e.cwd, proj.display().to_string());
}

/// bl-48f8: every verb stamps its own §7.3 origin on the row it logs, and the
/// stamp is the verb's **subject** — `bl` acts on a ball, `lernie` on a
/// conversation. Exhaustive over the two families, because the surfaces filter
/// on exactly this: a verb that forgot to say would banner on the composer by
/// the parser's default and accuse a surface that did nothing.
#[test]
fn every_verb_stamps_the_origin_of_its_own_subject() {
    let w = World::new("lernie", OK_BODY);
    message(&w.bound(), w.state.path(), "TS", "a-1", "hi").unwrap();
    stop(&w.bound(), w.state.path(), "TS", "a-1", false).unwrap();
    scan(&w.bound(), w.state.path(), "TS").unwrap();
    for e in opslog::tail(w.state.path(), 8) {
        assert_eq!(e.origin, opslog::Origin::Conversation, "{:?}", e.argv);
    }

    let w = World::new("bl", "#!/bin/sh\nprintf 'bl-9zzz\\n'\nexit 0\n");
    let proj = w.cwd.clone();
    close(&w.cli, w.state.path(), "TS", &proj, "bl-7", "amber").unwrap();
    unclaim(&w.cli, w.state.path(), "TS", &proj, "bl-7", "amber").unwrap();
    assign(&w.cli, w.state.path(), "TS", &proj, "bl-7", "cobalt").unwrap();
    create(
        &w.cli,
        w.state.path(),
        "TS",
        &proj,
        "amber",
        &BallCreate {
            title: "t".into(),
            ..BallCreate::default()
        },
    )
    .unwrap();
    update(
        &w.cli,
        w.state.path(),
        "TS",
        &proj,
        "bl-7",
        "amber",
        &BallUpdate::default(),
    )
    .unwrap();
    let ops = opslog::tail(w.state.path(), 16);
    assert_eq!(ops.len(), 5, "one row per bl verb");
    for e in ops {
        assert_eq!(e.origin, opslog::Origin::Balls, "{:?}", e.argv);
    }
}
