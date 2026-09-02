//! **The role's two tuning knobs** (DESIGN §9.4, bl-23bd): how much reasoning
//! its model calls request, and whether they ask the provider's priority lane.
//!
//! Both are optional fields of the same `roles.<r>` assignment `/model` writes
//! (litany ARCH §4.3, upstream bl-acba and bl-f587), on the same lineage,
//! through the same `litany config` commit — so this is a sibling of
//! [`pick`](super::pick), not a widening of it. Two gestures rather than a
//! wider `/model` for two reasons that agree: REMOTE §3 makes a new op free and
//! a changed shape a version bump, and a toggle that forced the operator to
//! restate provider and model would be a knob you cannot reach without
//! re-asserting two facts you did not come to change.
//!
//! **They carry the operator's vocabulary, and no dialect's.** `effort` and
//! `priority` are what the config says; `reasoning_effort`, `thinking.budget_tokens`,
//! `service_tier`, `flex` and `batch` are wire spellings that stay inside the
//! adapter, exactly as litany's own ARCH §4.3 rules. yog spells neither — it
//! writes the config's two words and the engine resolves the rest.
//!
//! **No capability gate stands on the write.** The §9.4 rows state whether a
//! provider takes each knob ([`ProviderRowView`](crate::config_edit::brazen::ProviderRowView)),
//! and that fact decides whether a *control* is offered — never whether a write
//! is allowed. The config field is always lawful, a level a model declines is
//! the provider's own refusal in the §7.3 banner, and refusing here would be a
//! surface refusing on the strength of a question that went unanswered (§9.4's
//! caveat discipline, the rule `is_unknown_row` keeps).
//!
//! **Off is the absent field, and there is no third spelling.** litany reads
//! `effort` and `priority` as `Option`s and states that `false` and omitted are
//! one fact; writing `priority: false` would be a second spelling of absence
//! that the engine would read identically and an operator would read as a
//! third state. So `off` **removes the line**
//! ([`remove_field`](super::grammar::remove_field)), which is also what makes
//! the gesture idempotent: turning off what is already off is the same world.

use super::grammar::{self, PROVIDERS_YAML, ROLES};
use crate::model_pick::GrammarError;

/// A role's requested reasoning-effort level — the closed vocabulary litany's
/// `effort:` takes, in its own lowercase spelling.
///
/// yog's own enum rather than a re-export, for litany's stated reason one layer
/// down: the engine defines its own instead of re-exporting brazen's because the
/// linked crate's type does not carry what it needs, and here the crate's type
/// is not on litany's public surface at all. The bridge is
/// [`as_str`](Self::as_str) and [`parse`](Self::parse), one match each, and the
/// three words are the contract — a fourth level is an upstream act first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effort {
    Low,
    Medium,
    High,
}

/// What `/effort`'s level word may be, said once so the line's refusal, the
/// codec's refusal and the help page cannot disagree.
pub const LEVELS: &str = "low|medium|high|off";

impl Effort {
    /// The word this level is written as, in the config and on the wire alike.
    pub fn as_str(&self) -> String {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
        .to_owned()
    }

    /// One level word read back, or `None` for anything else — including
    /// `off`, which is not a level but the absence of one and is read by the
    /// caller that knows it has an off state.
    pub fn parse(word: &str) -> Option<Self> {
        match word {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            _ => None,
        }
    }
}

/// The §9.4 tuning family: one gesture per knob, folded onto one carrier.
///
/// One variant on the §8.5 roster over two, the fold the monitor's, the
/// fleet's, the routing leg's, the §3.8 fan's and the `bl` family's each take
/// (§12): every layer beneath reads these two as a pair — one line reader, one
/// codec file, one executor, one help section, one file written — so the
/// carrier says what those layers already say. The fold is in the carrier and
/// never in the surface: each still spells as its own slash verb and its own
/// envelope `op`, which is what makes a new op free of a version bump.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tuning {
    /// `/effort <role> <low|medium|high|off>` — set the role's reasoning level,
    /// or remove it. `None` is `off`, and it is an absence rather than a value
    /// because that is what the engine reads.
    Effort {
        workspace: String,
        role: String,
        level: Option<Effort>,
    },
    /// `/priority <role> <on|off>` — ask the provider's priority lane for this
    /// role's calls, or stop asking. A checkbox and not a tri-state: `off`
    /// removes the line, because asking for the *standard* lane is a different
    /// intent that no config key expresses.
    Priority {
        workspace: String,
        role: String,
        on: bool,
    },
}

impl Tuning {
    /// The workspace slot REMOTE §8.2's name→path rewrite borrows, the shape
    /// [`monitor::Verb`](crate::monitor::Verb) and
    /// [`fleet::Verb`](crate::fleet::Verb) already have: both members name one,
    /// so the address table delegates instead of matching.
    ///
    /// `pub(crate)` for their reason too — it hands back a borrow, which house
    /// rule 2 keeps off the `pub` surface, and the only caller is the table one
    /// module over.
    pub(crate) fn workspace_slot(&mut self) -> &mut String {
        match self {
            Self::Effort { workspace, .. } | Self::Priority { workspace, .. } => workspace,
        }
    }

    /// The role both members act on.
    pub fn role(&self) -> String {
        match self {
            Self::Effort { role, .. } | Self::Priority { role, .. } => role.clone(),
        }
    }
}

/// The `providers.yaml` text one tuning gesture produces, or a decline —
/// [`plan`](super::plan)'s sibling, and pure in exactly the same way: text in,
/// whole text out, no disk and no network.
///
/// **It gates nothing itself, and that is the point.** The value is closed at
/// the line and at the codec, so nothing unspellable reaches here; the role is
/// required by the two grammar primitives below, both of which refuse an entry
/// the file does not declare with one sentence. An earlier draft checked the
/// role here as well, so that the *clear* path could not report success for a
/// write that reached nothing — but that made one rule live in two places and
/// left the primitives' own refusal untested. The asymmetry belongs where the
/// lines are: the entry must exist, the field need not.
///
/// It carries **no provider gate** either: see this module's note on why a
/// capability decides a control and never a write.
pub fn plan(providers_yaml: &str, tuning: &Tuning) -> Result<String, GrammarError> {
    let role = tuning.role();
    match tuning {
        Tuning::Effort {
            level: Some(level), ..
        } => set(providers_yaml, &role, grammar::EFFORT, &level.as_str()),
        Tuning::Effort { level: None, .. } => clear(providers_yaml, &role, grammar::EFFORT),
        Tuning::Priority { on: true, .. } => set(providers_yaml, &role, grammar::PRIORITY, "true"),
        Tuning::Priority { on: false, .. } => clear(providers_yaml, &role, grammar::PRIORITY),
    }
}

/// One knob set: the field is written whether or not the role already carried
/// it, which is the whole of what an *optional* assignment field needs and the
/// whole of why [`grammar::upsert_field`] had to exist.
fn set(text: &str, role: &str, name: &'static str, value: &str) -> Result<String, GrammarError> {
    grammar::upsert_field(PROVIDERS_YAML, text, ROLES, role, name, value)
}

/// One knob cleared: the line goes, and a role that never carried it is
/// returned as it stands — while a role that is not there refuses, which is the
/// asymmetry [`grammar::remove_field`] owns.
fn clear(text: &str, role: &str, name: &str) -> Result<String, GrammarError> {
    grammar::remove_field(PROVIDERS_YAML, text, ROLES, role, name)
}
