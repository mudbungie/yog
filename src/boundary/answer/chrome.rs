//! **The altitude-0 chrome** (§11, REMOTE §9.7) — what
//! [`Query::Workspaces`](crate::boundary::Query::Workspaces) answers, split off
//! the chokepoint at §12's per-file budget (bl-b4b5) on the seam the roster
//! already draws: [`super`] is the table of which derivation answers which
//! query, and this is one of them.
//!
//! The answer grew twice in this family's own idiom — a payload gains a field,
//! never a near-duplicate question. bl-296f put the §4.1 pin rank on the row so
//! the tab bar could hoist without re-reading `ui.json`; bl-b4b5 put the §2.2
//! lineage tip beside it and the §7.2 currency of the derivation itself on the
//! answer, because this is the one read every window makes every frame and the
//! staleness of what a seat is showing costs it nothing to say.

use std::path::Path;

use crate::app::Snapshot;
use crate::attention;
use crate::git_tree::AgentState;
use crate::nav::ws_key;
use crate::ui_state::UiState;

use crate::boundary::reply::WsRow;

/// **The altitude-0 chrome** — the `workspaces` answer in full (REMOTE §9.7,
/// bl-b4b5): the enumeration below, plus the §7.2 currency of the derivation it
/// was made from. `now_unix` is the caller's wall clock; the snapshot carries
/// its completion in the same unit
/// ([`derived_at_unix`](Snapshot::derived_at_unix)), so the age is a
/// subtraction here rather than an `Instant` no wire could carry.
///
/// A derivation stamped **after** the caller's clock — a fake clock wound back
/// under a test, two boxes disagreeing by a second — is not stale, and reads as
/// an age of zero rather than as a negative one.
pub fn workspaces(
    snap: &Snapshot,
    ui: &UiState,
    now_unix: i64,
) -> crate::boundary::reply::Workspaces {
    let age = std::time::Duration::from_secs(
        now_unix
            .saturating_sub(snap.derived_at_unix)
            .try_into()
            .unwrap_or(0),
    );
    crate::boundary::reply::Workspaces {
        rows: ws_rows(snap, ui),
        stale: crate::app::stale_label(age, snap.cadence.stale_after()),
        growth: crate::app::growth_label(&snap.growth),
    }
}

/// Every enumerated workspace with its rollup — the `workspaces` answer's rows.
pub fn ws_rows(snap: &Snapshot, ui: &UiState) -> Vec<WsRow> {
    // The §4.1 pin list, read once for the whole listing rather than per row.
    // Its keys are paths (durable state whose re-keying is its own migration,
    // bl-7407), which is exactly why the *rank* crosses and the key does not.
    let pinned = ui.pinned();
    snap.workspaces
        .iter()
        .map(|w| {
            let (attention, agents, running) = workspace_stats(snap, ui, &w.path);
            let key = ws_key(&w.path);
            WsRow {
                workspace: crate::naming::leaf(&w.path),
                kind: w.kind.clone(),
                attention,
                agents,
                running,
                pinned: pinned.iter().position(|k| *k == key),
                // The §2.2 lineage tip (§9.4): the newest commit of the
                // `HEAD` → `config/default` walk the tree derivation already
                // made — two strings off a fold nobody re-runs.
                config_tip: snap
                    .trees
                    .get(&w.path)
                    .and_then(|t| t.commits.last())
                    .map(|c| crate::model_pick::ConfigTip {
                        oid: c.oid.clone(),
                        short_oid: c.short_oid.clone(),
                    }),
            }
        })
        .collect()
}

/// One workspace's §6 rollup: attention-bearing agents, agent count, whether
/// anything runs — the tab bar's numbers, by parameter.
pub fn workspace_stats(snap: &Snapshot, ui: &UiState, ws: &Path) -> (usize, usize, bool) {
    let Some(tree) = snap.trees.get(ws) else {
        return (0, 0, false);
    };
    let key = ws_key(ws);
    let seen = |k, w: &str, a: &str, o: &str| ui.is_seen(k, w, a, o);
    let mut count = 0;
    let mut running = false;
    for a in &tree.agents {
        if attention::attention(a, &key, &seen).any() {
            count += 1;
        }
        running |= matches!(a.state, AgentState::Live | AgentState::InFlight);
    }
    (count, tree.agents.len(), running)
}
