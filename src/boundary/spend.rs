//! **The §3.5 spend family's executors** (DESIGN §3.5, §4.1; REMOTE
//! §9.23; bl-53d1): the one derivation the table read answers and
//! both acts receipt with, the two write-throughs, and the ceiling's release.
//!
//! **One derivation, three askers.** `/prices`, `/price` and `/ceiling` all
//! answer [`view`]: the table flattened, the bound, and the world's priced
//! spend against it — re-derived from the `ui.json` the act just moved rather
//! than echoed, the `marks` precedent. The spend figure is
//! [`spend::of_world`](crate::spend::of_world)'s fresh walk, because it is the
//! same figure the ceiling's gate compares and the release below decides on,
//! and a gate compares against the world as it is (bl-56d5): one walk, on a
//! gesture, never per frame.
//!
//! **The release is the number moving** (§3.5). Past the ceiling the §8.6
//! consult parks every conversation at its next tool call, carrying
//! [`CEILING_HEAD`]'s sentence as the mark's reason. A `/ceiling` that leaves
//! the world under the number — raised, deleted, or with no priced table to
//! compare against — enumerates every `refs/litany/held/*` mark in every
//! workspace, keeps the ones whose reason starts with that head, and drives
//! each through the same detached `litany advance` the answer gesture spends
//! ([`control::advance`](super::control::advance)): a re-consult under a
//! number the world is now under answers `pass`, and the branch moves. A
//! floor's park and a policy hold carry other words and are not touched.
//! The count of marks so driven rides back as `released`; each launch is its
//! own §4.2 row, so a driver that failed to fork is on the trail rather than
//! subtracted from a number nobody could reconcile.

use std::path::PathBuf;

use crate::spend::{CEILING_HEAD, Price, Prices};
use crate::ui_state::UiState;

use super::dispatch::Deps;
use super::reply::{PricesView, Reply};

/// The whole answer, off the durable document and one fresh walk of the
/// world. `released` is the act's own count, `None` for a read.
pub(super) fn view(deps: &Deps, ui: &UiState, released: Option<usize>) -> PricesView {
    let prices = ui.prices();
    PricesView {
        rows: prices.rows(),
        ceiling: ui.ceiling(),
        spent: crate::spend::of_world(&roster(deps), &prices),
        released,
    }
}

/// Write or delete one entry, then answer the re-read table.
pub(super) fn price(
    deps: &Deps,
    ui: &mut UiState,
    provider: &str,
    model: &str,
    rates: Option<Price>,
) -> Reply {
    let mut table: Prices = ui.prices();
    table.set(provider, model, rates);
    ui.set_prices(&table);
    Reply::Prices(view(deps, ui, None))
}

/// Write or delete the number, release what it parked if the world is now
/// under it, and answer the re-read table with the count.
pub(super) fn ceiling(deps: &Deps, ui: &mut UiState, ts: &str, micro_usd: Option<u64>) -> Reply {
    ui.set_ceiling(micro_usd);
    let roster = roster(deps);
    let prices = ui.prices();
    let over = ui.ceiling().refusal(&roster, &prices).is_some();
    let released = if over { 0 } else { release(deps, ts, &roster) };
    Reply::Prices(view(deps, ui, Some(released)))
}

/// Drive every conversation the ceiling parked, in every workspace, and say
/// how many. Selection is by the mark's own reason — the sentence yog wrote —
/// so nothing parked for another reason moves.
fn release(deps: &Deps, ts: &str, roster: &[PathBuf]) -> usize {
    let mut count = 0;
    for workspace in roster {
        for (agent, held) in crate::control::hold::all(workspace) {
            if !held.reason.starts_with(CEILING_HEAD) {
                continue;
            }
            // Best-effort, like the answer gesture's own release: the launch
            // writes its row either way, and the trail is where a driver that
            // could not fork is read.
            let _ = super::control::advance(deps, ts, workspace, &agent);
            count += 1;
        }
    }
    count
}

/// The §3.1 roster, enumerated now rather than read off the snapshot — the
/// gate's own reading (`dispatch::doors`), so the figure this answers and
/// the figure a birth is refused on are one walk over one roster.
fn roster(deps: &Deps) -> Vec<PathBuf> {
    crate::binding::workspaces(&deps.yog_data_root, &deps.world.litany_data_root())
        .into_iter()
        .map(|w| w.path)
        .collect()
}

#[cfg(test)]
mod tests;
