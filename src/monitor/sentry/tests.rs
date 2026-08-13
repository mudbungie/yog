//! The level trigger's whole contract, driven by hand: unarmed costs nothing,
//! a moved tip costs exactly one check, an unmoved tip costs none, and a failed
//! check leaves the tip unchecked so the next tick re-fires.

use super::*;
use crate::app::Snapshot;
use crate::binding::{Workspace, WorkspaceKind};
use crate::git_tree::{AgentState, GitTree};
use crate::monitor::{Called, Verdict};
use crate::ui_state::SystemClock;
use std::sync::Mutex;
use std::time::Instant;
use tempfile::tempdir;

/// A caller that counts its calls and answers a canned `bz` stdout.
struct Fake {
    stdout: String,
    exit: i32,
    calls: Arc<Mutex<usize>>,
}

impl Caller for Fake {
    fn call(&self, _: &std::path::Path, _: Vec<String>) -> Called {
        *self
            .calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) += 1;
        Called {
            exit: self.exit,
            stdout: self.stdout.clone(),
            stderr: "nope".to_owned(),
        }
    }
}

fn said(verdict: &str) -> String {
    format!(
        "{}\n{{\"type\":\"end\"}}\n",
        serde_json::json!({
            "type": "content_delta", "index": 0,
            "delta": {"text_delta": format!("{verdict}: because")},
        })
    )
}

fn agent(id: &str, tip: &str) -> crate::git_tree::Agent {
    crate::git_tree::Agent {
        branch_name: format!("agents/{id}"),
        agent_id: id.to_owned(),
        tip_oid: tip.to_owned(),
        tip_short_oid: tip.chars().take(8).collect(),
        tip_timestamp_unix: 1,
        last_action_unix: 1,
        messages: 0,
        steps: Vec::new(),
        preview: None,
        stream: crate::git_tree::Stream::default(),
        tool_calls: Vec::new(),
        state: AgentState::Quiescent,
        state_uncertain: false,
        pending: Vec::new(),
        conflicted_oid: None,
        budget_oid: None,
        abandoned_oid: None,
        notify_oid: None,
        held: None,
        goal_ball: None,
        name: None,
        goal_name: None,
        call_start_unix: None,
    }
}

/// A world with one workspace holding one agent at `tip`, published.
fn published(ws: &std::path::Path, tip: &str) -> SnapshotCell {
    let mut snap = Snapshot::empty(Instant::now());
    snap.workspaces = vec![Workspace {
        path: ws.to_path_buf(),
        kind: WorkspaceKind::Named {
            name: "otter".to_owned(),
        },
    }];
    snap.trees.insert(
        ws.to_path_buf(),
        GitTree {
            commits: Vec::new(),
            agents: vec![agent("a-1", tip)],
        },
    );
    crate::state::new_snapshot_cell(Arc::new(snap))
}

struct Rig {
    ctx: SentryCtx,
    calls: Arc<Mutex<usize>>,
    state_root: PathBuf,
    ws: PathBuf,
}

