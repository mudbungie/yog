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

use super::rows;
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
    /// echo is the newest thing said, and what it appends is **every** send the
    /// echo stands for ([`super::Echo::deposits`]) — the one that made it and
    /// each §3.4 held follow-up, in the order they were said.
    ///
    /// **A start's echo is folded here too** (bl-56c6). It used to be declined
    /// on the premise that *"its conversation has no id yet, so no seat is
    /// asking this question about it"* — a premise bl-2e8f had already
    /// invalidated by making the start focus its minted name: the composer aims
    /// at that name, asks `Query::Inbox` about it, is refused, and painted an
    /// **empty queue for the whole start window** — the operator's first
    /// message with no representation at the one seat §7.2 names as the seat
    /// they meant. A name and an id are two spellings of one target
    /// ([`super::Echo::addresses`]), and the seat asks about whichever it holds.
    ///
    /// **And it yields the moment the answer carries the deposit**
    /// ([`super::Echo::deposited`], bl-78d8). The §8.2 verb is piped, so a follow-up's
    /// file is on disk before the receipt that mints the echo; what lags is
    /// this seat's own ask, and the answer that catches up *is* the row. Folding
    /// unconditionally painted both — one solid, one faded, same words — until
    /// a whole-workspace derivation seconds later retired the echo, which is
    /// §7.2's "brightening is that same row at full strength" broken at the one
    /// seat it was written for.
    pub fn echoed_pending(&self, agent: &str, pending: Vec<InboxEntry>) -> Vec<InboxEntry> {
        let Some(ws) = self.focused_workspace() else {
            return pending;
        };
        let mine = self.started.as_ref().filter(|echo| {
            echo.ws == ws && echo.addresses(agent) && !echo.deposited(pending.len())
        });
        match mine {
            Some(echo) => pending.into_iter().chain(echo.deposits()).collect(),
            None => pending,
        }
    }
}
