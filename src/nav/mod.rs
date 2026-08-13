//! The altitude-0 navigator view-models (DESIGN §11): the workspace **tab
//! bar** ([`tabs`]) and the focused workspace's **conversation list**
//! ([`convs`]), the §11 context-menu seat roster ([`menu`]), plus the shared
//! row/key types. No egui and no git/ui_state
//! dependency: the caller ([`AppModel`](crate::AppModel)) derives the input
//! facts from the snapshot map + attention + the §3.5 join, and the shell
//! renders the outputs. Every branch is table-tested with plain data.

pub mod convs;
pub mod menu;
pub mod tabs;

use std::path::Path;

/// One bound ball rendered in the balls section (DESIGN §3.5, §11): its id and
/// the §3.5 join badge (e.g. "delivered"), the caller having resolved both via
/// [`crate::projects::join::badge`]. `badge` is `None` on a state needing none
/// (a plain Bound row).
///
/// The row also carries what **acting on it** needs (§11 ball-row menu): the
/// project its `bl` verbs run in, the claimant they stamp `--as` (§3.2), and the
/// §3.5 state the enablement predicates read. A row that can be right-clicked
/// must name its own object — re-deriving it from the focus is exactly what a
/// pointer-targeted menu may not do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundBall {
    pub id: String,
    pub badge: Option<String>,
    pub project: std::path::PathBuf,
    pub owner: String,
    pub state: crate::projects::join::JoinState,
}

/// The ui_state key for a workspace (its path) — the pin key, collapse key
/// prefix input, seen key, and focus key, one string everywhere (§4.1).
pub fn ws_key(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
