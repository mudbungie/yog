//! The query chokepoint (§8.5): every boundary read, over the published
//! [`Snapshot`] + the durable `ui.json` ([`UiState`]) + the caller's clock —
//! **the snapshot derivation run without a frame** (VISION §4.8). The frame's
//! view-models ([`AppModel`](crate::AppModel)) delegate to these same
//! functions, which is the parity discipline: one implementation, two
//! serializations — the GUI renders the returned rows, the headless transport
//! encodes them ([`super::reply`]).
//!
//! **Most queries are pure over the snapshot; the §9 config family's three
//! (bl-0164) are not** — a destination's bytes, the §16.3 knob and brazen's
//! provider table are read from the world at the moment they are asked,
//! exactly as their write already is (§8.5's "asked, never stored"), so
//! [`answer`] takes the same [`Deps`] [`dispatch`](super::dispatch::dispatch)
//! does rather than the bare snapshot alone — `Deps` already carries one
//! ([`Deps::snapshot`]). That is also why this can refuse: a config read can
//! fail exactly as its write can (an unreadable file, an unprimed project).
//!
//! Also home to the §3.6 confirmation derivation both dispatch and the dialog
//! read — the delete gate is one derivation wherever it is asked.

use crate::app::Snapshot;
use crate::attention;
use crate::delete::{self, Claim, Confirmation};
use crate::git_tree::AgentState;
use crate::nav::{self, convs::ConvBall, convs::ConvRow, ws_key};
use crate::projects::join::{self, JoinState};
use crate::ui_state::UiState;
use std::collections::HashSet;
use std::path::Path;

use super::dispatch::Deps;
use super::reply::{Reply, WsRow};
use super::{Query, config, help};

/// The §6 decision queue: the roster both frontends walk, the queue it filters
/// to, and the acknowledgement that answers one row (VISION §5 V5.2).
pub mod queue;

/// Answer one query (§8.5). Total over [`Query`]; `now_unix` is the caller's
/// wall clock (minted at the process boundary, so the derivation stays
/// clock-free and deterministic under test). `Err` is a refusal — the same
/// class [`dispatch`](super::dispatch::dispatch) returns, and for the same
/// reason: the three config reads can fail exactly as their writes can.
pub fn answer(query: &Query, deps: &Deps, ui: &UiState, now_unix: i64) -> Result<Reply, String> {
    let snap = &deps.snapshot;
    Ok(match query {
        Query::Workspaces => Reply::Workspaces(ws_rows(snap, ui)),
        Query::Conversations { workspace } => {
            Reply::Conversations(conversations(snap, ui, workspace, now_unix))
        }
        Query::Balls => Reply::Balls(snap.join_rows.clone()),
        // The V4 board — the same snapshot, one altitude up. The window's
        // `AppModel::board` is this same call (§8.5's parity discipline).
        Query::Board => Reply::Board(crate::board::build(snap, ui, now_unix)),
        // The §6 attention strip made addressable (VISION §5 V5.2) — the same
        // predicate the window counts, listed.
        Query::Attention => Reply::Attention(queue::queue(snap, ui, now_unix)),
        // The one query with no world to read (§8.5): its subject is the
        // interface. The consumer answers it exactly as any seat does — same
        // function, so the deposited spelling and the typed one cannot differ.
        Query::Help { verb } => Reply::Help(help::rows(verb.as_deref())),
        // The one query whose subject is the world's bytes rather than this
        // snapshot's derivations (§8.5): the snapshot says where everything is
        // and [`search::run`] re-reads it. Answered straight through here
        // because every seat that reaches this function is already off-frame —
        // the deposit consumer's thread, or a `yog gesture` process with
        // nothing else to do, so nothing supersedes it and nothing waits.
        // The window's seat is [`AppModel::answer`](crate::AppModel::answer),
        // which asks its searcher instead and renders the landed answer.
        Query::Search { text } => Reply::Search(crate::search::run(snap, text, &|| true)),
        // The other world-bytes query (§5.1 #32): the snapshot says which
        // balls this workspace claims, the project repos say what changed.
        // Answered straight through for the same reason search is — every
        // seat reaching here is already off-frame.
        Query::WorkDiff { workspace, file } => {
            let attempts = crate::workdiff::read(snap, workspace);
            let patch = file
                .as_ref()
                .and_then(|f| crate::workdiff::patch(&attempts, f));
            Reply::WorkDiff { attempts, patch }
        }
        Query::Ops { max } => {
            let skip = snap.ops.len().saturating_sub(*max);
            Reply::Ops(snap.ops.iter().skip(skip).cloned().collect())
        }
        // The §9 config family's reads (§8.5, bl-0164): asked of the world at
        // the moment they are asked, exactly as the writes beside them are.
        Query::ReadConfig { file } => return config::read(deps, file),
        Query::Marks { workspace } => return Ok(config::read_marks(deps, workspace)),
        Query::Providers { workspace } => config::providers(deps, workspace),
        // The §9.3 browse and the §9.4 roster (bl-dff8), on the same terms as
        // the three above: asked of the world — this workspace's git, this
        // wall's brazen — at the moment they are asked, and answered straight
        // through because every seat here is already off-frame.
        Query::Lineages { workspace } => return config::lineages(workspace),
        Query::Models {
            workspace,
            provider,
        } => return config::models(deps, workspace, provider),
    })
}

