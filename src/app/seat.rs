//! What a **seat** reads about its own selection (REMOTE §9.4, bl-1eb0) — the
//! frame's half of the client/server line.
//!
//! The line this file draws: [`AppModel`] may **orchestrate** — hold the focus,
//! resolve it against the published snapshot, memoize, hand the frame something
//! it can paint without waiting — but what crosses into paint is a payload the
//! wire could carry. Before this, the centre pane took `focused_tree()` and
//! `focused_agent()` and derived names, marks, liveness and verb gates out of
//! `GitTree` on the frame thread; a thin client holds no `GitTree` and never
//! will, so every one of those seats was window-only by construction.
//!
//! Nothing here derives anything. Each accessor resolves the focus and folds
//! the snapshot the frame is already holding.
//!
//! **What is left is one accessor** (bl-b4b5). The composer's pending queue,
//! the §3.3 title table and the §2.2 lineage tip left with the accessory tail:
//! `focused_pending` is `Query::Inbox`' answer (the §11 Inbox tab's own standing
//! question, so the two seats are one ask), `agent_titles` is `Titles::of_rows`
//! over the landed forest, and `config_tip` is a field on `Query::Workspaces`'
//! row. Each was a fold of the engine's `GitTree` on the paint thread; none
//! needed a question of its own.
//!
//! **`focused_conversation` is gone** (bl-48ae). It was the last in-process read
//! of REMOTE §11's residual — the whole seat view, re-derived per frame off the
//! window's own snapshot — and it did not become a standing question but a
//! *split*: the facts that name the target or gate a gesture are picked out of
//! the landed `Query::Conversations` forest, and the selection's own detail is a
//! standing `Query::Agent`. Both halves live at the seat
//! ([`crate::shell::seat`]), which is where the ruling about what each may cost
//! is written.
//!
//! The two facts that stay in RAM rather than crossing: the focus itself (§13.1
//! — per-instance, never durable) and the viewport's folds (§5.3). A seat's
//! *selection* is not a world fact, which is why it is the parameter these
//! answers take rather than something the boundary is asked for.

use super::AppModel;

impl AppModel {
    /// The selected conversation's id (§11 Altitude-2), owned — the target
    /// every gesture on this surface names. `None` with nothing selected, and
    /// for a selection the published snapshot does not carry (a moved or
    /// unfetched tree).
    pub fn focused_agent_id(&self) -> Option<String> {
        self.focused_agent().map(|a| a.agent_id.clone())
    }
}
