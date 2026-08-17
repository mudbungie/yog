//! The fixture the board's tables are built from: one project, balls across
//! the four columns, workspaces holding claims, and a `steps/` fold already
//! walked. Split out of [`super`] at §12's cap — the seam is real, being the
//! difference between what the world *is* and what the board makes of it.

use super::*;
use crate::app::Snapshot;
use crate::binding::{Workspace, WorkspaceKind};
use crate::budgets::{BudgetSpend, StepBill};
use crate::git_tree::{Agent, AgentState, GitTree};
use crate::projects::balls::{Ball, Blocker};
use crate::projects::join::{JoinRow, JoinState};
use std::collections::HashMap;
use std::path::PathBuf;

pub(super) const PROJECT: &str = "/proj";
pub(super) const WS_A: &str = "/ws/alfa";
pub(super) const WS_B: &str = "/ws/bravo";

pub(super) fn ball(id: &str, claimant: Option<&str>, blockers: Vec<Blocker>) -> Ball {
    Ball {
        id: id.to_owned(),
        title: format!("{id} title"),
        body: String::new(),
        claimant: claimant.map(str::to_owned),
        blockers,
        parent: None,
        priority: 2,
        tags: vec![],
        created: Some(0),
        updated: Some(0),
        root_commit: None,
    }
}

pub(super) fn blocks(id: &str, on: &str) -> Blocker {
    Blocker {
        id: id.to_owned(),
        on: on.to_owned(),
    }
}

pub(super) fn join(
    id: &str,
    state: JoinState,
    ws: Option<&str>,
    claimant: Option<&str>,
) -> JoinRow {
    JoinRow {
        project: crate::naming::leaf(Path::new(PROJECT)),
        ball_id: id.to_owned(),
        state,
        workspace: ws.map(|w| crate::naming::leaf(Path::new(w))),
        claimant: claimant.map(str::to_owned),
        title: Some(format!("{id} title")),
    }
}

pub(super) fn agent(id: &str, ball: Option<&str>, name: Option<&str>) -> Agent {
    Agent {
        goal_ball: ball.map(str::to_owned),
        name: name.map(str::to_owned),
        ..crate::boundary::tests::agent(id, AgentState::Quiescent, 0)
    }
}

pub(super) fn bill(conv: &str, input: u64) -> StepBill {
    StepBill {
        conv: conv.to_owned(),
        seq: "001".to_owned(),
        model: Some("opus".to_owned()),
        spend: BudgetSpend {
            input_tokens: input,
            ..BudgetSpend::default()
        },
        last_usage: BudgetSpend::default(),
        wall_secs: 0,
    }
}

/// The §4.1 durable a board is built against: the price table above, and no
/// ceiling — the ungated default every world starts in.
pub(super) fn ui() -> crate::ui_state::UiState {
    ui_doc(r#"{"v":1,"prices":{"opus":{"input":1}}}"#)
}

/// The same, with whatever `ui.json` a test needs.
pub(super) fn ui_doc(doc: &str) -> crate::ui_state::UiState {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("ui.json");
    std::fs::write(&path, doc).expect("write");
    crate::ui_state::UiState::open(path)
}

/// The wall clock every board fixture is dated against.
pub(super) const NOW: i64 = 1_000_000;

/// The whole fixture: one project, four balls across the four columns, two
/// workspaces holding claims, and a `steps/` fold already walked.
pub(super) struct World {
    pub(super) snap: Snapshot,
}

impl World {
    pub(super) fn board(&self) -> Board {
        build(&self.snap, &ui(), NOW)
    }
}

pub(super) fn world(
    balls: Vec<Ball>,
    joins: Vec<JoinRow>,
    agents: Vec<(&str, Vec<Agent>)>,
) -> World {
    let mut trees = HashMap::new();
    let mut bills = HashMap::new();
    for (ws, members) in agents {
        bills.insert(
            PathBuf::from(ws),
            members
                .iter()
                .map(|a| bill(&a.agent_id, 1_000_000))
                .collect(),
        );
        trees.insert(
            PathBuf::from(ws),
            GitTree {
                commits: vec![],
                agents: members,
            },
        );
    }
    let mut balls_by_project = HashMap::new();
    balls_by_project.insert(PathBuf::from(PROJECT), balls);
    World {
        snap: Snapshot {
            windows: std::collections::BTreeMap::default(),
            // Both walls are enumerated: a join row addresses its workspace
            // by §3.1 name since bl-b4b5, and the board resolves that name back
            // through the snapshot's own round trip.
            workspaces: [WS_A, WS_B]
                .into_iter()
                .map(|path| Workspace {
                    path: PathBuf::from(path),
                    kind: WorkspaceKind::Named {
                        name: crate::naming::leaf(Path::new(path)),
                    },
                })
                .collect(),
            projects: vec![PathBuf::from(PROJECT)],
            trees,
            bills,
            balls_by_project,
            closed_by_project: HashMap::new(),
            join_rows: joins,
            ops: vec![],
            growth: vec![],
            ui_bytes: None,
            derived_at_unix: 0,
            cadence: crate::app::Cadence::default(),
            fleet: std::collections::BTreeMap::new(),
        },
    }
}
