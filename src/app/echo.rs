//! The **pending echo** (DESIGN §7.2, §3.4, bl-915e): the operator's own last
//! send, held until the derivation shows it.
//!
//! A [`Snapshot`] is what a completed derivation read off disk, and that was
//! the only source a frame had — so between Enter and the detached driver's
//! first write, the text the operator had just typed existed nowhere in yog's
//! model. Operator: *"you send the message, but before it goes into the inbox,
//! it's just missing for a minute."* Nothing was blocked; there was nothing to
//! render.
//!
//! This is not a synchronous write and not a spinner — the frame still does no
//! IO and still renders a completed derivation. It is an optimistic echo,
//! reconciled by the next snapshot, and it is the **same value** as the §3.4
//! start claim rather than a second pending concept beside it: one thing names
//! the conversation, holds the text, and is retired by one predicate.
//!
//! [`compose`] is the one place snapshot and pending meet. What it writes is
//! the fact an unflushed message already is — a pending deposit (§5.1 #11) —
//! so the `✉n` badge, the Inbox tab and the §11 inbox-composer queue carry it
//! with no new seat. A start has no agent to hang it on, so the fold mints a
//! **pending conversation** keyed by the minted §3.3 name: one row in the §11
//! list, in the operator's own words.

use super::Snapshot;
use std::path::{Path, PathBuf};

/// **Whether the world has caught up** (§7.2) — the reconciliation predicates
/// and the one lookup they share, in their own file at §12's budget on the seam
/// this module's doc already draws: above is what an echo *is*, there is what
/// retires it.
mod reconcile;

use reconcile::index_of;

/// Who an echo addresses. The two arms are the two things that can be true when
/// a message is sent, and the difference between them is real: a start has no
/// agent id yet (only the §3.3 name it minted), and a start focuses what it
/// started (§3.4) while a follow-up must not — the operator was already there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Target {
    /// A §3.4 start: the minted §3.3 name, the only identity the conversation
    /// has until the detached driver writes its branch.
    Conversation(String),
    /// A §8.2 follow-up: the agent id already on the roster.
    Agent(String),
}

/// One message yog has sent that the derivation has not shown yet — the §3.4
/// start claim, carrying the operator's text (§7.2). Per-instance RAM (§5.3,
/// §13.1); nothing about it is written down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Echo {
    pub(crate) ws: PathBuf,
    pub(crate) target: Target,
    /// The minted §3.3 name a **start** was born under, for a start; `None` for
    /// a follow-up, which was born addressing an id. Unlike
    /// [`target`](Self::target) it never changes — it is what the conversation
    /// *was called*, not what it *is* — so the two are one fact each rather than
    /// two copies of one (bl-56c6). Two seats still need it after the swap: the
    /// composer's draft is keyed by the identity it was typed against, and the
    /// §11 row leads by the minted name until the answered list catches up.
    pub(crate) born: Option<String>,
    /// The operator's text, verbatim — the payload the composer sent.
    pub(crate) text: String,
    /// **Sends the operator made at this conversation before it had an
    /// address** (§3.4, bl-56c6) — held by yog, never fired at a name that
    /// resolves nowhere, and posted in this order the instant the start
    /// resolves ([`held`](self::held)). Empty for every echo that is not an
    /// unresolved start, which is the general path at zero items.
    pub(crate) held: Vec<held::HeldSend>,
    /// How many messages had ever landed on the target when this was made —
    /// the `NNN` counter's high-water mark (§5.1 #12), the reconciliation
    /// baseline. A high-water rather than a file count (bl-fde5): compaction
    /// deletes files mid-flight, and a shrunken count would strand the echo
    /// behind a baseline no landing could ever pass. Zero for a start, whose
    /// root does not exist.
    pub(crate) baseline: usize,
    /// How many deposits the §11 queue seat could show when the act was
    /// **queued** — the second, narrower baseline (§7.2, bl-78d8), counted over
    /// the inbox listing rather than over `messages/`. Zero for a start, which
    /// is queued against no inbox at all, and zero for a seat whose standing
    /// ask had not answered yet: both showed nothing, which is the general path
    /// at zero items rather than a case of its own.
    pub(crate) queue_baseline: usize,
    /// Wall-clock seconds at the send: the deposit header's `at`, and the
    /// recency that lifts the row.
    pub(crate) at_unix: i64,
}

