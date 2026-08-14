//! The §11 conversation-list **row** (§15 Z9): the projection of one
//! **subtree** to what the list paints — the aggregated state badge, the capped
//! first-line preview, the age, the §11 live-activity class, the attention
//! count and the §3.3 ball. Pure over the injected snapshot, the seen closure
//! and the caller's clock. [`super`] answers the structural questions this
//! builds on (which agents form a conversation, what it is called, is it live,
//! what is in flight in it).
//!
//! The subtree is **any** member's, not only a root's (bl-fa82): a row is the
//! subtree rooted at its agent, and the depth-0 case is the whole conversation.
//! [`build`] is the all-collapsed list — one row per root — which is
//! [`expand::visible_rows`](super::expand::visible_rows) with an empty set.

use super::display_name;
use super::flight::{Flight, flight};
use crate::attention;
use crate::git_tree::{Agent, AgentState, DescentRow, children_of};
use crate::projects::join::JoinState;
use crate::transcript::Tone;
use crate::ui_state::SeenKind;

/// First-line preview cap (characters); the row is a glance, not a transcript.
const PREVIEW_CAP: usize = 80;

/// The conversation's associated start-flow ball (DESIGN §3.2, §3.3, §3.5): the
/// `id` is the goal stamp — source 1, the *only* per-conversation attribution
/// that exists (§3.2: agent-picked balls bind at the workspace level, not the
/// conversation). The join facts (`state`/`title`/`badge`) come from the §3.5
/// claimant join and are `None` when no live/closed ball matches the stamped id
/// here (project unfetched, or a stray id) — the badge still renders from source 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvBall {
    pub id: String,
    pub state: Option<JoinState>,
    pub title: Option<String>,
    pub badge: Option<String>,
}

/// One list row (§11): the id of the agent this row is rooted at, the state
/// badge aggregated over **its** subtree (+ §10 uncertainty), the first-line
/// preview, seconds since that subtree's last activity, the live-activity
/// class, its attention count, its member count (this agent + its descendants),
/// how deep it hangs and how many children it dispatched itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvRow {
    /// The agent this row is the subtree of — the conversation root at depth 0,
    /// and the member itself at every depth below (bl-fa82).
    pub root_id: String,
    pub state: AgentState,
    pub uncertain: bool,
    pub preview: String,
    pub age_secs: i64,
    /// What kind of work is in flight here (§5.1 #28), by the operator's
    /// priority `inference > tools > subagents`; `None` when the conversation
    /// is at rest. The one carrier of "this row pulses" — there is no separate
    /// streaming flag, because `Some(Inference)` *is* that fact.
    pub flight: Option<Flight>,
    pub attention: usize,
    pub members: usize,
    /// How far this row hangs under its conversation root (§11's per-depth
    /// indent and its `↳` elbow): 0 for a root, +1 per descent generation.
    pub depth: usize,
    /// How many agents this one dispatched **itself** — the strict §5.1 #8
    /// [`children_of`] count, never the Stop menu's looser prefix test. The
    /// subagent field's first number; the second is [`total`](Self::total),
    /// derived from `members` rather than stored beside it.
    pub direct: usize,
    /// The conversation's start-flow ball (§3.3), derived from the root's goal
    /// stamp and coloured by the §3.5 join. `None` for a bare/path conversation.
    pub ball: Option<ConvBall>,
    /// The conversation's own name (§3.3): the root's
    /// [`name_fact`](crate::git_tree::Agent::name_fact) — the lernie-stored
    /// blob, else the legacy goal-stamp parse — the display ladder's first
    /// rung. `None` for a foreign or hand-typed root, which lands on the
    /// payload line or the id.
    pub name: Option<String>,
    /// Whether [`name`](ConvRow::name) is the **legacy display-only rung**
    /// (bl-8068, [`Agent::name_display_only`]): a goal-stamp parse no
    /// lernie-stored fact backs, so peers cannot message the conversation by
    /// this name. The row hovers [`crate::theme::NAME_DISPLAY_ONLY`] on it,
    /// and the boundary withholds the `name` key entirely.
    pub name_display_only: bool,
    /// The conversation's **standing alignment verdict** (VISION §4.9, rung
    /// V6): the worst of its members' latest monitor checks, with the reason
    /// and the sha it read. Derived from the ops tail on every build, never a
    /// stored flag — so an unarmed workspace, or one whose verdicts have aged
    /// out of the rendered tail, simply carries `None` and renders nothing.
    pub verdict: Option<crate::monitor::Check>,
    /// Whether §8.2's `Stop` is offered on this row — its agent holds a driver
    /// right now ([`stop_enabled`](crate::actions::stop_enabled)). **Not
    /// [`state`](ConvRow::state)**, which is the badge aggregated over the
    /// row's whole subtree: a quiet root with a working child paints Live and
    /// has nothing to kill. On the row since bl-1eb0, because a seat that
    /// cannot answer it cannot paint the row's own menu.
    pub stoppable: bool,
    /// Whether the `+children` cascade is offered beside it — some other agent
    /// id here descends from this one by the Stop menu's **looser prefix test**
    /// ([`stop_children_offered`](crate::actions::stop_children_offered)),
    /// which is `lernie stop --children`'s own rule and deliberately not the
    /// strict §5.1 #8 descent [`members`](ConvRow::members) counts.
    pub stop_children: bool,
    /// How solid the row paints (§11, bl-915e): [`Tone::Weak`] while this is
    /// §7.2's **pending conversation** — a start yog has fired whose driver has
    /// not written a branch, so the row is only yog's own word for it — and
    /// [`Tone::Plain`] once the derivation carries it. The one input is
    /// [`Agent::in_memory`], so nothing here decides; it reads.
    pub tone: Tone,
}

