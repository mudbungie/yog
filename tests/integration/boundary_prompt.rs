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
        world: yog::world::compose(&hermetic(state_root)),
        home: state_root.join("home"),
        yog_data_root: state_root.join("data"),
        snapshot: Arc::new(Snapshot {
            bills: HashMap::default(),
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

/// The fixture's world, built from two named variables and nothing of the
/// process's own (bl-2067). The `Prompt` door's *sign in first* rung reads
/// brazen's routing table through this snapshot, and brazen's ambient
/// credential discovery reads it too: a foreign tool's sign-in file is found
/// by expanding `~/` against the snapshot's `HOME`, and a key by the name of a
/// variable in it. Handed the process environment, the rung answered for the
/// box the suite ran on — a box holding a sign-in for a built-in preset got a
/// one-row table that does not carry the fixture's keyless row, and a refusal
/// where a box holding none got an empty table and a fire.
fn hermetic(state_root: &Path) -> yog::xdg::Env {
    yog::xdg::Env::of(named(state_root))
}

/// The two variables [`hermetic`] names, both inside the fixture.
fn named(state_root: &Path) -> Vec<(String, String)> {
    let under = |leaf: &str| state_root.join(leaf).to_string_lossy().into_owned();
    vec![
        ("HOME".to_owned(), under("home")),
        ("XDG_DATA_HOME".to_owned(), under("share")),
    ]
}

fn prepared(ws: &Path, cwd: &Path) -> Prepared {
    Prepared {
        workspace: yog::naming::leaf(ws),
        binding: Some(cwd.to_path_buf()),
        goal: "prefill".into(),
        origin: yog::opslog::Origin::Conversation,
        lineage: None,
        role: None,
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

/// **The door answers from the fixture's snapshot, in both directions**
/// (bl-2067). The two beats above pass because that snapshot names no
/// credential; this one names one — a built-in preset's own key variable — and
/// is refused exactly as they were on a box that held a sign-in: the table is
/// now that preset's rows, and the keyless row the role names is not among
/// them. So what the rung reads is what the fixture wrote, and never the box.
#[test]
fn a_credential_in_the_fixtures_own_snapshot_is_the_one_the_door_reads() {
    let state = tempdir().unwrap();
    let ws = tempdir().unwrap();
    crate::support::keyless_wall(ws.path());
    let mut d = deps(&Cli::new("/no/such/litany"), state.path(), &[ws.path()]);
    let mut vars = named(state.path());
    vars.push(("OPENAI_API_KEY".to_owned(), "notreal-fixture".to_owned()));
    d.world = yog::world::compose(&yog::xdg::Env::of(vars));
    let err = dispatch(
        &d,
        &mut UiState::open(state.path().join("ui.json")),
        "T0",
        &Action::Prompt {
            prepared: prepared(ws.path(), ws.path()),
            goal: "g".into(),
            seed: Some(7),
        },
    )
    .unwrap_err();
    assert!(
        err.contains("`ollama`") && err.contains("declares no such row"),
        "{err}"
    );
}
