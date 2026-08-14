//! Which project a gesture mutates (§8.2): the one table over [`Action`] the
//! after-verb ball refresh reads. Its own file at §12's 300-line cap (bl-dc0c);
//! it was always a query on the enum rather than a part of it.

use super::Action;
use crate::start::Payload;
use std::path::PathBuf;

impl Action {
    /// The project a `bl`-family action mutates — the §8.2 after-verb ball
    /// refresh target. `None` for the lernie/workspace families.
    pub fn project(&self) -> Option<PathBuf> {
        match self {
            Action::Close { project, .. }
            | Action::Assign { project, .. }
            | Action::Release { project, .. }
            | Action::Move { project, .. }
            | Action::Create { project, .. }
            | Action::Update { project, .. } => Some(project.clone()),
            Action::Prepare { payload, .. } => match payload {
                Payload::Ball { project, .. } => Some(project.clone()),
                Payload::Bare | Payload::Path { .. } => None,
            },
            // `SetMarks` named a project until the per-agent ruling re-keyed
            // it to the agent (§16.3): it now repoints one agent's OWN space,
            // which is a different clone bundle from the one the §3.5
            // projection reads, so no board row can move because of it.
            Action::SetMarks { .. }
            | Action::Message { .. }
            | Action::Stop { .. }
            | Action::Scan { .. }
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
            | Action::PickModel { .. } => None,
        }
    }
}
