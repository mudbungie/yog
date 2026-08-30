//! **The one listing row the boundary itself owns** (§8.5) — the
//! [`Query::Workspaces`](crate::boundary::Query::Workspaces) answer's element,
//! split off the [reply roster](super) at §12's per-file budget.
//!
//! Every other listed thing is somebody else's type read back out of its own
//! module — a `ConvRow` is `nav`'s, a `JoinRow` is `projects`', an `OpRow` is
//! `opslog`'s — and this is the one whose *subject* exists only as an answer:
//! a workspace named, classified, rolled up and ranked is a thing no seat and
//! no derivation holds until one is asked for. Its two spellings live in
//! [`rows`](super::rows) beside every other row's, unchanged.

use crate::binding::WorkspaceKind;

/// One workspace row (§3.1 classification + the §6 rollups the tab bar shows):
/// the [`Query::Workspaces`](super::Query::Workspaces) answer's element.
///
/// **It names the workspace, it does not locate it** (REMOTE §8, bl-f5f6). It
/// carried the whole [`Workspace`] — path and all — and a path is meaningless
/// on a thin client and a disclosure besides. §3.1 makes the leaf the name, so
/// the `name` this row used to carry *beside* the path became the row's whole
/// identity and the path went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsRow {
    /// The workspace's name (§3.1) — the token every gesture addresses it by.
    pub workspace: String,
    /// How §3.1 classifies it: yog's own, litany's, or a read-only replay.
    pub kind: WorkspaceKind,
    /// Attention-bearing agents in it (§6).
    pub attention: usize,
    /// Root-and-member agent count.
    pub agents: usize,
    /// Whether anything in it is Live/InFlight right now.
    pub running: bool,
    /// **Where the operator pinned it** (§4.1 `pinned`, REMOTE §9.7 class 2;
    /// bl-296f): its rank in the durable pin list, `None` for a workspace that
    /// is not pinned.
    ///
    /// A pin is **durable operator state**, not a viewport fold, which is what
    /// makes it a lawful field here where the §11 expanded set is not (DESIGN
    /// §8.5, §5.3): it lives in `ui.json` beside the §6 acknowledgements this
    /// row's [`attention`](Self::attention) already folds, and the chokepoint
    /// reads that document anyway. It rides as a **rank** rather than a flag
    /// because the tab bar hoists pinned tabs *in pin order*, and a seat given
    /// only a boolean would have to re-read the pin list to sort them — which
    /// is the seat joining an answer back against the engine's own document,
    /// the exact shape bl-7407 refused for the workspace path.
    pub pinned: Option<usize>,
    /// **The §2.2 config-lineage tip** (§9.4, bl-1eb0's named residual; closed
    /// bl-b4b5): the commit `litany prompt` forks the next conversation in this
    /// workspace off — what the §11 model picker calls the *workspace default*
    /// and what a pick advances. `None` for a workspace with no lineage derived
    /// yet, which paints no row rather than a row about nothing.
    ///
    /// A field rather than a query of its own, because it is a fact **about a
    /// workspace** exactly as [`running`](Self::running) is, and this is the
    /// question that answers those. The §9.4 drift clause reads it against the
    /// conversation's own frozen commit (`Query::Governing`), which is the
    /// *other* fact and stays where it is.
    pub config_tip: Option<crate::model_pick::ConfigTip>,
}

/// **The altitude-0 chrome, as one answer** (REMOTE §9.7, bl-b4b5) — the
/// [`Query::Workspaces`](crate::boundary::Query::Workspaces) reply: the
/// enumeration, and how current the derivation behind it is.
///
/// The two notes are the §7.2 instrumentation the §11 activity accessory paints
/// above its chip, and they were the last in-process reads in the tail
/// (`AppModel::staleness`, `AppModel::growth_note`). They ride **here** rather
/// than on a question of their own for bl-296f's own rule — a payload grows by
/// a field, never by a near-duplicate question — and the seam is exact: this is
/// the one read every window makes on every frame, so the currency of an answer
/// costs the wire nothing to say.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Workspaces {
    /// Every enumerated workspace with its rollups.
    pub rows: Vec<WsRow>,
    /// **How stale the derivation this answer came from is** (§7.2), or `None`
    /// while it is current — which is the ordinary case and renders nothing.
    ///
    /// It crosses as the rendered line rather than as an age and a threshold,
    /// for [`FlightStrip::facts`](crate::nav::convs::FlightStrip)' reason: the
    /// wording is one derivation's (`app::drift::stale_label`) with a bound the
    /// operator tunes in `cadence.yaml`, and a wire spelling of the parts would
    /// be a second place that decides when a derivation is late.
    pub stale: Option<String>,
    /// **What grew since the previous derivation** (§7.2, bl-ee0a), or `None`
    /// when nothing did. A dispatch storm is a fact about a *conversation*, and
    /// the line names the biggest grower and counts the rest.
    pub growth: Option<String>,
}
