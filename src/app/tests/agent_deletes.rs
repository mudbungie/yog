//! [`AppModel`]'s §3.6 agent-delete wiring (bl-f17a): the named-workspace
//! scope, the fail-closed liveness gate, the blast-radius arming (`--children`
//! rides only on the typed name), and the aftermath — stderr verbatim, focus
//! off the dead subtree.

use super::Harness;
use crate::cli_outbound::Cli;
use crate::git_tree::AgentState;
use crate::opslog;
use crate::test_support::spawn_guard;
use std::fs;
use std::os::unix::fs::PermissionsExt;
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
    let err = model
        .delete_agent(&unused_bl(), &unused_bl(), &h.ws, "c-1", "", "TS")
        .unwrap_err();
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

    let err = model
        .delete_agent(&unused_bl(), &unused_bl(), &named, "c-1", "", "TS")
        .unwrap_err();
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
    model
        .delete_agent(&lernie, &unused_bl(), &named, "c-1", "", "TS")
        .unwrap();
    assert_eq!(logged_argv(&bin), format!("delete {} c-1", named.display()));
    assert!(
        model.focused_agent().is_none(),
        "focus never points into the deleted subtree"
    );

    // The typed conversation name — and nothing else — unlocks --children.
    model
        .delete_agent(&lernie, &unused_bl(), &named, "c-1", " hi ", "TS")
        .unwrap();
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
    let err = model
        .delete_agent(&lernie, &unused_bl(), &named, "c-1", "", "TS")
        .unwrap_err();
    assert_eq!(err, decline, "the substrate's own words, not a paraphrase");

    let mute = tempdir().unwrap();
    let silent = fake_lernie(&mute, "", "", 3);
    let err = model
        .delete_agent(&silent, &unused_bl(), &named, "c-1", "", "TS")
        .unwrap_err();
    assert_eq!(err, "lernie delete failed (exit 3)");

    // A spawn that never launched rides back too — its synthetic ops row is
    // already written by the logged runner.
    let gone = Cli::new(bin.path().join("no-such-lernie"));
    let err = model
        .delete_agent(&gone, &unused_bl(), &named, "c-1", "", "TS")
        .unwrap_err();
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
    model
        .delete_agent(&lernie, &unused_bl(), &named, "r-aa", "", "TS")
        .unwrap();
    assert!(model.focused_agent().is_none());

    // …and a focus on a neighbouring conversation stands.
    model.focus_agent(&named, "z-zz");
    model
        .delete_agent(&lernie, &unused_bl(), &named, "r-aa", "", "TS")
        .unwrap();
    assert_eq!(
        model.focused_agent().map(|a| a.agent_id.clone()),
        Some("z-zz".to_owned())
    );
}
