//! The §3.6 executor's own tables: the per-step `ops.jsonl` records, the aborts
//! that stop before the removal, and the refusal wording. Everything here needs
//! the effect world (a fake `bl`, a state root, a real workspace dir); the pure
//! half — the confirmation's gate and the plan's load-bearing order — is
//! `super::super`'s.

use super::super::{Claim, DELETE_STEP, DeleteError, Step, confirmation, execute, plan};
use crate::cli_outbound::Cli;
use crate::opslog::{self, OpEntry, SYNTHETIC_EXIT, YOG_STEP};
use crate::ui_state::UiState;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

use super::super::tests::{NAME, claim};

/// The effect world: a fake-binary dir, a yog state root (`ops.jsonl` +
/// `ui.json`), a project dir, and a real workspace directory to remove.
struct World {
    bin: TempDir,
    state: TempDir,
    project: TempDir,
    names: TempDir,
}

impl World {
    fn new() -> Self {
        let me = Self {
            bin: tempdir().unwrap(),
            state: tempdir().unwrap(),
            project: tempdir().unwrap(),
            names: tempdir().unwrap(),
        };
        fs::create_dir_all(me.workspace().join("repo.git")).unwrap();
        me
    }

    fn workspace(&self) -> PathBuf {
        self.names.path().join(NAME)
    }

    fn ui(&self) -> UiState {
        UiState::open(self.state.path().join("ui.json"))
    }

    fn ops(&self) -> Vec<OpEntry> {
        opslog::tail(self.state.path(), 16)
    }

    /// A `bl` that exits `code`, echoing its args back on stderr.
    fn bl(&self, code: i32) -> Cli {
        let path = self.bin.path().join("bl");
        fs::write(&path, format!("#!/bin/sh\necho \"$@\" 1>&2\nexit {code}\n")).unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        Cli::new(path)
    }

    /// This world's wall root — the sphere's brazen state (§16.2 as amended),
    /// removed with the sphere.
    fn wall_root(&self) -> std::path::PathBuf {
        self.names.path().join("world")
    }

    /// This workspace's own wall dir (§16.2), materialized with a credential in
    /// it — the thing a deletion must take down with the sphere.
    fn seed_wall(&self) -> PathBuf {
        let wall = crate::world::wall::root_under(&self.wall_root(), NAME);
        let creds = crate::config_edit::brazen::BrazenPaths::in_wall(&wall).credentials_dir;
        fs::create_dir_all(&creds).unwrap();
        fs::write(creds.join("openai.json"), b"{}").unwrap();
        wall
    }

    /// The plan for this world's workspace, carrying `claims`.
    fn steps(&self, claims: Vec<Claim>) -> Vec<Step> {
        plan(
            &confirmation(NAME, &[], claims),
            &self.workspace(),
            &self.wall_root(),
            &[self.project.path().to_path_buf()],
        )
    }
}

#[test]
fn execute_releases_prunes_and_removes_leaving_one_step_row() {
    let w = World::new();
    let key = w.workspace().to_string_lossy().into_owned();
    let mut ui = w.ui();
    ui.set_pinned(vec![key.clone()]);

    let claims = vec![claim(&crate::naming::leaf(w.project.path()), "bl-7")];
    execute(&w.steps(claims), &w.bl(0), &mut ui, w.state.path(), "TS").unwrap();

    assert!(!w.workspace().exists(), "the sphere wall is down");
    assert!(ui.pinned().is_empty(), "the dead workspace's pin is pruned");
    let ops = w.ops();
    assert_eq!(&ops[0].argv[1..], &["unclaim", "bl-7", "--as", NAME]);
    assert_eq!(ops[0].cwd, w.project.path().display().to_string());
    assert_eq!(&ops[1].argv, &[YOG_STEP.to_owned(), DELETE_STEP.to_owned()]);
    assert_eq!(ops[1].exit, 0, "a completed step states its status");
    assert_eq!(ops[1].cwd, w.names.path().display().to_string());
}

