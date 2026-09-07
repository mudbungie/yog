//! The **judgment vocabulary** (VISION §4.11 items 4, 6, 7; DESIGN §8.6): a
//! ruling, the shipped default table, and the **scope** an operator's answer
//! stands over. The fold that reads the operator's answers off the trail is
//! [`answers`].
//!
//! The control writes nothing, ever — the seam re-adjudicates a held invocation
//! on every later drive, so a consult with a side effect would answer
//! differently the second time. Everything it needs therefore has a home
//! somewhere else already:
//!
//! - the **request** is litany's own hold mark, written by the seam;
//! - **standing policy** is the shipped [`Table`], overridden row by row by the
//!   workspace's own [`Policy`](super::policy::Policy) when it declares one —
//!   absence *is* the defaults, the `cadence.yaml` severability pattern;
//! - **answers** are `ops.jsonl` rows, which are at once the audit and the
//!   fold's memory. No new durable artifact; I2 holds at three.
//!
//! **An answer has a scope, and the scope is the whole of what a wider one
//! costs** (bl-94a5). Before it, every answer was one call, so an operator
//! holding a conversation answered the same question for every call of a kind
//! they had already decided about — eleven holds and eleven releases of one
//! narrow routed tool in a single measured run. Three scopes now:
//!
//! 1. [`Call`](Scope::Call) — the held `tool_use` id, which is today's answer
//!    and the default. The id is provider-unique, so the grant needs no
//!    consumption and cannot race.
//! 2. [`Conversation`](Scope::Conversation) — the **class** of the held call
//!    ([`class_key`]: the same tool at the same reach) over that conversation
//!    and its whole descent, matched by the descent prefix the floor is.
//! 3. [`Workspace`](Scope::Workspace) — that class over every conversation in
//!    the workspace the answer was given in.
//!
//! Two rules bound the two wide scopes, and neither is a new floor:
//!
//! - **Loss and credentials take [`Call`](Scope::Call) only.** Those are the
//!   two classes the shipped table refuses outright, and they are exactly the
//!   ones an operator must not be able to decide once and forget
//!   ([`Answer::permits`]).
//! - **A raised floor suspends every standing answer** ([`answers`]). §4.9's
//!   fifth rung means *walk me through each call from here*, and a standing
//!   grant is the auto-approval it revoked. That is also what makes a standing
//!   answer revocable: `/revoke` parks the next call of the class, the operator
//!   answers it again at the same scope, `/restore` lowers the floor.

use super::classify::Effect;

/// The trail fold: the operator's answers, read back off `ops.jsonl`.
pub mod answers;
/// The shipped default table — the ruling for a class nobody has answered for.
pub mod table;
pub use answers::{Answers, Standing};
pub use table::Table;

/// What the policy says about a class, before a reason is attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ruling {
    Pass,
    Hold,
    Refuse,
}

impl Ruling {
    /// The ruling as an ops row spells it — the same word both directions, so
    /// the writer (bl-765d) and this reader cannot drift.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Ruling::Pass => "pass",
            Ruling::Hold => "hold",
            Ruling::Refuse => "refuse",
        }
    }

    /// The ruling a row's word names, or `None` for anything else.
    pub fn of(word: &str) -> Option<Ruling> {
        [Ruling::Pass, Ruling::Hold, Ruling::Refuse]
            .into_iter()
            .find(|r| r.word() == word)
    }

    /// This ruling **addressed to the party that can act on it** (bl-1772).
    ///
    /// A refusal is a sentence handed to the model as a tool result marked
    /// ERROR, and the model is then the only party deciding whether to try
    /// again differently. On the engine's own machine that is the right answer:
    /// the operator is sitting at it, the blast radius is in front of them, and
    /// an in-band decline costs no park. On a **routed** leg it is not an answer
    /// at all — the drive that filed this had a bare `rm -f <dir>/*` refused
    /// destructive, and the model then probed the confinement over three steps
    /// and ran `cd <dir> && rm -f -- *` instead: 115 MB gone from another
    /// machine, with no hold, no attention item, and nothing the operator could
    /// have answered. The refusal was delivered to the one party with an
    /// interest in rephrasing it.
    ///
    /// So on that leg a refusal becomes a **hold**: the operator is asked, which
    /// is also the only way they can say *yes* to the one destructive act a foot
    /// exists to make safe — freeing disk on a server. Under a refusal there is
    /// nothing for `answer` to release, so that yes could not be said at all.
    /// Never the other direction: a hold is softened into a refusal nowhere.
    pub(crate) fn for_the_operator(self) -> Ruling {
        match self {
            Ruling::Refuse => Ruling::Hold,
            Ruling::Pass | Ruling::Hold => self,
        }
    }
}

