//! The attention model (DESIGN §6, §15 Y10): the derived per-agent predicate,
//! its per-signal detail for badges, the workspace/strip rollups, the
//! jump-to-next-attention control, and the roster sort.
//!
//! Everything here is a **pure function** of injected snapshots — the
//! [`git_tree::Agent`](crate::git_tree::Agent) views plus a `seen`-lookup
//! closure over the `ui.json` watermarks (§4.1). The narrowest coupling: the
//! module needs exactly one query from `ui_state` — "is this evidence oid
//! acknowledged?" ([`UiState::is_seen`](crate::ui_state::UiState::is_seen)) —
//! so it takes that one closure, never the whole document nor a `Clock`.
//!
//! # The predicate (DESIGN §6)
//!
//! [`attention`] is true when any signal fires:
//!
//! 1. **notify** — `notify_oid` present and unseen.
//! 2. **stopped** — the agent is **at rest** (`Quiescent | Stopped`), **not**
//!    abandoned (`abandoned_oid` absent), and its branch tip oid is unseen (the
//!    §6/§4.1 evidence for a rest is the branch tip). Ruled bl-2194: the strip
//!    is a **turn queue**, so rule 2 fires on rest, not on the wound — a clean
//!    turn-end and a failed one differ in the state badge, never in whether
//!    your turn has come. The field, the [`AttentionKind`] and the `ui.json`
//!    key keep the historical name `stopped`; the watermark's identity is the
//!    tip oid, unchanged, which is what makes the widening migration-free.
//! 3. **budget** — `budget_oid` present and unseen.
//! 4. **conflicted** — `conflicted_oid` present and unseen.
//! 5. **mail** — a non-empty `pending` listing **and** the lock is definitely `Free`
//!    (a driver-absence stall). Signals 1–4 are seen-gated on `ui.json`; **mail
//!    is not** — it self-clears when a driver drains the inbox (§6 rule 5).
//! 6. **held** — the capability control parked a tool invocation before it
//!    executed (`refs/lernie/held/<id>`, §8.6). **Not seen-gated**, on mail's
//!    own precedent and for a stronger reason: a park costs the drone no
//!    process and no tokens and *nothing but an answer releases it*, so a
//!    watermark could only hide a conversation that cannot move. It self-clears
//!    when lernie lifts the mark — which happens exactly when the answer lands
//!    and the branch re-adjudicates.
//!
//! Signals 1–4 "re-arm" automatically: the watermark is an oid, so a moved ref
//! (new oid ≠ the seen one) fires again (§4.1 "A moved ref re-notifies").

mod roster;
pub use roster::{
    RosterKey, next_attention, roster_order, sorted_roster, step, strip_total, workspace_count,
};

use crate::git_tree::{Agent, AgentState};
use crate::ui_state::SeenKind;

// The seen-lookup every predicate here takes — does `(kind, ws, agent, oid)`
// carry an acknowledgement watermark in `ui.json` (§6)? Threaded as a bare
// `&dyn Fn(..)` (a type alias `dyn Fn` would bake in `'static` and reject the
// shell's `self`-capturing closure; `&dyn` defaults to the reference's own,
// elided lifetime). Production passes `&|k, w, a, o| ui_state.is_seen(k, w, a, o)`.

/// One firing signal — the per-badge detail (§6, §3.5 badge rendering).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttentionKind {
    Notify,
    Stopped,
    Budget,
    Conflicted,
    Mail,
    /// A tool invocation is parked at the capability boundary (§6 rule 6,
    /// §8.6) — the one signal an answer, not an acknowledgement, clears.
    Held,
}

impl AttentionKind {
    /// The rule in words — why this signal is asking (§6). The **one** home for
    /// that sentence, so the seats that state it rather than badge it (the
    /// bl-e160 desktop alert today) cannot word the same rule two ways. Written
    /// as a clause that completes *"this conversation …"*, since every seat that
    /// spends it has already named the conversation.
    ///
    /// `pub(crate)` per AGENTS.md rule 2: an internal accessor is demoted
    /// rather than cloned to own — the sentence is a `'static` literal and its
    /// only consumer is in-crate ([`crate::alert`]).
    pub(crate) fn says(self) -> &'static str {
        match self {
            Self::Notify => "raised a notify mark",
            Self::Stopped => "came to rest — your turn",
            Self::Budget => "exhausted its budget",
            Self::Conflicted => "has a conflicted branch",
            Self::Mail => "has mail queued and no driver taking it",
            Self::Held => "parked a tool invocation for your answer",
        }
    }
}

/// The per-agent attention detail: which of the six signals fire. The bare
/// predicate is [`Attention::any`]; [`Attention::kinds`] lists the firing
/// kinds for badge rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attention {
    pub notify: bool,
    pub stopped: bool,
    pub budget: bool,
    pub conflicted: bool,
    pub mail: bool,
    pub held: bool,
}

