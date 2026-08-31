//! **The §3.6 gates, fired at the chokepoint** (§8.5, bl-f17a) — the arm every
//! seat crosses (`boundary::dispatch::delete_exec`), driven from where the
//! engine stands.
//!
//! Both classes gate at FIRE time, off the published snapshot, never off the
//! confirmation that offered the verb: a seat's dialog can be a frame old, and
//! what the gate reads is what is true now. Each refusal attempts nothing —
//! that is what fail-closed means here — and the spawn's own echo is the
//! witness that it did or did not happen.

use super::Harness;
use crate::boundary::{Action, reply::Reply};
use crate::cli_outbound::Cli;
use crate::git_tree::AgentState;
use crate::test_support::engine;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tempfile::{TempDir, tempdir};

/// A fake binary that echoes its own argv, so a recorded `stdout` is exactly
/// what the act spawned — and an empty one is proof it spawned nothing.
fn fake(dir: &Path, name: &str) -> Cli {
    let path = dir.join(name);
    fs::write(&path, "#!/bin/sh\nprintf '%s\\n' \"$*\"\nexit 0\n").unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    Cli::new(path)
}

/// A world holding one **named** workspace (`cobalt`, one stopped agent `c-1`)
/// beside the harness's own ad-hoc one — the two sides of §3.6's scope in one
/// fixture, since the refusal and the removal are the same arm's two answers.
struct World {
    bin: TempDir,
    h: Harness,
    named: PathBuf,
}

fn world() -> World {
    let bin = tempdir().unwrap();
    let mut h = Harness::new();
    let named = h.mint_named("cobalt", "c-1");
    World { bin, h, named }
}

impl World {
    fn deps(&self, rig: &crate::AppModel) -> crate::boundary::dispatch::Deps {
        rig.boundary_deps(
            &fake(self.bin.path(), "litany"),
            &fake(self.bin.path(), "bl"),
        )
    }
}

/// A workspace outside yog's own named set has no confirmation at all, so the
/// arm refuses before it reads a gate — the scope §3.6 draws, and the same one
/// the agent class draws.
#[test]
fn an_unnamed_workspace_is_refused_before_any_gate() {
    let w = world();
    let (_c, m) = w.h.model();
    let deps = w.deps(&m);
    for action in [
        Action::DeleteWorkspace {
            workspace: "ws".to_owned(),
            typed: "ws".to_owned(),
        },
        Action::DeleteAgent {
            workspace: "ws".to_owned(),
            agent: "c-1".to_owned(),
            typed: String::new(),
        },
    ] {
        let err = engine::act(&m, &deps, "TS", &action).unwrap_err();
        assert!(err.contains("not a yog-named workspace"), "{err}");
    }
    assert!(w.h.ws.exists(), "a refusal attempts nothing");
}

/// The typed name is the arming, checked at fire time. An unarmed fire removes
/// nothing; the armed one takes the wall down.
#[test]
fn a_workspace_delete_needs_its_own_name_typed() {
    let w = world();
    let (_c, m) = w.h.model();
    let deps = w.deps(&m);
    let unmake = |typed: &str| {
        engine::act(
            &m,
            &deps,
            "TS",
            &Action::DeleteWorkspace {
                workspace: "cobalt".to_owned(),
                typed: typed.to_owned(),
            },
        )
    };
    let err = unmake("spare").unwrap_err();
    assert!(err.contains("type the workspace's name"), "{err}");
    assert!(w.named.exists(), "a refusal attempts nothing");

    assert!(matches!(unmake(" cobalt ").unwrap(), Reply::Deleted));
    assert!(
        !w.named.exists(),
        "the armed fire unmakes the workspace's own directory"
    );
}

/// A live **member** refuses both classes, and names it. The liveness is the
/// snapshot's, so the gate is what the derivation last published rather than
/// what a dialog was told — and a live child refuses its root's deletion for
/// the same reason the workspace's: nothing under it may be removed while
/// something is holding it.
#[test]
fn a_live_conversation_refuses_the_workspace_and_the_conversation() {
    let w = world();
    let (_c, mut rig) = w.h.model();
    rig.deriver
        .trees
        .entry(w.named.clone())
        .or_default()
        .agents
        .push(super::agent("c-1-x-2", AgentState::Live));
    rig.publish();
    let deps = w.deps(&rig.model);
    let ws_err = engine::act(
        &rig.model,
        &deps,
        "TS",
        &Action::DeleteWorkspace {
            workspace: "cobalt".to_owned(),
            typed: "cobalt".to_owned(),
        },
    )
    .unwrap_err();
    assert!(
        ws_err.contains("live conversations") && ws_err.contains("hi"),
        "the refusal names what is live, by the §3.3 display name a seat shows \
         rather than by an id: {ws_err}"
    );
    let agent_err = engine::act(
        &rig.model,
        &deps,
        "TS",
        &Action::DeleteAgent {
            workspace: "cobalt".to_owned(),
            agent: "c-1".to_owned(),
            typed: String::new(),
        },
    )
    .unwrap_err();
    assert!(agent_err.contains("stop them first"), "{agent_err}");
    assert!(w.named.exists(), "neither refusal attempted anything");
}

/// The conversation-deep class: the verb is litany's, and `--children` rides
/// exactly when the typed name re-states the conversation's own — the census
/// the substrate computes at the moment it acts, never a stale dialog's.
#[test]
fn an_agent_delete_spawns_the_litany_verb_and_arms_children_by_name() {
    let w = world();
    let (_c, m) = w.h.model();
    let deps = w.deps(&m);
    let fire = |typed: &str| {
        let reply = engine::act(
            &m,
            &deps,
            "TS",
            &Action::DeleteAgent {
                workspace: "cobalt".to_owned(),
                agent: "c-1".to_owned(),
                typed: typed.to_owned(),
            },
        )
        .expect("the agent delete answers a captured run");
        let Reply::Outcome(outcome) = reply else {
            panic!("the agent delete answers a captured run");
        };
        outcome.stdout
    };
    let bare = fire("");
    assert!(bare.contains("delete"), "{bare}");
    assert!(
        !bare.contains("--children"),
        "unarmed is the bare verb, and litany declines a subtree nobody \
         confirmed: {bare}"
    );
    // The arming is the conversation's own §3.3 DISPLAY name — what a seat
    // shows and an operator retypes — never its id.
    let armed = fire("hi");
    assert!(armed.contains("--children"), "{armed}");
}
