//! Boundary-level tables ([`Action::project`]) and the shared fixtures the
//! sibling test modules build snapshots from.

/// What a gesture addresses (REMOTE §8, bl-f5f6): the workspace table and the
/// query one, over the whole roster.
mod address;
/// The §8.1 start family driven the way a terminal must drive it — two real
/// `yog gesture` invocations, the second carrying the first's reply (bl-44d8).
mod start_terminal;

use super::*;
use crate::app::Snapshot;
use crate::binding::{Workspace, WorkspaceKind};
use crate::git_tree::{Agent, AgentState, GitTree};
use crate::projects::join::{JoinRow, JoinState};
use crate::start::{BallSpec, Payload, Prepared};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

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
    let mut projects: Vec<std::path::PathBuf> = join.iter().map(|r| r.project.clone()).collect();
    projects.sort();
    projects.dedup();
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
        derived_at: Instant::now(),
        cadence: crate::app::Cadence::default(),
        fleet: std::collections::BTreeMap::new(),
    }
}

/// A join row bound to `ws` — the §3.5 fixture cell.
pub(crate) fn bound_row(project: &Path, id: &str, ws: &Path, claimant: &str) -> JoinRow {
    JoinRow {
        project: project.to_path_buf(),
        ball_id: id.to_owned(),
        state: JoinState::Bound,
        workspace: Some(ws.to_path_buf()),
        claimant: Some(claimant.to_owned()),
        title: Some(format!("title of {id}")),
    }
}

#[test]
fn the_bl_family_names_its_project_and_nothing_else_does() {
    let p = "p".to_owned();
    let ws = "ws".to_owned();
    let bl_family = [
        Action::Close {
            project: p.clone(),
            id: "b-1".into(),
            name: "n".into(),
        },
        Action::Assign {
            project: p.clone(),
            id: "b-1".into(),
            name: "n".into(),
        },
        Action::Release {
            project: p.clone(),
            id: "b-1".into(),
            name: "n".into(),
        },
        Action::Move {
            project: p.clone(),
            id: "b-1".into(),
            from: "a".into(),
            to: "b".into(),
        },
        Action::Create {
            project: p.clone(),
            title: "t".into(),
            name: "n".into(),
            body: None,
        },
        Action::Update {
            project: p.clone(),
            id: "b-1".into(),
            name: "n".into(),
            title: None,
            body: None,
            note: None,
        },
    ];
    for action in bl_family {
        assert_eq!(action.project(), Some(p.clone()), "{action:?}");
    }
    let lernie_family = [
        Action::Message {
            workspace: ws.clone(),
            agent: "c".into(),
            content: "hi".into(),
        },
        Action::Stop {
            workspace: ws.clone(),
            agent: "c".into(),
            children: false,
        },
        Action::Scan {
            workspace: ws.clone(),
        },
        Action::Retarget {
            workspace: ws.clone(),
            agent: "c".into(),
        },
        Action::DeleteWorkspace {
            workspace: ws.clone(),
            typed: "n".into(),
        },
        Action::DeleteAgent {
            workspace: ws.clone(),
            agent: "c".into(),
            typed: "n".into(),
        },
        Action::Fork {
            workspace: ws.clone(),
            parent: "c".into(),
            attempt: crate::fork::Attempt::default(),
            goal: "g".into(),
        },
        Action::Ack,
        Action::MarkSeen {
            workspace: ws.clone(),
            agent: "c".into(),
        },
        Action::ClearTrail,
    ];
    for action in lernie_family {
        assert_eq!(action.project(), None, "{action:?}");
    }
}

#[test]
fn a_ball_rung_prepare_carries_its_project_and_the_other_rungs_none() {
    let p = "proj".to_owned();
    let ball = Action::Prepare {
        workspace: "ws".to_owned(),
        payload: Payload::Ball {
            project: p.clone(),
            ball: BallSpec::New {
                title: "t".into(),
                body: String::new(),
            },
        },
    };
    assert_eq!(ball.project(), Some(p.clone()));
    for payload in [
        Payload::Bare,
        Payload::Path {
            dir: Path::new("/d").to_path_buf(),
        },
    ] {
        let a = Action::Prepare {
            workspace: "ws".to_owned(),
            payload,
        };
        assert_eq!(a.project(), None, "{a:?}");
    }
    let prompt = Action::Prompt {
        prepared: Prepared {
            workspace: "ws".into(),
            binding: None,
            goal: "g".into(),
            origin: crate::opslog::Origin::Conversation,
        },
        goal: "g".into(),
    };
    assert_eq!(prompt.project(), None);
}

/// The §4.10 fan's two act in a project's refs rather than on its board, and
/// the §3.5 projection reads that project — so they name it too.
#[test]
fn the_fan_family_names_its_project() {
    let p = "p".to_owned();
    let obligation = crate::fan::Obligation {
        project: p.clone(),
        ball: Some("b-1".into()),
    };
    for action in [
        Action::Fan {
            prepared: Prepared {
                workspace: "ws".into(),
                binding: None,
                goal: "g".into(),
                origin: crate::opslog::Origin::Balls,
            },
            obligation: obligation.clone(),
            n: 3,
        },
        Action::Retire {
            obligation,
            handle: "at-0badcafe".into(),
        },
    ] {
        assert_eq!(action.project(), Some(p.clone()), "{action:?}");
    }
}