impl Echo {
    /// The echo a fired §3.4 start leaves: the minted name, the goal verbatim,
    /// and a zero baseline — the root does not exist, so any landed message
    /// under that name is the one this stands in for.
    pub(crate) fn started(ws: &Path, conversation: &str, goal: &str, at_unix: i64) -> Self {
        Self {
            ws: ws.to_path_buf(),
            target: Target::Conversation(conversation.to_owned()),
            born: Some(conversation.to_owned()),
            text: goal.to_owned(),
            held: Vec::new(),
            baseline: 0,
            queue_baseline: 0,
            at_unix,
        }
    }

    /// The echo a §8.2 `message` leaves: the agent it was aimed at, the counter
    /// high-water already landed there — read off the derivation the gesture
    /// was fired against — and `queued`, the queue seat's own baseline, which
    /// only the seat that fired can know (it is what that seat's standing ask
    /// showed *before* the piped verb ran).
    pub(crate) fn messaged(
        snap: &Snapshot,
        ws: &Path,
        agent: &str,
        content: &str,
        queued: usize,
        at_unix: i64,
    ) -> Self {
        let target = Target::Agent(agent.to_owned());
        let baseline = index_of(snap, ws, &target)
            .and_then(|i| snap.trees.get(ws)?.agents.get(i))
            .map_or(0, |a| a.messages);
        Self {
            ws: ws.to_path_buf(),
            target,
            born: None,
            text: content.to_owned(),
            held: Vec::new(),
            baseline,
            queue_baseline: queued,
            at_unix,
        }
    }

    /// Whether this echo speaks for the conversation `agent` names — by the id
    /// it has, or by the §3.3 name it was born under while it has none. The one
    /// predicate every seat asks before folding its optimism on, so a seat
    /// holding a minted name and a seat holding an id are asking one question.
    pub(crate) fn addresses(&self, agent: &str) -> bool {
        match &self.target {
            Target::Conversation(name) => name == agent,
            Target::Agent(id) => id == agent,
        }
    }

    /// **The identity the pending row is minted under**, when there is one
    /// (§3.4, bl-56c6): `(what addresses it, what it is called)`.
    ///
    /// Two arms, one rule — *a conversation yog started and the world has not
    /// shown yet is a row of its own*:
    ///
    /// - an **unresolved** start is addressed and named by the minted §3.3
    ///   name, the only identity it has;
    /// - a **resolved** one is addressed by the id its branch brought and still
    ///   named by the name it was born under, because the answered §11 list
    ///   lands an ask period behind the derivation that resolved it — without
    ///   this the conversation blinks out of the list at exactly the handover
    ///   (bl-56c6 D9), and it is no invention: the derivation is what said the
    ///   root exists.
    ///
    /// `None` for a follow-up, whose conversation the world either carries or
    /// has lost — inventing a row for it would be a false definite.
    pub(crate) fn pending_identity(&self) -> Option<(String, String)> {
        let name = self.born.clone()?;
        match &self.target {
            Target::Conversation(_) => Some((name.clone(), name)),
            Target::Agent(id) => Some((id.clone(), name)),
        }
    }
}

/// **The sends yog holds while a conversation has no address** (§3.4, bl-56c6)
/// — the window between Enter and the driver's branch, in its own file at §12's
/// budget. Nothing here is a second pending concept: a held send is one more
/// deposit on the same echo, and releasing it is the claim resolving.
pub(crate) mod held;

/// **What a pending conversation looks like to a seat** (§3.4, §5.1 #11) — the
/// deposits an echo stands for and the synthetic agent that carries them, cut
/// off this file at §12's budget on the seam it already draws: above is what an
/// echo *is* and when it retires, there is the shape it wears on the glass.
mod pending;

/// **The echo folded into the snapshot** (§7.2) — the fold this module's doc
/// calls the one place snapshot and pending meet, in its own file at §12's
/// budget (bl-78d8). Re-exported so `echo::compose` stays the one name every
/// caller and every citation knows it by.
mod fold;
pub(crate) use fold::compose;

/// **The echo at the row altitude** (REMOTE §9.7, bl-44e9) — the same fold, over
/// an answered §11 list instead of over a snapshot, because that surface reads a
/// `Reply` now. Its own file at §12's cap; the reasoning is its own doc.
pub(crate) mod rows;

/// **What the frame asks of the echo** (bl-b4b5) — the two `AppModel` doors the
/// shell folds an answer through, cut off this file at §12's budget on the seam
/// it already had: above is what an echo *is* and how it retires, and there is
/// what a seat does with one.
mod seat;

#[cfg(test)]
mod tests;
