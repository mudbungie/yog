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
use crate::inboxview::InboxEntry;
use crate::model_pick::ConfigTip;
use crate::nav::convs::Titles;

impl AppModel {
    /// The selected conversation's id (§11 Altitude-2), owned — the target
    /// every gesture on this surface names. `None` with nothing selected, and
    /// for a selection the published snapshot does not carry (a moved or
    /// unfetched tree).
    pub fn focused_agent_id(&self) -> Option<String> {
        self.focused_agent().map(|a| a.agent_id.clone())
    }

    /// The selected conversation's undelivered deposits (§5.1 #11), oldest
    /// first — the composer's pending queue and the `✉n` badge's listing.
    ///
    /// Off the **snapshot**, not disk: the listing is gathered when the tree is
    /// (§3.5 stateless re-read) precisely so the render path never stats, which
    /// is why this is not the [`Inbox`](crate::boundary::Query::Inbox) answer's
    /// own `list_inbox` call. Same rows, one tick older, no I/O on the frame.
    pub fn focused_pending(&self) -> Vec<InboxEntry> {
        self.focused_agent()
            .map(|a| a.pending.clone())
            .unwrap_or_default()
    }

    /// The §3.3 ladder for every agent in the focused workspace — what a seat
    /// resolves a *third party's* id against: a deposit's sender, in the two
    /// places mail is painted. One table per frame, so no seat holds the agent
    /// set to answer the same question per row.
    pub fn agent_titles(&self) -> Titles {
        self.focused_tree()
            .map(|tree| Titles::of(&tree.agents))
            .unwrap_or_default()
    }

    /// The focused workspace's config-lineage tip (§2.2) — the commit
    /// `lernie prompt` forks the next conversation off, which is what the §11
    /// model picker shows and what a pick advances. `None` for a workspace with
    /// no config lineage derived yet.
    pub fn config_tip(&self) -> Option<ConfigTip> {
        self.focused_tree()?.commits.last().map(|c| ConfigTip {
            oid: c.oid.clone(),
            short_oid: c.short_oid.clone(),
        })
    }
}
