//! [`AppModel`]'s §3.6 wiring: who may be unmade, the fail-closed gate, the
//! typed-name arming, and the convergence after the wall comes down.

use super::Harness;
use crate::cli_outbound::Cli;
use crate::git_tree::AgentState;
use crate::git_tree::tests::fixture::Fixture;
use crate::opslog::{self, YOG_STEP};
use std::path::PathBuf;

/// A `bl` that is never spawned: every test here deletes a workspace with no
/// bound balls, so the plan carries no `unclaim` step.
fn unused_bl() -> Cli {
    Cli::new("bl")
}

/// One of **yog's own** named workspaces (§3.1: `<yog_data>/workspaces/<name>`,
/// the leaf being the name) carrying one agent. The directory itself is real —
/// only its contents are symlinked in from a fixture — because the §3.6 unmaking
/// removes that directory, and `remove_dir_all` refuses a symlink. The fixture
/// is leaked so it outlives the model.
fn add_named(h: &Harness, name: &str, agent: &str) -> PathBuf {
    let fx = Fixture::new();
    fx.build_agent(agent, "hi");
    let ws = crate::binding::names_root(&h.roots.yog_data).join(name);
    std::fs::create_dir_all(&ws).unwrap();
    for entry in std::fs::read_dir(&fx.path).unwrap().flatten() {
        let from = entry.path();
        let leaf = from.file_name().unwrap_or_default().to_owned();
        std::os::unix::fs::symlink(&from, ws.join(leaf)).unwrap();
    }
    std::mem::forget(fx);
    ws
}

#[test]
fn the_verb_is_offered_only_on_yogs_own_named_workspaces() {
    // §3.6 scope: foreign workspaces are lernie's retention-governed territory
    // and replays are read-only — yog may not delete what it did not place.
    let mut h = Harness::new();
    let named = add_named(&h, "alba-koi", "c-1");
    let replay = h.add_replay("20260101T-rr", "c-9");
    let (_c, model) = h.model();

    let confirm = model.delete_confirmation(&named).unwrap();
    assert_eq!(confirm.name, "alba-koi");
    assert_eq!(confirm.conversations, ["hi"], "named by its preview (§11)");
    assert!(confirm.ball_ids().is_empty());
    assert!(model.delete_confirmation(&h.ws).is_none(), "foreign");
    assert!(model.delete_confirmation(&replay).is_none(), "replay");
    assert!(model.delete_confirmation(&named.join("nope")).is_none());
}

#[test]
fn a_workspace_yog_did_not_place_is_refused_outright() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    let err = model
        .delete_workspace(&unused_bl(), &unused_bl(), &h.ws, "ws", "TS")
        .unwrap_err();
    assert_eq!(err, "not a yog-named workspace");
    assert!(h.ws.exists());
}

#[test]
fn a_live_driver_refuses_the_unmaking_and_names_the_conversation() {
    // §3.6's gate, fail-closed: an `rm` under a flock-holding driver races a
    // running process, and folding a Stop into the delete would destroy running
    // work across two substrates. Verbs stay orthogonal.
    let h = Harness::new();
    let named = add_named(&h, "alba-koi", "c-1");
    let (_c, mut model) = h.model();
    for agent in &mut model.deriver.trees.get_mut(&named).unwrap().agents {
        agent.state = AgentState::Live;
    }
    model.publish();

    let err = model
        .delete_workspace(&unused_bl(), &unused_bl(), &named, "alba-koi", "TS")
        .unwrap_err();
    assert_eq!(err, "refused \u{2014} live conversations: hi");
    assert!(named.exists(), "the wall stands until the driver stops");
}

#[test]
fn the_typed_name_is_the_safety_mechanism() {
    let h = Harness::new();
    let named = add_named(&h, "alba-koi", "c-1");
    let (_c, mut model) = h.model();
    let err = model
        .delete_workspace(&unused_bl(), &unused_bl(), &named, "alba", "TS")
        .unwrap_err();
    assert_eq!(err, "type the workspace's name to confirm");
    assert!(named.exists());
}

#[test]
fn the_unmaking_removes_the_wall_moves_the_focus_and_leaves_the_trail() {
    let h = Harness::new();
    let named = add_named(&h, "alba-koi", "c-1");
    let (_c, mut model) = h.model_focused(Some(named.clone()));
    assert_eq!(model.focused_workspace(), Some(named.as_path()));

    model
        .delete_workspace(&unused_bl(), &unused_bl(), &named, "alba-koi", "TS")
        .unwrap();
    // The unmaking names the roots it changed; the worker re-enumerates on its
    // next pass (§7.2) — the removal is already real on disk either way.
    model.tick();

    assert!(!named.exists(), "the workspace directory is gone");
    assert!(
        !model.workspaces().iter().any(|w| w.path == named),
        "the removal IS the de-registration (§3.1)"
    );
    assert_ne!(
        model.focused_workspace(),
        Some(named.as_path()),
        "focus never points at a gone directory"
    );
    // The trail survives its subject (§3.6, §4.2).
    let ops = opslog::tail(&h.roots.yog_state, 8);
    assert_eq!(
        ops.last().map(|e| e.argv.clone()),
        Some(vec![YOG_STEP.to_owned(), "delete-workspace".to_owned()])
    );
    assert!(model.ops_rows().iter().any(|r| !r.failed()));
}
