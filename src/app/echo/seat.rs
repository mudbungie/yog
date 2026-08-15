//! **What the frame asks of the echo** (§3.4, §7.2; REMOTE §9.7) — the two
//! doors a seat folds an answer through, cut off [`super`] at §12's per-file
//! budget on the seam that module already draws: what an echo *is*, what it
//! stands in for and when it retires lives there; this is the orchestration a
//! frame performs with one.
//!
//! Both are orchestration, never derivation (§8.5's paint-side line): the rows
//! and the entries are the engine's, and the optimism is the seat's. An
//! unfocused window echoes nothing, because an echo belongs to the workspace it
//! was fired in and there is none to compare against.

use super::{Target, rows};
use crate::inboxview::InboxEntry;

impl crate::app::AppModel {
    /// The §11 list **as this seat paints it**: what the boundary answered for
    /// the focused workspace, with this window's own pending echo folded on
    /// (§3.4, §7.2). Orchestration, never derivation (§8.5's paint-side line):
    /// the rows are the engine's and the optimism is the seat's.
    ///
    /// An unfocused window echoes nothing, because an echo belongs to the
    /// workspace it was fired in and there is none to compare against.
    pub fn echoed(
        &self,
        rows: Vec<crate::nav::convs::ConvRow>,
        now_unix: i64,
    ) -> Vec<crate::nav::convs::ConvRow> {
        let Some(ws) = self.focused_workspace() else {
            return rows;
        };
        rows::with_echo(self.started.as_ref(), &ws, rows, now_unix)
    }

    /// **The composer's queue as this seat paints it** (§3.4, §5.1 #11;
    /// bl-b4b5) — one agent's answered `Query::Inbox` listing with this
    /// window's own echo folded on, when the echo is a message *to that agent*.
    ///
    /// The third projection of one fact, for [`rows::with_echo`]'s reason
    /// exactly: the queue reads a `Reply` now, and *a seat's optimism reaches
    /// whatever that seat actually reads* (bl-44e9). Without it a typed message
    /// would vanish for an ask period between Enter and the deposit file, which
    /// is the §11 faded-send ruling deleted at the one surface it exists for.
    ///
    /// It appends rather than reordering: the queue is oldest-first and the
    /// echo is the newest thing said. A **start**'s echo adds nothing here —
    /// its conversation has no id yet, so no seat is asking this question about
    /// it, and the row it does mint is [`rows::with_echo`]'s.
    pub fn echoed_pending(&self, agent: &str, pending: Vec<InboxEntry>) -> Vec<InboxEntry> {
        let Some(ws) = self.focused_workspace() else {
            return pending;
        };
        let mine = self
            .started
            .as_ref()
            .filter(|echo| echo.ws == ws && echo.target == Target::Agent(agent.to_owned()));
        match mine {
            Some(echo) => pending.into_iter().chain([echo.deposit()]).collect(),
            None => pending,
        }
    }
}
