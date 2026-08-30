//! The **judgment fold** (VISION §4.11 items 4, 6, 7; DESIGN §8.6): a class, the
//! shipped default table, and the two kinds of operator answer the ops trail
//! already carries.
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
//! - **answers** are `ops.jsonl` rows, which are at once the audit and this
//!   fold's memory. No new durable artifact; I2 holds at three.
//!
//! Two answer kinds, and only two:
//!
//! 1. A **once-answer** scoped to one `tool_use` id. The id is provider-unique,
//!    so the grant needs no consumption and cannot race: the same id is never
//!    asked twice by two different invocations.
//! 2. A **floor** on a conversation — the alignment monitor's revoke rung
//!    (bl-94b4) — under which every class above read adjudicates to a hold. It
//!    matches by descent prefix, so revoking a conversation revokes its whole
//!    subtree without enumerating one.
//!
//! Precedence is the operator's: a once-answer to *this exact* invocation wins
//! over the floor and over the table. The floor then raises whatever the table
//! said; it never lowers it, so a refusal stays a refusal.
//!
//! **Revocation binds at the next consult, never mid-window.** A verdict already
//! passed runs its one call; recalling it would mean stopping the agent, and a
//! stop mid-tool-window wedges the branch permanently.

use std::collections::HashMap;

use super::classify::Effect;
use super::wire::Verdict;
use crate::opslog::{OpEntry, YOG_CONTROL};

/// What the policy says about a class, before a reason is attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

    /// The verdict this ruling carries, given the classification's clause.
    pub fn verdict(self, why: &str) -> Verdict {
        match self {
            Ruling::Pass => Verdict::Pass,
            Ruling::Hold => Verdict::Hold(why.to_owned()),
            Ruling::Refuse => Verdict::Refuse(why.to_owned()),
        }
    }
}

/// The class → ruling table: **everything passes except loss and credentials**.
/// An unattended drone is there to work, and a shipped hold on open-world made
/// the operator answer for every `python` and every fetch — approving what they
/// were always going to approve. So the four classes that are the job pass, and
/// only irreversible loss and credential access decline in band: those two are
/// what a drone must not decide for itself, and neither is answerable by
/// reflex.
///
/// **Hold is no longer standing policy; it is imposed.** Two mechanisms carry
/// the weight the shipped hold used to, and both aim it at the conversation
/// that earned it rather than at all of them:
///
/// - a workspace that wants the parked default writes one line of
///   `capability.yaml` — `table:` / `  open-world: hold` (see
///   [`Policy`](super::policy::Policy)); severability still runs the right way,
///   with absence the (now permissive) default and the file the override;
/// - the alignment monitor's revoke rung raises a per-conversation floor, under
///   which every class above read holds ([`Answers::floored`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table;

impl Table {
    /// This class's ruling.
    pub fn ruling(effect: Effect) -> Ruling {
        match effect {
            Effect::Read | Effect::TargetWrite | Effect::Process | Effect::OpenWorld => {
                Ruling::Pass
            }
            Effect::Destructive | Effect::Secret => Ruling::Refuse,
        }
    }
}

/// The ops-row verb naming a once-answer to one held `tool_use`.
const ANSWER: &str = "answer";
/// The ops-row verb naming a per-conversation floor, raised or lowered.
const FLOOR: &str = "floor";
/// The floor's two states, as its row spells them.
const RAISE: &str = "raise";
const LOWER: &str = "lower";

/// The operator's answers, folded from the trail. Later rows supersede earlier
/// ones for the same key — the log is append-only, so the fold is the state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Answers {
    once: HashMap<String, Ruling>,
    floors: HashMap<String, bool>,
}

impl Answers {
    /// Fold every `yog-control` row in `entries`, oldest first.
    pub fn fold(entries: &[OpEntry]) -> Answers {
        let mut answers = Answers::default();
        for argv in entries.iter().map(|e| &e.argv) {
            let words: Vec<&str> = argv.iter().map(String::as_str).collect();
            match words.as_slice() {
                [YOG_CONTROL, ANSWER, key, word] => {
                    if let Some(ruling) = Ruling::of(word) {
                        answers.once.insert((*key).to_owned(), ruling);
                    }
                }
                [YOG_CONTROL, FLOOR, conv, state @ (RAISE | LOWER)] => {
                    answers.floors.insert((*conv).to_owned(), *state == RAISE);
                }
                _ => {}
            }
        }
        answers
    }

    /// Whether a floor stands over `agent_id` — its own conversation's, or that
    /// of any ancestor in its hyphenated descent.
    pub fn floored(&self, agent_id: &str) -> bool {
        self.floors.iter().any(|(conv, raised)| {
            *raised
                && (agent_id == conv
                    || agent_id
                        .strip_prefix(conv.as_str())
                        .is_some_and(|rest| rest.starts_with('-')))
        })
    }

    /// The ruling for one invocation: the once-answer if the operator gave one,
    /// else the workspace's table raised by any standing floor.
    pub fn ruling(
        &self,
        tool_use_id: &str,
        agent_id: &str,
        effect: Effect,
        policy: &super::policy::Policy,
    ) -> Ruling {
        if let Some(once) = self.once.get(tool_use_id) {
            return *once;
        }
        let table = policy.ruling(effect);
        if effect > Effect::Read && self.floored(agent_id) {
            return table.max(Ruling::Hold);
        }
        table
    }
}

#[cfg(test)]
mod tests;