impl Attention {
    /// The §6 predicate: any signal firing.
    pub fn any(self) -> bool {
        self.notify || self.stopped || self.budget || self.conflicted || self.mail || self.held
    }

    /// The firing kinds, in fixed badge order (notify, stop, budget, conflict,
    /// mail, held).
    pub fn kinds(self) -> Vec<AttentionKind> {
        [
            (self.notify, AttentionKind::Notify),
            (self.stopped, AttentionKind::Stopped),
            (self.budget, AttentionKind::Budget),
            (self.conflicted, AttentionKind::Conflicted),
            (self.mail, AttentionKind::Mail),
            (self.held, AttentionKind::Held),
        ]
        .into_iter()
        .filter_map(|(on, kind)| on.then_some(kind))
        .collect()
    }
}

/// The §6 per-agent predicate over injected snapshots. `ws` is the workspace's
/// seen-key path (§4.1 `seen[ws][agent]`).
pub fn attention(
    agent: &Agent,
    ws: &str,
    seen: &dyn Fn(SeenKind, &str, &str, &str) -> bool,
) -> Attention {
    let id = agent.agent_id.as_str();
    let unseen = |kind, oid: &str| !seen(kind, ws, id, oid);
    Attention {
        notify: agent
            .notify_oid
            .as_deref()
            .is_some_and(|o| unseen(SeenKind::Notify, o)),
        stopped: rest_evidence(agent).is_some_and(|o| unseen(SeenKind::Stopped, &o)),
        budget: agent
            .budget_oid
            .as_deref()
            .is_some_and(|o| unseen(SeenKind::Budget, o)),
        conflicted: agent
            .conflicted_oid
            .as_deref()
            .is_some_and(|o| unseen(SeenKind::Conflicted, o)),
        mail: !agent.pending.is_empty() && lock_free(agent),
        // Rule 6 (§8.6): the park itself, unqualified by state. A held branch
        // has *no* driver — the seam exits after writing the mark — so gating
        // this on rest would only restate what the mark already asserts.
        held: agent.held.is_some(),
    }
}

/// The §6 **at rest** state class: a conversation that is not executing —
/// `Quiescent` (it came to rest cleanly) or `Stopped` (it came to rest wounded).
/// Rest is the general condition rule 2 fires on; *which way* it came to rest is
/// the state badge's job, never attention's (ruled bl-2194).
fn at_rest(state: AgentState) -> bool {
    matches!(state, AgentState::Quiescent | AgentState::Stopped)
}

/// The §6 rule-2 evidence for `agent`: the branch tip oid it is resting at, or
/// `None` when it is still running or has been abandoned (`refs/lernie/abandoned`
/// is the will-not-retry assertion that suppresses the rule). **The one home for
/// rule 2's non-watermark gate** — the predicate here and the acknowledgement in
/// `app::focus` both call it, so the two can never drift into two answers.
pub fn rest_evidence(agent: &Agent) -> Option<String> {
    (at_rest(agent.state) && agent.abandoned_oid.is_none()).then(|| agent.tip_oid.clone())
}

/// The present acknowledgement evidence for one agent (§6): every signal oid
/// that exists right now — notify, the rest tip (unless abandoned), budget,
/// conflicted. Recording these as seen is what quiets attention; a later moved
/// ref is a different oid and re-arms (§4.1).
///
/// **The one definition**, read by both entries that acknowledge: the window's
/// focus tick ([`AppModel::focus_agent`](crate::AppModel::focus_agent)) and the
/// boundary's `seen` action
/// ([`queue::mark_seen`](crate::boundary::answer::queue::mark_seen)). Rules 5
/// (mail) and 6 (held) have no oid and appear here by design — each self-clears
/// when the world moves (a driver drains the inbox; lernie lifts the hold mark
/// on the answer's re-adjudication), and no watermark may pretend to answer
/// them.
pub fn evidence(agent: &Agent) -> Vec<(SeenKind, String)> {
    [
        (SeenKind::Notify, agent.notify_oid.clone()),
        (SeenKind::Stopped, rest_evidence(agent)),
        (SeenKind::Budget, agent.budget_oid.clone()),
        (SeenKind::Conflicted, agent.conflicted_oid.clone()),
    ]
    .into_iter()
    .filter_map(|(kind, oid)| oid.map(|oid| (kind, oid)))
    .collect()
}

/// The §6 rule-5 driver-absence condition: the executor lock is definitely
/// `Free`, not merely `Unknown`. The classifier (`git_tree::state`) collapses a
/// `Free` probe to a framing state (the [`at_rest`] pair) with the uncertainty
/// flag *clear*, whereas `Unknown` yields the same framing state with the flag
/// *set* (DESIGN §10). So `Free ⟺ at-rest ∧ ¬uncertain` — a `Live` / `InFlight`
/// agent (lock `Held`) is never mail-stalled; an `Unknown` one is hidden, never
/// a false stall.
fn lock_free(agent: &Agent) -> bool {
    at_rest(agent.state) && !agent.state_uncertain
}

#[cfg(test)]
mod tests;