impl ConvRow {
    /// Whether this row's agent has any descent below it — the §11 gate for
    /// painting the subagent field at all, so a lone root's row carries no
    /// arrow and nothing to unfold.
    pub fn has_children(&self) -> bool {
        self.members > 1
    }

    /// The subagent field's **total**: every agent under this one, at any
    /// depth. Derived from `members` (this agent plus its descendants) rather
    /// than stored beside it — two integers deriving from each other are two
    /// chances to disagree.
    pub fn total(&self) -> usize {
        self.members.saturating_sub(1)
    }

    /// The row's display name — [`display_name`]'s ladder over this row's own
    /// rungs, the one naming rule every seat shares.
    pub fn display_name(&self) -> String {
        display_name(self.name.as_deref(), &self.preview, &self.root_id)
    }

    /// The weak subtitle beside the title (§11: "the name with its preview weak
    /// beside it"): the first payload line — empty when the ladder already spent
    /// it as the title, so no row ever says the same thing twice.
    pub fn subtitle(&self) -> String {
        match self.name {
            Some(_) => self.preview.clone(),
            None => String::new(),
        }
    }
}

/// Build the §11 conversation list for one workspace's agents. `ws` is the
/// seen-key path (§4.1) and `now_unix` the caller's wall clock (the age input
/// — injected so the derivation stays pure).
pub fn build(
    agents: &[Agent],
    ws: &str,
    seen: &dyn Fn(SeenKind, &str, &str, &str) -> bool,
    now_unix: i64,
    ball: &dyn Fn(&str) -> ConvBall,
    checks: &[crate::monitor::Check],
) -> Vec<ConvRow> {
    super::expand::visible_rows(
        agents,
        ws,
        seen,
        now_unix,
        ball,
        checks,
        &std::collections::HashSet::new(),
    )
}

/// A compact age label for a row: `42s`, `7m`, `3h`, `2d`. Negative (clock
/// skew) clamps to `0s`.
pub fn age_label(age_secs: i64) -> String {
    let s = age_secs.max(0);
    match s {
        0..=59 => format!("{s}s"),
        60..=3599 => format!("{}m", s / 60),
        3600..=86399 => format!("{}h", s / 3600),
        _ => format!("{}d", s / 86400),
    }
}

