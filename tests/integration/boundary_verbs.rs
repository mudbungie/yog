//! The §8.5 action chokepoint end-to-end: every short-verb [`Action`] through
//! `boundary::dispatch` spawns its exact §8.2 argv and leaves its ops row —
//! the same table `stories_s1_t3` pinned on the pre-boundary dispatchers —
//! and one deposit consumed headlessly is the same spawn plus its reply file
//! (VISION §4.8: one surface, two serializations).

#![allow(clippy::unwrap_used)]

/// The `bl` family's own beats, split off at §12's cap (bl-49bc's line budget)
/// on the seam `codec/balls.rs` already draws.
mod balls;

use crate::support::Recorder;
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;
use yog::app::Snapshot;
use yog::boundary::dispatch::{Deps, dispatch};
use yog::boundary::reply::Reply;
use yog::boundary::{Action, consume, deposit};
use yog::cli_outbound::Cli;
use yog::opslog;
use yog::ui_state::UiState;

/// The conversation every gesture here addresses — **id-shaped** (ARCH §2.3's
/// stamp), which is what the §8.5 conversation resolution reads as an id on its
/// own, with no enumeration behind it (bl-49bc).
const AGENT: &str = "20260101T000000Z-c1";

/// The enumerated sets a gesture's workspace/project NAME resolves against
/// (REMOTE §8, bl-f5f6): the wire carries no paths, so a fixture publishes the
/// spheres and repos it acts on exactly as the worker publishes what it found.
fn snapshot_of(workspaces: &[&Path], projects: &[&Path]) -> Arc<Snapshot> {
    Arc::new(Snapshot {
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
        projects: projects.iter().map(|p| (*p).to_path_buf()).collect(),
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
    })
}

fn deps(lernie: &Cli, bl: &Cli, state_root: &Path, snapshot: Arc<Snapshot>) -> Deps {
    Deps {
        lernie: lernie.clone(),
        bl: bl.clone(),
        state_root: state_root.to_path_buf(),
        yog_binary: state_root.join("yog"),
        world: yog::world::compose(&yog::xdg::Env::from_env()),
        home: state_root.join("home"),
        yog_data_root: state_root.join("data"),
        balls_state_root: state_root.join("balls"),
        snapshot,
        caller: yog::boundary::dispatch::Caller::default(),
    }
}

fn ui() -> UiState {
    UiState::open(std::path::PathBuf::from("/nonexistent/ui.json"))
}

/// The lernie-family variants spawn their §8.2 argv through the chokepoint.
#[test]
fn the_lernie_actions_spawn_their_exact_argv_and_ops_rows() {
    let bin = tempdir().unwrap();
    let state = tempdir().unwrap();
    let ws = tempdir().unwrap();
    let ws_s = ws.path().to_string_lossy().to_string();
    let rec = Recorder::new(bin.path(), "lernie");
    let lernie = Cli::new(rec.path());
    let d = deps(
        &lernie,
        &Cli::new("/no/bl"),
        state.path(),
        snapshot_of(&[ws.path()], &[]),
    );

    let actions = [
        Action::Message {
            workspace: yog::naming::leaf(ws.path()),
            agent: AGENT.into(),
            content: "ping".into(),
        },
        Action::Stop {
            workspace: yog::naming::leaf(ws.path()),
            agent: AGENT.into(),
            children: true,
        },
        Action::Scan {
            workspace: yog::naming::leaf(ws.path()),
        },
    ];
    for (i, action) in actions.iter().enumerate() {
        match dispatch(&d, &mut ui(), &format!("T{i}"), action).unwrap() {
            Reply::Outcome(outcome) => assert!(outcome.ok(), "{action:?}"),
            other => panic!("a verb answers an outcome, got {other:?}"),
        }
    }
    let argv: Vec<Vec<String>> = rec.invocations().into_iter().map(|i| i.argv).collect();
    assert_eq!(
        argv,
        vec![
            vec![
                "message".to_owned(),
                ws_s.clone(),
                AGENT.into(),
                "ping".into()
            ],
            vec![
                "stop".to_owned(),
                ws_s.clone(),
                AGENT.into(),
                "--stop-children".into()
            ],
            vec!["scan".to_owned(), ws_s.clone()],
        ],
        "the §8.2 argv, verbatim"
    );
    let ops = opslog::tail(state.path(), 8);
    assert_eq!(ops.len(), 3, "one ops row per spawn (§4.2)");
    assert!(ops.iter().all(|e| e.exit == 0));
}

/// One deposit consumed headlessly is the identical spawn plus its reply file
/// — the §8.5 transport round trip.
#[test]
fn a_deposited_message_converges_to_the_same_spawn_and_a_reply() {
    let bin = tempdir().unwrap();
    let state = tempdir().unwrap();
    let ws = tempdir().unwrap();
    let rec = Recorder::new(bin.path(), "lernie");
    let lernie = Cli::new(rec.path());
    let bl = Cli::new("/no/bl");
    let d = deps(&lernie, &bl, state.path(), snapshot_of(&[ws.path()], &[]));

    deposit::deposit(
        state.path(),
        "g-msg",
        &json!({
            "op": "message",
            "workspace": yog::naming::leaf(ws.path()),
            "agent": AGENT,
            "content": "from headless",
        }),
    )
    .unwrap();
    assert_eq!(consume::consume(&d, &mut ui(), "T9", 100), 1);

    let reply = deposit::read_reply(state.path(), "g-msg").unwrap();
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["kind"], "outcome");
    let inv = rec.invocations();
    assert_eq!(inv.len(), 1);
    assert_eq!(
        inv[0].argv,
        [
            "message",
            ws.path().to_string_lossy().as_ref(),
            AGENT,
            "from headless"
        ]
    );
    let ops = opslog::tail(state.path(), 8);
    assert_eq!(ops.len(), 1, "the audit: the deposit + this row (§8.5)");
}
