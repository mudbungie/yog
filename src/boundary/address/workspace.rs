//! **The workspace a gesture names, in both directions** (REMOTE §8, §8.2):
//! the table over [`Action`] and [`Query`] answering *which workspace*, and the
//! rewrite that replaces it. Its own file beside [`agent`](super::agent)'s
//! third noun, on the seam that module's doc already draws — one file per noun,
//! standing once ahead of each chokepoint's match — and here the noun differs
//! from its two siblings by a **direction**: a workspace is the one addressed
//! thing that is also *written*, when a §8.2 entry's client-side leaf differs
//! from the name that workspace bears on its host.
//!
//! **One table, not two.** [`Action::workspace_slot`] is the whole of it and
//! the read answers *through* the write. Two exhaustive matches over the same
//! thirty-odd variants would be two representations of one fact, and the arm
//! that drifted would send a client's own leaf to a host that never heard of
//! it. The reader pays a clone for that, which is the trade the house standard
//! names outright: the performance given up was never why we chose Rust.
//!
//! **The rewrite is spent at the channel boundary and nowhere else** (§8.2:
//! *"the mapping between the two names is spent at exactly one place, the
//! channel boundary, in both directions"*). It is neither a face capability nor
//! a verb — REMOTE §3's ban on wire-only vocabulary is untouched, because this
//! edits a field on a gesture the boundary already carries rather than adding
//! one it does not.

use crate::boundary::{Action, Gesture, Query};

impl Gesture {
    /// The workspace this gesture names, whichever half it is — the two tables
    /// below, read as one (bl-8bbc).
    ///
    /// The wire's scoped intake asks it: a gesture's address is what
    /// authorization is decided over (REMOTE §4), and asking it here means the
    /// scope and the dispatch chokepoint read the **same** table rather than
    /// two that could disagree about which workspace a variant names.
    pub fn workspace(&self) -> Option<String> {
        match self {
            Gesture::Act(action) => action.workspace(),
            Gesture::Ask(query) => query.workspace(),
        }
    }
}

impl Action {
    /// The **workspace** this gesture names (§3.1), or `None` when it names
    /// none — the ack, the trail clear, the two `bl`-only families, and the
    /// three §9 destinations that name no world.
    pub fn workspace(&self) -> Option<String> {
        let mut named = self.clone();
        named.workspace_slot().map(std::mem::take)
    }

