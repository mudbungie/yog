//! The §3.5 spend ceiling's **second seat** (DESIGN §8.6's last bullet,
//! bl-4b48): the ceiling read at the consult, so a world at or over the
//! operator's number parks every conversation in it instead of spending to the
//! end of its backlog.
//!
//! The first seat is [`crate::boundary::ceiling`] and it gates a *birth*. What
//! it cannot reach is the fleet already alive: past the number every drone kept
//! spending until its own natural end, which for a `/fleet` over a backlog is
//! the whole backlog. This seat is the §4.9 floor's mechanism aimed at every
//! conversation at once — same verdict (`hold`), same mark, same queue, same
//! `/answer pass` walk-through. The difference is the predicate and its scope:
//! a floor holds every class above `read` over one descent, and the ceiling
//! holds **every** class over every conversation in the world.
//!
//! **It is the one comparison, called from a third place.**
//! [`Ceiling::verdict`](crate::spend::Ceiling::verdict) is the at-or-over test
//! and the sentence on the mark is its own, unaltered — fixed head `spend
//! ceiling reached:`. That is how the `/ceiling` act finds what the ceiling
//! parked and releases exactly that, and how the attention queue reads the same
//! words the board's own ceiling verdict already says. Nothing here re-compares
//! and nothing here re-words.
//!
//! **What it must not do, and does not.** It never stops and never signals — a
//! stop mid-tool-window wedges the branch permanently (litany's own bl-b98d) —
//! and it writes nothing, like every other half of this consult. A park costs
//! no process and no tokens, which is the whole reason the seat is here rather
//! than at a hook. The number is the operator's and the mark is litany's.
//!
//! **The cost, stated.** While the world is bounded *and* priced, one consult
//! costs one `steps/` walk of the world — once per tool call, on the tool
//! call's own process, never on a render or a derivation pass. While it is
//! unbounded or unpriced it costs one `ui.json` read and no walk at all, which
//! is [`Ceiling::armed`](crate::spend::Ceiling::armed)'s whole job: the roster
//! is not even enumerated. The overshoot the seat leaves is **one step per live
//! conversation** — the park lands at the next tool call, and a step that ends
//! with no tool call ends the branch.

use std::path::PathBuf;

use crate::ui_state::UiState;
use crate::xdg::Env;

/// The ceiling's sentence for `env`'s world, or `None` when nothing parks —
/// no `ceiling` key, an empty `prices` table, or a world still under the
/// number.
///
/// Resolved **once per consult** ([`Consult::new`](super::Consult::new)) and
/// carried on the consult as a value, so the judgment stays a pure function of
/// an owned struct.
pub(crate) fn parked(env: &Env) -> Option<String> {
    // `ui.json` under the state root, addressed through the one address book
    // that spells it (§7.1's roots, pure path arithmetic over the composed
    // world — nothing here reads disk yet).
    let ui = UiState::open(crate::app::Roots::of(env).ui_json());
    let (ceiling, prices) = (ui.ceiling(), ui.prices());
    if !ceiling.armed(&prices) {
        return None;
    }
    // The §3.1 roster, enumerated here at the instant of the judgment exactly
    // as the `Prompt` door enumerates it: the comparison's scope is the world's
    // (bl-a80a), never this one workspace's, and a bound compares against the
    // world as it is rather than a snapshot a debounce window old.
    let world: Vec<PathBuf> =
        crate::binding::workspaces(&env.yog_data_root(), &env.litany_data_root())
            .into_iter()
            .map(|w| w.path)
            .collect();
    ceiling.refusal(&world, &prices)
}
