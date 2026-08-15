//! The §3.6 confirmation derivations both dispatch and the dialog read — split
//! from [`super`] at §12's per-file budget (bl-6233) on the seam that module's
//! own doc already draws: the chokepoint answers queries, and this answers
//! *what an unmaking would destroy*, which no query asks and both gates need.
//!
//! One derivation for the dialog and the dispatch gate alike; re-derived at
//! fire time, fail-closed.

use std::path::Path;

use crate::app::Snapshot;
use crate::delete::{self, Claim, Confirmation};
use crate::nav;
use crate::projects::join::JoinState;

/// The §3.6 confirmation for `ws` — what dies, what is released, what is live.
/// `None` for anything not one of yog's own named workspaces (§3.6 scope).
pub fn confirmation_of(snap: &Snapshot, ws: &Path) -> Option<Confirmation> {
    let name = named_leaf(snap, ws)?;
    let agents = snap.trees.get(ws).map_or(&[][..], |t| t.agents.as_slice());
    Some(delete::confirmation(
        &name,
        &nav::convs::liveness(agents),
        bound_claims(snap, &name),
    ))
}

/// The §3.6 agent-delete confirmation for one conversation (bl-f17a): its
/// display name and its live members. `None` outside yog's own named
/// workspaces — the same scope as the workspace verb (§3.6: foreign
/// workspaces are another driver's territory, replays read-only), and how
/// every carrier decides whether to offer the verb.
pub fn agent_confirmation_of(
    snap: &Snapshot,
    ws: &Path,
    root: &str,
) -> Option<delete::agent::AgentConfirmation> {
    named_leaf(snap, ws)?;
    let agents = snap.trees.get(ws).map_or(&[][..], |t| t.agents.as_slice());
    Some(delete::agent::confirmation(root, agents))
}

/// `ws`'s own name iff it is one of yog's own — [`crate::binding::named_of`]'s
/// question, asked of this snapshot's workspace set.
fn named_leaf(snap: &Snapshot, ws: &Path) -> Option<String> {
    crate::binding::named_of(&snap.workspaces, ws)
}

/// The live bound balls the unmaking releases (§3.6 step 1): the join's
/// [`Bound`](JoinState::Bound) rows for this workspace.
fn bound_claims(snap: &Snapshot, name: &str) -> Vec<Claim> {
    snap.join_rows
        .iter()
        .filter(|r| r.workspace.as_deref() == Some(name) && r.state == JoinState::Bound)
        .map(|r| Claim {
            project: r.project.clone(),
            id: r.ball_id.clone(),
        })
        .collect()
}
