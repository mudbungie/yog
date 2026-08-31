//! The §11 **unfold** (bl-fa82; re-cut at the boundary's altitude by bl-44e9):
//! the conversation list's rows over the descent forest, and the seat-side fold
//! that picks the visible ones out of them.
//!
//! The reframe this module is: **a list row is the subtree rooted at its
//! agent**, and the root-only list every earlier version painted is the
//! all-collapsed case — [`super::build`] is literally [`visible`] over the whole
//! forest with an empty set. Expansion reveals a row's direct children as rows
//! of the same anatomy, recursively, so there is no second row kind and no
//! child-rendering path.
//!
//! **The derivation and the fold are two functions, and that is REMOTE §9.7's
//! altitude ruling** (bl-44e9). [`forest_rows`] answers the whole forest with
//! its per-row rollups — the shape a `Reply::Conversations` carries — and
//! [`visible`] is the *seat's* pure selection out of that answer, over rows
//! rather than over a snapshot. So the fold never crosses the boundary (DESIGN
//! §8.5: *views gain no boundary representation*), one derivation still serves
//! every seat, and a seat with no fold at all selects the root subset.
//!
//! A pre-order walk
//! that stops descending at a node the caller's set does not name, with the set
//! passed **in** rather than stored — expansion is viewport ephemera (§5.3,
//! §13.1) and never rides a [`ConvRow`].
//!
//! The **mutation** is that module's too: a click on the §11 subagent field
//! flips one id's membership in that set, the crate's one
//! disclosure-set toggle, already shared by the transcript and queue folds. A
//! second spelling of "flip a membership" would be a second thing to keep
//! honest, so this module only ever *reads* a set.
//!
//! Membership is §5.1 #8's strict descent-id rule ([`descent_order`] /
//! [`children_of`](crate::git_tree::children_of)), never the looser prefix test
//! the Stop menu's `+children` seat uses.

use std::collections::HashSet;

use super::row::{ConvRow, row};
use super::{ConvBall, conversations};
use crate::git_tree::{Agent, DescentRow, descent_order};
use crate::monitor::Check;
use crate::ui_state::SeenKind;

/// The §11 list at its **whole altitude**: every member of the workspace's
/// descent forest, each projected over its own subtree, in paint order. The
/// answer a `Reply::Conversations` carries (REMOTE §9.7) — no fold reaches it,
/// because the fold is the seat's ([`visible`]).
///
/// Order is §11's, both halves of it: depth-0 subtrees by **recency alone,
/// descending, then root id** for the deterministic tail (I9, bl-cad5), and
/// within a subtree the §2.3 descent order [`descent_order`] already yields
/// (id-sorted siblings, each one's children directly beneath it). Every fold of
/// it is therefore a contiguous *subsequence*, which is what makes [`visible`] a
/// selection rather than a second sort.
pub fn forest_rows(
    agents: &[Agent],
    ws: &str,
    seen: &dyn Fn(SeenKind, &str, &str, &str) -> bool,
    now_unix: i64,
    ball: &dyn Fn(&str) -> ConvBall,
    checks: &[Check],
) -> Vec<ConvRow> {
    let mut convs: Vec<(i64, Vec<ConvRow>)> = Vec::new();
    for subtree in conversations(agents) {
        let mut rows = Vec::new();
        let mut last_active = 0;
        for at in 0..subtree.len() {
            let (t, r) = row(
                agents,
                slice_at(&subtree, at),
                ws,
                seen,
                now_unix,
                ball,
                checks,
            );
            // The conversation's sort key is its **root's** — the subtree fold
            // (bl-cad5), which is the depth-0 row and no other. Unfolding a
            // conversation may not move it in the list.
            if at == 0 {
                last_active = t;
            }
            rows.push(r);
        }
        convs.push((last_active, rows));
    }
    convs.sort_by(|(ta, a), (tb, b)| tb.cmp(ta).then_with(|| head_id(a).cmp(&head_id(b))));
    convs.into_iter().flat_map(|(_, rows)| rows).collect()
}

