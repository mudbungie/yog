//! The §3.6 agent delete (bl-f17a): the named-workspace scope, the fail-closed
//! liveness gate, the blast-radius arming (`--children` rides only on the typed
//! name), and the aftermath — the substrate's own words, focus off the dead
//! subtree.
//!
//! **Driven from where the engine stands** (bl-1747): the dialog posts the
//! gesture and folds its receipt, so what is exercised here is
//! `Action::DeleteAgent` through the one chokepoint plus
//! [`AppModel::deleted_agent`], the convergence a clean receipt earns. The
//! sentence the dialog paints for a declined verb is one projection
//! (`shell::act::trouble`, bl-afa9) and is asserted where it is painted; what
//! this file asserts is the outcome that projection reads.

use super::Harness;
use crate::AppModel;
use crate::boundary::{Action, reply::Reply};
use crate::cli_outbound::Cli;
use crate::git_tree::AgentState;
use crate::opslog;
use crate::test_support::{engine, spawn_guard};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::{TempDir, tempdir};

/// A fake `lernie`: logs `$@` beside itself, prints `stdout`, exits `code`.
fn fake_lernie(dir: &TempDir, stdout: &str, stderr: &str, code: i32) -> Cli {
    let path = dir.path().join("lernie");
    fs::write(
        &path,
        format!(
            "#!/bin/sh\necho \"$@\" > {}/argv.log\nprintf '%s\\n' '{stdout}'\nprintf '%s' '{stderr}' 1>&2\nexit {code}\n",
            dir.path().display()
        ),
    )
    .unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    Cli::new(path)
}

fn logged_argv(dir: &TempDir) -> String {
    fs::read_to_string(dir.path().join("argv.log"))
        .unwrap_or_default()
        .trim()
        .to_owned()
}

/// A `bl` that is never spawned: no step of an agent delete runs `bl`.
fn unused_bl() -> Cli {
    Cli::new("bl")
}

/// The delete as the §3.6 dialog fires it (bl-1747): the gesture through the
/// engine's own chokepoint, then the convergence a receipt earns.
fn delete(
    model: &mut AppModel,
    lernie: &Cli,
    ws: &Path,
    root: &str,
    typed: &str,
) -> Result<Reply, String> {
    let deps = model.boundary_deps(lernie, &unused_bl());
    let action = Action::DeleteAgent {
        workspace: model.snap.ws_name(ws),
        agent: root.to_owned(),
        typed: typed.to_owned(),
    };
    let landed = engine::act(model, &deps, "TS", &action);
    // The act's own root, marked by the receipt for every act alike
    // (`AppModel::settle_acts`): an agent delete names no project, so it is the
    // yog-state root's ordinary routing (§7.1).
    model.after_lernie_verb();
    model.deleted_agent(ws, root);
    landed
}

/// The captured run behind a clean reply — what the dialog reads to decide
/// whether the removal happened.
fn outcome(landed: Result<Reply, String>) -> crate::actions::verbs::Outcome {
    match landed.expect("the verb ran") {
        Reply::Outcome(o) => o,
        other => panic!("a verb answers an outcome, not {other:?}"),
    }
}

#[test]
fn the_confirmation_is_offered_only_inside_named_workspaces() {
    // The §3.6 scope, one conversation deep: same rule as the workspace verb —
    // foreign workspaces are another driver's territory, replays read-only.
    let mut h = Harness::new();
    let named = h.mint_named("alba-koi", "c-1");
    let replay = h.add_replay("20260101T-rr", "c-9");
    let (_c, model) = h.model();

    let confirm = model.agent_delete_confirmation(&named, "c-1").unwrap();
    assert_eq!(confirm.name, "hi", "the display ladder names the dialog");
    assert!(!confirm.refused());
    assert!(
        model.agent_delete_confirmation(&h.ws, "c-1").is_none(),
        "foreign"
    );
    assert!(
        model.agent_delete_confirmation(&replay, "c-9").is_none(),
        "replay"
    );
}

#[test]
fn an_unnamed_workspace_is_refused_outright() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    let err = delete(&mut model, &unused_bl(), &h.ws, "c-1", "").unwrap_err();
    assert_eq!(err, "not a yog-named workspace");
}

