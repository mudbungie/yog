//! **How full is this conversation's context?** (DESIGN §5.1 #35, §11's
//! settings rows.) The context-window percentage is shown per chat.
//!
//! Derived, never stored, and pure: a query over the worker's already-walked
//! [`StepBill`]s (§3.5, bl-9dd4), and nothing else — the window is on the bill
//! too, read off the same usage line as the prompt (bl-9c8a). Yog keeps no
//! counter, no cache and no table of windows — asking twice re-derives, like
//! every other §5.1 fact.
//!
//! **The denominator is the provider's fact, served in band.** brazen stamps
//! the resolved model row's context window on every `Usage` event (its
//! model-discovery §5.5, 0.0.9), and litany's own `window_percent` compaction
//! trigger divides by exactly that number off the same record. A window yog
//! declared on its own would be a second representation of one fact that the
//! engine compacting the context could not see — the percentage here and the
//! compaction one layer down would disagree about one context. So there is no
//! declaration: a row that states no window renders no figure, and the seat to
//! state one is upstream, on brazen's provider row, where for Ollama it is
//! already the `num_ctx` in force.
//!
//! **Fullness is not spend.** [`crate::budgets`] sums every counter of every
//! attempt of every step of a whole descent — that is what exhausts
//! `max_total_tokens`, and it grows without bound while a context is compacted
//! back down. Fullness is one number off **one** step: the prompt the *latest*
//! step of the root agent actually sent. Two questions, two derivations, one
//! walk.
//!
//! **The latest step of the root, not of the descent.** A dispatched child runs
//! its own context in its own `steps/<root>-<child>/` tree; folding the descent
//! in would answer no question anyone asked. The conversation's context is the
//! conversation's.
//!
//! **The prompt is `max(input, cache_read + cache_write)`, and that formula
//! has one home:** [`BudgetSpend::prompt_tokens`], which carries the whole
//! why — brazen's canonical `Usage` is deliberately *unnormalized* about
//! overlap because its providers disagree, so the maximum is exact where the
//! cached slice is contained and a floor where the slices are disjoint. This
//! module used to state it a second time and [`crate::budgets`] summed all four
//! counters instead; that divergence was the double-count bl-6621 closed. Two
//! questions, two derivations — one reading of a prompt.

use crate::budgets::StepBill;

/// One conversation's context as of its latest step (§5.1 #35).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fullness {
    /// The wire model that step ran on — the id `request.json` named.
    pub model: String,
    /// The prompt that step sent, by the reading in this module's header. A
    /// floor on a provider that slices its prompt counters disjointly.
    pub prompt_tokens: u64,
    /// The window the step's own usage lines state for that model. Never
    /// zero: an unstated or zero window yields no [`Fullness`] at all rather
    /// than a division to guess at.
    pub window: u64,
}

impl Fullness {
    /// The figure itself, rounded to whole percent. Not clamped: a context that
    /// has outgrown its declared window is a fact worth reading as `140%`,
    /// which says *the row's window is wrong or the provider compacted* —
    /// where a clamp to 100 would render an overflow and a full context alike.
    pub fn percent(&self) -> u64 {
        self.prompt_tokens.saturating_mul(100) / self.window
    }
}

/// **One agent's** fullness, or `None` when nothing honest can be said: it has
/// taken no step, its latest step names no model, or its usage lines state no
/// window. **Render nothing, never an estimate** — the whole point of the
/// figure is that it is measured.
///
/// One agent and never a descent (§5.1 #35: *"a dispatched child runs its own
/// context in its own tree"*), and since bl-131d the agent is the one the
/// caller named rather than that agent's root — a child asked about its own
/// context was answered with its parent's, off the same one filter that could
/// have answered honestly.
pub fn of_agent(bills: &[StepBill], agent_id: &str) -> Option<Fullness> {
    let latest = bills
        .iter()
        .filter(|b| b.conv == agent_id)
        .max_by(|a, b| a.seq.cmp(&b.seq))?;
    Some(Fullness {
        model: latest.model.clone()?,
        prompt_tokens: latest.last_usage.prompt_tokens(),
        window: latest.window?,
    })
}

#[cfg(test)]
mod tests;