/// **The seat's fold** (REMOTE §9.7): the rows of an answered forest that this
/// viewport's `expanded` set makes visible, in the order the answer gave them.
///
/// A row is painted unless some ancestor of it is shut, and the cut is by
/// **depth** — pre-order says a row's ancestors are exactly the shallower rows
/// above it — so one pass needs no recursion, no stack and no snapshot. A row
/// whose id the set does not name closes over its whole descent; an empty set is
/// therefore the root subset, which is the all-collapsed list a seat with no
/// viewport at all reads.
pub fn visible(rows: &[ConvRow], expanded: &HashSet<String>) -> Vec<ConvRow> {
    let mut out = Vec::new();
    // The depth of the collapsed row we are currently skipping beneath, if any.
    let mut cut: Option<usize> = None;
    for r in rows {
        match cut {
            Some(depth) if r.depth > depth => continue,
            _ => cut = None,
        }
        out.push(r.clone());
        if !expanded.contains(&r.root_id) {
            cut = Some(r.depth);
        }
    }
    out
}

/// A conversation's root id — the sort's deterministic tie-break. Empty for the
/// unreachable rowless conversation ([`conversations`] emits none).
fn head_id(rows: &[ConvRow]) -> String {
    rows.first().map(|r| r.root_id.clone()).unwrap_or_default()
}

/// The contiguous pre-order slice rooted at `subtree[at]`: that row, then every
/// row beneath it — the ones deeper than it, up to the next row at its own
/// depth or shallower. Total by construction rather than by a guard: an
/// out-of-range position takes depth 0, cuts at the very next row and then
/// slices nothing, which is the empty answer without a branch to reach for it
/// (no caller is out of range — the positions come from [`visible_indices`]
/// over this same slice).
fn slice_at(subtree: &[DescentRow], at: usize) -> &[DescentRow] {
    let depth = subtree.get(at).map_or(0, |r| r.depth);
    let rest = subtree.get(at + 1..).unwrap_or_default();
    let len = rest
        .iter()
        .position(|r| r.depth <= depth)
        .unwrap_or(rest.len());
    subtree.get(at..=at + len).unwrap_or_default()
}

/// Step `delta` rows (±1 for ↓/↑, §11) from `selected` through the **visible**
/// rows in paint order, wrapping. The walk never expands and never collapses:
/// a collapsed subtree contributes one row here, so `↓` from a collapsed parent
/// lands on the next row at the same level and `↓` after a `→` enters the first
/// child — the operator's ruling with no branch to implement it (bl-fa82).
/// A `None`/unknown selection starts before the front, so `+1` lands on the
/// first row and `-1` on the last; an empty list yields `None`.
pub fn step(rows: &[ConvRow], selected: Option<&str>, delta: isize) -> Option<String> {
    let n = rows.len();
    if n == 0 {
        return None;
    }
    let here = selected.and_then(|id| rows.iter().position(|r| r.root_id == id));
    let next = match here {
        Some(i) => (i as isize + delta).rem_euclid(n as isize) as usize,
        None if delta >= 0 => 0,
        None => n - 1,
    };
    rows.get(next).map(|r| r.root_id.clone())
}

/// The row `id` hangs under, read off the painted rows themselves: the nearest
/// row **above** it at a shallower depth (§11's `←` paging back up to the last
/// level). Purely structural — a visible row's parent is visible by
/// construction, so no snapshot lookup is needed. `None` at depth 0 and for an
/// id this list does not paint.
pub fn parent_of(rows: &[ConvRow], id: &str) -> Option<String> {
    let at = rows.iter().position(|r| r.root_id == id)?;
    let depth = rows.get(at)?.depth;
    rows.get(..at)?
        .iter()
        .rev()
        .find(|r| r.depth < depth)
        .map(|r| r.root_id.clone())
}

/// The descent-id chain above `agent_id`, outermost first — what a **jump**
/// expands so its landing is on a visible row (§11's visible-selection
/// invariant, §6: arriving somewhere you cannot see leaves *why am I here*
/// unanswered). Read off [`descent_order`] rather than re-deriving the grammar,
/// so it is the same parentage the list itself renders. Empty for a root, and
/// for an id this snapshot does not carry.
///
/// The walk carries the open chain as a stack, truncated to each row's depth —
/// pre-order says every row's ancestors are exactly the rows above it that are
/// shallower, so the truncation *is* the parentage and nothing re-derives it.
pub fn ancestors(agents: &[Agent], agent_id: &str) -> Vec<String> {
    let mut stack: Vec<String> = Vec::new();
    for r in descent_order(agents) {
        // Total by the same reasoning as everywhere else here: a `DescentRow`
        // indexes the slice it was built from, so the miss is unreachable and
        // needs no arm of its own.
        let id = agents.get(r.index).map_or("", |a| a.agent_id.as_str());
        stack.truncate(r.depth);
        if id == agent_id {
            return stack;
        }
        stack.push(id.to_owned());
    }
    Vec::new()
}
