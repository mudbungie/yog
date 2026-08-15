//! **What a gesture addresses** (§8.2, REMOTE §8): the two tables over
//! [`Action`] that answer *which workspace* and *which project* it names. Its
//! own file at §12's 300-line cap (bl-dc0c, widened to both nouns by bl-f5f6);
//! each was always a query on the enum rather than a part of it.
//!
//! Both answer **names**, because that is what the boundary now carries: a
//! path is meaningless across machines and a disclosure besides (REMOTE §8).
//! One table per noun means the resolution stands **once, ahead of the
//! dispatch table** ([`dispatch`](super::dispatch::dispatch)) instead of being
//! re-derived inside twenty arms — and it is the same table the frame's
//! after-verb refresh reads, so "which project did that touch" has one answer
//! wherever it is asked.

use super::{Action, Gesture, Query};

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
    /// none — the ack, the trail clear, and the two `bl`-only families. The
    /// nested payloads answer through it: a deferred prompt and a fan both
    /// name the workspace their [`Prepared`](crate::start::Prepared) was
    /// prepared in, and the monitor/fleet verbs name their own.
    pub fn workspace(&self) -> Option<String> {
        match self {
            Action::Message { workspace, .. }
            | Action::Stop { workspace, .. }
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
            | Action::Floor { workspace, .. } => Some(workspace.clone()),
            Action::Prompt { prepared, .. } | Action::Fan { prepared, .. } => {
                Some(prepared.workspace.clone())
            }
            Action::Monitor(verb) => Some(verb.workspace()),
            Action::Fleet(verb) => Some(verb.workspace()),
            // An advertisement names its CLIENT, never a workspace (REMOTE §5,
            // bl-4e08): a tool set is a fact about the machine, and which
            // workspaces see it is the registration listing that already exists.
            // The routing leg's two, for the advertisement's reason exactly
            // (bl-024b): a call addresses a MACHINE, and the queue of calls to
            // one is a fact about that machine, not about a workspace.
            Action::Advertise { .. }
            | Action::Route(_)
            // The §9 config family answers through its destination instead
            // ([`config::ConfigFile::workspace`](super::config::ConfigFile)):
            // two of the five name a wall and three name no world at all, so
            // the table would have to read the file to answer here anyway.
            | Action::ApplyConfig { .. }
            | Action::Close { .. }
            | Action::Assign { .. }
            | Action::Release { .. }
            | Action::Move { .. }
            | Action::Create { .. }
            | Action::Update { .. }
            | Action::Retire { .. }
            | Action::Ack
            | Action::ClearTrail => None,
        }
    }

    /// The project a `bl`-family action mutates — the §8.2 after-verb ball
    /// refresh target. `None` for the lernie/workspace families.
    pub fn project(&self) -> Option<String> {
        match self {
            Action::Close { project, .. }
            | Action::Assign { project, .. }
            | Action::Release { project, .. }
            | Action::Move { project, .. }
            | Action::Create { project, .. }
            | Action::Update { project, .. } => Some(project.clone()),
            // A fan claims nothing and a retirement delivers nothing, but both
            // act in a project's refs — and the §3.5 projection reads that
            // project's board, so both refresh it.
            Action::Fan { obligation, .. } | Action::Retire { obligation, .. } => {
                Some(obligation.project.clone())
            }
            Action::Prepare { payload, .. } => payload.project(),
            // `SetMarks` named a project until the per-agent ruling re-keyed
            // it to the agent (§16.3): it now repoints one agent's OWN space,
            // which is a different clone bundle from the one the §3.5
            // projection reads, so no board row can move because of it.
            Action::SetMarks { .. }
            | Action::Message { .. }
            | Action::Stop { .. }
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

impl Query {
    /// The **workspace** this read is aimed at (§3.1), or `None` for the reads
    /// that span the world (the roster, the board, the trail, search) and the
    /// one whose subject is the interface (help). The §9 config family answers
    /// through its destination instead
    /// ([`ConfigFile`](super::config::ConfigFile)), for the reason
    /// [`Action::workspace`] gives.
    ///
    /// The mirror of the action table above, and it exists for the same reason:
    /// one resolution stands ahead of [`answer`](super::answer::answer)'s table
    /// rather than being re-derived inside a dozen arms.
    pub fn workspace(&self) -> Option<String> {
        match self {
            Query::Conversations { workspace }
            | Query::WorkDiff { workspace, .. }
            | Query::Lineages { workspace }
            | Query::Models { workspace, .. }
            | Query::Marks { workspace }
            | Query::Transcript { workspace, .. }
            | Query::Steps { workspace, .. }
            | Query::Step { workspace, .. }
            | Query::Files { workspace, .. }
            | Query::Governing { workspace, .. }
            | Query::Rail { workspace, .. }
            | Query::Inbox { workspace, .. }
            | Query::Agent { workspace, .. }
            | Query::Providers { workspace }
            | Query::WorkspaceBalls { workspace }
            | Query::Clients { workspace } => Some(workspace.clone()),
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
