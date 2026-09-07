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

use super::classify::{self, Effect};
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
///
/// **One exception, and it is not about a reach** (bl-72bd): the seventh class
/// [`Opaque`](Effect::Opaque) holds, because it is what the classifier says
/// when it could not read the invocation at all. bl-1ef1's argument does not
/// reach it — that argument was about parking effects the operator was always
/// going to approve, and this class is the one where nobody knows what is
/// being approved. A workspace that wants the old, open answer writes
/// `table:` / `  opaque: pass`, the same one line, the same way round.
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
            // The one shipped hold, and it is not a policy about a reach — it
            // is what the control says when it could not read one (bl-72bd).
            // A refusal would be a claim about the invocation this control has
            // no basis for; a pass is the arm the routed leg fell off into for
            // a year. So it parks, and the operator answers once.
            Effect::Opaque => Ruling::Hold,
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
    /// else the workspace's table — addressed to the leg the name runs on
    /// ([`Ruling::for_the_operator`], bl-1772) and raised by any standing floor.
    ///
    /// **The floor does not reach the compactor's checkpoint pair** (bl-a821,
    /// VISION §4.11 item 7). `revoke` takes auto-approval from a conversation
    /// *and its descendants*, and the compactor is a descendant — so a floor
    /// raised on a long conversation held `write_summary`, which is the one act
    /// no role can declare and litany injects from its own procedure. The
    /// operator was queued a machinery act they have no basis to judge and did
    /// not ask for, and until they answered it the floored conversation could
    /// not compact: a floor set out of worry stalled the conversation on
    /// context rather than on policy. A floor is a statement about what the
    /// AGENT may do to the world; this pair touches the conversation's own
    /// compactor branch and nothing else ([`classify::checkpoint`]). Everything
    /// else about descent propagation stands — a dispatched child's calls are
    /// the agent's acts, and the floor still reaches them.
    ///
    /// The exemption is the floor's alone. The table still rules the pair, so a
    /// workspace that writes `target-write: hold` gets what it asked for; what
    /// is dissolved is the hold nobody asked for.
    pub fn ruling(
        &self,
        tool_use_id: &str,
        agent_id: &str,
        name: &str,
        effect: Effect,
        policy: &super::policy::Policy,
    ) -> Ruling {
        if let Some(once) = self.once.get(tool_use_id) {
            return *once;
        }
        // The once-answer stands ahead of the leg on purpose: an operator who
        // answered this exact `tool_use` id has MADE the decision, and
        // re-parking it would ask them what they just answered. What the leg
        // addresses is the table's answer, which nobody has read yet.
        let table = match classify::Leg::of(name) {
            classify::Leg::Engine => policy.ruling(effect),
            classify::Leg::Routed => policy.ruling(effect).for_the_operator(),
        };
        if effect > Effect::Read && !classify::checkpoint(name) && self.floored(agent_id) {
            return table.max(Ruling::Hold);
        }
        table
    }
}

#[cfg(test)]
mod tests;
