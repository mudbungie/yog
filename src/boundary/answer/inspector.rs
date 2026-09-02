//! The §11 **inspector family's** derivations (bl-6233, REMOTE §9 step 1;
//! extended bl-13f9) — one conversation's transcript, steps, files, spine,
//! mail and the policy it resolves.
//!
//! These surfaces were reachable from no seat but the window: the frame's
//! view-models read disk directly, so the chats themselves had no headless
//! spelling. What was missing was never the reading — every read below is one
//! call into a module that already tests it — but a *shared home* for the two
//! things the frame did on its own: folding the live tail onto the committed
//! transcript, and gathering the spine's inputs off the snapshot. Both live
//! here now, at the boundary's altitude.
//!
//! **And now the window reads them the same way any seat does** (REMOTE §9.7,
//! bl-13f9): the shell declares each as a standing wire question rather than
//! calling in and memoizing, so the last §11 surface that had two paths has
//! one. That is §8.5's parity discipline in its literal form — one
//! implementation, two serializations — with nothing left on the window's side
//! of it but the pin, which is a fold.
//!
//! Everything here is a pure function of the published snapshot plus the
//! workspace's own bytes, answered straight through at the chokepoint because
//! every seat reaching it is already off-frame — the [`Search`] and
//! [`WorkDiff`](crate::boundary::Query::WorkDiff) precedent.
//!
//! [`Search`]: crate::boundary::Query::Search

use std::path::{Path, PathBuf};

use crate::app::Snapshot;
use crate::budgets::{Scope, bills, total};
use crate::config_edit::branch::governing_config;
use crate::files_view::{self, FilesView, Preview};
use crate::git_tree::{Agent, AgentState, Stream, children_of};
use crate::nav::convs::{display_name_of, root_of};
use crate::rail::{self, ChildInput, Rail};
use crate::steps_view::{self, StepsView};
use crate::transcript::{self, Transcript};

/// One agent's whole conversation: the committed `messages/` read, with the
/// **live streaming tail folded on** when a call is in flight.
///
/// **Why the tail is folded rather than dropped** (bl-6233). The tail is not a
/// disk read: it is the rendered snapshot's own [`Stream`], which the §7.2
/// follower keeps fresh and the `Deps` already carry — so folding it costs a
/// clone of the committed entries and nothing else. Answering the committed
/// half alone would have been cheaper still and *wrong*: the window folds it,
/// so a headless seat that did not would describe a different moment than the
/// GUI does of the same instant, which is precisely the divergence §8.5's one
/// implementation exists to prevent. A settled step's trailing text is already
/// committed, so the fold is gated on [`AgentState::InFlight`] — merging it
/// otherwise paints the last answer twice.
pub fn transcript(snap: &Snapshot, ws: &Path, agent: &str) -> Transcript {
    let committed = transcript::build(ws, agent);
    match live_tail(snap, ws, agent) {
        Some(stream) => committed.with_live(&stream),
        None => committed,
    }
}

/// The live streaming tail this agent is producing right now, off the
/// snapshot — `None` unless the agent is [`InFlight`](AgentState::InFlight),
/// and `None` for an agent the snapshot does not carry.
///
/// **[`transcript`] is its only caller** since bl-13f9. The frame used to read
/// it beside a memoized committed half, the two having different clocks (§7.2);
/// the frame now asks for the whole conversation over the wire and this fold
/// happens once, here, at the boundary that already owned the ruling — so it is
/// `pub(crate)` and the two clocks became one, the asker's.
pub(crate) fn live_tail(snap: &Snapshot, ws: &Path, agent: &str) -> Option<Stream> {
    let found = agent_of(snap, ws, agent)?;
    (found.state == AgentState::InFlight).then(|| found.stream.clone())
}

/// The agent's steps list. Its liveness comes off the snapshot rather than a
/// parameter: a driver at work is still filling its newest step, so that step's
/// unanswered shape is a call in flight and not a §7.3 wound — and a seat that
/// could *state* the liveness could contradict the world (§10: never a false
/// definite). An agent the snapshot does not carry reads as
/// [`Stopped`](AgentState::Stopped), which is what an untracked tree's newest
/// step honestly is.
pub fn steps(snap: &Snapshot, ws: &Path, agent: &str) -> StepsView {
    steps_view::build(ws, agent, state_of(snap, ws, agent))
}

/// The agent worktree's listing, and the named path's preview when the listing
/// carries it as a file — **live, or as of the commit `at` names** (VISION V1.2
/// config-frozen-at's sibling; REMOTE §9.7, bl-44e9).
///
/// One derivation with the tree as a parameter, rather than two reads: pinned
/// and live differ only in *which* tree is enumerated, and the containment rule
/// below is the same rule over either. The window's own pinned Files tab used to
/// reach `rail::files_at` in process because no query spelled this; it is
/// answered here now, so the two seats read one implementation.
///
/// **The path is resolved against the listing, never joined blind.** A boundary
/// caller names a path and yog opens only what this same answer just enumerated
/// — the containment `workdiff::patch` already gives the work diff. A path that
/// is not there, or names a directory, answers `None` rather than refusing: the
/// listing beside it already says why.
pub fn files(
    ws: &Path,
    agent: &str,
    path: Option<&str>,
    at: Option<&str>,
) -> (FilesView, Option<Preview>) {
    let view = match at {
        Some(commit) => rail::files_at(ws, commit),
        None => files_view::build(ws, agent),
    };
    let preview = path
        .filter(|path| listed(&view, path))
        .map(|path| match at {
            Some(commit) => rail::preview_at(ws, commit, path),
            None => files_view::preview(&files_view::agent_worktree(ws, agent).join(path)),
        });
    (view, preview)
}

