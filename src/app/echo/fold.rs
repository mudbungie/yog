//! **The echo folded into the SNAPSHOT** (§7.2) — the projection every seat
//! that reads `AppModel::snap` gets, cut off [`super`] at §12's per-file budget
//! on the seam that module already draws three ways (bl-78d8). Its siblings are
//! [`rows`](super::rows) (the same optimism over an answered §11 list) and
//! [`seat`](super::seat) (the two `AppModel` doors); what an echo *is* and when
//! it retires stays in [`super`].
//!
//! One file per altitude, and still one module owning every fold: *"what does a
//! frame see that disk does not say?"* has one place to read.

use super::{Echo, Snapshot, index_of};
use std::path::Path;
use std::sync::Arc;

/// **The one place the derivation and the non-derived facts meet** (§7.2): the
/// snapshot a frame paints is the worker's, with the pending `echo` and the
/// §3.4 `raised` wall ([`crate::app::raise`]) folded in. Every render seat reads
/// the result and neither of them knows either exists.
///
/// **There were three** until bl-73e7: the focused conversation's live tail was
/// folded here too, by a follower thread in the window. It is an *answer* now
/// (`wire::lane`, spliced at `Transcript::with_live`), so the fold that reached
/// every seat by writing `Agent::stream` reaches the one seat that paints it by
/// being asked for — which is what the remote split already made true of every
/// other §11 read.
///
/// The two are folded here rather than each somewhere convenient because that
/// is the whole partition: **one function writes the painted snapshot**, so
/// "what does a frame see that disk does not say?" has one answer to read, and
/// a third such fact is a third argument here rather than a third mechanism.
///
/// With nothing pending and nothing raised this is a pointer clone, so the
/// ordinary case allocates nothing and the rendered `Arc` is the derived one —
/// which is also why the caller may only run this when one of its inputs moved:
/// a fresh `Arc` every frame would make `SnapMemo` rebuild per frame, the exact
/// cost bl-e90a removed.
pub(crate) fn compose(
    derived: &Arc<Snapshot>,
    echo: Option<&Echo>,
    raised: Option<&Path>,
) -> Arc<Snapshot> {
    if echo.is_none() && raised.is_none() {
        return Arc::clone(derived);
    }
    let mut snap = (**derived).clone();
    if let Some(ws) = raised {
        crate::app::raise::fold(&mut snap, ws);
    }
    let Some(echo) = echo else {
        return Arc::new(snap);
    };
    let tree = snap.trees.entry(echo.ws.clone()).or_default();
    match index_of(derived, &echo.ws, &echo.target) {
        // The target is on the roster: the echo is one more undelivered
        // deposit on it *until the listing carries it* (bl-78d8) — past that
        // the deposit is the derivation's and pushing would double-count it in
        // the `✉n` badge and the Inbox tab. The row rises either way: the send
        // is an action whoever ends up telling the story of it.
        Some(i) => {
            if let Some(agent) = tree.agents.get_mut(i) {
                if !echo.deposited(agent.pending.len()) {
                    agent.pending.push(echo.deposit());
                }
                agent.last_action_unix = agent.last_action_unix.max(echo.at_unix);
            }
        }
        // It is not: a start whose branch does not exist yet, which is the
        // whole of what the operator could not see. Which identity that row
        // wears is the echo's own answer (`pending_identity`), so this altitude
        // and the row altitude mint one row from one rule.
        None => {
            if let Some((id, name)) = echo.pending_identity() {
                tree.agents.push(echo.pending_conversation(&id, &name));
            }
        }
    }
    Arc::new(snap)
}