#[test]
fn the_sphere_takes_its_wall_down_with_it() {
    // §3.6 as amended: a workspace's providers and sign-ins are its own, so an
    // orphaned wall would hand a dead sphere's credentials to the next
    // workspace that takes the name.
    let w = World::new();
    let wall = w.seed_wall();
    let mut ui = w.ui();
    execute(
        &w.steps(Vec::new()),
        &w.bl(0),
        &mut ui,
        w.state.path(),
        "TS",
    )
    .unwrap();
    assert!(!wall.exists(), "the wall went with the workspace");
    assert!(!w.workspace().exists());
}

#[test]
fn a_wall_that_cannot_be_removed_aborts_the_unmaking() {
    // The removal is one step: a wall that refuses (here, a *file* where the
    // wall dir belongs) leaves the workspace standing rather than half-unmade,
    // and the synthetic step row records it.
    let w = World::new();
    let wall = crate::world::wall::root_under(&w.wall_root(), NAME);
    fs::create_dir_all(wall.parent().expect("the walls dir")).unwrap();
    fs::write(&wall, b"not a dir").unwrap();
    let mut ui = w.ui();
    let err = execute(
        &w.steps(Vec::new()),
        &w.bl(0),
        &mut ui,
        w.state.path(),
        "TS",
    )
    .unwrap_err();
    assert!(matches!(err, DeleteError::Io(_)));
    assert!(w.workspace().exists(), "nothing was half-unmade");
    assert_eq!(w.ops()[0].exit, SYNTHETIC_EXIT);
}

#[test]
fn a_refused_release_aborts_before_the_removal() {
    let w = World::new();
    let mut ui = w.ui();
    let claims = vec![claim(&crate::naming::leaf(w.project.path()), "bl-7")];
    let err = execute(&w.steps(claims), &w.bl(3), &mut ui, w.state.path(), "TS").unwrap_err();

    assert!(matches!(err, DeleteError::ReleaseFailed { ref id, .. } if id == "bl-7"));
    assert!(
        err.to_string()
            .starts_with("`bl unclaim bl-7` failed (exit 3)")
    );
    assert!(
        w.workspace().exists(),
        "releases first, removal last — the wall stands"
    );
    assert_eq!(w.ops().len(), 1, "only the refused unclaim's own row");
}

#[test]
fn a_failed_removal_leaves_a_synthetic_step_row() {
    let w = World::new();
    let mut ui = w.ui();
    let steps = plan(
        &confirmation(NAME, &[], Vec::new()),
        &w.names.path().join("never-minted"),
        &w.wall_root(),
        &[],
    );
    let err = execute(&steps, &w.bl(0), &mut ui, w.state.path(), "TS").unwrap_err();

    assert!(matches!(err, DeleteError::Io(_)));
    let ops = w.ops();
    assert_eq!(&ops[0].argv, &[YOG_STEP.to_owned(), DELETE_STEP.to_owned()]);
    assert_eq!(ops[0].exit, SYNTHETIC_EXIT);
    assert!(!ops[0].stderr.is_empty(), "the failure text is the record");
}

#[test]
fn a_spawn_failure_rides_back_as_io() {
    let w = World::new();
    let mut ui = w.ui();
    let claims = vec![claim(&crate::naming::leaf(w.project.path()), "bl-7")];
    let bl = Cli::new(w.bin.path().join("no-such-bl"));
    let err = execute(&w.steps(claims), &bl, &mut ui, w.state.path(), "TS").unwrap_err();

    assert!(matches!(err, DeleteError::Io(_)));
    assert!(w.workspace().exists());
    assert_eq!(
        w.ops()[0].exit,
        SYNTHETIC_EXIT,
        "the un-launched spawn's row"
    );
}

#[test]
fn the_refusal_errors_say_what_they_refused() {
    assert_eq!(
        DeleteError::NotArmed.to_string(),
        "type the workspace's name to confirm"
    );
    assert_eq!(
        DeleteError::Unnamed.to_string(),
        "not a yog-named workspace"
    );
}
