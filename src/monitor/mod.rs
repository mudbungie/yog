//! The alignment monitor (VISION §4.9, story rung V6): a second, cheaper model
//! reads what an agent is doing and answers one question — **does the recent
//! work serve the stated goal?**
//!
//! Everything the monitor needs already has a durable home, which is why it
//! adds no artifact of its own:
//!
//! | Fact | Home | Read by |
//! |---|---|---|
//! | the assignment | the branch's `goal.md` (committed, immutable) | [`check::request`] |
//! | what the agent did | the committed `messages/` transcript | [`check::request`] |
//! | armed-ness and policy | the `cadence.yaml` `monitor:` entry | [`arming`] |
//! | every verdict ever | `ops.jsonl` rows | [`row`] |
//!
//! **THE ANTI-REINVENTION LAW (VISION §4.9, stated as law).** The tier-0 check
//! has no tools, no retry machinery and no transcript. Its **retry is the
//! level-trigger itself** — a failed check leaves the last-checked sha behind
//! the branch tip, so the next tick simply re-fires; its **audit is the ops
//! row**; and **any response that requires a decision is a dispatch**. The
//! moment the check wants a second lernie feature — a tool, a multi-step chain,
//! a memory — it has become an agent, and the design's answer already exists:
//! dispatch one. Nothing in this module may grow one.
//!
//! **One row is three things.** Every check appends exactly one `ops.jsonl`
//! line ([`row`]) naming the sha it read, the verdict and the reason. That line
//! is at once the audit trail, the level-trigger's memory (the last-checked sha
//! is *derived* from it, never stored beside it) and the tuning dataset — so
//! the monitor stores nothing a query could answer.
//!
//! **Unarmed, the mechanism is absent.** No `cadence.yaml` entry means no
//! model call, no row, and nothing rendered; severability is deleting the
//! entry (or the prompt file it names), never editing code.
//!
//! The pieces: [`arming`] is the config tie-point, [`verdict`] the three-valued
//! answer and its parse, [`check`] the one bounded tool-less call, [`row`] the
//! ops-row encoding and the standing-verdict derivation, and [`sentry`] the
//! off-thread level trigger that fires a check only when a branch tip moved.

pub mod arming;
pub mod check;
pub mod row;
pub mod sentry;
pub mod verdict;
pub mod window;

/// The monitor's gestures, as the control boundary carries them (VISION §4.9,
/// DESIGN §8.5). One boundary [`Action`](crate::boundary::Action) variant holds
/// this enum rather than three holding its arms: the three are one family —
/// same subject, same config file, same trail — and folding them here keeps the
/// boundary's four tables (codec, line, dispatch, help) one row wider apiece
/// instead of three, while every gesture still carries its whole parameter set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verb {
    /// Arm one workspace on `model`: write its `cadence.yaml` monitor entry and
    /// seed the policy file that entry names.
    Arm { workspace: String, model: String },
    /// Disarm it: delete that entry. Its own gesture rather than an arm with no
    /// pin — arming with no model is a different instruction, and a gesture is
    /// an instruction, never an absence to be read.
    Disarm { workspace: String },
    /// Raise an attention item on one conversation, with its reason, as its own
    /// ops row. The responder's floor grant (bl-7aef): signalling out is a call
    /// whose schema types the signal, never prose yog would have to parse.
    Flag {
        workspace: String,
        agent: String,
        reason: String,
    },
}

impl Verb {
    /// **The workspace this verb names** (REMOTE §8) — every one of the three
    /// does, so the boundary's own table
    /// ([`Action::workspace`](crate::boundary::Action)) answers through here
    /// rather than re-matching the arms.
    ///
    /// Borrowed rather than read, because the boundary both reads this name and
    /// **writes** it: REMOTE §8.2's channel-boundary rewrite replaces it when a
    /// client-side entry's leaf differs from the name its host knows. One table
    /// rather than two, for the reason `boundary::address::workspace`'s doc
    /// gives — two exhaustive matches over one fact are two things that drift.
    pub(crate) fn workspace_slot(&mut self) -> &mut String {
        match self {
            Verb::Arm { workspace, .. }
            | Verb::Disarm { workspace }
            | Verb::Flag { workspace, .. } => workspace,
        }
    }

    /// The **conversation** this verb names (REMOTE §8 as amended, bl-49bc) —
    /// only the flag does, arming and disarming being facts about a workspace.
    /// The boundary's own conversation table
    /// ([`Action::agent`](crate::boundary::Action)) answers through here for
    /// [`workspace`](Self::workspace)'s reason exactly.
    pub fn agent(&self) -> Option<String> {
        match self {
            Verb::Flag { agent, .. } => Some(agent.clone()),
            Verb::Arm { .. } | Verb::Disarm { .. } => None,
        }
    }
}

pub use arming::Watch;
pub use check::{BzCaller, Called, Caller};
pub use row::Check;
pub use sentry::{Sentry, SentryCtx};
pub use verdict::{Reply, Verdict};
