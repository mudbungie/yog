//! The conversation-list view-model (DESIGN §11 altitude 0, §15 Z9).
//!
//! A **conversation** is a root agent in the focused workspace (§1); its
//! subtree (the §2.3 hyphenated descent) rides with it. One row per **visible
//! member of the descent forest** ([`expand`], bl-fa82) — with nothing expanded
//! that is one row per root, which is all this list ever was: state badge
//! (aggregated over the row's own subtree — InFlight > Live > its agent's
//! settled state), the §3.3 [`display_name`] ladder with the first payload line
//! weak beside it, age, and the §11 live-activity class ([`Flight`]) pulsing
//! while any member is working.
//! Sort: **recency alone** — last action of any kind, descending (§11 as
//! amended by bl-cad5); attention and liveness are badges, not ranks.
//! Pure over the injected agent snapshot + the seen closure; the shell paints
//! [`ConvRow`]s; [`members`] is the subtree fold every row's aggregate reads.
//!
//! The §3.3 ladder itself — what a conversation is *called*, and the when-seat
//! read out of the same id — is [`naming`]: this module answers the structural
//! questions (which agents form a conversation, which root one belongs to, is
//! it live), that one answers what to write on the row.

use crate::git_tree::{Agent, AgentState, DescentRow, descent_order};

/// The **census** folds a §3.6 gate and the §3.3 mint read off an answered
/// forest (REMOTE §9.7, bl-b4b5) — [`expand`]'s and [`select`]'s third sibling.
pub mod census;
pub mod doing;
pub mod expand;
pub mod flight;
pub mod group;
/// The §3.3 **display ladder** and the id-derived when-seat, the one naming
/// rule every seat falls through together (bl-63a1, bl-16da).
pub mod naming;
pub mod row;
/// The **selection's** own facts, picked out of the answered forest (REMOTE
/// §9.7, bl-48ae) — [`expand::visible`]'s sibling fold.
pub mod select;
/// The §3.3 ladder as a seat holds it — id→title, no agent set (bl-1eb0).
pub mod titles;

pub use census::{liveness_of_rows, names_in_rows};
pub use doing::{Doing, Seat, doing, seats};
pub use expand::{ancestors, forest_rows, parent_of, step, visible};
pub use flight::{Flight, FlightStrip, STRIP_HOVER, conversation_flight, strip};
pub use naming::{StartedAt, display_name, display_name_of, member_title, preview_of, started_at};
/// The ladder's floor spelling and the stamp predicate it shares with the §3.3
/// acceptance scan — internal, because both hand back a borrow of their
/// argument (AGENTS rule 2) and every caller is in this crate.
pub(crate) use naming::{id_floor, is_stamp};
pub use row::{ConvBall, ConvRow, age_label, build};
pub use select::{Selection, selection};
pub use titles::Titles;

/// A conversation reduced to what a **verb** needs (§3.6): its display name and
/// whether it holds a driver. Deliberately not [`ConvRow`] — that one is the §11
/// list's projection and needs a clock, the seen closure and the §3.5 join; the
/// deletion gate asks one question about a workspace nobody may be looking at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conversation {
    pub name: String,
    pub live: bool,
}

/// The workspace's conversations as a verb's gate reads them (§3.6): one entry
/// per root, `live` true when any member probes Live/InFlight **or** carries the
/// §10 uncertainty — an unobservable probe counts as live, so the gate fails
/// closed rather than racing an `rm` against a flock-holding driver.
pub fn liveness(agents: &[Agent]) -> Vec<Conversation> {
    let mut out = Vec::new();
    for subtree in conversations(agents) {
        let members: Vec<&Agent> = subtree.iter().filter_map(|r| agents.get(r.index)).collect();
        let root = members.first().copied();
        out.push(Conversation {
            name: display_name_of(agents, root.map_or("", |a| a.agent_id.as_str())),
            live: members
                .iter()
                .any(|a| running(a.state) || a.state_uncertain),
        });
    }
    out
}

/// The conversation's rendered subtree (root first, §2.3 descent order) — the
/// center's descent-tree source. Empty when `root_id` is not a root here.
pub fn members(agents: &[Agent], root_id: &str) -> Vec<DescentRow> {
    conversations(agents)
        .into_iter()
        .find(|subtree| {
            subtree
                .first()
                .and_then(|r| agents.get(r.index))
                .is_some_and(|a| a.agent_id == root_id)
        })
        .unwrap_or_default()
}

/// The conversation root an agent belongs to — the selected member's
/// conversation identity (§11 center header). `None` for an unknown id.
pub fn root_of(agents: &[Agent], agent_id: &str) -> Option<String> {
    for subtree in conversations(agents) {
        let root = subtree.first().and_then(|r| agents.get(r.index))?;
        if subtree
            .iter()
            .filter_map(|r| agents.get(r.index))
            .any(|a| a.agent_id == agent_id)
        {
            return Some(root.agent_id.clone());
        }
    }
    None
}

/// Segment the descent order into per-conversation subtrees: each depth-0 row
/// starts a conversation, its descendants follow it (pre-order, §2.3). Each
/// subtree's depths are its own — the root at 0 — so a slice of one is a
/// well-formed subtree in its own right ([`expand`]).
fn conversations(agents: &[Agent]) -> Vec<Vec<DescentRow>> {
    let mut out: Vec<Vec<DescentRow>> = Vec::new();
    for r in descent_order(agents) {
        if r.depth == 0 {
            out.push(vec![r]);
        } else if let Some(current) = out.last_mut() {
            current.push(r);
        }
    }
    out
}

fn running(state: AgentState) -> bool {
    matches!(state, AgentState::Live | AgentState::InFlight)
}

#[cfg(test)]
pub(crate) mod tests;
