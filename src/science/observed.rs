//! **What a bound conversation can be asked** (§3.9, bl-40ab) — every column of
//! the projection that is a fact about the *agent* rather than about the refs.
//!
//! Split from [`super::bound`] at §12's budget on the seam that module's doc
//! draws: it answers *which* conversation an attempt is bound to, this answers
//! *what about it*, and the two share nothing but the agent id.
//!
//! Every one is read from the authority that already owns it, and the
//! step-record columns are read from **no disk at all**: `Snapshot::bills` is
//! the walk the derivation worker already made (bl-9dd4), so usage, wall time
//! and the step count are one in-memory filter over it.

use std::path::Path;

use crate::app::Snapshot;
use crate::budgets::{BudgetSpend, Scope, total, wall};
use crate::transcript::{Block, Entry, EntryKind, Transcript};

/// The agent worktree's goal file (litany ARCH §2.2) — the frozen input, as of
/// the dispatch commit this worktree is a checkout of.
const GOAL_FILE: &str = "goal.md";
/// Workspace subdirectory holding the per-agent worktrees (litany ARCH §2.2).
const AGENTS_DIR: &str = "agents";

/// What the bound conversation can be asked — every column of the projection
/// that is a fact about the *agent* rather than about the refs. Default is the
/// honest answer for an attempt with no conversation: nothing was frozen,
/// nothing was spent and nothing was said.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Observed {
    pub(super) goal: Option<String>,
    pub(super) governing: Option<String>,
    pub(super) usage: BudgetSpend,
    pub(super) wall_secs: u64,
    pub(super) steps: usize,
    pub(super) response: Option<String>,
    pub(super) verdicts: Vec<super::Verdict>,
    pub(super) compacted: usize,
}

/// Read all of it for one agent. Three sources: the worktree's `goal.md`, the
/// §5.1 #17 config walk, the published bills, and the committed transcript —
/// each already the one home of what it answers.
pub(super) fn observed(snap: &Snapshot, workspace: &Path, agent: &str) -> Observed {
    let bills: Vec<_> = snap
        .bills
        .get(workspace)
        .map(|all| {
            let scope = Scope::Tree(agent.to_owned());
            all.iter()
                .filter(|b| scope.wants(&b.conv))
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let transcript = crate::transcript::build(workspace, agent);
    Observed {
        goal: goal(workspace, agent),
        governing: governing(snap, workspace, agent),
        usage: total(&bills),
        wall_secs: wall(&bills),
        steps: bills.len(),
        response: response(&transcript),
        verdicts: verdicts(&transcript),
        compacted: compacted(&transcript),
    }
}

/// The goal this agent was fired with, verbatim from its own worktree.
fn goal(workspace: &Path, agent: &str) -> Option<String> {
    let path = workspace.join(AGENTS_DIR).join(agent).join(GOAL_FILE);
    std::fs::read_to_string(path).ok()
}

/// The config commit this agent is frozen on (§5.1 #17) — the walk from its own
/// branch tip, exactly as the §11 Config tab asks it. `None` for an agent the
/// snapshot does not carry and for a workspace whose git will not answer: the
/// projection says the freeze is unreadable rather than naming some other
/// commit.
fn governing(snap: &Snapshot, workspace: &Path, agent: &str) -> Option<String> {
    let tip = snap
        .trees
        .get(workspace)?
        .agents
        .iter()
        .find(|a| a.agent_id == agent)?
        .tip_oid
        .clone();
    crate::config_edit::branch::governing_config(workspace, &tip)
        .ok()
        .map(|gov| gov.oid)
}

/// The last committed model turn's text — the attempt's terminal response.
/// `None` when the conversation has no model turn yet, and when its last turn
/// was tool calls and reasoning with no answer in it: an empty answer is not an
/// answer, and a `Some("")` would read as one.
fn response(transcript: &Transcript) -> Option<String> {
    let text = transcript
        .entries
        .iter()
        .rev()
        .find_map(model_text)
        .unwrap_or_default();
    (!text.is_empty()).then_some(text)
}

/// One entry's model text, when it is a model turn: its [`Block::Text`] blocks
/// joined. Reasoning and tool calls are not the answer, so they do not ride.
fn model_text(entry: &Entry) -> Option<String> {
    let EntryKind::Model { blocks, .. } = &entry.kind else {
        return None;
    };
    let said: Vec<&str> = blocks
        .iter()
        .filter_map(|b| match b {
            Block::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    Some(said.join("\n"))
}

/// Every message delivered into this conversation, oldest first. No wording is
/// judged: a verdict is a message (VISION V3.1), and which messages an operator
/// counts as verdicts is the reader's question, not yog's.
fn verdicts(transcript: &Transcript) -> Vec<super::Verdict> {
    transcript
        .entries
        .iter()
        .filter_map(|entry| match &entry.kind {
            EntryKind::Delivered { sender, body, .. } => Some(super::Verdict {
                sender: sender.clone(),
                body: body.clone(),
            }),
            _ => None,
        })
        .collect()
}

/// How many entries the counter proves compacted away (§5.1 #12) — the sum of
/// the spliced markers' spans, and the whole of what disk can say about what
/// [`verdicts`] and [`response`] no longer see. Verdicts delivered in a
/// squashed span are deleted files: they are not recovered and not guessed at,
/// and this figure is the projection saying so (bl-fde5).
fn compacted(transcript: &Transcript) -> usize {
    transcript
        .entries
        .iter()
        .filter_map(|entry| match &entry.kind {
            EntryKind::Compacted { first, last, .. } => {
                Some(last.saturating_sub(*first).saturating_add(1))
            }
            _ => None,
        })
        .sum()
}
