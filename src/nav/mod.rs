//! The altitude-0 navigator view-models (DESIGN §11): the workspace **tab
//! bar** ([`tabs`]) and the focused workspace's **conversation list**
//! ([`convs`]), plus the shared
//! row/key types. No egui and no git/ui_state
//! dependency: the caller ([`AppModel`](crate::AppModel)) derives the input
//! facts from the snapshot map + attention + the §3.5 join, and the shell
//! renders the outputs. Every branch is table-tested with plain data.

pub mod balls;
pub mod convs;
pub mod tabs;

use std::path::Path;

/// One bound ball rendered in the balls section (DESIGN §3.5, §11): its id and
/// the §3.5 join badge (e.g. "delivered"), the caller having resolved both via
/// [`crate::projects::join::badge`]. `badge` is `None` on a state needing none
/// (a plain Bound row).
///
/// The row also carries what **acting on it** needs: the project its `bl` verbs
/// run in, the claimant they stamp `--as` (§3.2), and the §3.5 `state` a seat
/// reads the assign/release/close gates off (bl-33e9 — the gates are derivable
/// from the row, so REMOTE §9.4 leaves them to the seat and this crate holds
/// none of them). A row a seat can act on must name its own object — re-deriving
/// it from the focus is exactly what a pointer-targeted gesture may not do.
///
/// **It is `Query::WorkspaceBalls`' answer row since bl-b4b5**, not a
/// frame-side projection: `project` is the §5.1 #1 wire name a `bl` verb takes
/// rather than the clone's path, and the §3.5 [`spend`](Self::spend) figure
/// rides here rather than being asked per row — one workspace's balls and what
/// each has cost are one question, and the seat that paints the strip and the
/// seat that paints the figures were two reads of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundBall {
    pub id: String,
    pub badge: Option<String>,
    pub project: String,
    pub owner: String,
    pub state: crate::projects::join::JoinState,
    /// The ball's priced figure as this workspace can attribute it (§3.5's
    /// ruling): the conversations whose goal stamps it when any does, else the
    /// whole workspace, the figure's own `attribution` saying which.
    pub spend: crate::spend::Figure,
}

/// The ui_state key for a workspace (its path) — the pin key, collapse key
/// prefix input, seen key, and focus key, one string everywhere (§4.1).
pub fn ws_key(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
