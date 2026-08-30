//! What one tick **does**: the thread it runs on, the moves it fires through
//! the ordinary boundary doors, and the one row each leaves behind.
//!
//! Split from [`super`] at §12's cap, on a real seam — that file is the
//! decision (pure, table-shaped), this one is the effect (fake substrate,
//! real spawns, the trail read back). Cut once more at the same cap along the
//! loop's own two moves: the reap and the thread are here, and [`birth`] —
//! taking work — is beside it.

use super::super::*;
use crate::app::Snapshot;
use crate::boundary::dispatch::Deps;
use crate::boundary::tests::agent;
use crate::cli_outbound::Cli;
use crate::projects::join::JoinState;
use crate::ui_state::SystemClock;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

const NOW: i64 = 1_000_000;
const WS: &str = "/names/otter";
const PROJECT: &str = "/dev/yog";

fn ctx(state_root: &Path, snapshot: Snapshot) -> PilotCtx {
    let cell = crate::state::new_snapshot_cell(Arc::new(snapshot));
    PilotCtx {
        deps: Deps {
            litany: Cli::new("/no/such/litany"),
            bl: Cli::new("/no/such/bl"),
            state_root: state_root.to_path_buf(),
            yog_binary: PathBuf::from("/no/such/yog"),
            world: crate::test_support::no_world(),
            home: state_root.join("home"),
            yog_data_root: state_root.join("data"),
            balls_state_root: state_root.join("balls"),
            snapshot: Arc::new(Snapshot::empty(0)),
            caller: crate::boundary::dispatch::Caller::default(),
        },
        cell,
        clock: Arc::new(SystemClock),
        ui_path: state_root.join("ui.json"),
    }
}

/// **The burden check, mechanically.** An unarmed world's tick does nothing and
/// leaves nothing — no row, no file, no call — which is what makes deleting the
/// `cadence.yaml` entry delete the loop rather than a code path.
#[test]
fn an_unarmed_tick_acts_on_nothing_and_writes_nothing() {
    let root = tempdir().expect("tempdir");
    let ctx = ctx(root.path(), Snapshot::empty(0));
    assert!(!ctx.pass());
    assert!(
        !root.path().join("ops.jsonl").exists(),
        "an unarmed loop leaves no trace at all"
    );
    assert_eq!(ctx.period(), crate::app::Cadence::default().full_sweep);
}

/// An armed loop with nothing to do is equally quiet: the arm is the mechanism,
/// not the noise.
#[test]
fn an_armed_tick_with_no_work_does_nothing() {
    let root = tempdir().expect("tempdir");
    let mut snapshot = Snapshot::empty(0);
    snapshot.fleet.insert(
        WS.to_owned(),
        crate::fleet::Policy {
            project: PathBuf::from(PROJECT),
            cap: 2,
            lease: None,
        },
    );
    let ctx = ctx(root.path(), snapshot);
    assert!(!ctx.pass(), "no ready ball, no claim: nothing to do");
    assert!(!root.path().join("ops.jsonl").exists());
}

#[test]
fn the_thread_ticks_and_stops_on_drop() {
    let root = tempdir().expect("tempdir");
    let pilot = Pilot::spawn(ctx(root.path(), Snapshot::empty(0)));
    drop(pilot); // joins cleanly — the Drop is the shutdown
}

/// A world whose board carries exactly one row: `ball` in `project`, claimed by
/// `ws`'s name with one quiet conversation on it when `drone` says so, or ready
/// and unclaimed when it does not.
fn armed_world(ws: &Path, project: &Path, claimed: bool, lease: Option<Duration>) -> Snapshot {
    let name = "otter";
    let mut agents = Vec::new();
    if claimed {
        let mut drone = agent("root-1", crate::git_tree::AgentState::Stopped, NOW - 3600);
        drone.goal_ball = Some("bl-1".to_owned());
        agents.push(drone);
    }
    let join = crate::projects::join::JoinRow {
        project: crate::naming::leaf(project),
        ball_id: "bl-1".to_owned(),
        state: if claimed {
            JoinState::Bound
        } else {
            JoinState::ReadyStartable
        },
        workspace: claimed.then(|| crate::naming::leaf(ws)),
        claimant: claimed.then(|| name.to_owned()),
        title: Some("the one ball".to_owned()),
    };
    let ball = crate::projects::balls::Ball {
        id: "bl-1".to_owned(),
        title: "the one ball".to_owned(),
        body: "do the thing".to_owned(),
        claimant: claimed.then(|| name.to_owned()),
        blockers: vec![],
        parent: None,
        priority: 2,
        tags: vec![],
        created: Some(0),
        updated: Some(0),
        root_commit: None,
    };
    let mut snap = crate::boundary::tests::snapshot(ws, name, agents, vec![join]);
    // The §5.1 #1 naming set the row's project name was derived over — the
    // round trip the board and the start flow both spend (bl-b4b5).
    snap.projects = vec![project.to_path_buf()];
    snap.balls_by_project
        .insert(project.to_path_buf(), vec![ball]);
    snap.fleet.insert(
        crate::nav::ws_key(ws),
        crate::fleet::Policy {
            project: project.to_path_buf(),
            cap: 1,
            lease,
        },
    );
    snap
}

/// Write `body` as an executable `name` in `dir` and hand back its [`Cli`].
fn fake(dir: &Path, name: &str, body: &str) -> Cli {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write");
    let mut perms = std::fs::metadata(&path).expect("stat").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).expect("chmod");
    Cli::new(path)
}

/// A reap that lands leaves exactly one row, and the row's reason is the
/// comparison — the whole durable the loop keeps.
#[test]
fn a_landed_reap_releases_the_claim_and_leaves_one_row() {
    let root = tempdir().expect("tempdir");
    let project = root.path().join("proj");
    let ws = root.path().join("ws");
    std::fs::create_dir_all(&project).expect("mkdir");
    let bl = fake(root.path(), "bl", "#!/bin/sh\nexit 0\n");
    let mut ctx = ctx(
        root.path(),
        armed_world(&ws, &project, true, Some(Duration::from_mins(30))),
    );
    ctx.deps.bl = bl;
    assert!(ctx.pass(), "the ball is an hour quiet against a 30m lease");
    let trail = std::fs::read_to_string(root.path().join("ops.jsonl")).expect("trail");
    assert!(trail.contains("yog-fleet"), "{trail}");
    assert!(trail.contains("reap"), "{trail}");
    assert!(
        trail.contains("lease expired"),
        "the reason is the comparison: {trail}"
    );
}

/// A reap the substrate refuses leaves **no** loop row: the failure is already
/// on the trail as `bl`'s own, and the next tick simply decides again.
#[test]
fn a_refused_reap_writes_no_loop_row() {
    let root = tempdir().expect("tempdir");
    let project = root.path().join("proj");
    let ws = root.path().join("ws");
    std::fs::create_dir_all(&project).expect("mkdir");
    let bl = fake(
        root.path(),
        "bl",
        "#!/bin/sh\nprintf 'boom\\n' 1>&2\nexit 3\n",
    );
    let mut ctx = ctx(
        root.path(),
        armed_world(&ws, &project, true, Some(Duration::from_mins(30))),
    );
    ctx.deps.bl = bl;
    assert!(!ctx.pass());
    let trail = std::fs::read_to_string(root.path().join("ops.jsonl")).unwrap_or_default();
    assert!(
        !trail.contains("yog-fleet"),
        "the executor's own failure row is the record: {trail}"
    );
}

mod birth;
