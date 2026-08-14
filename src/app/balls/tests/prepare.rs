//! The start flow's focus adoption ([`AppModel::prepare_start`], §3.4/§8.1 step 1):
//! a start focuses the workspace it resolved, so the tab bar, the conversation list
//! and both composers derive from one fact.
//!
//! The regression these pin (bl-2826): New workspace raised a workspace and left
//! the focus on the previous one, so the bottom composer's bare rung still resolved
//! to the workspace the operator had just walked away from — Enter fired the goal
//! into the wrong sphere, silently, with the new workspace left an unprompted husk.
//!
//! Hermetic: the world is pre-seeded (`models.yaml` present ⇒ `lernie prime` skips,
//! §16.6 W3), so the only spawn is `lernie new` — a fake that materializes the same
//! `<ws>/repo.git` marker the real one does, or fails, per test.

use super::{model_focused, world};
use crate::cli_outbound::Cli;
use crate::test_support::{authoring_new_arm, spawn_guard};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::tempdir;

/// A fake `lernie` authoring `new`'s workspace like the real one (ARCH §2.2,
/// the shared [`authoring_new_arm`]). Every other verb exits 0.
fn news() -> String {
    format!(
        "#!/bin/sh\ncase \"$1\" in\n{}esac\nexit 0\n",
        authoring_new_arm()
    )
}
/// A fake `lernie` that refuses — the failed-prepare path.
const FAILS: &str = "#!/bin/sh\nprintf 'boom\\n' 1>&2\nexit 3\n";

/// Write `body` as an executable `lernie` in `dir` and hand back its [`Cli`].
fn fake_lernie(dir: &Path, body: &str) -> Cli {
    let path = dir.join("lernie");
    fs::write(&path, body).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    Cli::new(path)
}

/// Lay the world's seed marker so `prime` short-circuits (§16.6 W3) — the ordinary
/// case, and the one that keeps the fake `lernie` to a single verb.
fn seed(yog_data_root: &Path) {
    let lernie = crate::world::layout_under(yog_data_root).lernie;
    fs::create_dir_all(&lernie).unwrap();
    fs::write(lernie.join("models.yaml"), b"models: {}\n").unwrap();
}

#[test]
fn a_raise_focuses_the_raised_workspace_and_retargets_the_bare_rung() {
    let bin = tempdir().unwrap();
    let w = world();
    seed(&w.roots.yog_data);
    // The model is built BEFORE the guard: `AppModel::new` derives every
    // workspace snapshot through `git`, which forks via `spawn_locked` — taking
    // `SPAWN_LOCK` here first would deadlock against it (the mutex is not
    // reentrant). The guard covers only what needs it: the fake script's
    // write-then-exec window (the ETXTBSY race `test_support` documents).
    let (_c, mut m) = model_focused(&w, &w.ws_cobalt);
    assert_eq!(m.focused_workspace(), Some(w.ws_cobalt.as_path()));
    let _g = spawn_guard();

    let inputs = m.new_workspace_inputs("ops");
    let prepared = m
        .prepare_start(
            &fake_lernie(bin.path(), &news()),
            &Cli::new("/no/bl"), // the bare rung mutates no ball
            &inputs.workspace,
            &inputs.payload,
            "TS",
        )
        .unwrap();

    assert_eq!(
        prepared.workspace, "ops",
        "the operator's own name (§3.1) — the address and the stamp are one"
    );
    // The wall the raise just made: a name resolves against the *published*
    // set, and this snapshot predates the `lernie new` that founded it.
    let raised = crate::binding::workspace_path(&w.roots.yog_data, &prepared.workspace);
    assert_ne!(raised, w.ws_cobalt, "the raise always raises a fresh wall");
    assert!(raised.join("repo.git").is_dir(), "`lernie new` ran");
    assert_eq!(
        m.focused_workspace(),
        Some(raised.as_path()),
        "focus follows the raise — the tab bar and conversation list move with it"
    );
    // The bug's sharp end: the BOTTOM composer's bare rung derives from the focus,
    // so Enter now fires into the newly created workspace, not the abandoned one.
    assert_eq!(
        m.start_bare_inputs().workspace,
        raised,
        "the bare rung names the start's own workspace"
    );
}

#[test]
fn a_failed_prepare_moves_the_focus_nowhere() {
    let bin = tempdir().unwrap();
    let w = world();
    seed(&w.roots.yog_data);
    let (_c, mut m) = model_focused(&w, &w.ws_cobalt);
    let _g = spawn_guard(); // after the model, per the note above

    let inputs = m.new_workspace_inputs("ops");
    let err = m
        .prepare_start(
            &fake_lernie(bin.path(), FAILS),
            &Cli::new("/no/bl"),
            &inputs.workspace,
            &inputs.payload,
            "TS",
        )
        .unwrap_err();

    assert!(err.contains("boom"), "the refusal rode back");
    assert_eq!(
        m.focused_workspace(),
        Some(w.ws_cobalt.as_path()),
        "nothing was resolved, so nothing moved"
    );
}

/// The frame's door resolves the payload's project **name** exactly as the
/// dispatch table's Prepare arm does (REMOTE §8, bl-f5f6) — and a name this
/// snapshot enumerates no project for is a refusal saying so, before a `bl`
/// runs. The click-glue and a deposit therefore fail the same way on the same
/// token, which is the whole point of one resolution.
#[test]
fn a_ball_rung_whose_project_this_world_does_not_enumerate_refuses_by_name() {
    let bin = tempdir().unwrap();
    let w = world();
    seed(&w.roots.yog_data);
    let (_c, mut m) = model_focused(&w, &w.ws_cobalt);
    let _g = spawn_guard();

    let inputs = m.new_ball_inputs(Path::new("/nowhere/at/all"), "Fresh", "do it");
    let err = m
        .prepare_start(
            &fake_lernie(bin.path(), &news()),
            &Cli::new("/no/bl"),
            &inputs.workspace,
            &inputs.payload,
            "TS",
        )
        .unwrap_err();

    assert_eq!(err, "unknown project \"/nowhere/at/all\"");
}
