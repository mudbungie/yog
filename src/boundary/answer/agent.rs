//! One conversation **as a seat sees it** (REMOTE §9.4, bl-1eb0) — the
//! [`Agent`](crate::git_tree::Agent)'s wire projection.
//!
//! The §11 centre pane paints an identity line, a mark row, a live badge and a
//! pair of §8.2 verb gates about whatever is selected. Every one of those was
//! derived on the frame thread out of `GitTree::agents` — the engine's fat,
//! disk-derived tree, which a thin client will never hold. Nothing was missing
//! from the *derivations*; what was missing was a **spelling**, so this is that
//! and nothing more: scalars and two small lists off the published snapshot,
//! each one a call into the module that already owns it (§2.3 descent, §3.3
//! naming, §3.5 liveness, §6 marks, §8.2 enablement, §5.1 #28/#28b activity).
//!
//! It is the seventh member of the conversation-addressed family
//! ([`super::inspector`]'s six), so it shares their address and their envelope
//! and adds no vocabulary of its own — REMOTE §3's rule that a capability a
//! client needs is added to the boundary, on every face, never to a wire.
//!
//! **Absence is a value here, not a refusal.** An agent the snapshot does not
//! carry — an untracked tree, an id that has been deleted, a workspace not yet
//! derived — reads as its own root, unnamed, [`Stopped`](AgentState::Stopped),
//! unmarked and unstoppable, which is what such a conversation honestly is
//! (the same ruling [`super::inspector::steps`] makes for its liveness).

use std::path::Path;

use crate::actions::{nudge_enabled, stop_children_offered, stop_enabled};
use crate::app::Snapshot;
use crate::control::hold::Held;
use crate::git_tree::{Agent, AgentMark, AgentState};
use crate::nav::convs::{
    Flight, ancestors, conversation_flight, display_name_of, root_of, seats, strip,
};
use crate::ui_state::UiState;

/// One conversation's seat facts: who is selected, what its conversation is
/// called, what it is doing, what it wears and what may be done to it — plus,
/// since bl-296f, the two §11 accessories that describe the same subtree's live
/// activity (the mark's seats and the in-flight strip).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentView {
    /// The agent asked about — echoed back, so a reply identifies itself.
    pub agent_id: String,
    /// The conversation root it belongs to (§2.3 descent); itself for a root,
    /// and for an id this workspace does not carry.
    pub root: String,
    /// The descent-id chain **above** it, outermost first — what §11's
    /// visible-selection invariant unfolds so a selection lands on a row the
    /// operator can see. Empty for a root and for an id this workspace does not
    /// carry, which is the same answer: nothing to open.
    pub ancestors: Vec<String>,
    /// What the conversation is called: the §3.3 ladder over the **root**, the
    /// one function every naming seat reads — the centre header's title, the
    /// composer's `→ message <name>` line and the transcript's speaker are one
    /// answer, not three.
    pub name: String,
    /// Whether [`name`](Self::name) is the legacy display-only rung (bl-8068):
    /// prose off the root's goal stamp that no litany-stored name fact backs,
    /// so peers cannot address the conversation by it.
    pub display_only: bool,
    /// The agent branch's tip oid — the §5.1 #17 which-config-governs
    /// derivation's input, and empty for an agent the snapshot does not carry.
    pub tip: String,
    /// Its **own** §3.5 liveness, not the subtree aggregate a
    /// [`ConvRow`](crate::nav::convs::ConvRow) badge carries.
    pub state: AgentState,
    /// The `refs/litany/*` marks it wears (§6), in badge order — the derived
    /// badge list, exactly as [`Agent::marks`] folds it.
    pub marks: Vec<AgentMark>,
    /// The parked invocation itself (§8.6, ARCH §3.3) — the one mark that
    /// carries a **value** rather than a watermark oid, because a park is not
    /// acknowledgeable and what the operator needs is what the blob says. The
    /// [`Held`](AgentMark::Held) badge above is this fact folded; the answer
    /// controls need the sentence, so both ride, exactly as they do on
    /// [`Agent`].
    pub held: Option<Held>,
    /// What is in flight anywhere in its **conversation** (§5.1 #28) — the
    /// header badge's class, `None` at rest.
    pub flight: Option<Flight>,
    /// Whether the published snapshot carries this agent at all — the roster
    /// half of §8.2's message gate ([`message_enabled`](crate::actions::message_enabled);
    /// the text half is the composer's, and no boundary can know it).
    pub present: bool,
    /// Whether §8.2's `Nudge` is offered — a settled conversation the model can
    /// be run on again ([`nudge_enabled`]).
    pub nudgeable: bool,
    /// Whether §8.2's `Stop` is offered (it holds a driver right now).
    pub stoppable: bool,
    /// Whether the `+children` cascade is offered beside it — the Stop menu's
    /// looser prefix test, not the strict §5.1 #8 descent.
    pub stop_children: bool,
    /// **The §11 live mark's seats** (§5.1 #28b, bl-296f): the eye — the agent
    /// the operator is talking to — then its subagents in §2.3 descent order,
    /// each with what it is doing right now. Empty for an id this workspace
    /// does not carry, which is the mark at rest and not a case of its own.
    ///
    /// It rides here rather than on a [`ConvRow`](crate::nav::convs::ConvRow)
    /// for the reason bl-48ae gave for `marks` and `held`: it is a fact about
    /// **one conversation's** agents, one per circle, and putting a per-agent
    /// activity list on every row of a workspace's forest to serve the one row
    /// that is selected is the altitude mistake `ConvRow`'s own definition
    /// exists to prevent.
    pub seats: Vec<crate::nav::convs::Seat>,
    /// **The latest turn was refused at the provider rung** (bl-b43b) — the
    /// §3.5 fact read off the same bytes the state was, carried here because
    /// this is the answer that says what may be done to a conversation and a
    /// refusal is the one rest whose remedy is not a gesture on it at all.
    /// `false` under a held lock, where the question is not asked.
    ///
    /// The provider **row** is deliberately not here: it costs a git read of
    /// the governing roles, it is a fact about one *step*, and the steps
    /// surface already answers it as `auth_row`.
    pub refused: bool,
    /// **Why that latest model call failed**, in one clause (bl-9b88) — the
    /// words behind [`refused`](Self::refused), and the whole of the fact when
    /// the failure was not auth-shaped at all. `None` when the call did not
    /// fail. bl-b43b answered the class here and left the sentence in a
    /// `driver.log` no seat reads.
    pub failure: Option<String>,
    /// **The §11 bottom in-flight strip** (§5.1 #28, bl-905f): the live
    /// characteristics of what is running in this conversation, `None` at rest
    /// — which is what makes an idle window paint no strip at all.
    ///
    /// Its elapsed segment is stamped at the moment the answer is derived
    /// rather than re-rendered per frame, so it advances at the asker's cadence
    /// (REMOTE §9.7's live-tail ruling, taken again at a coarser unit): the
    /// segment is a compact `5s`/`2m` label, and half a second of lag in a
    /// figure that ticks in seconds is not a difference a reader can feel.
    pub strip: Option<crate::nav::convs::FlightStrip>,
    /// **The conversation's priced whole-tree figure** (§3.5, bl-b4b5): the
    /// root agent and its descent (ARCH §6), attributed to itself. It rides
    /// here for [`seats`](Self::seats)' reason exactly — a fact about **one
    /// conversation's** subtree, like the strip beside it — and its workspace
    /// twin is `Query::WorkspaceBalls`' per-ball figure, the other half of the
    /// §3.5 pair the §11 settings band paints.
    pub spend: crate::spend::Figure,
    /// **How full its context is** (§5.1 #35): the prompt its root agent's
    /// latest step sent, against the window `models.yaml` declares for the
    /// model that sent it. `None` when nothing measured can be said, which is
    /// the ordinary answer for a conversation that has not run yet.
    ///
    /// Deliberately apart from [`spend`](Self::spend) in what it answers:
    /// spend is the whole descent's cumulative burn, fullness is this
    /// conversation's one current prompt.
    pub context: Option<crate::context::Fullness>,
}

