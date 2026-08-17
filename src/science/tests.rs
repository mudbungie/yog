//! The §3.9 projection's tests, split by what each half asserts: [`join`] the
//! whole row against a real project repo and a real fan (every agent-side
//! column, and the diff column it composes), [`outcome`] the four arms over
//! their three git facts, and [`wire`] the spelling's own refusals.
//!
//! The project fixture is `workdiff`'s — a real balls-shaped project repo with
//! real attempts — because the projection *is* a join over those reads, and a
//! mocked one would test the mock.

mod join;
mod outcome;
mod wire;

use std::collections::HashMap;
use std::path::Path;

use crate::app::Snapshot;
use crate::budgets::{BudgetSpend, StepBill};
use crate::git_tree::{Agent, AgentState, GitTree};
use crate::opslog::OpEntry;
use crate::workdiff::tests::{Project, ball, snap as ws_snap, xdg};

pub(super) const NAME: &str = "lab";
pub(super) const BALL: &str = "bl-1";
/// The minted conversation name a fire carries, and the id its driver wrote.
pub(super) const CONV: &str = "otter-one";
pub(super) const AGENT: &str = "20260815T101112Z-abcd1234";

/// The claim row the obligation is read from, plus one fire row per binding —
/// each with a `--pin` so the frozen-input column has something to say.
pub(super) fn trail(ws: &Path, project: &Path, bindings: &[(&str, &Path)]) -> Vec<OpEntry> {
    let mut entries = vec![OpEntry {
        argv: ["bl", "claim", BALL, "--as", NAME]
            .map(str::to_owned)
            .to_vec(),
        cwd: project.to_string_lossy().into_owned(),
        ..OpEntry::default()
    }];
    for (conv, binding) in bindings {
        entries.push(OpEntry {
            argv: [
                "lernie",
                "prompt",
                "--name",
                conv,
                "--cwd",
                &binding.to_string_lossy(),
                "--pin",
                "instructions/00-AGENTS.md=/p/AGENTS.md",
                "--pin",
                "instructions/01-AGENTS.md=/p/src/AGENTS.md",
                "/ws",
                "the goal",
            ]
            .map(str::to_owned)
            .to_vec(),
            cwd: ws.to_string_lossy().into_owned(),
            ..OpEntry::default()
        });
    }
    entries
}

/// The snapshot both halves read: one named workspace, one project's balls,
/// and — when `agents` says so — a derived tree with its `steps/` bills.
pub(super) fn snap(
    ws: &Path,
    project: &Path,
    agents: Vec<Agent>,
    bills: Vec<StepBill>,
) -> Snapshot {
    let mut snap = ws_snap(ws, NAME, project, vec![ball(BALL, Some(NAME), None)]);
    snap.trees = HashMap::from([(
        ws.to_path_buf(),
        GitTree {
            agents,
            ..GitTree::default()
        },
    )]);
    snap.bills = HashMap::from([(ws.to_path_buf(), bills)]);
    snap
}

/// One agent whose §3.3 name fact is `CONV` — what the binding join resolves
/// the fire's `--name` through.
pub(super) fn named_agent() -> Agent {
    let mut agent = crate::nav::convs::tests::agent(AGENT, AgentState::Quiescent, 1);
    agent.name = Some(CONV.to_owned());
    agent
}

/// One step's bill under `conv`, with a wall span and one distinct counter.
pub(super) fn bill(conv: &str, seq: &str, input: u64, wall_secs: u64) -> StepBill {
    StepBill {
        conv: conv.to_owned(),
        seq: seq.to_owned(),
        model: Some("opus".to_owned()),
        spend: BudgetSpend {
            input_tokens: input,
            ..BudgetSpend::default()
        },
        last_usage: BudgetSpend::default(),
        wall_secs,
    }
}

/// The balls layout under a throwaway root, and the state root the claim
/// worktree formula mirrors a project under.
pub(super) fn layout(root: &Path) -> (balls::layout::Xdg, std::path::PathBuf) {
    (xdg(root), root.join("state").join("balls"))
}

/// Write the agent worktree bytes the projection reads: its frozen `goal.md`
/// and, for each entry, one `messages/` file in lernie's own naming.
pub(super) fn worktree(ws: &Path, agent: &str, goal: &str, messages: &[(&str, &str)]) {
    let dir = ws.join("agents").join(agent);
    std::fs::create_dir_all(dir.join("messages")).unwrap();
    std::fs::write(dir.join("goal.md"), goal).unwrap();
    for (i, (name, body)) in messages.iter().enumerate() {
        let file = format!("{:03}-{name}", i + 1);
        std::fs::write(dir.join("messages").join(file), body).unwrap();
    }
}

/// A live project repo with `work/<ball>` on it — the claim attempt's source.
pub(super) fn claimed_project() -> Project {
    let project = Project::new();
    project.switch(&balls::delivery_path::work_branch(BALL));
    project.commit("src/a.rs", "fn a() {}\n");
    project.checkout(crate::workdiff::tests::MAIN);
    project
}
