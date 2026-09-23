//! The spend ceiling (DESIGN §3.5; VISION spend attribution): the **policy**
//! half — the operator's number and the comparison. [`Ceiling::verdict`] is
//! that comparison and there is exactly one of it; the *seats* that spend it
//! live elsewhere, and since bl-4b48 there are two:
//!
//! - [`crate::boundary::ceiling`] — the spawn gate, the one chokepoint every
//!   birth crosses (§3.5). It **refuses** a birth.
//! - [`crate::control::ceiling`] — the §8.6 capability control's consult, which
//!   runs before every tool call of every live conversation. It **parks** one.
//!
//! The ruling this implements: the ceiling **never kills a running drone**,
//! because killing mid-ball destroys uncommitted work and early termination is
//! the expensive failure. The second seat does not soften that. A hold is
//! litany's park — the invocation waits *before* it executes, and the branch
//! keeps its tree, its history and every uncommitted byte — and it closes the
//! gap the first seat left open: a fleet already alive spent to the end of its
//! backlog however far past the number it was (§3.5's fourth bullet). Nothing
//! is stopped, signalled or written by either seat, and the bound on the model
//! call itself is litany's own, one layer down, where the loop that spends it
//! lives.
//!
//! **Severable in the strong sense, and in two directions.** The ceiling is
//! one `ui.json` number beside the price table (§4.1 `ceiling`); deleting the
//! key deletes the gate, not a code path — [`Ceiling::refusal`] is a `None`
//! away from an ungated yog. Deleting `prices` deletes it too, and that is not
//! an accident: a ceiling is a dollar figure, and yog refuses to bound dollars
//! it cannot compute rather than inventing a proxy.
//!
//! **The figure it compares is the whole world's** (bl-a80a). It was the target
//! workspace's, and that was the one scope a spawn names outright — but the
//! number it is compared against is *world* config: one `ui.json`, one
//! `ceiling` key, beside the one `prices` table it is denominated in (§4.1).
//! A world-level number read as a per-sphere bound gives one operator figure as
//! many meanings as there are workspaces, so arming a second project silently
//! doubled the allowance the operator wrote. One key, one comparison, one gate,
//! one home: the multiplication is gone by construction rather than policed by
//! a second ceiling over the same concern — which is exactly the shape bl-56af
//! deleted.
//!
//! The cost is real and is the ruling's, not an oversight: an idle workspace is
//! refused because a busy one spent. That is the safe direction — a ceiling that
//! binds sooner refuses a *birth*, and a birth is the one thing it may refuse.

use std::path::PathBuf;

use serde_json::Value;

use super::{Cost, Prices};

/// **The fixed head of every sentence the ceiling writes** (bl-53d1): the
/// birth refusal's, and the reason the §8.6 consult parks a running
/// conversation under. yog wrote it and yog reads it back — a `/ceiling` act
/// that lifts the world back under the number selects the marks to release by
/// this prefix, so a floor's park or a policy hold, which carry other words,
/// are never touched.
pub const CEILING_HEAD: &str = "spend ceiling reached:";

/// The operator's spend ceiling, in micro-USD. `None` — the absent key — is
/// **no gate at all**, which is the default and the severability §3.5 demands.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Ceiling {
    limit: Option<u64>,
}

impl Ceiling {
    /// Read the `ui.json` value (§4.1 `ceiling`): a quoted USD number. Absent,
    /// non-numeric or negative all read as *no ceiling* — the forgiving read
    /// every `ui.json` key gets, so a typo costs the gate and never the window.
    /// A literal `0` is honored as written: the deliberate hard stop.
    pub fn from_json(value: Option<&Value>) -> Self {
        Self {
            limit: super::prices::quoted(value),
        }
    }

    /// The number itself, in micro-USD, or `None` for no ceiling — what the
    /// `prices` reply says back and what a write-through stores (bl-53d1).
    pub fn micro_usd(self) -> Option<u64> {
        self.limit
    }

    /// Whether this ceiling can bind at all against `prices`: a number to bind
    /// at, and a table to denominate it in. **The question a seat asks before
    /// it pays for the walk** — the fold behind [`refusal`](Self::refusal) is a
    /// `steps/` walk of every workspace in the world, and §8.6's consult spends
    /// it once per *tool call*, so an unbounded or unpriced world has to cost
    /// one `ui.json` read and nothing else. It is not a second comparison: it
    /// answers whether there is one to make.
    pub fn armed(&self, prices: &Prices) -> bool {
        self.limit.is_some() && !prices.is_empty()
    }

    /// The refusal the next birth anywhere in this world earns, or `None` to
    /// let it fly. `workspaces` is the world's roster (§3.1's three roots),
    /// walked here at the instant of the refusal — and only when [`armed`](
    /// Self::armed) says the walk can change the answer.
    ///
    /// Three ways to fly: no ceiling configured, no price table (an unpriceable
    /// figure bounds nothing), or a world whose priced spend is still under the
    /// number. The comparison is against the figure's **floor** — tokens the
    /// table cannot price are reported by the §11 render and never guessed at
    /// here, so the gate refuses only on spend it can actually name.
    pub fn refusal(&self, workspaces: &[PathBuf], prices: &Prices) -> Option<String> {
        self.armed(prices)
            .then(|| self.verdict(super::of_world(workspaces, prices)))
            .flatten()
    }

    /// The same judgement over a total someone else already folded — the
    /// **rendering** half (bl-66fb): the V4 board says where the ceiling will
    /// bind on the next spawn, and it must say it with the gate's own words and
    /// the gate's own comparison rather than a second opinion that could drift.
    /// The gate above is this function with the walk in front of it, and both
    /// arms fold the same scope — every workspace — since bl-a80a.
    ///
    /// **The sentence is the mark** (bl-4b48). §8.6's consult hands this text
    /// to litany's hold mark unaltered, so the fixed head `spend ceiling
    /// reached:` is what the `/ceiling` release selects a parked conversation
    /// by — a floor's park or a policy hold carries a different reason and is
    /// not touched. Write it once, here, or the seat that reads it back and the
    /// seat that wrote it drift.
    pub fn verdict(&self, spent: Option<Cost>) -> Option<String> {
        let limit = self.limit?;
        let cost = spent?;
        (cost.micro_usd >= limit).then(|| {
            let ceiling = Cost {
                micro_usd: limit,
                unpriced_tokens: 0,
            };
            format!(
                "{CEILING_HEAD} this world has spent {} across every workspace against a {} \
                 ceiling (ui.json `ceiling`), so nothing new is started anywhere and \
                 everything already running parks at its next tool call. Nothing is killed \
                 and no work is lost — raise the ceiling or delete the key to release it.",
                cost.usd(),
                ceiling.usd(),
            )
        })
    }
}

#[cfg(test)]
mod tests;
