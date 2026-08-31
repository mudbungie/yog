//! The rig one tick is driven against: a `bz` caller that counts its calls and
//! answers a canned stdout, an agent at a tip, the published world holding it,
//! and the armed state root its policy is seeded into. Split from the contract
//! at §12's budget on the seam between *the world a tick reads* and *what the
//! level trigger does in it* — every beat in this corpus builds the same rig.

use super::super::{CADENCE_YAML, Caller, SentryCtx, arming, row};
use crate::app::Snapshot;
use crate::binding::{Workspace, WorkspaceKind};
use crate::git_tree::{AgentState, GitTree};
use crate::monitor::Called;
use crate::opslog;
use crate::state::SnapshotCell;
use crate::ui_state::SystemClock;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
        truncated: false,
        refused: false,
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
    let mut snap = Snapshot::empty(0);
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

pub(super) struct Rig {
    pub(super) ctx: SentryCtx,
    pub(super) calls: Arc<Mutex<usize>>,
    pub(super) state_root: PathBuf,
    pub(super) ws: PathBuf,
}

impl Rig {
    pub(super) fn calls(&self) -> usize {
        *self
            .calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(super) fn checks(&self) -> Vec<row::Check> {
        row::of_entries(&opslog::tail(&self.state_root, 100))
    }
}

/// A rig with the workspace armed and its policy seeded, unless `armed` is off.
pub(super) fn rig(root: &std::path::Path, tip: &str, verdict: &str, exit: i32, armed: bool) -> Rig {
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
