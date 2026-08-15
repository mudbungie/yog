//! **The selection's own facts, selected out of the answered forest** (REMOTE
//! §9.7, bl-48ae) — [`visible`](super::expand::visible)'s sibling: a second pure
//! selection over the same `Reply::Conversations` payload, this one keyed by
//! which row the operator has picked rather than by which rows are open.
//!
//! Until bl-48ae the §11 seat asked this of `AppModel::focused_conversation`,
//! which re-derived [`AgentView`](crate::boundary::answer::agent::AgentView) off
//! the window's own snapshot on the frame thread — the last in-process read of
//! REMOTE §11's residual, and the one that could not simply become a standing
//! question: four of its consumers are **frame-synchronous**. The composer's
//! target line names the conversation, §11's visible-selection invariant unfolds
//! the ancestors, and two act gates (`x` stops the selection, the §3.6 danger
//! row aims at its root) are read at the click. A fact that landed an ask period
//! after the selection would blink a name, hide the row the operator was just
//! sent to, or refuse a gesture they had just made.
//!
//! **None of them is a wire read, because the forest already answers them.**
//! Since bl-44e9 `Query::Conversations` lands the whole descent forest with
//! per-row rollups, and a `ConvRow` has carried the §8.2 gates since bl-1eb0 —
//! so every one of these facts is a *selection* out of an answer this seat is
//! already holding, and the ask that pays for them is the list's own. Nothing
//! new is asked, nothing is latched, and the answer changes in the same frame
//! the selection does.
//!
//! What is **not** here is the selection's own detail — its config freeze, its
//! §6 marks, its §8.6 park, its `Nudge` gate. Those are facts about one agent
//! rather than about the list, they gate no gesture (an unpainted button cannot
//! be clicked), and they ride the standing `Query::Agent` at the seat
//! ([`crate::shell::seat`]) exactly as the transcript beside them does.
//!
//! Two rules keep this honest. **Depth is the parentage**: the answer is
//! pre-order, so a row's ancestors are exactly the shallower rows above it —
//! the same fact [`parent_of`](super::expand::parent_of) reads one generation
//! at a time. And **absence is a value, not a refusal**, which is
//! [`agent`](crate::boundary::answer::agent::agent)'s own ruling: a selection
//! this forest does not carry reads as its own root, unnamed, unstoppable and
//! not present, because that is what such a conversation honestly is.

use super::row::ConvRow;
use crate::nav::convs::Flight;

/// What a seat knows about its selection the instant it is made: the identity,
/// the conversation it belongs to, the chain §11 unfolds to keep it visible,
/// what the conversation is called, what is in flight in it, and the three gates
/// the composer's verbs read.
///
/// Every field is [`AgentView`](crate::boundary::answer::agent::AgentView)'s
/// under the same name — the parity is pinned in
/// `boundary::answer::agent::tests`, so the two projections of one derivation
/// cannot come to disagree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// The agent asked about — echoed back, so a seat holding this needs
    /// nothing else to name its target.
    pub agent_id: String,
    /// The conversation root it belongs to (§2.3 descent); itself for a root,
    /// and for an id this forest does not carry.
    pub root: String,
    /// The descent chain **above** it, outermost first — what §11's
    /// visible-selection invariant unfolds. Empty for a root and for an absent
    /// id, which is the same answer: nothing to open.
    pub ancestors: Vec<String>,
    /// What the conversation is called: the §3.3 ladder over the **root** row,
    /// which is the one the composer's `→ message <name>` line spends.
    pub name: String,
    /// Whether [`name`](Self::name) is the legacy display-only rung (bl-8068).
    pub display_only: bool,
    /// What is in flight anywhere in its conversation (§5.1 #28) — the root
    /// row's rollup, `None` at rest.
    pub flight: Option<Flight>,
    /// Whether the answered forest carries this agent at all — the roster half
    /// of §8.2's message gate.
    pub present: bool,
    /// Whether §8.2's `Stop` is offered on it (it holds a driver right now).
    pub stoppable: bool,
    /// Whether the `+children` cascade is offered beside it.
    pub stop_children: bool,
}

/// Read `agent_id`'s seat facts out of an answered forest. Pure and total: no
/// row for the id is answered rather than refused, on
/// [`agent`](crate::boundary::answer::agent::agent)'s own ruling.
pub fn selection(rows: &[ConvRow], agent_id: &str) -> Selection {
    let at = rows.iter().position(|r| r.root_id == agent_id);
    let own = at.and_then(|i| rows.get(i));
    let chain = at.map(|i| chain(rows, i)).unwrap_or_default();
    // The conversation's row: the outermost ancestor, else the selection itself
    // when it is already a root. Absent, the ladder's floor answers the name —
    // which is `display_name_of`'s own miss arm, reached the same way.
    let root_row = chain.first().copied().or(own);
    Selection {
        root: root_row.map_or_else(|| agent_id.to_owned(), |r| r.root_id.clone()),
        ancestors: chain.iter().map(|r| r.root_id.clone()).collect(),
        name: root_row.map_or_else(
            || super::id_floor(agent_id).to_owned(),
            ConvRow::display_name,
        ),
        display_only: root_row.is_some_and(|r| r.name_display_only),
        flight: root_row.and_then(|r| r.flight),
        present: own.is_some(),
        stoppable: own.is_some_and(|r| r.stoppable),
        stop_children: own.is_some_and(|r| r.stop_children),
        agent_id: agent_id.to_owned(),
    }
}

/// The rows above `at` that are its ancestors, outermost first: walking back,
/// each row shallower than the shallowest seen so far is the next generation up.
/// Pre-order makes that the whole of the parentage — the rule
/// [`parent_of`](super::expand::parent_of) reads one step at a time, iterated to
/// depth 0.
fn chain(rows: &[ConvRow], at: usize) -> Vec<&ConvRow> {
    let mut want = rows.get(at).map_or(0, |r| r.depth);
    let mut out: Vec<&ConvRow> = Vec::new();
    for r in rows.get(..at).unwrap_or_default().iter().rev() {
        if r.depth < want {
            want = r.depth;
            out.push(r);
            if want == 0 {
                break;
            }
        }
    }
    out.reverse();
    out
}
