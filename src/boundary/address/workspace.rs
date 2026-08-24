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

    /// This gesture with the workspace it names replaced by `name` — the §8.2
    /// mapping's write direction, spent by the seat that is about to encode it
    /// for one channel ([`wire::seat`](crate::wire::seat)).
    ///
    /// A gesture naming no workspace comes back unchanged: the general path
    /// with nothing to rewrite, not a case of its own.
    pub(crate) fn with_workspace(self, name: &str) -> Self {
        match self {
            Gesture::Act(action) => Gesture::Act(action.with_workspace(name)),
            Gesture::Ask(query) => Gesture::Ask(query.with_workspace(name)),
        }
    }
}

impl Action {
    /// The **workspace** this gesture names (§3.1), or `None` when it names
    /// none — the ack, the trail clear, and the two `bl`-only families.
    pub fn workspace(&self) -> Option<String> {
        let mut named = self.clone();
        named.workspace_slot().map(std::mem::take)
    }

    /// [`Self::workspace`] written: this action with its workspace field set to
    /// `name`, or itself when it has none.
    pub(crate) fn with_workspace(mut self, name: &str) -> Self {
        if let Some(slot) = self.workspace_slot() {
            name.clone_into(slot);
        }
        self
    }

    /// **The one table.** The field naming this action's workspace, borrowed so
    /// the read above and the rewrite beside it cannot disagree about which
    /// arms have one. The nested payloads answer through it: a deferred prompt
    /// and a fan both name the workspace their
    /// [`Prepared`](crate::start::Prepared) was prepared in, and the
    /// monitor/fleet verbs name their own.
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
            Action::Prompt { prepared, .. }
            | Action::Fan(crate::fan::Verb::Spread { prepared, .. }) => {
                Some(&mut prepared.workspace)
            }
            Action::Monitor(verb) => Some(verb.workspace_slot()),
            Action::Fleet(verb) => Some(verb.workspace_slot()),
            // An advertisement names its CLIENT, never a workspace (REMOTE §5,
            // bl-4e08): a tool set is a fact about the machine, and which
            // workspaces see it is the registration listing that already exists.
            // The routing leg's two, for the advertisement's reason exactly
            // (bl-024b): a call addresses a MACHINE, and the queue of calls to
            // one is a fact about that machine, not about a workspace.
            Action::Advertise { .. }
            | Action::Route(_)
            // The §9 config family answers through its destination instead
            // ([`config::ConfigFile::workspace`](crate::boundary::config::ConfigFile)):
            // two of the five name a wall and three name no world at all, so
            // the table would have to read the file to answer here anyway.
            | Action::ApplyConfig { .. }
            | Action::Close { .. }
            | Action::Assign { .. }
            | Action::Release { .. }
            | Action::Create { .. }
            | Action::Update { .. }
            | Action::Fan(
                crate::fan::Verb::Retire { .. } | crate::fan::Verb::Deliver { .. },
            )
            | Action::Ack
            | Action::ClearTrail => None,
        }
    }
}

impl Query {
    /// The **workspace** this read is aimed at (§3.1), or `None` for the reads
    /// that span the world (the roster, the board, the trail, search) and the
    /// one whose subject is the interface (help). The §9 config family answers
    /// through its destination instead
    /// ([`ConfigFile`](crate::boundary::config::ConfigFile)), for the reason
    /// [`Action::workspace`] gives.
    ///
    /// The mirror of the action table above, and it exists for the same reason:
    /// one resolution stands ahead of [`answer`](crate::boundary::answer::answer)'s
    /// table rather than being re-derived inside a dozen arms.
    pub fn workspace(&self) -> Option<String> {
        let mut named = self.clone();
        named.workspace_slot().map(std::mem::take)
    }

    /// [`Self::workspace`] written — [`Action::with_workspace`]'s mirror.
    pub(crate) fn with_workspace(mut self, name: &str) -> Self {
        if let Some(slot) = self.workspace_slot() {
            name.clone_into(slot);
        }
        self
    }

    /// The one table for reads, for [`Action::workspace_slot`]'s reason exactly.
    fn workspace_slot(&mut self) -> Option<&mut String> {
        match self {
            Query::Conversations { workspace }
            | Query::WorkDiff { workspace, .. }
            | Query::Science { workspace }
            | Query::Lineages { workspace }
            | Query::Models { workspace, .. }
            | Query::Marks { workspace }
            | Query::Transcript { workspace, .. }
            | Query::Follow { workspace, .. }
            | Query::Steps { workspace, .. }
            | Query::Step { workspace, .. }
            | Query::Files { workspace, .. }
            | Query::Governing { workspace, .. }
            | Query::Rail { workspace, .. }
            | Query::Inbox { workspace, .. }
            | Query::Agent { workspace, .. }
            | Query::Providers { workspace }
            | Query::WorkspaceBalls { workspace }
            | Query::Clients { workspace } => Some(workspace),
            Query::Workspaces
            | Query::Balls
            | Query::Board
            | Query::Attention
            | Query::Ops { .. }
            | Query::Search { .. }
            | Query::Help { .. }
            | Query::ReadConfig { .. }
            // The routing leg's two reads: one is answered to the intake's own
            // identity, the other to a handle — neither names a world.
            | Query::Invocations
            | Query::Capture { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests;
