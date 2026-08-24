//! The shared fixtures the sibling test modules build snapshots from. The
//! boundary's own table — [`Action::project`] — is [`project`], split off at
//! §12's cap on the seam this file's own summary already named: a fixture is
//! read by every sibling, a table is read by none.

/// What a gesture addresses (REMOTE §8, bl-f5f6): the workspace table and the
/// query one, over the whole roster.
mod address;
/// Which gestures name a project (§3.5) and which name none.
mod project;
/// The §8.1 start family driven the way a terminal must drive it — two real
/// `yog gesture` invocations, the second carrying the first's reply (bl-44d8).
mod start_terminal;

use super::*;
use crate::app::Snapshot;
use crate::binding::{Workspace, WorkspaceKind};
use crate::git_tree::{Agent, AgentState, GitTree};
use crate::projects::join::{JoinRow, JoinState};
use std::collections::HashMap;
use std::path::Path;

/// One agent row, the conversation-list fixture shape (§2.3).
pub(crate) fn agent(id: &str, state: AgentState, ts: i64) -> Agent {
    Agent {
        branch_name: format!("agents/{id}"),
        agent_id: id.to_string(),
        tip_oid: "a".repeat(40),
        tip_short_oid: "aaaaaaaa".into(),
        tip_timestamp_unix: ts,
        last_action_unix: ts,
        messages: 0,
        steps: vec![],
        preview: None,
        stream: crate::git_tree::Stream::default(),
        tool_calls: vec![],
        state,
        state_uncertain: false,
        truncated: false,
        pending: vec![],
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

/// A snapshot around one named workspace with `agents`, plus `join_rows` —
/// the general fixture every answer/dispatch table reads.
pub(crate) fn snapshot(ws: &Path, name: &str, agents: Vec<Agent>, join: Vec<JoinRow>) -> Snapshot {
    let mut trees = HashMap::new();
    trees.insert(
        ws.to_path_buf(),
        GitTree {
            commits: vec![],
            agents,
        },
    );
    // The §5.1 #1 naming set is the caller's: a join row says a project *name*
    // since bl-b4b5, and only the fixture that minted the row knows which
    // directory that name was derived over. Empty is the ordinary case — most
    // tables here never resolve one.
    let projects: Vec<std::path::PathBuf> = Vec::new();
    Snapshot {
        workspaces: vec![Workspace {
            path: ws.to_path_buf(),
            kind: WorkspaceKind::Named {
                name: name.to_owned(),
            },
        }],
        // The enumerated project set a name resolves over (REMOTE §8): whatever
        // this fixture's join rows name, which is what its gestures address.
        projects,
        trees,
        bills: HashMap::new(),
        windows: std::collections::BTreeMap::default(),
        balls_by_project: HashMap::new(),
        closed_by_project: HashMap::new(),
        join_rows: join,
        ops: vec![],
        growth: vec![],
        ui_bytes: None,
        derived_at_unix: 0,
        cadence: crate::app::Cadence::default(),
        fleet: std::collections::BTreeMap::new(),
    }
}

/// A join row bound to `ws` — the §3.5 fixture cell.
pub(crate) fn bound_row(project: &Path, id: &str, ws: &Path, claimant: &str) -> JoinRow {
    JoinRow {
        project: crate::naming::leaf(project),
        ball_id: id.to_owned(),
        state: JoinState::Bound,
        workspace: Some(crate::naming::leaf(ws)),
        claimant: Some(claimant.to_owned()),
        title: Some(format!("title of {id}")),
    }
}
