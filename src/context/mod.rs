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
//! **Why the prompt is `max(input, cache_read + cache_write)`.** brazen's
//! canonical `Usage` is deliberately *unnormalized* about overlap, because its
//! providers disagree: Anthropic reports `input_tokens`,
//! `cache_read_input_tokens` and `cache_creation_input_tokens` as three
//! **disjoint** slices of one prompt, while OpenAI's `prompt_tokens` and
//! Google's `promptTokenCount` already **contain** the cached slice they report
//! beside it. So summing all three over-states OpenAI by the cached prefix, and
//! taking `input_tokens` alone under-states Anthropic by nearly the whole
//! prompt — brazen marks Anthropic prompts for caching unconditionally
//! (`protocol/anthropic/encode/cache.rs`: *"caching is brazen-owned POLICY with
//! zero canonical surface"*), so mid-conversation `input_tokens` is only the
//! uncached tail. The maximum of the two readings is exact wherever the cached
//! slice is contained (OpenAI, Google, Ollama, and any provider reporting no
//! cache counters at all — where it degrades to plain `input_tokens`) and a
//! **floor** where the slices are disjoint (Anthropic, short by that same
//! uncached tail). It never over-states, and yog already renders a figure it
//! cannot complete as a floor rather than as an answer (§3.5's
//! `+N tok unpriced`). One formula, no per-provider branch — normalizing that
//! overlap is brazen's to do, not yog's to guess at.

/// The egui line the settings rows paint.
pub mod render;

use crate::budgets::{BudgetSpend, StepBill};
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

/// The prompt one `Usage` reading describes — the module header's rule, at its
/// one home.
pub fn prompt_tokens(usage: BudgetSpend) -> u64 {
    usage.input_tokens.max(
        usage
            .cache_read_tokens
            .saturating_add(usage.cache_write_tokens),
    )
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
        prompt_tokens: prompt_tokens(latest.last_usage),
        window,
    })
}

#[cfg(test)]
mod tests;
