//! **How full is this conversation's context?** (DESIGN §5.1 #35, §11's
//! settings rows.) The context-window percentage is shown per chat.
//!
//! Derived, never stored, and pure: a query over the worker's already-walked
//! [`StepBill`]s (§3.5, bl-9dd4) and the windows `models.yaml` declares. Yog
//! keeps no counter and no cache — asking twice re-derives, like every other
//! §5.1 fact.
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
use std::collections::BTreeMap;

/// One conversation's context as of its latest step (§5.1 #35).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fullness {
    /// The wire model that step ran on — the id `request.json` named, which is
    /// also the key the window was found under.
    pub model: String,
    /// The prompt that step sent, by the reading in this module's header. A
    /// floor on a provider that slices its prompt counters disjointly.
    pub prompt_tokens: u64,
    /// The window that model declares. Never zero: an undeclared or zero
    /// window yields no [`Fullness`] at all rather than a division to guess at.
    pub window: u64,
}

impl Fullness {
    /// The figure itself, rounded to whole percent. Not clamped: a context that
    /// has outgrown its declared window is a fact worth reading as `140%`,
    /// which says *the declaration is wrong or the provider compacted* — where
    /// a clamp to 100 would render an overflow and a full context alike.
    pub fn percent(&self) -> u64 {
        self.prompt_tokens.saturating_mul(100) / self.window
    }
}

/// One conversation's fullness, or `None` when nothing honest can be said: the
/// root has taken no step, its latest step names no model, or that model
/// declares no window. **Render nothing, never an estimate** — the whole point
/// of the figure is that it is measured.
pub fn of_conversation(
    bills: &[StepBill],
    root_id: &str,
    windows: &BTreeMap<String, u64>,
) -> Option<Fullness> {
    let latest = bills
        .iter()
        .filter(|b| b.conv == root_id)
        .max_by(|a, b| a.seq.cmp(&b.seq))?;
    let model = latest.model.clone()?;
    let window = *windows.get(&model)?;
    Some(Fullness {
        model,
        prompt_tokens: latest.last_usage.prompt_tokens(),
        window,
    })
}

#[cfg(test)]
mod tests;
