//! **What a gesture addresses** (§8.2, REMOTE §8): the tables over [`Action`]
//! and [`Query`] answering which named thing a gesture is aimed at — one file
//! per noun, each a query *on* the enum rather than a part of it. Split off at
//! §12's cap (bl-dc0c) and widened from one noun to three as the boundary
//! learned to speak names (bl-f5f6, bl-49bc, bl-4e31).
//!
//! What is left here is the **project** table. The other two nouns are their
//! own files because each differs from this one in a way worth stating where
//! it lives:
//!
//! - [`workspace`] — the noun that is also **written** (REMOTE §8.2): a §8.2
//!   entry's client-side leaf may differ from the name that workspace bears on
//!   its host, so the table is borrowed rather than read, and one rewrite
//!   spends the mapping at the channel boundary.
//! - [`agent`] — the **conversation** (bl-49bc), addressed by an agent id or
//!   the unique stored name a living agent wears rather than over an
//!   enumerated set of paths, so it carries a resolution ladder beside its
//!   table.
//!
//! All three answer **names**, because that is what the boundary now carries: a
//! path is meaningless across machines and a disclosure besides (REMOTE §8).
//! One table per noun means the resolution stands **once, ahead of the
//! dispatch table** ([`dispatch`](super::dispatch::dispatch)) instead of being
//! re-derived inside twenty arms — and it is the same table the frame's
//! after-verb refresh reads, so "which project did that touch" has one answer
//! wherever it is asked.

use super::Action;

/// The conversation noun's own table and resolution (bl-49bc) — see the module
/// doc above for why each noun is a file rather than a third table here.
mod agent;
/// The workspace noun's table, and the one rewrite that writes it (REMOTE §8.2,
/// bl-4e31).
mod workspace;
pub(super) use agent::resolve_agent;

impl Action {
    /// The project a `bl`-family action mutates — the §8.2 after-verb ball
    /// refresh target. `None` for the lernie/workspace families.
    pub fn project(&self) -> Option<String> {
        match self {
            Action::Close { project, .. }
            | Action::Assign { project, .. }
            | Action::Release { project, .. }
            | Action::Create { project, .. }
            | Action::Update { project, .. } => Some(project.clone()),
            // A fan claims nothing, a retirement delivers nothing, and a
            // delivery closes nothing — but all three act in a project's refs,
            // and the §3.5 projection reads that project's board, so all three
            // refresh it.
            Action::Fan(
                crate::fan::Verb::Spread { obligation, .. }
                | crate::fan::Verb::Retire { obligation, .. }
                | crate::fan::Verb::Deliver { obligation, .. },
            ) => Some(obligation.project.clone()),
            Action::Prepare { payload, .. } => payload.project(),
            // `SetMarks` named a project until the per-agent ruling re-keyed
            // it to the agent (§16.3): it now repoints one agent's OWN space,
            // which is a different clone bundle from the one the §3.5
            // projection reads, so no board row can move because of it.
            Action::SetMarks { .. }
            | Action::Message { .. }
            | Action::Stop { .. }
            | Action::Interrupt { .. }
            | Action::Scan { .. }
            | Action::Nudge { .. }
            | Action::Retarget { .. }
            | Action::Prompt { .. }
            | Action::DeleteWorkspace { .. }
            | Action::DeleteAgent { .. }
            | Action::Monitor(_)
            // Arming writes one config entry and claims nothing; the loop's own
            // spawns and reaps are ordinary `bl` actions and refresh on their own.
            | Action::Fleet(_)
            | Action::AnswerHold { .. }
            | Action::Floor { .. }
            | Action::Fork { .. }
            | Action::Ack
            | Action::MarkSeen { .. }
            | Action::ClearTrail
            | Action::ApplyConfig { .. }
            | Action::Advertise { .. }
            | Action::Route(_)
            | Action::PickModel { .. } => None,
        }
    }
}