    /// **The one table.** The field naming this action's workspace, borrowed so
    /// the read above and the rewrite beside it cannot disagree about which
    /// arms have one. The nested payloads answer through it: a deferred prompt
    /// and a fan both name the workspace their
    /// [`Prepared`](crate::start::Prepared) was prepared in, the monitor/fleet
    /// verbs name their own, and the §9 family names the wall its destination
    /// lands in ([`ConfigFile`](crate::boundary::config::ConfigFile)).
    fn workspace_slot(&mut self) -> Option<&mut String> {
        match self {
            Action::Message { workspace, .. }
            | Action::Stop { workspace, .. }
            | Action::Interrupt { workspace, .. }
            | Action::Scan { workspace }
            | Action::Nudge { workspace, .. }
            | Action::Retarget { workspace, .. }
            | Action::Fork { workspace, .. }
            | Action::Prepare { workspace, .. }
            | Action::DeleteWorkspace { workspace, .. }
            | Action::DeleteAgent { workspace, .. }
            | Action::MarkSeen { workspace, .. }
            | Action::SetMarks { workspace, .. }
            | Action::PickModel { workspace, .. }
            | Action::AnswerHold { workspace, .. }
            | Action::Floor { workspace, .. } => Some(workspace),
            // The §9.4 tuning pair delegates, as the monitor's and the fleet's
            // families do: both members name a workspace, so the carrier
            // answers and this table does not match on the pair (bl-23bd).
            Action::Tune(tuning) => Some(tuning.workspace_slot()),
            Action::Prompt { prepared, .. }
            | Action::Fan(crate::fan::Verb::Spread { prepared, .. }) => {
                Some(&mut prepared.workspace)
            }
            // An enrollment names the workspace it SEATS the new client in
            // (REMOTE §1.4 as amended, §4.1): the act creates the registration,
            // and a registration is the pair — so it is addressed like every
            // other gesture, scoped like one, and renamed at a §8.2 entry's
            // channel boundary like one.
            Action::Enroll(request) => Some(&mut request.workspace),
            Action::Monitor(verb) => Some(verb.workspace_slot()),
            Action::Fleet(verb) => Some(verb.workspace_slot()),
            // The §9 config family answers through its destination (bl-523f):
            // the wall a provider config or a lineage file belongs to is the
            // workspace the act is aimed AT, so it is the address §8.2 rewrites
            // and the entry routing reads. Two of the five name a wall and
            // three name no world at all, so the row is the destination's own
            // table rather than an arm per variant.
            Action::ApplyConfig { file, .. } => file.workspace_slot(),
            // An advertisement names its CLIENT, never a workspace (REMOTE §5,
            // bl-4e08): a tool set is a fact about the machine, and which
            // workspaces see it is the registration listing that already exists.
            // The routing leg's two, for the advertisement's reason exactly
            // (bl-024b): a call addresses a MACHINE, and the queue of calls to
            // one is a fact about that machine, not about a workspace.
            Action::Advertise { .. }
            | Action::Route(_)
            | Action::Ball(_)
            | Action::Fan(crate::fan::Verb::Retire { .. } | crate::fan::Verb::Deliver { .. })
            | Action::Ack
            | Action::ClearTrail => None,
        }
    }
}

impl Query {
    /// The **workspace** this read is aimed at (§3.1), or `None` for the reads
    /// that span the world (the roster, the board, the trail, search) and the
    /// one whose subject is the interface (help). The §9 config read answers
    /// through its destination
    /// ([`ConfigFile`](crate::boundary::config::ConfigFile)), exactly as
    /// [`Action::workspace`] does.
    ///
    /// The mirror of the action table above, and it exists for the same reason:
    /// one resolution stands ahead of [`answer`](crate::boundary::answer::answer)'s
    /// table rather than being re-derived inside a dozen arms.
    pub fn workspace(&self) -> Option<String> {
        let mut named = self.clone();
        named.workspace_slot().map(std::mem::take)
    }

    /// The one table for reads, for [`Action::workspace_slot`]'s reason exactly.
    fn workspace_slot(&mut self) -> Option<&mut String> {
        match self {
            Query::Conversations { workspace }
            | Query::WorkDiff { workspace, .. }
            | Query::Science { workspace }
            | Query::Transcript { workspace, .. }
            | Query::Follow { workspace, .. }
            | Query::Steps { workspace, .. }
            | Query::Step { workspace, .. }
            | Query::Files { workspace, .. }
            | Query::Governing { workspace, .. }
            | Query::Rail { workspace, .. }
            | Query::Inbox { workspace, .. }
            | Query::Agent { workspace, .. }
            | Query::WorkspaceBalls { workspace }
            | Query::Clients { workspace } => Some(workspace),
            // The §9 family delegates (bl-719a), and its answer is an `Option`
            // because one member of it addresses through a *destination*
            // exactly as the write does (bl-523f) — one row on each side of the
            // one table, so a read and a write of the same file cross the same
            // channel, and the other four name a workspace outright.
            Query::Config(read) => read.workspace_slot(),
            Query::Workspaces
            | Query::Balls
            | Query::Board
            | Query::Attention
            | Query::Ops { .. }
            | Query::Search { .. }
            | Query::Help { .. }
            // The routing leg's two reads: one is answered to the intake's own
            // identity, the other to a handle — neither names a world.
            | Query::Invocations
            | Query::Capture { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests;
