//! The spine's headless tests — S10's derivation half.
//!
//! Split by what each half asserts: [`build`] the notches and the two edges,
//! [`cards`] the card those edges carry — the two cut at §12's budget on the
//! `mod`/`cards` seam the production modules have; [`cohort`] their grouping
//! into V2's fans; [`pin`] the fold that
//! threads one notch through the inspector; [`place`] where each notch sits in
//! the chat, which is the pairing bl-1802 corrected. The paint half moved to
//! `transcript::tests::spine` with the seat itself.

mod build;
mod cards;
mod cohort;
mod pin;
mod place;
mod tree;

use crate::budgets::BudgetSpend;
use crate::git_tree::{AgentState, Framing, StepCommit};
use crate::rail::ChildInput;
use crate::steps_view::{Orphan, StepSummary, StepsView, Wound};
use crate::transcript::{Block, Entry, EntryKind, Transcript, Usage};

/// A commit on a branch; oid and timestamp are the only fields the rail reads.
pub(super) fn commit(oid: &str, at: i64) -> StepCommit {
    StepCommit {
        oid: oid.to_owned(),
        short_oid: oid.chars().take(8).collect(),
        timestamp_unix: at,
        subject: "step".to_owned(),
    }
}

/// A step whose read-state commit is `oid` (or none) and which billed
/// `tokens`.
pub(super) fn step(seq: &str, oid: Option<&str>, tokens: u64) -> StepSummary {
    StepSummary {
        seq: seq.to_owned(),
        framing: Framing::Complete,
        attempts: 1,
        tokens: BudgetSpend {
            input_tokens: tokens,
            ..BudgetSpend::default()
        },
        commit: oid.map(str::to_owned),
        started_at: None,
        ended_at: None,
        wound: Wound::None,
    }
}

pub(super) fn steps(rows: Vec<StepSummary>) -> StepsView {
    StepsView {
        steps: rows,
        orphan: Orphan::default(),
    }
}

/// One delivered message and one model reply per step — the settled shape a
/// conversation of `turns` completed calls has, so every notch gets its seat
/// in the chat and the `NNN` counter runs as litany's does.
pub(super) fn chat(turns: usize) -> Transcript {
    let entry = |name: String, kind: EntryKind| Entry {
        name,
        raw: b"x".to_vec(),
        kind,
    };
    let mut entries = Vec::new();
    for turn in 0..turns {
        entries.push(entry(
            format!("{:03}-user.md", turn * 2 + 1),
            EntryKind::Delivered {
                sender: "user".to_owned(),
                epitaph: None,
                body: "hi".to_owned(),
            },
        ));
        entries.push(entry(
            format!("{:03}-opus.json", turn * 2 + 2),
            EntryKind::Model {
                model_id: "opus".to_owned(),
                blocks: vec![Block::Text("ok".to_owned())],
                usage: Usage::new(),
            },
        ));
    }
    Transcript { entries }
}

/// The row key of the `turn`-th chat's delivered message — where that turn's
/// notch paints its rule.
pub(super) fn seat(turn: usize) -> String {
    format!("tx/{:03}-user.md#0", turn * 2 + 1)
}

/// A child whose branch carries `commits`, named `name`.
pub(super) fn child(name: &str, commits: Vec<StepCommit>) -> ChildInput {
    ChildInput {
        agent_id: format!("root-{name}"),
        name: name.to_owned(),
        state: AgentState::Quiescent,
        streaming_text: None,
        commits,
        tokens: 0,
        config_label: None,
    }
}
