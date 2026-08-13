//! The epic-tree spend rollup (DESIGN §3.5's recorded follow-on, bl-9dd4).
//!
//! §3.5 fixed both the shape and the reason: *"a rollup crosses workspaces (a
//! ball's children may be claimed anywhere), so its enumeration source is the
//! board's join, not this one."* So it is here, over `Snapshot`, and not in
//! [`crate::spend`], which knows one workspace at a time.
//!
//! Two rules make it honest.
//!
//! **The tree is the live one.** Descendants are read off balls' own `parent`
//! pointer — the containment fact, no index — and a closed ball has no file, so
//! it leaves the live set and its spend leaves the rollup with it. The figure
//! therefore says what the epic is *still* spending, which is a true statement;
//! reconstructing the closed subtree from the on-demand closed listing would
//! make the number depend on whether that listing had been fetched, which is
//! not a property a figure may have.
//!
//! **A whole-workspace slice absorbs the tree slices inside it.** §3.5 accepts
//! workspace-granularity attribution for a ball claimed mid-conversation. Two
//! such balls in one workspace are each "the whole workspace", and summing both
//! would bill that workspace twice. So each workspace contributes exactly once:
//! whole, if any member there attributes workspace-wide, else the union of the
//! stamped roots. Summing an upper bound is still an upper bound; summing it
//! twice is just wrong.

use crate::app::Snapshot;
use crate::projects::balls::Ball;
use crate::spend::{Attribution, Figure, Prices};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

/// Every live descendant of `id` within one project's live set, transitively,
/// by balls' `parent` pointer. Excludes `id` itself. Ordered and deduplicated,
/// and a parent cycle terminates because a ball is visited at most once.
pub fn descendants(id: &str, by_id: &HashMap<&str, &Ball>) -> Vec<String> {
    let mut children: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for ball in by_id.values() {
        if let Some(parent) = ball.parent.as_deref() {
            children.entry(parent).or_default().push(&ball.id);
        }
    }
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut frontier = vec![id];
    while let Some(next) = frontier.pop() {
        for child in children.get(next).into_iter().flatten() {
            if seen.insert((*child).to_owned()) {
                frontier.push(child);
            }
        }
    }
    seen.into_iter().collect()
}

/// The rollup figure for `id`: itself plus its live descendants, folded across
/// every workspace any of them is bound to. `None` twice over — when the ball
/// has no live descendants (a leaf's rollup is its own figure, and a second
/// copy of a number is not a fact), and when nothing in the subtree is bound
/// anywhere (there is no spend to roll up, which is different from zero).
pub(super) fn of(
    snap: &Snapshot,
    prices: &Prices,
    id: &str,
    by_id: &HashMap<&str, &Ball>,
) -> Option<Figure> {
    let mut members = descendants(id, by_id);
    if members.is_empty() {
        return None;
    }
    members.push(id.to_owned());
    fold(snap, prices, &members)
}

/// One workspace's contribution: the stamped roots to keep, or `whole` — the
/// §3.5 workspace-granularity arm, which subsumes any roots beside it.
#[derive(Default)]
struct Slice {
    whole: bool,
    roots: BTreeSet<String>,
}

/// Fold the members into one figure. Per workspace, one slice; per slice, the
/// bills its roots claim (or all of them when the slice is whole); then the one
/// [`crate::spend::figure`] fold over the concatenation.
fn fold(snap: &Snapshot, prices: &Prices, members: &[String]) -> Option<Figure> {
    let mut slices: BTreeMap<PathBuf, Slice> = BTreeMap::new();
    for row in &snap.join_rows {
        let Some(ws) = row.workspace.clone() else {
            continue;
        };
        if !members.contains(&row.ball_id) {
            continue;
        }
        let roots = super::stamped_roots(&snap.trees, &ws, &row.ball_id);
        let slice = slices.entry(ws).or_default();
        if roots.is_empty() {
            slice.whole = true;
        }
        slice.roots.extend(roots);
    }
    if slices.is_empty() {
        return None;
    }
    let mut bills = Vec::new();
    let mut roots = 0;
    let mut whole = false;
    for (ws, slice) in &slices {
        let keep: Vec<String> = if slice.whole {
            whole = true;
            Vec::new()
        } else {
            roots += slice.roots.len();
            slice.roots.iter().cloned().collect()
        };
        bills.extend(crate::spend::select(
            &super::rows::bills_of(snap, ws),
            &keep,
        ));
    }
    let attribution = if whole {
        Attribution::Workspace
    } else {
        Attribution::Conversations(roots)
    };
    Some(crate::spend::figure(&bills, prices, attribution))
}
