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
    /// How §3.1 classifies it: yog's own, lernie's, or a read-only replay.
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
}
