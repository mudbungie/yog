//! **The census folds, over an answered forest** (REMOTE §9.7, bl-b4b5) —
//! [`expand::visible`](super::expand::visible) and
//! [`select::selection`](super::select::selection)'s third sibling: pure
//! selections out of a `Query::Conversations` reply, keyed neither by which rows
//! are open nor by which one is picked, but by *what a conversation contains*.
//!
//! Two seats wanted the same thing and each was asking `AppModel` for it: the
//! §3.6 delete gate (which conversations die, and is any of them live) and the
//! §3.3 mint (which names are taken). Both folded the engine's own agent set on
//! the paint thread; both are answered by rows this window already holds.
//!
//! **Depth is the containment**, exactly as it is the parentage in
//! [`select`](super::select): the answer is pre-order, so a row's subtree is the
//! run of deeper rows below it, and every member carries its own §3.5 state and
//! §10 uncertainty. Nothing here re-derives; it selects.

use super::{ConvRow, Conversation, running};

/// **The same, off an answered forest** (REMOTE §9.7, bl-b4b5) — [`liveness`]'s
/// seat-side twin, and the same kind of thing [`visible`] and [`selection`] are:
/// a pure fold over `Query::Conversations`' rows rather than over the engine's
/// agent set.
///
/// The answer is pre-order and every member carries its own §3.5 state and §10
/// uncertainty, so a root's subtree is exactly the run of deeper rows below it
/// and "is anything in this conversation live" is that run's disjunction — the
/// derivation `liveness` makes, made from what a seat holds. The two are pinned
/// equal by `delete::tests`, because two projections of one gate are two facts
/// waiting to disagree.
pub fn liveness_of_rows(rows: &[ConvRow]) -> Vec<Conversation> {
    let mut out: Vec<Conversation> = Vec::new();
    for (at, row) in rows.iter().enumerate() {
        if row.depth != 0 {
            continue;
        }
        out.push(Conversation {
            name: row.display_name(),
            live: subtree(rows, at).any(|r| running(r.state) || r.uncertain),
        });
    }
    out
}

/// The rows of `at`'s own subtree, itself first: the pre-order run of rows
/// below it that hang deeper than it does. The one rule every seat-side subtree
/// fold reads, so a member census and a liveness gate cannot disagree about
/// what a conversation contains.
pub(crate) fn subtree(rows: &[ConvRow], at: usize) -> impl Iterator<Item = &ConvRow> {
    let depth = rows.get(at).map_or(0, |r| r.depth);
    rows.get(at..)
        .unwrap_or_default()
        .iter()
        .enumerate()
        .take_while(move |(i, r)| *i == 0 || r.depth > depth)
        .map(|(_, r)| r)
}

/// **The §3.3 occupied name set off an answered forest** (§3.3, bl-b4b5) — what
/// the conversation mint may not re-use, as a seat reads it.
///
/// Every member counts, and must: litany refuses a name any living agent
/// already wears, so a mint that ignored a named child would fail at fire. A
/// row's `name` is the root-or-member's own `name_fact` — the very fold
/// `answer::names_in` collects — so this is one derivation addressed from the
/// other end, not a second one.
pub fn names_in_rows(rows: &[ConvRow]) -> Vec<String> {
    rows.iter().filter_map(|r| r.name.clone()).collect()
}

#[cfg(test)]
mod tests;