/// Project one subtree to its row, returning `(last_active, row)` — the sort
/// key rides beside the row so the age is derived once.
///
/// `subtree` is a pre-order slice whose **first** element is the row's own
/// agent (§2.3 descent order), so its depth is the row's depth and the slice is
/// exactly what the row aggregates. A depth-0 slice is a whole conversation;
/// any deeper slice is a member's own descent, projected identically — that
/// sameness is the whole of bl-fa82's reframe.
pub(super) fn row(
    agents: &[Agent],
    subtree: &[DescentRow],
    ws: &str,
    seen: &dyn Fn(SeenKind, &str, &str, &str) -> bool,
    now_unix: i64,
    ball: &dyn Fn(&str) -> ConvBall,
    checks: &[crate::monitor::Check],
) -> (i64, ConvRow) {
    let members: Vec<&Agent> = subtree.iter().filter_map(|r| agents.get(r.index)).collect();
    // `conversations` only emits non-empty subtrees; a filtered-empty one
    // yields a harmless placeholder root reference via `first`.
    let root = members.first().copied();
    let root_id = root.map(|a| a.agent_id.clone()).unwrap_or_default();
    // The per-conversation ball is the *root's* goal stamp (§3.2): a child's
    // goal.md is its own sub-task, never a yog ball attribution.
    let conv_ball = root.and_then(|a| a.goal_ball.as_deref()).map(ball);
    let (state, uncertain) = agg_state(&members, root);
    // One recency fact, gathered at snapshot time (§3.5) and spent twice: it
    // orders the list and it dates the row. `last_action_unix` already folds
    // the tip commit, the newest `messages/` entry and the live tail (bl-cad5),
    // so nothing here stats disk from the render path.
    let last_active = members
        .iter()
        .map(|a| a.last_action_unix)
        .max()
        .unwrap_or(0);
    let attention_count = members
        .iter()
        .filter(|a| attention::attention(a, ws, seen).any())
        .count();
    let row = ConvRow {
        // The strict descent-id children (§5.1 #8) — the subagent field's
        // `direct`, read from the one home that answers it.
        direct: children_of(agents, &root_id).len(),
        // The row's own §8.2 gates (bl-1eb0), through the same two predicates
        // the composer's buttons and the key bindings run — one implementation,
        // answered where the row is derived rather than re-derived per paint
        // against a roster the seat had to be holding.
        stoppable: crate::actions::stop_enabled(Some(&root_id), agents),
        stop_children: crate::actions::stop_children_offered(&root_id, agents),
        depth: subtree.first().map_or(0, |r| r.depth),
        root_id,
        state,
        uncertain,
        preview: preview(root),
        age_secs: (now_unix - last_active).max(0),
        flight: flight(&members),
        attention: attention_count,
        members: members.len(),
        ball: conv_ball,
        // Faded while the conversation is only in memory (§11, the faded-send
        // ruling): a row with no branch behind it is a send yog has
        // made and the world has not yet confirmed.
        tone: if root.is_some_and(Agent::in_memory) {
            Tone::Weak
        } else {
            Tone::Plain
        },
        name: root.and_then(crate::git_tree::Agent::name_fact),
        name_display_only: root.is_some_and(Agent::name_display_only),
        verdict: crate::monitor::row::worst(
            checks,
            ws,
            &members
                .iter()
                .map(|a| a.agent_id.clone())
                .collect::<Vec<_>>(),
        ),
    };
    (last_active, row)
}

/// The aggregated badge state (§11): InFlight if any member streams, else Live
/// if any member holds a driver, else the root's settled state. The
/// uncertainty flag rides with whichever agent decided.
fn agg_state(members: &[&Agent], root: Option<&Agent>) -> (AgentState, bool) {
    for want in [AgentState::InFlight, AgentState::Live] {
        if let Some(a) = members.iter().find(|a| a.state == want) {
            return (want, a.state_uncertain);
        }
    }
    root.map_or((AgentState::Stopped, false), |a| {
        (a.state, a.state_uncertain)
    })
}

/// The row's first-line preview: the root's request preview, else its live
/// streaming text, else empty — first line only, capped at [`PREVIEW_CAP`].
pub(super) fn preview(root: Option<&Agent>) -> String {
    let text = root
        .and_then(|a| a.preview.clone().or_else(|| a.stream.text.clone()))
        .unwrap_or_default();
    text.lines()
        .next()
        .unwrap_or("")
        .chars()
        .take(PREVIEW_CAP)
        .collect()
}
