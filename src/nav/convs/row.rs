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
//! [`expand::visible`](super::expand::visible) over
//! [`expand::forest_rows`](super::expand::forest_rows) with an empty set.

use super::flight::flight;
use crate::attention;
use crate::git_tree::{Agent, AgentState, DescentRow, children_of};
use crate::nav::convs::Tone;
use crate::ui_state::SeenKind;

/// The row's own inert shapes — what the §11 list paints, and the two facts
/// derived off a row rather than stored beside it.
pub mod model;
pub use model::{ConvBall, ConvRow};

/// First-line preview cap (characters); the row is a glance, not a transcript.
const PREVIEW_CAP: usize = 80;

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
    super::expand::visible(
        &super::expand::forest_rows(agents, ws, seen, now_unix, ball, checks),
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
    // One recency fact, gathered at snapshot time (§3.5) and spent three ways:
    // it orders the list, it ages the row and it dates it.
    // `last_action_unix` already folds
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
        // …and spent a third time, unsubtracted (bl-b7d9): a roster that can
        // only say a distance cannot stamp a row, and a client subtracting its
        // own clock would say a different time than the engine's.
        last_active_unix: last_active,
        flight: flight(&members),
        attention: attention_count,
        members: members.len(),
        ball: conv_ball,
        // Faded while the conversation is only in memory (§11, the faded-send
        // ruling): a row with no branch behind it is a send yog has
        // made and the world has not yet confirmed.
        tone: tone_of(root),
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

/// **How solid, and how well, the row paints** (§11, bl-915e; widened bl-b43b).
///
/// [`Tone::Weak`] while the row is §7.2's *pending conversation* — a start yog
/// has fired whose driver has not written a branch, so the row is only yog's
/// own word for it. [`Tone::Bad`] for a conversation whose latest turn was
/// **refused at the provider rung**: the badge set is frozen at four (§5.1 #9)
/// so the refusal comes to rest `stopped` like an operator's own `/stop`, and
/// the roster is the operator's one *passive* sighting of it — a list where the
/// two read identically is a list that cannot be scanned. The hue is the
/// sighting and never the explanation (§11 glyph doctrine): the word is §6's
/// `refused` signal and the provider row is the steps surface's `auth_row`.
///
/// Nothing here decides; it reads two facts the derivation already carries.
fn tone_of(root: Option<&Agent>) -> Tone {
    match root {
        Some(agent) if agent.in_memory() => Tone::Weak,
        Some(agent) if agent.refused => Tone::Bad,
        _ => Tone::Plain,
    }
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
