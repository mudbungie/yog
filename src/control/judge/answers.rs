//! The **trail fold** (VISION §4.11 items 4, 6, 7): the operator's answers and
//! floors, read back out of `ops.jsonl`. Later rows supersede earlier ones for
//! the same key — the log is append-only, so the fold is the state, and no
//! fourth durable artifact exists.
//!
//! Two row grammars, and only two:
//!
//! ```text
//! ["yog-control","answer",<key>,"pass"|"hold"|"refuse",<scope>]
//! ["yog-control","floor",<conversation>,"raise"|"lower"]
//! ```
//!
//! The answer row's `<key>` is what its scope stands over — the held `tool_use`
//! id at [`Call`](Scope::Call), `<agent> <class>` at
//! [`Conversation`](Scope::Conversation), the bare `<class>` at
//! [`Workspace`](Scope::Workspace), where the workspace is the row's own `cwd`
//! rather than a second copy of it in the argv. A four-word row is a
//! [`Call`](Scope::Call) answer written before bl-94a5 and reads as one.
//!
//! **Precedence, in one list.** A once-answer to *this exact* invocation wins
//! over everything: it is the operator looking at the call in front of them.
//! Then a raised floor, which **suspends every standing answer** — §4.9's fifth
//! rung means walk me through each call from here, and a standing grant is the
//! auto-approval it revoked; it is also what makes a standing answer revocable,
//! since `/revoke` parks the next call of the class so it can be answered
//! again. Then the standing answers, nearest first — the longest matching
//! conversation prefix, then the workspace. Then the table.

use std::collections::HashMap;

use super::{Ruling, Scope, class_key};
use crate::control::classify::{self, Effect};
use crate::control::wire::Request;
use crate::opslog::{OpEntry, YOG_CONTROL};

/// The ops-row verb naming an answer to a held call.
const ANSWER: &str = "answer";
/// The ops-row verb naming a per-conversation floor, raised or lowered.
const FLOOR: &str = "floor";
/// The floor's two states, as its row spells them.
const RAISE: &str = "raise";
const LOWER: &str = "lower";

/// A ruling and the scope the thing that produced it stands over — what the
/// refusal sentence names, so the model is told how wide the decision is
/// rather than only that one call failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Standing {
    pub ruling: Ruling,
    pub scope: Scope,
}

/// The operator's answers, folded from the trail.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Answers {
    once: HashMap<String, Ruling>,
    /// `(scope, subject, class)` → ruling. The subject is the answering
    /// conversation or the answering workspace; the class is [`class_key`].
    standing: HashMap<(Scope, String, String), Ruling>,
    floors: HashMap<String, bool>,
}

impl Answers {
    /// Fold every `yog-control` row in `entries`, oldest first.
    pub fn fold(entries: &[OpEntry]) -> Answers {
        let mut answers = Answers::default();
        for entry in entries {
            let words: Vec<&str> = entry.argv.iter().map(String::as_str).collect();
            match words.as_slice() {
                [YOG_CONTROL, ANSWER, key, word] => answers.answer(key, word, Scope::Call, entry),
                [YOG_CONTROL, ANSWER, key, word, scope] => {
                    if let Some(scope) = Scope::of(scope) {
                        answers.answer(key, word, scope, entry);
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

    /// One answer row, filed under the map its scope reads out of.
    fn answer(&mut self, key: &str, word: &str, scope: Scope, entry: &OpEntry) {
        let Some(ruling) = Ruling::of(word) else {
            return;
        };
        match scope {
            Scope::Call => {
                self.once.insert(key.to_owned(), ruling);
            }
            Scope::Conversation => {
                if let Some((conv, class)) = key.split_once(' ') {
                    self.standing
                        .insert((scope, conv.to_owned(), class.to_owned()), ruling);
                }
            }
            Scope::Workspace => {
                self.standing
                    .insert((scope, entry.cwd.clone(), key.to_owned()), ruling);
            }
        }
    }

    /// Whether a floor stands over `agent_id` — its own conversation's, or that
    /// of any ancestor in its hyphenated descent.
    pub fn floored(&self, agent_id: &str) -> bool {
        self.floors
            .iter()
            .any(|(conv, raised)| *raised && (agent_id == conv || descends(agent_id, conv)))
    }

    /// The ruling for one invocation, and the scope it came from: the operator's
    /// answers in precedence order, else the workspace's table.
    pub fn ruling(
        &self,
        request: &Request,
        ws: &str,
        effect: Effect,
        policy: &super::super::policy::Policy,
    ) -> Standing {
        if let Some(once) = self.once.get(&request.id) {
            return Standing {
                ruling: *once,
                scope: Scope::Call,
            };
        }
        // The table's answer is addressed to the leg the name runs on
        // (bl-1772): on a routed leg a refusal is delivered to the one party
        // with an interest in rephrasing it, so it becomes a hold and the
        // operator is asked. The once-answer stands ahead of this on purpose —
        // an operator who answered this exact id has made the decision.
        let table = match classify::Leg::of(&request.name) {
            classify::Leg::Engine => policy.ruling(effect),
            classify::Leg::Routed => policy.ruling(effect).for_the_operator(),
        };
        // The compaction procedure's own pair is never floored (bl-a821): it is
        // machinery, not the agent's act, and holding it queues the operator a
        // decision they have no basis to make while the conversation it belongs
        // to cannot compact.
        if effect > Effect::Read
            && !classify::checkpoint(&request.name)
            && self.floored(&request.agent_id)
        {
            return Standing {
                ruling: table.max(Ruling::Hold),
                scope: Scope::Conversation,
            };
        }
        let class = class_key(&request.name, effect);
        self.nearest(&request.agent_id, &class)
            .or_else(|| {
                self.standing
                    .get(&(Scope::Workspace, ws.to_owned(), class))
                    .map(|ruling| Standing {
                        ruling: *ruling,
                        scope: Scope::Workspace,
                    })
            })
            .unwrap_or(Standing {
                ruling: table,
                scope: Scope::Workspace,
            })
    }

    /// The conversation-scoped answer for `class` written nearest to
    /// `agent_id`: the longest ancestor prefix that has one. Nearest rather
    /// than latest, because two answers on one descent are two statements about
    /// two different subtrees and the more specific one is the one aimed here.
    fn nearest(&self, agent_id: &str, class: &str) -> Option<Standing> {
        self.standing
            .iter()
            .filter(|((scope, conv, key), _)| {
                *scope == Scope::Conversation
                    && key == class
                    && (agent_id == conv || descends(agent_id, conv))
            })
            .max_by_key(|((_, conv, _), _)| conv.len())
            .map(|(_, ruling)| Standing {
                ruling: *ruling,
                scope: Scope::Conversation,
            })
    }
}

/// Whether `agent_id` is a descendant of `conv` — the hyphenated descent litany
/// mints, so a longer name that merely starts with the same letters is another
/// agent entirely.
fn descends(agent_id: &str, conv: &str) -> bool {
    agent_id
        .strip_prefix(conv)
        .is_some_and(|rest| rest.starts_with('-'))
}

#[cfg(test)]
mod tests;
