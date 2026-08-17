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
use crate::nav::{self, convs::ConvBall, convs::ConvRow, ws_key};
use crate::projects::join;
use crate::ui_state::UiState;

use std::path::Path;

use super::dispatch::Deps;
use super::reply::Reply;
use super::{Query, config, help};

/// One conversation as a seat sees it (REMOTE §9.4, bl-1eb0) — the
/// [`Agent`](crate::git_tree::Agent)'s wire projection.
pub mod agent;
/// One workspace's bound balls with their §3.5 figures (REMOTE §9.7, bl-b4b5).
pub mod balls;
/// The §11 altitude-0 chrome: the enumeration, its §6 rollups, and how current
/// the derivation behind them is.
mod chrome;
/// The §3.6 unmaking's own derivations — what a delete would destroy, read by
/// the dialog and by the dispatch gate alike.
mod confirm;
/// The §11 inspector family (bl-6233): one conversation's transcript, steps,
/// files, spine and mail — the derivations the frame's view-models delegate to.
pub mod inspector;
/// The §6 decision queue: the roster both frontends walk, the queue it filters
/// to, and the acknowledgement that answers one row (VISION §5 V5.2).
pub mod queue;

pub use chrome::{workspace_stats, workspaces, ws_rows};
pub use confirm::{agent_confirmation_of, confirmation_of};

/// Answer one query (§8.5). Total over [`Query`]; `now_unix` is the caller's
/// wall clock (minted at the process boundary, so the derivation stays
/// clock-free and deterministic under test). `Err` is a refusal — the same
/// class [`dispatch`](super::dispatch::dispatch) returns, and for the same
/// reason: the three config reads can fail exactly as their writes can.
pub fn answer(query: &Query, deps: &Deps, ui: &UiState, now_unix: i64) -> Result<Reply, String> {
    let snap = &deps.snapshot;
    // **The one resolution** (REMOTE §8, bl-f5f6). Every read that names a
    // workspace names it by *name*; this turns that name into the path the
    // derivations below read, once, ahead of the table — never once per arm.
    // A query that names no workspace resolves to nothing and no arm reads it:
    // the general path with no input, not a case of its own.
    let ws: &std::path::Path = &match query.workspace() {
        Some(name) => snap.ws_path(&name)?,
        None => std::path::PathBuf::new(),
    };
    // **And the conversation's** (bl-49bc), on the same terms one noun down:
    // the §11 inspector family and the seat's own read are aimed at an agent
    // **id**, and a `Started` receipt hands back a *name* — so the resolution
    // stands here too, once, ahead of the table. A read naming no conversation
    // resolves to nothing and no arm reads it.
    let agent: &str = &super::address::resolve_agent(snap, ws, query.agent())?;
    Ok(match query {
        Query::Workspaces => Reply::Workspaces(workspaces(snap, ui, now_unix)),
        Query::Conversations { .. } => Reply::Conversations(conversations(snap, ui, ws, now_unix)),
        Query::Balls => Reply::Balls(snap.join_rows.clone()),
        // The same binding facts one workspace deep, with each ball's figure
        // (REMOTE §9.7, bl-b4b5) — the §11 balls section's whole content.
        Query::WorkspaceBalls { .. } => Reply::WorkspaceBalls(balls::ws_balls(snap, ui, ws)),
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
        Query::WorkDiff { file, .. } => {
            // The claim rows come off the snapshot; the fan's candidate rows
            // (bl-c2bd) come off the same two yog-owned facts the §8.6
            // writable root reads — the trail's claim row and its fire rows —
            // so the trail is read here, where the question is asked.
            let entries = crate::opslog::tail(&deps.state_root, usize::MAX);
            let xdg = deps.world.balls_layout();
            let attempts = crate::workdiff::read(snap, ws, &entries, &xdg);
            let patch = file
                .as_ref()
                .and_then(|f| crate::workdiff::patch(snap, &attempts, f));
            Reply::WorkDiff { attempts, patch }
        }
        // The projection over those same attempts (§3.9, bl-40ab): the diff
        // rows joined with the conversation each attempt was bound to. It reads
        // the trail for the same reason the work diff does — the binding
        // pointer is a fire row — and the step-record columns off the
        // snapshot's own pre-walked bills, so the join costs no second pass.
        Query::Science { .. } => {
            let entries = crate::opslog::tail(&deps.state_root, usize::MAX);
            let xdg = deps.world.balls_layout();
            Reply::Science(crate::science::project(
                snap,
                ws,
                &entries,
                &xdg,
                &deps.balls_state_root,
            ))
        }
        // The §11 inspector family (bl-6233, REMOTE §9 step 1): the
        // conversation's own reads, which had no headless spelling at all —
        // so no seat but the window could read a chat. World-bytes queries
        // like the two above, answered straight through for the same reason,
        // over the derivations in [`inspector`] the frame delegates to.
        Query::Transcript { .. } => Reply::Transcript(inspector::transcript(snap, ws, agent)),
        Query::Steps { .. } => Reply::Steps(inspector::steps(snap, ws, agent)),
        Query::Step { seq, .. } => Reply::Step(crate::steps_view::detail(ws, agent, seq)),
        Query::Files { path, at, .. } => {
            let (view, preview) = inspector::files(ws, agent, path.as_deref(), at.as_deref());
            Reply::Files { view, preview }
        }
        Query::Rail { .. } => {
            let steps = inspector::steps(snap, ws, agent);
            let tx = inspector::transcript(snap, ws, agent);
            Reply::Rail(inspector::rail(snap, ws, agent, &steps, &tx))
        }
        Query::Inbox { .. } => Reply::Inbox(crate::inboxview::list_inbox(ws, agent)),
        // Config-frozen-at (VISION V1.2, bl-13f9): the §5.1 #17 derivation
        // asked at whichever commit the seat named, the agent's tip when it
        // named none. It **refuses** where its siblings answer absent, because
        // its walk is the workspace's own git and a conversation with no
        // policy at all is not a reading (the `Lineages` shape).
        Query::Governing { at, .. } => {
            return inspector::governing(snap, ws, agent, at.as_deref()).map(Reply::Governing);
        }
        // The seat's own read of its selection (REMOTE §9.4, bl-1eb0) — pure
        // over the snapshot, unlike the five above, because everything it says
        // was already derived when the tree was.
        Query::Agent { .. } => Reply::Agent(agent::agent(snap, ui, ws, agent, now_unix)),
        Query::Ops { max } => {
            let skip = snap.ops.len().saturating_sub(*max);
            Reply::Ops(snap.ops.iter().skip(skip).cloned().collect())
        }
        // The §9 config family's reads (§8.5, bl-0164): asked of the world at
        // the moment they are asked, exactly as the writes beside them are.
        Query::ReadConfig { file } => return config::read(deps, file),
        Query::Marks { .. } => return Ok(config::read_marks(deps, ws)),
        Query::Providers { .. } => config::providers(deps, ws),
        // The §9.3 browse and the §9.4 roster (bl-dff8), on the same terms as
        // the three above: asked of the world — this workspace's git, this
        // wall's brazen — at the moment they are asked, and answered straight
        // through because every seat here is already off-frame.
        Query::Lineages { .. } => return config::lineages(ws),
        Query::Models { provider, .. } => return config::models(deps, ws, provider),
        // REMOTE §5's roster (bl-4e08): the §4.1 registration listing, the
        // wire's presence RAM and each client's advertised set, joined at the
        // moment they are asked. It reads the *name* rather than the resolved
        // path, because a registration is keyed by name — and the resolution
        // above still stands, so an unregistered workspace refuses in the
        // resolver's own words before this runs.
        Query::Clients { workspace } => Reply::Clients(crate::registry::roster::roster(
            &deps.state_root,
            &deps.caller.presence,
            workspace,
        )),
        // REMOTE §3's routing leg (bl-024b): the follow-class read that waits
        // for this client's next work, and the asker's poll for what one
        // captured. Neither names a world, so neither reads the resolution
        // above.
        Query::Invocations => return super::routing::invocations(deps),
        Query::Capture { invocation } => return super::routing::capture(deps, invocation),
    })
}