/// **Where this conversation's work actually lands, when that is not the
/// worktree [`files`] just listed** (bl-1015).
///
/// The one channel is litany's `refs/litany/cwd/<agent-id>` mark (DESIGN §3.3:
/// *"the creation-seeded mark … is the one channel — no misleading
/// redundancy"*), which a path or ball rung's fire seeds at creation and the
/// executor reads back at every tool spawn. So this is the same read
/// [`crate::control::root::agent_cwd`] already makes for operand resolution —
/// one home for the fact, asked here for the other consumer.
///
/// `None` where the listing IS the working directory, which is both the unset
/// mark (a bare start) and a mark that names the agent worktree itself (an
/// agent that `cd`ed home). One answer for "the promise holds", so the reply
/// carries the path exactly when there is somewhere else to name.
pub fn working_dir(ws: &Path, agent: &str) -> Option<PathBuf> {
    crate::control::root::agent_cwd(ws, agent)
        .filter(|at| at != &files_view::agent_worktree(ws, agent))
}

/// Whether the listing carries `path` as a file of its own.
fn listed(view: &FilesView, path: &str) -> bool {
    match view {
        FilesView::Present { entries, .. } => entries
            .iter()
            .any(|entry| !entry.is_dir && entry.rel_path == path),
        FilesView::AbsentWorktree => false,
    }
}

/// The config commit this conversation resolves its policy from (§9.3, §5.1
/// #17) — **at the commit `at` names, or at the agent's own branch tip**
/// (VISION V1.2 config-frozen-at; REMOTE §9.7, bl-13f9).
///
/// One derivation with the commit as a parameter, the [`files`] shape: pinned
/// and unpinned differ only in *which* commit the walk starts from, and the
/// window's Config tab used to make that choice in process because no query
/// spelled it. Since bl-e654 that walk ends in the followed lineage's head
/// rather than the fork commit, so both readings move when the lineage does. `None` resolves to the tip off the published snapshot, so a seat
/// asks without knowing one — and an agent the snapshot does not carry has an
/// empty tip, which the derivation refuses in git's own words rather than
/// answering about some other commit.
pub fn governing(
    snap: &Snapshot,
    ws: &Path,
    agent: &str,
    at: Option<&str>,
) -> Result<crate::config_edit::branch::GoverningConfig, String> {
    let tip = at.map_or_else(
        || {
            agent_of(snap, ws, agent)
                .map(|a| a.tip_oid)
                .unwrap_or_default()
        },
        str::to_owned,
    );
    governing_config(ws, &tip).map_err(|e| e.to_string())
}

/// The step spine (VISION V1) for one conversation: its notches, and the child
/// cards hanging off them. `steps` and `transcript` are passed in because the
/// chokepoint already holds them, off the two calls above — re-reading either
/// here would double the disk cost of answering one conversation.
pub fn rail(snap: &Snapshot, ws: &Path, agent: &str, steps: &StepsView, tx: &Transcript) -> Rail {
    let agents = snap
        .trees
        .get(ws)
        .map(|tree| tree.agents.clone())
        .unwrap_or_default();
    let parent_commits = agents
        .iter()
        .find(|a| a.agent_id == agent)
        .map(|a| a.steps.clone())
        .unwrap_or_default();
    let children: Vec<ChildInput> = children_of(&agents, agent)
        .into_iter()
        .filter_map(|index| agents.get(index))
        .map(|child| child_input(ws, &agents, child))
        .collect();
    rail::build(
        &speaker(&agents, agent),
        &parent_commits,
        steps,
        tx,
        &children,
    )
}

/// Who the conversation's model turns are (bl-2335): the §3.3 display ladder
/// over the selection's *conversation root*, exactly as the composer's target
/// line derives it. A selection the snapshot does not carry is its own root and
/// lands on the ladder's last rung, which is the agent id.
pub fn speaker(agents: &[Agent], agent: &str) -> String {
    let root = root_of(agents, agent).unwrap_or_else(|| agent.to_owned());
    display_name_of(agents, &root)
}

/// One child's card inputs. Its spend is the **per-agent** fold of
/// `steps/<id>` (VISION V1.5), so a fork's shared prefix cost stays with the
/// ancestor; its config label is the which-config-governs derivation (§5.1
/// #17), which names the lineage the child FOLLOWS — absent only where a
/// divergence holds it and there is no one lineage to name (bl-e654).
fn child_input(ws: &Path, agents: &[Agent], child: &Agent) -> ChildInput {
    ChildInput {
        agent_id: child.agent_id.clone(),
        name: display_name_of(agents, &child.agent_id),
        state: child.state,
        streaming_text: child.stream.text.clone(),
        commits: child.steps.clone(),
        tokens: total(&bills(ws, &Scope::Agent(child.agent_id.clone()))).total_tokens(),
        config_label: governing_config(ws, &child.tip_oid)
            .ok()
            .and_then(|gov| gov.followed_lineage()),
    }
}

/// The agent's row in the published snapshot, if this workspace is derived and
/// carries it.
fn agent_of(snap: &Snapshot, ws: &Path, agent: &str) -> Option<Agent> {
    snap.trees
        .get(ws)?
        .agents
        .iter()
        .find(|a| a.agent_id == agent)
        .cloned()
}

/// The agent's derived §3.5 liveness, or [`Stopped`](AgentState::Stopped) for
/// one the snapshot does not carry.
fn state_of(snap: &Snapshot, ws: &Path, agent: &str) -> AgentState {
    agent_of(snap, ws, agent).map_or(AgentState::Stopped, |a| a.state)
}

#[cfg(test)]
mod tests;
