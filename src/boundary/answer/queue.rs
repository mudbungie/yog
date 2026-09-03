//! The decision queue (VISION §5 V5.2, DESIGN §8.5): **the §6 attention strip
//! made addressable**. The strip is a count and a jump control at the window;
//! here it is a list of rows an agent can read, answer and hand on — the same
//! predicate, the same order, no second model of "what needs you".
//!
//! Three functions, and the third is the reason this file exists:
//!
//! - [`roster`] is the §6 flattened world roster, the one derivation the ↓/↑
//!   keys walk *and* the queue filters — so the queue can never disagree with
//!   the order the window steps through.
//! - [`queue`] is that roster's attention-bearing subsequence, each entry
//!   carrying why it fires and what it last said.
//! - [`mark_seen`] is the **answer**: it writes the very watermarks
//!   [`focus_agent`](crate::AppModel::focus_agent) writes, from the one
//!   evidence definition ([`attention::evidence`]), so a headless
//!   acknowledgement and a windowed one are the same bytes in `ui.json` and I0
//!   converges the two frontends over one disk.

use crate::app::Snapshot;
use crate::attention::{self, AttentionKind, RosterKey};
use crate::git_tree::{Agent, AgentState};
use crate::nav::{convs, ws_key};
use crate::ui_state::UiState;
use std::path::{Path, PathBuf};

/// One thing waiting on the operator (§6): where it is, what it is called, why
/// it is asking, and what it last said. The address is the pair every
/// conversation gesture already takes (`workspace` + `agent`), so a row is
/// directly answerable — `/message`, `/stop`, `/seen` — with nothing to look up
/// in between. Per **agent**, not per conversation root: the strip counts
/// agents, the ↓ key lands on agents, and a child that raised its hand is the
/// one that must be answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueRow {
    /// The workspace's **name** (§3.1: its leaf), never its path (REMOTE §8,
    /// bl-f5f6, regression bl-22ab). It is the token
    /// [`Action::MarkSeen`](crate::boundary::Action) and every other
    /// conversation gesture takes, so a row's address copies straight into the
    /// next gesture; an engine-local path could not be resolved by the seat
    /// that read it and disclosed the engine's home root besides.
    pub workspace: String,
    pub agent: String,
    /// The §3.3 display ladder's answer for this agent — the one naming rule
    /// every seat shares ([`convs::member_title`]), never a raw id.
    pub display: String,
    pub state: AgentState,
    /// The §10 unobservable-probe flag: the state is the best framing-only
    /// reading, never a false definite.
    pub uncertain: bool,
    /// Which of the §6 signals fire, in badge order.
    pub signals: Vec<AttentionKind>,
    /// The conversation's first payload line — what the window paints beside
    /// the row, so a queue reader sees what a looker sees.
    pub preview: String,
    pub age_secs: i64,
    /// Undelivered inbox deposits (§6 rule 5): the `mail` signal's own count.
    pub pending: usize,
    /// The invocation parked at this conversation's capability boundary (§6
    /// rule 6, §8.6), when one is: which `tool_use`, which tool, and the
    /// control's sentence — the tool, an input summary, the computed effect
    /// class and the evidence. `None` for every row nothing is holding.
    ///
    /// It rides the row rather than a query of its own because §6 already
    /// *is* "what needs you", and a second list of parked drones would be a
    /// second model of that — the thing this module exists to prevent. A
    /// headless teleoperator therefore sees a park and answers it
    /// (`/answer pass`) without learning a new read.
    pub held: Option<crate::control::hold::Held>,
    /// **Why this conversation's latest model call failed**, in one clause
    /// (bl-9b88) — `None` when it did not. It rides the row for `held`'s own
    /// reason: §6 already *is* "what needs you", and the whole point of a
    /// queue row is that it is answerable without a second read. A `refused`
    /// signal says the class and this says the sentence, so an operator sees
    /// *which* credential the world is waiting on rather than that some
    /// credential is.
    pub failure: Option<String>,
    /// **The flag raised on this conversation** (§6 rule 7, VISION §4.9,
    /// bl-6f2f): when, and why in the raiser's own words. `None` for every row
    /// nobody flagged.
    ///
    /// It rides the row for `held`'s reason exactly — a queue row exists to be
    /// answerable without a second read, and a signal that says "look at this"
    /// and cannot say why is a signal the operator must go hunting through
    /// `/ops` to act on. That hunt was the whole defect: the row was written
    /// and nothing carried it anywhere an operator looks.
    pub flag: Option<crate::monitor::Flag>,
}

