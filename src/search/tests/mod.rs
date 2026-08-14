//! The §8.5 search derivation, end to end: the corpus it spans, the order it
//! ranks in, the bytes it re-reads, the gaps it names, and the abandonment a
//! superseded ask earns.

mod cell;
mod corpus;
mod rank;
mod render;

use super::*;
use crate::app::{Cadence, Snapshot};
use crate::binding::{Workspace, WorkspaceKind};
use crate::git_tree::{Agent, AgentState, GitTree};
use crate::projects::balls::Ball;
use crate::projects::join::{JoinRow, JoinState};
use crate::state::SearchCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tempfile::tempdir;

const AGENT: &str = "20260427T120000Z-aaaa";

fn ball(id: &str, title: &str, body: &str) -> Ball {
    Ball {
        id: id.to_owned(),
        title: title.to_owned(),
        body: body.to_owned(),
        claimant: None,
        blockers: vec![],
        parent: None,
        priority: 3,
        tags: vec![],
        created: None,
        updated: None,
        root_commit: None,
    }
}

fn agent(id: &str, name: Option<&str>) -> Agent {
    Agent {
        branch_name: format!("agents/{id}"),
        agent_id: id.to_owned(),
        tip_oid: "a".repeat(40),
        tip_short_oid: "aaaaaaaa".to_owned(),
        tip_timestamp_unix: 100,
        last_action_unix: 100,
        messages: 0,
        call_start_unix: None,
        steps: vec![],
        preview: None,
        stream: crate::git_tree::Stream::default(),
        tool_calls: vec![],
        state: AgentState::Quiescent,
        state_uncertain: false,
        pending: vec![],
        conflicted_oid: None,
        budget_oid: None,
        abandoned_oid: None,
        notify_oid: None,
        held: None,
        goal_ball: None,
        name: name.map(str::to_owned),
        goal_name: None,
    }
}

/// A world: one named workspace with `agents` derived, and one project with
/// `live`/`closed` ball sets.
fn world(ws: &Path, agents: Vec<Agent>, live: Vec<Ball>, closed: Vec<Ball>) -> Snapshot {
    let project = PathBuf::from("/proj");
    let mut trees = HashMap::new();
    trees.insert(
        ws.to_path_buf(),
        GitTree {
            commits: vec![],
            agents,
        },
    );
    Snapshot {
        workspaces: vec![Workspace {
            path: ws.to_path_buf(),
            kind: WorkspaceKind::Named {
                name: crate::naming::leaf(ws),
            },
        }],
        projects: vec![project.clone()],
        trees,
        bills: HashMap::new(),
        windows: std::collections::BTreeMap::default(),
        balls_by_project: HashMap::from([(project.clone(), live)]),
        closed_by_project: HashMap::from([(project, closed)]),
        join_rows: vec![],
        ops: vec![],
        growth: vec![],
        ui_bytes: None,
        derived_at: Instant::now(),
        cadence: Cadence::default(),
        fleet: std::collections::BTreeMap::new(),
    }
}

fn always() -> impl Fn() -> bool {
    || true
}

fn write(path: &Path, bytes: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}
