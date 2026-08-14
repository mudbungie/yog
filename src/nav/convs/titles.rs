//! The §3.3 display ladder as a **seat** holds it (REMOTE §9.4, bl-1eb0): one
//! `(agent id, title)` pair per agent in a workspace, so a face that paints a
//! name it did not derive needs no agent set at all.
//!
//! Every seat that renders somebody's *name* — a deposit's sender, a
//! conversation's target, a transcript speaker — was asking
//! [`display_name_of`](super::display_name_of) against the frame's own
//! `&[Agent]`. That slice is the engine's tree derivation: fat, disk-derived,
//! and unspellable on a wire, which is exactly what REMOTE §9.4 retires from
//! paint code. The ladder itself does not move — [`super::display_name`] stays
//! its one home — only its *input* narrows to the two strings the answer is
//! made of.
//!
//! **It is a projection of an answer a seat already holds**: `root_id` +
//! `display` is every [`ConvRow`] pair, so [`Titles::of_rows`] builds the same
//! table out of a decoded [`Conversations`](crate::boundary::Query::Conversations)
//! reply. Nothing here is a second fact — it is one fact addressed by id.

use super::{ConvRow, id_floor, member_title};
use crate::git_tree::Agent;

/// A workspace's agents paired with what each is called (§3.3).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Titles {
    /// `(agent id, title)`, in the order the roster was read.
    pub rows: Vec<(String, String)>,
}

impl Titles {
    /// The table off the engine's own agent set — the derivation side.
    pub fn of(agents: &[Agent]) -> Self {
        Self {
            rows: agents
                .iter()
                .map(|a| (a.agent_id.clone(), member_title(a)))
                .collect(),
        }
    }

    /// The same table off a [`ConvRow`] listing — the seat side. A row's
    /// `display` **is** the ladder's answer for its own agent
    /// ([`ConvRow::display_name`]), so a fully-unfolded conversations reply
    /// carries this whole table already.
    pub fn of_rows(rows: &[ConvRow]) -> Self {
        Self {
            rows: rows
                .iter()
                .map(|r| (r.root_id.clone(), r.display_name()))
                .collect(),
        }
    }

    /// What `id` is called: its title when this roster carries it, else the
    /// ladder's floor — [`id_floor`]'s terminal generation, the same last rung
    /// every other seat lands on for an id nobody here holds (`user`, a foreign
    /// or deleted agent).
    pub fn name(&self, id: &str) -> String {
        self.rows
            .iter()
            .find(|(agent, _)| agent == id)
            .map_or_else(|| id_floor(id).to_owned(), |(_, title)| title.clone())
    }
}