/// Derive one conversation's seat facts off the published snapshot. Pure: every
/// field is a fold over the workspace's agent set, so this costs no disk read
/// and answers the same thing at the chokepoint as it does on a frame.
///
/// `now_unix` is the caller's wall clock, and only the §11
/// [`strip`](AgentView::strip)'s elapsed segment spends it — everything else
/// here is structural. `ui` is the durable document, read for the §3.5 price
/// table alone (bl-b4b5): a figure's money column is `ui.json`'s severability
/// gate, so an empty table renders tokens and nothing else.
pub fn agent(snap: &Snapshot, ui: &UiState, ws: &Path, agent: &str, now_unix: i64) -> AgentView {
    let agents = snap
        .trees
        .get(ws)
        .map(|tree| tree.agents.clone())
        .unwrap_or_default();
    let root = root_of(&agents, agent).unwrap_or_else(|| agent.to_owned());
    let found = agents.iter().find(|a| a.agent_id == agent);
    // The §3.5/§5.1 #35 pair, over the worker's already-walked bills — the same
    // fold every other figure is a filter over (bl-9dd4), so a conversation's
    // budget line costs this answer no disk read.
    let bills = snap.bills.get(ws).cloned().unwrap_or_default();
    AgentView {
        spend: crate::spend::of_conversation(&bills, &root, &ui.prices()),
        context: crate::context::of_conversation(&bills, &root, &snap.windows),
        name: display_name_of(&agents, &root),
        display_only: agents
            .iter()
            .any(|a| a.agent_id == root && a.name_display_only()),
        tip: found.map(|a| a.tip_oid.clone()).unwrap_or_default(),
        state: found.map_or(AgentState::Stopped, |a| a.state),
        refused: found.is_some_and(Agent::refused),
        failure: found.and_then(|a| a.failure.as_deref().map(crate::git_tree::clause)),
        marks: found.map(Agent::marks).unwrap_or_default(),
        held: found.and_then(|a| a.held.clone()),
        present: found.is_some(),
        nudgeable: nudge_enabled(Some(agent), &agents),
        flight: conversation_flight(&agents, &root),
        stoppable: stop_enabled(Some(agent), &agents),
        stop_children: stop_children_offered(agent, &agents),
        ancestors: ancestors(&agents, agent),
        seats: seats(&agents, &root),
        strip: strip(&agents, &root, now_unix),
        agent_id: agent.to_owned(),
        root,
    }
}

#[cfg(test)]
mod tests;
