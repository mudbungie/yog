//! The §8.5 Prompt action end-to-end: the boundary's deferred detached fire
//! is `litany prompt --name <minted> <ws> <goal>` — the goal verbatim, bl-6920
//! (§8.1) — the minted name
//! rides back as the reply, and a fork that never lands is a refusal with its
//! §4.2 synthetic ops row — through the same chokepoint the GUI's Send uses.

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;
use yog::app::Snapshot;
use yog::boundary::Action;
use yog::boundary::dispatch::{Deps, dispatch};
use yog::boundary::reply::Reply;
use yog::cli_outbound::Cli;
use yog::opslog::{self, DETACHED_EXIT};
use yog::start::{Payload, Prepared};
use yog::ui_state::UiState;

/// `workspaces` is the enumerated set a gesture's workspace NAME resolves
/// against (REMOTE §8, bl-f5f6) — the wire carries no paths, so a fixture that
/// acts on a sphere must publish it exactly as the worker publishes what it
/// found on disk.
fn deps(litany: &Cli, state_root: &Path, workspaces: &[&Path]) -> Deps {
    Deps {
        litany: litany.clone(),
        bl: Cli::new("/no/bl"),
        state_root: state_root.to_path_buf(),
        yog_binary: state_root.join("yog"),
        world: yog::world::compose(&yog::xdg::Env::from_env()),
        home: state_root.join("home"),
        yog_data_root: state_root.join("data"),
        balls_state_root: state_root.join("balls"),
        snapshot: Arc::new(Snapshot {
            bills: HashMap::default(),
            windows: std::collections::BTreeMap::default(),
            workspaces: workspaces
                .iter()
                .map(|path| yog::binding::Workspace {
                    path: (*path).to_path_buf(),
                    kind: yog::binding::WorkspaceKind::Named {
                        name: yog::naming::leaf(path),
                    },
                })
                .collect(),
            projects: vec![],
            trees: HashMap::new(),
            balls_by_project: HashMap::new(),
            closed_by_project: HashMap::new(),
            join_rows: vec![],
            ops: vec![],
            growth: vec![],
            ui_bytes: None,
            derived_at_unix: 0,
            cadence: yog::app::Cadence::default(),
            fleet: std::collections::BTreeMap::new(),
        }),
        caller: yog::boundary::dispatch::Caller::default(),
    }
}

fn prepared(ws: &Path, cwd: &Path) -> Prepared {
    Prepared {
        workspace: yog::naming::leaf(ws),
        binding: Some(cwd.to_path_buf()),
        goal: "prefill".into(),
        origin: yog::opslog::Origin::Conversation,
        lineage: None,
    }
}

#[test]
fn the_prompt_action_fires_detached_and_returns_the_minted_name() {
    let bin = tempdir().unwrap();
    let state = tempdir().unwrap();
    let ws = tempdir().unwrap();
    // The door's *sign in first* rung (§8.1, bl-2291) stands ahead of the
    // fire; a lineage naming a keyless row is what lets a hermetic beat past.
    crate::support::keyless_wall(ws.path());
    let rec = Recorder::new(bin.path(), "litany");
    let litany = Cli::new(rec.path());
    let d = deps(&litany, state.path(), &[ws.path()]);

    let action = Action::Prompt {
        prepared: prepared(ws.path(), ws.path()),
        goal: "make me a plan".into(),
        // The seat's own §3.3 prediction, carried on the gesture (bl-1747).
        seed: Some(7),
    };
    let Reply::Started { conversation } = dispatch(
        &d,
        &mut UiState::open(state.path().join("ui.json")),
        "T0",
        &action,
    )
    .unwrap() else {
        panic!("a prompt answers the minted name");
    };

    let inv = rec.wait(1);
    assert_eq!(inv[0].argv[0], "prompt");
    assert_eq!(
        &inv[0].argv[1..3],
        ["--name", conversation.as_str()],
        "the minted name rides --name to its litany home (§3.3, bl-08f2)"
    );
    assert_eq!(
        &inv[0].argv[3..5],
        ["--cwd", ws.path().to_string_lossy().as_ref()],
        "the typed work target rides --cwd (§3.3, bl-6654)"
    );
    assert_eq!(inv[0].argv[5], ws.path().to_string_lossy());
    assert_eq!(
        inv[0].argv[6], "make me a plan",
        "the edited goal rode the wire verbatim, still last — no identity line (bl-6920)"
    );
    let ops = opslog::tail(state.path(), 8);
    assert_eq!(ops.len(), 1);
    assert_eq!(
        ops[0].exit, DETACHED_EXIT,
        "a handoff logs the -2 sentinel and nothing else (§4.2)"
    );
}

#[test]
fn a_fork_that_never_lands_is_a_refusal_with_its_synthetic_row() {
    let state = tempdir().unwrap();
    let ws = tempdir().unwrap();
    crate::support::keyless_wall(ws.path());
    let litany = Cli::new("/no/such/litany");
    let d = deps(&litany, state.path(), &[ws.path()]);
    let action = Action::Prompt {
        prepared: prepared(ws.path(), ws.path()),
        goal: "g".into(),
        seed: Some(7),
    };
    let err = dispatch(
        &d,
        &mut UiState::open(state.path().join("ui.json")),
        "T0",
        &action,
    )
    .unwrap_err();
    assert!(!err.is_empty());
    let ops = opslog::tail(state.path(), 8);
    assert_eq!(ops.len(), 1, "the never-launched spawn's §4.2 line");
    assert!(ops[0].exit != 0);
}

/// The dispatch match's Prepare arm is the same typed door the frame uses —
/// a world that cannot seed refuses through it, ops row and all (§8.1).
#[test]
fn a_prepare_that_cannot_seed_refuses_through_the_dispatch_arm() {
    let state = tempdir().unwrap();
    let ws = tempdir().unwrap();
    let litany = Cli::new("/no/such/litany");
    let d = deps(&litany, state.path(), &[ws.path()]);
    let action = Action::Prepare {
        workspace: yog::naming::leaf(ws.path()),
        payload: Payload::Bare,
    };
    let err = dispatch(
        &d,
        &mut UiState::open(state.path().join("ui.json")),
        "T0",
        &action,
    )
    .unwrap_err();
    assert!(!err.is_empty());
}