/// How far one answer stands. Ordered narrowest first, which is also the
/// order the fold consults them in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scope {
    /// The held call, and nothing else.
    Call,
    /// The class of the held call, over this conversation and its descent.
    Conversation,
    /// The class of the held call, over this workspace. Also what the shipped
    /// table and a workspace's own `capability.yaml` stand over, which is why
    /// a ruling that came from neither an answer nor a floor reports this one.
    Workspace,
}

impl Scope {
    /// The scope as its row, its gesture field and its flag all spell it.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Scope::Call => "call",
            Scope::Conversation => "conversation",
            Scope::Workspace => "workspace",
        }
    }

    /// The scope a word names, or `None` for anything else.
    pub fn of(word: &str) -> Option<Scope> {
        [Scope::Call, Scope::Conversation, Scope::Workspace]
            .into_iter()
            .find(|s| s.word() == word)
    }

    /// What a decision at this scope covers, in the words a refusal hands the
    /// model. Never an invitation: it names the reach of the decision so the
    /// model can tell that another spelling of the same call is inside it.
    pub fn stands_for(self, tool: &str, effect: Effect) -> String {
        match self {
            Scope::Call => "this one call".to_owned(),
            Scope::Conversation => format!(
                "every {tool} call classified {} in this conversation and its descent",
                effect.word()
            ),
            Scope::Workspace => format!(
                "every {tool} call classified {} in this workspace",
                effect.word()
            ),
        }
    }

    /// Whether an answer at this scope may stand over `effect`, and why not
    /// where it may not. Loss and credentials take [`Call`](Scope::Call) only:
    /// they are the two classes the shipped table refuses outright, so an
    /// answer to one is the operator overriding the strictest thing the control
    /// says — a decision about the call in front of them, never about a class
    /// of calls to come. The refusal names the way to make it standing anyway:
    /// a `capability.yaml` row, which lives where policy is read and deleted.
    pub fn permits(self, effect: Effect) -> Result<(), String> {
        if self == Scope::Call || !matches!(effect, Effect::Destructive | Effect::Secret) {
            return Ok(());
        }
        Err(format!(
            "a {class} call takes --scope call only: {class} is what the control refuses \
             outright, so it is answered for the call in front of you and never for a class of \
             calls to come. To make it standing anyway, state the tool's reach in this \
             workspace's capability.yaml `rules:` block, where it is read and deleted.",
            class = effect.word(),
        ))
    }
}

/// One operator answer: the verdict, and how far it stands. The two travel
/// together everywhere — gesture, envelope, ops row, receipt — because they
/// are one decision, and a verdict whose scope was carried beside it would be
/// two facts to keep true.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Answer {
    pub ruling: Ruling,
    pub scope: Scope,
}

impl Answer {
    /// A once-answer: the default, and what every answer was before bl-94a5.
    pub fn once(ruling: Ruling) -> Answer {
        Answer {
            ruling,
            scope: Scope::Call,
        }
    }
}

/// The **class of a call**: the same tool at the same reach, in one token. It
/// is the key every scope wider than [`Call`](Scope::Call) stands over, and it
/// is deliberately not the tool alone — a `Bash` released for a read must not
/// carry a later destructive line, so the reach is half the key.
pub fn class_key(tool: &str, effect: Effect) -> String {
    format!("{tool}@{}", effect.policy_word())
}

#[cfg(test)]
mod tests;