#[test]
fn a_live_member_refuses_before_anything_spawns() {
    // Fail closed (§3.6): never reap the substrate beneath a running driver;
    // Stop keeps its own semantics, and nothing was attempted — no ops row.
    let mut h = Harness::new();
    let named = h.mint_named("alba-koi", "c-1");
    let (_c, mut model) = h.model();
    for agent in &mut model.deriver.trees.get_mut(&named).unwrap().agents {
        agent.state = AgentState::Live;
    }
    model.publish();

    let err = delete(&mut model, &unused_bl(), &named, "c-1", "").unwrap_err();
    assert_eq!(err, "refused \u{2014} live: c-1 \u{2014} stop them first");
    assert!(opslog::tail(&h.roots.yog_state, 8).is_empty());
}

#[test]
fn a_leaf_fires_the_bare_verb_and_the_typed_name_arms_the_subtree() {
    let _g = spawn_guard();
    let mut h = Harness::new();
    let named = h.mint_named("alba-koi", "c-1");
    let bin = tempdir().unwrap();
    let lernie = fake_lernie(
        &bin,
        "deleted c-1; descendants: 0; pending deposits: 0",
        "",
        0,
    );
    let (_c, mut model) = h.model_focused(Some(named.clone()));
    model.focus_agent(&named, "c-1");
    assert!(model.focused_agent().is_some());

    // Unarmed (`typed` restates nothing): the bare verb — a subtree nobody
    // confirmed would be declined by lernie itself, never assumed here.
    assert!(outcome(delete(&mut model, &lernie, &named, "c-1", "")).ok());
    assert_eq!(logged_argv(&bin), format!("delete {} c-1", named.display()));
    assert!(
        model.focused_agent().is_none(),
        "focus never points into the deleted subtree"
    );

    // The typed conversation name — and nothing else — unlocks --children.
    assert!(outcome(delete(&mut model, &lernie, &named, "c-1", " hi ")).ok());
    assert_eq!(
        logged_argv(&bin),
        format!("delete {} c-1 --children", named.display())
    );
    let ops = opslog::tail(&h.roots.yog_state, 8);
    assert_eq!(ops.len(), 2, "each removal leaves its ops row");
    assert!(ops.iter().all(|e| e.exit == 0));
}

#[test]
fn a_declined_verb_rides_back_its_stderr_verbatim() {
    let _g = spawn_guard();
    let mut h = Harness::new();
    let named = h.mint_named("alba-koi", "c-1");
    let bin = tempdir().unwrap();
    let decline = "agent \"c-1\" has 1 descendant(s); pass --children";
    let lernie = fake_lernie(&bin, "", decline, 2);
    let (_c, mut model) = h.model();
    let ran = outcome(delete(&mut model, &lernie, &named, "c-1", ""));
    assert!(!ran.ok(), "a non-zero verb is not a removal");
    assert_eq!(
        ran.stderr.trim(),
        decline,
        "the substrate's own words, not a paraphrase"
    );
    assert_eq!(ran.exit, 2);

    // A spawn that never launched rides back as the refusal itself — its
    // synthetic ops row is already written by the logged runner.
    let gone = Cli::new(bin.path().join("no-such-lernie"));
    let err = delete(&mut model, &gone, &named, "c-1", "").unwrap_err();
    assert!(err.contains("No such file"), "{err}");
}

#[test]
fn a_focus_on_a_descendant_clears_with_its_root_and_a_neighbour_survives() {
    let _g = spawn_guard();
    let mut h = Harness::new();
    let named = h.mint_named("alba-koi", "r-aa");
    h.last_added().build_agent("r-aa-c-bb", "child");
    h.last_added().build_agent("z-zz", "other");
    let bin = tempdir().unwrap();
    let lernie = fake_lernie(
        &bin,
        "deleted r-aa; descendants: 1 (r-aa-c-bb); pending deposits: 0",
        "",
        0,
    );

    // A focus inside the subtree clears with it…
    let (_c, mut model) = h.model_focused(Some(named.clone()));
    model.focus_agent(&named, "r-aa-c-bb");
    assert!(outcome(delete(&mut model, &lernie, &named, "r-aa", "")).ok());
    assert!(model.focused_agent().is_none());

    // …and a focus on a neighbouring conversation stands.
    model.focus_agent(&named, "z-zz");
    assert!(outcome(delete(&mut model, &lernie, &named, "r-aa", "")).ok());
    assert_eq!(
        model.focused_agent().map(|a| a.agent_id.clone()),
        Some("z-zz".to_owned())
    );
}