/// **`seen`'s receipt** (bl-5cfe): the item the acknowledgement landed on, and
/// the queue that remains after it. Both, because either alone lies — the
/// remainder cannot say what was acted on (the acted-on row is precisely the
/// row it no longer holds), and the item alone would cost the teleoperator the
/// read that makes the loop one gesture per decision.
///
/// The address is the pair every conversation gesture takes, in the §3.1
/// vocabulary [`QueueRow`] spells: the workspace's **name**, never its path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Acknowledged {
    pub workspace: String,
    pub agent: String,
    pub remaining: Vec<QueueRow>,
}

/// The §6 flattened roster across every enumerated workspace — **the** roster,
/// shared by the window's ↑/↓ and jump-to-next-attention
/// ([`AppModel::pick_roster`](crate::AppModel)) and by [`queue`]. Path order
/// across workspaces, [`sorted_roster`](attention::sorted_roster) within.
pub fn roster(snap: &Snapshot, ui: &UiState) -> Vec<RosterKey> {
    let mut keyed: Vec<(PathBuf, String)> = snap
        .workspaces
        .iter()
        .filter(|w| snap.trees.contains_key(&w.path))
        .map(|w| (w.path.clone(), ws_key(&w.path)))
        .collect();
    keyed.sort_by(|a, b| a.1.cmp(&b.1));
    let wss: Vec<(&str, &[Agent])> = keyed
        .iter()
        .filter_map(|(p, k)| snap.trees.get(p).map(|t| (k.as_str(), t.agents.as_slice())))
        .collect();
    let seen = |k, w: &str, a: &str, o: &str| ui.is_seen(k, w, a, o);
    attention::roster_order(&wss, &seen)
}

/// The queue: the roster's attention-bearing entries, in roster order. Empty is
/// the ordinary answer — nothing needs you — never a special case.
pub fn queue(snap: &Snapshot, ui: &UiState, now_unix: i64) -> Vec<QueueRow> {
    roster(snap, ui)
        .into_iter()
        .filter(|key| key.attention)
        .filter_map(|key| row(snap, ui, &key, now_unix))
        .collect()
}

/// One row from a roster entry. `None` only if the snapshot moved out from
/// under the roster it was just built from — unreachable in one pass, and a
/// dropped row rather than an invented one if it ever is.
fn row(snap: &Snapshot, ui: &UiState, key: &RosterKey, now_unix: i64) -> Option<QueueRow> {
    let workspace = PathBuf::from(&key.ws);
    let agents = &snap.trees.get(&workspace)?.agents;
    let agent = agents.iter().find(|a| a.agent_id == key.agent_id)?;
    let seen = |k, w: &str, a: &str, o: &str| ui.is_seen(k, w, a, o);
    Some(QueueRow {
        workspace: snap.ws_name(&workspace),
        agent: agent.agent_id.clone(),
        display: convs::member_title(agent),
        state: agent.state,
        uncertain: agent.state_uncertain,
        signals: attention::attention(agent, &key.ws, &seen).kinds(),
        preview: convs::preview_of(agent),
        age_secs: (now_unix - agent.last_action_unix).max(0),
        pending: agent.pending.len(),
        held: agent.held.clone(),
        failure: agent.failure.as_deref().map(crate::git_tree::clause),
        flag: agent.flagged.clone(),
    })
}

/// Acknowledge `(ws, agent)` — the headless spelling of what focusing a
/// conversation does (§6): record its present evidence oids as seen. One
/// definition of the evidence ([`attention::evidence`]) serves both entries, so
/// widening a signal could never leave one of them behind.
///
/// Refuses a conversation the published snapshot does not carry: a gesture is
/// an instruction, and an acknowledgement aimed at nothing must say so rather
/// than report a silent success.
pub fn mark_seen(snap: &Snapshot, ui: &mut UiState, ws: &Path, agent: &str) -> Result<(), String> {
    let marks = snap
        .trees
        .get(ws)
        .and_then(|t| t.agents.iter().find(|a| a.agent_id == agent))
        .map(attention::evidence)
        // The §3.1 name, never the path (REMOTE §8.1, bl-ef16) — the token the
        // gesture named this workspace by, and the one a remote seat can read.
        .ok_or_else(|| format!("no conversation {agent:?} in {:?}", snap.ws_name(ws)))?;
    ui.record_seen(&ws_key(ws), agent, &marks);
    Ok(())
}

#[cfg(test)]
mod tests;