/// The §11 conversation list of one workspace, **root rows only** — the
/// all-collapsed case, which is the answer a machine reader gets (§8.5: a
/// viewport's folds are not a boundary fact) and the shape this list had before
/// it unfolded. Aimed by parameter instead of focus; a workspace with no
/// derived tree is simply empty (§3.3's general path).
pub fn conversations(snap: &Snapshot, ui: &UiState, ws: &Path, now_unix: i64) -> Vec<ConvRow> {
    visible_conversations(snap, ui, ws, now_unix, &HashSet::new())
}

/// The same list as the frame paints it (§11, bl-fa82): every **visible** row
/// of the workspace's descent forest given the viewport's `expanded` set. One
/// derivation for both — [`conversations`] is this call with nothing expanded.
pub fn visible_conversations(
    snap: &Snapshot,
    ui: &UiState,
    ws: &Path,
    now_unix: i64,
    expanded: &HashSet<String>,
) -> Vec<ConvRow> {
    let Some(tree) = snap.trees.get(ws) else {
        return Vec::new();
    };
    let key = ws_key(ws);
    let seen = |k, w: &str, a: &str, o: &str| ui.is_seen(k, w, a, o);
    let ball = |id: &str| conv_ball(snap, id);
    // The standing verdicts, read off the same published ops tail the §11 pane
    // renders (VISION §4.9): a derivation per build, not a field on the world.
    let checks = crate::monitor::row::of_rows(&snap.ops);
    nav::convs::visible_rows(
        &tree.agents,
        &key,
        &seen,
        now_unix,
        &ball,
        &checks,
        expanded,
    )
}

/// Resolve a conversation's goal-stamp ball `id` to its render facts (§3.3,
/// §3.5): the id always renders; the join supplies status/title/badge when a
/// row matches, else those stay `None` — a pure read over the cached join.
pub fn conv_ball(snap: &Snapshot, id: &str) -> ConvBall {
    match snap.join_rows.iter().find(|r| r.ball_id == id) {
        Some(r) => ConvBall {
            id: id.to_owned(),
            state: Some(r.state),
            title: r.title.clone(),
            badge: join::badge(r.state, r.claimant.as_deref()),
        },
        None => ConvBall {
            id: id.to_owned(),
            state: None,
            title: None,
            badge: None,
        },
    }
}