impl Rig {
    fn calls(&self) -> usize {
        *self
            .calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn checks(&self) -> Vec<row::Check> {
        row::of_entries(&opslog::tail(&self.state_root, 100))
    }
}

/// A rig with the workspace armed and its policy seeded, unless `armed` is off.
fn rig(root: &std::path::Path, tip: &str, verdict: &str, exit: i32, armed: bool) -> Rig {
    let (state_root, ws) = (root.join("state"), root.join("ws"));
    std::fs::create_dir_all(&state_root).expect("state root");
    if armed {
        let text = arming::arm("", &crate::nav::ws_key(&ws), "haiku").expect("armable");
        std::fs::write(state_root.join(CADENCE_YAML), text).expect("settings");
        std::fs::write(state_root.join(arming::PROMPT_FILE), arming::TEMPLATE).expect("policy");
    }
    let calls = Arc::new(Mutex::new(0));
    Rig {
        ctx: SentryCtx {
            state_root: state_root.clone(),
            cell: published(&ws, tip),
            clock: Arc::new(SystemClock),
            caller: Box::new(Fake {
                stdout: said(verdict),
                exit,
                calls: Arc::clone(&calls),
            }),
        },
        calls,
        state_root,
        ws,
    }
}

#[test]
fn unarmed_there_is_no_monitor_at_all() {
    let root = tempdir().expect("tempdir");
    let rig = rig(root.path(), "tip1", "aligned", 0, false);
    assert!(!rig.ctx.pass(), "nothing to check");
    assert_eq!(rig.calls(), 0, "no call, no row, no cost");
    assert!(rig.checks().is_empty());
    assert_eq!(
        rig.ctx.period(),
        crate::app::Cadence::default().full_sweep,
        "and the tick is the clock's own slowest period"
    );
}

#[test]
fn an_armed_workspace_with_no_policy_file_is_not_armed() {
    let root = tempdir().expect("tempdir");
    let rig = rig(root.path(), "tip1", "aligned", 0, true);
    std::fs::write(rig.state_root.join(arming::PROMPT_FILE), "   \n").expect("emptied");
    assert!(!rig.ctx.pass(), "the policy is the mechanism");
    assert_eq!(rig.calls(), 0);
}

#[test]
fn a_moved_tip_is_checked_once_and_then_never_again() {
    let root = tempdir().expect("tempdir");
    let rig = rig(root.path(), "tip1", "diverged", 0, true);
    assert!(rig.ctx.pass(), "the tip has never been checked");
    assert_eq!(rig.calls(), 1);
    let checks = rig.checks();
    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].verdict, Verdict::Diverged);
    assert_eq!(checks[0].sha, "tip1");
    assert_eq!(checks[0].workspace, crate::nav::ws_key(&rig.ws));
    // Level-triggered: the tip has not moved, so the next tick does nothing.
    assert!(!rig.ctx.pass(), "an unmoved tip is not due");
    assert_eq!(rig.calls(), 1, "one checkpoint, one call");
}

#[test]
fn a_failed_check_leaves_the_tip_unchecked_so_the_next_tick_re_fires() {
    let root = tempdir().expect("tempdir");
    let rig = rig(root.path(), "tip1", "aligned", 70, true);
    assert!(rig.ctx.pass());
    assert!(
        rig.checks().is_empty(),
        "a failure is not a verdict and names no sha"
    );
    let tail = opslog::tail(&rig.state_root, 10);
    assert!(
        tail.iter().any(
            |e| e.argv.first().map(String::as_str) == Some(crate::opslog::YOG_STEP)
                && e.stderr.contains("a-1")
        ),
        "but it is audited: {tail:?}"
    );
    assert!(
        rig.ctx.pass(),
        "so the next tick re-fires — that IS the retry"
    );
    assert_eq!(rig.calls(), 2);
}

/// A workspace armed under a key the published world does not hold, and a
/// workspace the world holds with no derived tree: neither is a check.
#[test]
fn an_armed_key_with_nothing_behind_it_checks_nothing() {
    let root = tempdir().expect("tempdir");
    let rig = rig(root.path(), "tip1", "aligned", 0, true);
    let text = arming::arm("", "/nowhere", "haiku").expect("armable");
    std::fs::write(rig.state_root.join(CADENCE_YAML), text).expect("settings");
    assert!(!rig.ctx.pass());
    assert_eq!(rig.calls(), 0);
}

/// An armed workspace whose tree failed to derive has no agents to be due:
/// the pass skips it rather than guessing at a world it could not read.
#[test]
fn an_armed_workspace_with_no_derived_tree_checks_nothing() {
    let root = tempdir().expect("tempdir");
    let rig = rig(root.path(), "tip1", "aligned", 0, true);
    let mut snap = Snapshot::empty(Instant::now());
    snap.workspaces = vec![Workspace {
        path: rig.ws.clone(),
        kind: WorkspaceKind::Named {
            name: "otter".to_owned(),
        },
    }];
    crate::state::publish_snapshot(&rig.ctx.cell, Arc::new(snap));
    assert!(!rig.ctx.pass());
    assert_eq!(rig.calls(), 0);
}

/// The one thing a hand-driven pass cannot prove: the thread runs it and stops.
#[test]
fn the_thread_ticks_and_stops() {
    let root = tempdir().expect("tempdir");
    let rig = rig(root.path(), "tip1", "aligned", 0, true);
    let calls = Arc::clone(&rig.calls);
    let sentry = Sentry::spawn(rig.ctx);
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if *calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            > 0
        {
            break;
        }
        std::thread::yield_now();
    }
    drop(sentry);
    assert_eq!(
        *calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        1,
        "one tick fired one check, and the drop joined the thread"
    );
}
