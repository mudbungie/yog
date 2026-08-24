//! **Whether the world has caught up with an echo** (DESIGN §7.2, §3.4) — the
//! reconciliation predicates and the one derivation lookup all three share.
//!
//! Split off [`super`] at §12's budget on the seam that module's own doc keeps
//! drawing — *"above is what an echo is and when it retires"*. What an echo is,
//! and the two ways one is minted, stay there; the questions asked of a
//! **later** derivation are here, and they are asked in exactly three places:
//! a start's claim resolving into an id, the landed-message high-water that
//! retires the echo entire, and the queue-seat count that retires only its §11
//! seat (bl-78d8).
//!
//! Every one of them is a **count or a position**, never a text match: nothing
//! here reads a message body to decide whether the message it stands for has
//! arrived.

use super::{Echo, Snapshot, Target};
use std::path::Path;

impl Echo {
    /// The agent id a *start*'s target has acquired, once the roster carries
    /// the root wearing its minted §3.3 name — the §3.4 claim resolving. The
    /// echo then **takes that id** ([`Target::Agent`]): a conversation getting
    /// its id is what actually happened in the world, so it is one value
    /// changing, not a second one starting. `None` for an already-resolved
    /// target and for a root not written yet, which is the general path with
    /// the branch absent rather than a wait state.
    pub(crate) fn resolved(&self, derived: &Snapshot) -> Option<String> {
        let Target::Conversation(_) = &self.target else {
            return None;
        };
        let i = index_of(derived, &self.ws, &self.target)?;
        Some(derived.trees.get(&self.ws)?.agents.get(i)?.agent_id.clone())
    }

    /// Whether `derived` now shows the message this echo stands in for — the
    /// one reconciliation predicate (§7.2): the target is on the roster and
    /// its `messages/` counter high-water has passed the baseline the echo
    /// recorded. False holds the echo — including across a compaction, which
    /// deletes files but never lowers the counter (bl-fde5).
    pub(crate) fn landed(&self, derived: &Snapshot) -> bool {
        index_of(derived, &self.ws, &self.target)
            .and_then(|i| derived.trees.get(&self.ws)?.agents.get(i))
            .is_some_and(|a| a.messages > self.baseline)
    }

    /// Whether an inbox listing of `shown` deposits already carries the one
    /// this echo stands for (§7.2, bl-78d8) — the **queue seat's** narrower
    /// reconciliation, beside [`landed`](Self::landed)'s. Two predicates
    /// because there are two facts: `landed` asks whether the *derivation*
    /// shows the message (which is also the §3.4 claim's spend), and this asks
    /// only whether the seat can show the *deposit* — which happens far sooner,
    /// because the §8.2 verb is piped and writes the file before the receipt
    /// that mints this echo. Counted, never matched, exactly as `landed` is: a
    /// listing longer than the one the act was queued against is the deposit,
    /// and no text was read to say so.
    pub(crate) fn deposited(&self, shown: usize) -> bool {
        shown > self.queue_baseline
    }
}

/// The index of an echo's target in a workspace's derived agent list: by
/// `name_fact` for a start (the name is the identity until the branch exists),
/// by id for a follow-up. An index rather than a reference so no signature
/// grows a named lifetime (AGENTS.md rule 1).
pub(super) fn index_of(snap: &Snapshot, ws: &Path, target: &Target) -> Option<usize> {
    snap.trees
        .get(ws)?
        .agents
        .iter()
        .position(|a| match target {
            Target::Conversation(name) => a.name_fact().as_deref() == Some(name.as_str()),
            Target::Agent(id) => &a.agent_id == id,
        })
}
