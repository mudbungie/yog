//! The roster the §6 predicate feeds: its sort, its two rollups, and the
//! jump-to-next-attention walk (DESIGN §6, §11).
//!
//! [`super`] answers one agent's question — *is this one asking for me, and
//! why*. This answers the questions that only exist across a set: in what
//! order, how many per workspace, how many in total, and which one is next.
//! Every function is pure over the same injected snapshot and `seen` closure;
//! nothing here re-derives a signal, it only ranks and counts what [`super`]
//! already decided.

use super::attention;
use crate::git_tree::{Agent, AgentState, descent_order};
use crate::ui_state::SeenKind;

/// Sort rank for the roster (§6): attention (0) > running (1) > idle (2).
fn rank(agent: &Agent, ws: &str, seen: &dyn Fn(SeenKind, &str, &str, &str) -> bool) -> u8 {
    if attention(agent, ws, seen).any() {
        0
    } else if matches!(agent.state, AgentState::Live | AgentState::InFlight) {
        1
    } else {
        2
    }
}

/// The §6 roster sort within one workspace: attention > running > idle, ties
/// broken by descent order (§2.3). Returns the `agents` indices in that order —
/// stable, so the descent order survives within each rank group.
pub fn sorted_roster(
    agents: &[Agent],
    ws: &str,
    seen: &dyn Fn(SeenKind, &str, &str, &str) -> bool,
) -> Vec<usize> {
    let mut order: Vec<usize> = descent_order(agents)
        .into_iter()
        .map(|row| row.index)
        .collect();
    order.sort_by_key(|&i| agents.get(i).map(|a| rank(a, ws, seen)));
    order
}

/// The §6 workspace rollup: how many agents there have attention. The boolean
/// "workspace has attention" (§6 "max over its agents") is `count > 0`.
pub fn workspace_count(
    agents: &[Agent],
    ws: &str,
    seen: &dyn Fn(SeenKind, &str, &str, &str) -> bool,
) -> usize {
    agents
        .iter()
        .filter(|a| attention(a, ws, seen).any())
        .count()
}

/// The §6 strip total: attention-bearing agents summed across all workspaces,
/// each a `(seen-key path, agent set)` pair (§4.1).
pub fn strip_total(
    workspaces: &[(&str, &[Agent])],
    seen: &dyn Fn(SeenKind, &str, &str, &str) -> bool,
) -> usize {
    workspaces
        .iter()
        .map(|(path, agents)| workspace_count(agents, path, seen))
        .sum()
}

/// One entry in the flattened navigator roster (§6): the workspace's seen-key
/// path, an agent id, and whether that agent bears attention. Owned — it
/// outlives the snapshot borrows it derives from. The `attention` flag is
/// computed once as the roster is built (the rank sort needs it) and carried so
/// [`next_attention`] reads it rather than re-deriving.
#[derive(Clone)]
pub struct RosterKey {
    pub ws: String,
    pub agent_id: String,
    pub attention: bool,
}

impl RosterKey {
    /// Whether this entry is the `(ws, agent)` focus position.
    fn is_at(&self, focus: (&str, &str)) -> bool {
        self.ws == focus.0 && self.agent_id == focus.1
    }
}

/// The full derived roster across workspaces (§6): **path order across**,
/// [`sorted_roster`] **within**, flattened for the navigator and for
/// [`next_attention`]. Each workspace is a `(seen-key path, agent set)` pair.
pub fn roster_order(
    workspaces: &[(&str, &[Agent])],
    seen: &dyn Fn(SeenKind, &str, &str, &str) -> bool,
) -> Vec<RosterKey> {
    let mut wss: Vec<(&str, &[Agent])> = workspaces.to_vec();
    wss.sort_by_key(|(path, _)| *path);
    let mut out = Vec::new();
    for (path, agents) in wss {
        out.extend(
            sorted_roster(agents, path, seen)
                .into_iter()
                .filter_map(|i| agents.get(i))
                .map(|agent| RosterKey {
                    ws: path.to_string(),
                    agent_id: agent.agent_id.clone(),
                    attention: attention(agent, path, seen).any(),
                }),
        );
    }
    out
}

/// Jump-to-next-attention (§6): over the ordered `roster`, the next entry with
/// attention *after* `focus`, wrapping. `focus == None` starts from the front.
/// When `focus` is the only attention it is returned (a full wrap); when
/// nothing has attention, `None`. `focus` is a `(ws, agent)` position.
pub fn next_attention(roster: &[RosterKey], focus: Option<(&str, &str)>) -> Option<RosterKey> {
    let n = roster.len();
    if n == 0 {
        return None;
    }
    let mut start = 0;
    if let Some(f) = focus
        && let Some(i) = roster.iter().position(|e| e.is_at(f))
    {
        start = i + 1;
    }
    (0..n)
        .filter_map(|step| roster.get((start + step) % n))
        .find(|e| e.attention)
        .cloned()
}

/// Step `delta` entries (±1 for ↓/↑, §11 keyboard nav) from `focus` through the
/// ordered `roster`, wrapping. Unlike [`next_attention`] this visits *every*
/// entry, not only attention-bearing ones — it is plain roster traversal. A
/// `None`/unknown focus starts before the front, so `+1` lands on the first
/// entry and `-1` on the last; an empty roster yields `None`.
pub fn step(roster: &[RosterKey], focus: Option<(&str, &str)>, delta: isize) -> Option<RosterKey> {
    let n = roster.len();
    if n == 0 {
        return None;
    }
    let here = focus.and_then(|f| roster.iter().position(|e| e.is_at(f)));
    let next = match here {
        Some(i) => (i as isize + delta).rem_euclid(n as isize) as usize,
        None if delta >= 0 => 0,
        None => n - 1,
    };
    roster.get(next).cloned()
}