/// The §11 conversation list of one workspace, at the **forest** altitude
/// (REMOTE §9.7, bl-44e9): every member of the descent forest with its own
/// per-row rollups, in paint order. Aimed by parameter instead of focus; a
/// workspace with no derived tree is simply empty (§3.3's general path).
///
/// **This is the whole answer and it carries no fold.** A viewport's expanded
/// set is a view (§8.5: *views gain no boundary representation*), so it never
/// crosses and never rides a row — each seat selects its own visible rows out of
/// this with [`nav::convs::visible`], and a seat holding no fold at all selects
/// the root subset, which is the all-collapsed list this query used to answer.
pub fn conversations(snap: &Snapshot, ui: &UiState, ws: &Path, now_unix: i64) -> Vec<ConvRow> {
    let Some(tree) = snap.trees.get(ws) else {
        return Vec::new();
    };
    let key = ws_key(ws);
    let seen = |k, w: &str, a: &str, o: &str| ui.is_seen(k, w, a, o);
    let ball = |id: &str| conv_ball(snap, id);
    // The standing verdicts, read off the same published ops tail the §11 pane
    // renders (VISION §4.9): a derivation per build, not a field on the world.
    let checks = crate::monitor::row::of_rows(&snap.ops);
    nav::convs::forest_rows(&tree.agents, &key, &seen, now_unix, &ball, &checks)
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

#[cfg(test)]
mod tests;