/// One workspace's §6 rollup: attention-bearing agents, agent count, whether
/// anything runs — the tab bar's numbers, by parameter.
pub fn workspace_stats(snap: &Snapshot, ui: &UiState, ws: &Path) -> (usize, usize, bool) {
    let Some(tree) = snap.trees.get(ws) else {
        return (0, 0, false);
    };
    let key = ws_key(ws);
    let seen = |k, w: &str, a: &str, o: &str| ui.is_seen(k, w, a, o);
    let mut count = 0;
    let mut running = false;
    for a in &tree.agents {
        if attention::attention(a, &key, &seen).any() {
            count += 1;
        }
        running |= matches!(a.state, AgentState::Live | AgentState::InFlight);
    }
    (count, tree.agents.len(), running)
}

/// Every enumerated workspace with its rollup — the `workspaces` answer.
pub fn ws_rows(snap: &Snapshot, ui: &UiState) -> Vec<WsRow> {
    snap.workspaces
        .iter()
        .map(|w| {
            let (attention, agents, running) = workspace_stats(snap, ui, &w.path);
            WsRow {
                workspace: w.clone(),
                attention,
                agents,
                running,
            }
        })
        .collect()
}

/// The conversation mint's occupied set for a workspace (§3.3): the names its
/// living agents wear — each agent's `name_fact`, the lernie-stored blob with
/// the legacy goal-stamp fallback while pre-0.0.4 roots live. Children count
/// too, and must: lernie refuses a name any living agent already wears, so a
/// mint that ignored a named child would fail at fire. Empty for an underived
/// workspace — the general path with no inputs.
pub fn names_in(snap: &Snapshot, ws: &Path) -> Vec<String> {
    snap.trees
        .get(ws)
        .into_iter()
        .flat_map(|t| {
            t.agents
                .iter()
                .filter_map(crate::git_tree::Agent::name_fact)
        })
        .collect()
}

/// The §3.6 confirmation for `ws` — what dies, what is released, what is live.
/// `None` for anything not one of yog's own named workspaces (§3.6 scope). One
/// derivation for the dialog and the dispatch gate alike; re-derived at fire
/// time, fail-closed.
pub fn confirmation_of(snap: &Snapshot, ws: &Path) -> Option<Confirmation> {
    let name = named_leaf(snap, ws)?;
    let agents = snap.trees.get(ws).map_or(&[][..], |t| t.agents.as_slice());
    Some(delete::confirmation(
        &name,
        ws,
        &nav::convs::liveness(agents),
        bound_claims(snap, ws),
    ))
}

/// The §3.6 agent-delete confirmation for one conversation (bl-f17a): its
/// display name and its live members. `None` outside yog's own named
/// workspaces — the same scope as the workspace verb (§3.6: foreign
/// workspaces are another driver's territory, replays read-only), and how
/// every carrier decides whether to offer the verb. One derivation for the
/// dialog and the dispatch gate alike; re-derived at fire time, fail-closed.
pub fn agent_confirmation_of(
    snap: &Snapshot,
    ws: &Path,
    root: &str,
) -> Option<delete::agent::AgentConfirmation> {
    named_leaf(snap, ws)?;
    let agents = snap.trees.get(ws).map_or(&[][..], |t| t.agents.as_slice());
    Some(delete::agent::confirmation(root, agents))
}

/// `ws`'s minted name iff it is one of yog's own — [`crate::binding::named_of`]'s question,
/// asked of this snapshot's workspace set.
fn named_leaf(snap: &Snapshot, ws: &Path) -> Option<String> {
    crate::binding::named_of(&snap.workspaces, ws)
}

/// The live bound balls the unmaking releases (§3.6 step 1): the join's
/// [`Bound`](JoinState::Bound) rows for this workspace.
fn bound_claims(snap: &Snapshot, ws: &Path) -> Vec<Claim> {
    snap.join_rows
        .iter()
        .filter(|r| r.workspace.as_deref() == Some(ws) && r.state == JoinState::Bound)
        .map(|r| Claim {
            project: r.project.clone(),
            id: r.ball_id.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests;
