//! The §11 conversation list of one workspace, and the two reads that hang off
//! it — split from [`answer`](super) at §12's budget (bl-c088) on the seam the
//! chokepoint's own shape already draws: everything left in `answer` is the
//! `Query` table and the resolutions standing ahead of it, and these three are
//! a *derivation* the table calls, beside [`chrome`](super::chrome)'s and
//! [`balls`](super::balls)'.
//!
//! They are one subject rather than three: the forest rows, the ball facts each
//! row renders, and the occupied name set the §3.3 mint checks are all reads of
//! the same workspace's derived tree, and [`conversations`] spends
//! [`conv_ball`] itself.

use crate::app::Snapshot;
use crate::nav::{self, convs::ConvBall, convs::ConvRow, ws_key};
use crate::projects::join;
use crate::ui_state::UiState;

use std::path::Path;

/// The §11 conversation list of one workspace, at the **forest** altitude
/// (REMOTE §9.7, bl-44e9): every member of the descent forest with its own
/// per-row rollups, in paint order. Aimed by parameter instead of focus; a
/// workspace with no derived tree is simply empty (§3.3's general path).
///
/// **This is the whole answer and it carries no fold.** A viewport's expanded
/// set is a view (§8.5: *views gain no boundary representation*), so it never
/// crosses and never rides a row — each seat selects its own visible rows out of
/// this with [`nav::convs::visible`], and a seat holding no fold at all selects
/// the root subset, which is the all-collapsed list this query used to answer.
pub fn conversations(snap: &Snapshot, ui: &UiState, ws: &Path, now_unix: i64) -> Vec<ConvRow> {
    let Some(tree) = snap.trees.get(ws) else {
        return Vec::new();
    };
    let key = ws_key(ws);
    let seen = |k, w: &str, a: &str, o: &str| ui.is_seen(k, w, a, o);
    let ball = |id: &str| conv_ball(snap, id);
    // The standing verdicts, read off the same published ops tail the §11 pane
    // renders (VISION §4.9): a derivation per build, not a field on the world.
    let checks = crate::monitor::row::of_rows(&snap.ops);
    nav::convs::forest_rows(&tree.agents, &key, &seen, now_unix, &ball, &checks)
}

/// Resolve a conversation's goal-stamp ball `id` to its render facts (§3.3,
/// §3.5): the id always renders; the join supplies status/title/badge when a
/// row matches, else those stay `None` — a pure read over the cached join.
pub fn conv_ball(snap: &Snapshot, id: &str) -> ConvBall {
    match snap.join_rows.iter().find(|r| r.ball_id == id) {
        Some(r) => ConvBall {
            id: id.to_owned(),
            state: Some(r.state),
            title: r.title.clone(),
            badge: join::badge(r.state, r.claimant.as_deref()),
        },
        None => ConvBall {
            id: id.to_owned(),
            state: None,
            title: None,
            badge: None,
        },
    }
}

/// The conversation mint's occupied set for a workspace (§3.3): the names its
/// living agents wear — each agent's `name_fact`, the litany-stored blob with
/// the legacy goal-stamp fallback while pre-0.0.4 roots live. Children count
/// too, and must: litany refuses a name any living agent already wears, so a
/// mint that ignored a named child would fail at fire. Empty for an underived
/// workspace — the general path with no inputs.
pub fn names_in(snap: &Snapshot, ws: &Path) -> Vec<String> {
    snap.trees
        .get(ws)
        .into_iter()
        .flat_map(|t| {
            t.agents
                .iter()
                .filter_map(crate::git_tree::Agent::name_fact)
        })
        .collect()
}
