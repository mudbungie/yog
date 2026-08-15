//! One board row's derivation: the column, the gate, the drones, the figure.
//!
//! Split from [`super`] at the §12 budget. Everything here is a pure read of
//! the published snapshot — the walk that feeds the figure was the worker's
//! (`Snapshot::bills`, §3.5/§7.2), so a hundred-row board costs no disk.

use super::BoardRow;
use crate::app::Snapshot;
use crate::git_tree::GitTree;
use crate::projects::balls::{Ball, ladder};
use crate::projects::join::JoinRow;
use crate::spend::Prices;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

/// What holds a ball at its gate: the blocking ball, and what mints the gate —
/// that ball's own close (balls' `close` blocker edge, §10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    pub id: String,
    pub title: String,
}

/// A conversation working a ball. Named rather than re-rendered: `root_id` is
/// the very key `Query::Conversations` rows carry, so a seat shows the board's
/// drone as the conversation row it already paints (§11 — one object).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drone {
    pub root_id: String,
    /// The §3.3 display name — what the operator calls it.
    pub name: String,
}

/// Derive one row from its §3.5 join row (the binding) and its ball (the
/// ladder). `live`/`by_id` are the project's live set — the blocker resolver.
pub(super) fn row(
    snap: &Snapshot,
    prices: &Prices,
    join: &JoinRow,
    ball: &Ball,
    live: &HashSet<&str>,
    by_id: &HashMap<&str, &Ball>,
) -> BoardRow {
    let gates = gates(ball, by_id);
    // The binding is a §3.1 **name** since bl-b4b5; the derivations below read
    // the tree map, which is keyed by path, so the round trip is spelled once
    // here at the one seam that owns it (`Snapshot::ws_path`).
    let ws = join.workspace.clone();
    let path = ws.as_deref().and_then(|name| snap.ws_path(name).ok());
    let roots = path
        .as_deref()
        .map(|ws| stamped_roots(&snap.trees, ws, &ball.id))
        .unwrap_or_default();
    BoardRow {
        project: join.project.clone(),
        id: ball.id.clone(),
        title: ball.title.clone(),
        priority: ball.priority,
        column: super::column(ladder(ball, live), !gates.is_empty()),
        state: join.state,
        drones: path
            .as_deref()
            .map(|ws| drones(&snap.trees, ws, &roots))
            .unwrap_or_default(),
        spend: path
            .as_deref()
            .map(|ws| crate::spend::of_ball(&bills_of(snap, ws), &roots, prices)),
        rollup: super::rollup::of(snap, prices, &ball.id, by_id),
        workspace: ws,
        claimant: ball.claimant.clone(),
        parent: ball.parent.clone(),
        gates,
    }
}

/// The unresolved close-blockers, resolved exactly as the ladder resolves
/// claim-blockers: **the live set is the resolver** (a resolved ball's file is
/// gone, so it is simply absent from `by_id`). One rule, spelled once.
fn gates(ball: &Ball, by_id: &HashMap<&str, &Ball>) -> Vec<Gate> {
    ball.blockers
        .iter()
        .filter(|b| b.on == "close")
        .filter_map(|b| {
            by_id.get(b.id.as_str()).map(|g| Gate {
                id: b.id.clone(),
                title: g.title.clone(),
            })
        })
        .collect()
}

/// Every root in `ws` whose `goal.md` stamps `ball_id` (§3.3), deduplicated and
/// ordered. A stamp is resolved **to its root** before it counts: two stamps in
/// one descent are one tree, and summing both would bill it twice.
///
/// The one home for this question — [`crate::AppModel::stamped_roots`] and the
/// board both call it, so a ball's drones and a ball's spend are the same set
/// by construction rather than by agreement.
pub fn stamped_roots(trees: &HashMap<PathBuf, GitTree>, ws: &Path, ball_id: &str) -> Vec<String> {
    let Some(tree) = trees.get(ws) else {
        return Vec::new();
    };
    tree.agents
        .iter()
        .filter(|a| a.goal_ball.as_deref() == Some(ball_id))
        .filter_map(|a| crate::nav::convs::root_of(&tree.agents, &a.agent_id))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// The stamped roots as drone rows — the §3.3 display ladder applied to each.
fn drones(trees: &HashMap<PathBuf, GitTree>, ws: &Path, roots: &[String]) -> Vec<Drone> {
    let Some(tree) = trees.get(ws) else {
        return Vec::new();
    };
    roots
        .iter()
        .map(|root| Drone {
            root_id: root.clone(),
            name: crate::nav::convs::display_name_of(&tree.agents, root),
        })
        .collect()
}

/// One workspace's already-walked bills; empty for a workspace no pass reached.
pub(super) fn bills_of(snap: &Snapshot, ws: &Path) -> Vec<crate::budgets::StepBill> {
    snap.bills.get(ws).cloned().unwrap_or_default()
}
