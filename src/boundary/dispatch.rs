//! The action chokepoint (§8.5): one exhaustive match from [`Action`] to the
//! §8 executors. Both frontends land here — the GUI's click-glue constructs a
//! variant and calls [`dispatch`]; the deposit consumer decodes one and calls
//! the same function — so every mutating gesture has exactly one
//! implementation, and every attempt leaves its `ops.jsonl` line through the
//! executors' own §4.2 logging (nothing is logged twice here).
//!
//! [`Deps`] — the environment a gesture executes in — lives in its own file
//! beside this one (§12's budget); everything here is the match and the two
//! `pub` typed doors the frame's start glue enters through.

use crate::actions::verbs;
use crate::model_pick::Pick;
use crate::ui_state::UiState;

use super::reply::Reply;
use super::{Action, answer, config, control, fleet, monitor};

/// The §3.6 unmaking's two executors — the workspace and the one conversation
/// — split off at §12's budget when the §4.11 capability arm landed (bl-765d).
/// A real seam: everything else here routes, and these two *gate* — each
/// re-derives its confirmation at fire time and refuses fail-closed, whichever
/// frontend fired.
mod delete_exec;
mod deps;
/// The §8.1 start family's **two typed doors** — the other way into this
/// chokepoint. Their own file at §12's cap, on the seam the module doc already
/// draws: everything else here is the `Action` table, and these two are the
/// entrances the frame's start glue walks in through, each gated in its own
/// right (the §4.11 confinement refusal and the §3.5 spend ceiling ride
/// `prompt`). The `Prepare`/`Prompt` arms delegate here, so a line, a deposit
/// and a click all spend one body.
mod doors;
use delete_exec::{delete_agent, unmake};
pub use deps::Deps;
pub use doors::{prepare, prompt};

/// Dispatch one action (§8.5). The `Err` is a refusal or executor failure —
/// already a durable ops row wherever an executor ran; `ui` is the durable
/// `ui.json` the §3.6 unmaking prunes (write-through, either frontend's copy).
pub fn dispatch(deps: &Deps, ui: &mut UiState, ts: &str, action: &Action) -> Result<Reply, String> {
    let (bl, root) = (&deps.bl, deps.state_root.as_path());
    match action {
        // The §8.2 lernie arms spawn through [`Deps::bound`] and never through
        // `deps.lernie` itself (bl-bf79): a workspace verb's spawn owes its
        // workspace the wall and the name, stated at that one binding rather
        // than once per arm here — `Retarget` is the §9.4 exit (bl-2d19).
        Action::Message {
            workspace: ws,
            agent,
            content,
        } => outcome(verbs::message(&deps.bound(ws), root, ts, agent, content)),
        Action::Stop {
            workspace: ws,
            agent,
            children,
        } => outcome(verbs::stop(&deps.bound(ws), root, ts, agent, *children)),
        Action::Scan { workspace: ws } => outcome(verbs::scan(&deps.bound(ws), root, ts)),
        // The §8.2 nudge (bl-9bef): a detached `lernie advance`, which is the
        // §8.6 release's own launch — one body in [`control`], because "start a
        // driver on this conversation" is one act however it was asked for.
        // Detached and never piped: an advance runs the conversation until it
        // goes quiet, and no gesture may block a frame on that.
        Action::Nudge {
            workspace: ws,
            agent,
        } => control::advance(deps, ts, ws, agent).map(|()| Reply::Nudged),
        Action::Retarget { workspace, agent } => outcome(retarget(deps, ts, workspace, agent)),
        Action::Fork {
            workspace,
            parent,
            attempt,
            goal,
        } => fork(deps, ts, workspace, parent, attempt, goal),
        Action::Close { project, id, name } => {
            outcome(verbs::close(bl, root, ts, project, id, name))
        }
        Action::Assign { project, id, name } => {
            outcome(verbs::assign(bl, root, ts, project, id, name))
        }
        Action::Release { project, id, name } => {
            outcome(verbs::unclaim(bl, root, ts, project, id, name))
        }
        Action::Move {
            project,
            id,
            from,
            to,
        } => outcome(verbs::reassign(bl, root, ts, project, id, from, to)),
        Action::Create {
            project,
            title,
            name,
            body,
        } => outcome(verbs::create(
            bl,
            root,
            ts,
            project,
            title,
            name,
            body.as_deref(),
        )),
        Action::Update {
            project,
            id,
            name,
            title,
            body,
            note,
        } => {
            let fields = verbs::Update::of(title, body, note);
            outcome(verbs::update(bl, root, ts, project, id, name, &fields))
        }
        Action::Prepare { workspace, payload } => {
            prepare(deps, ts, workspace, payload).map(Reply::Prepared)
        }
        Action::Prompt { prepared, goal } => {
            prompt(deps, ui, ts, prepared, goal).map(|conversation| Reply::Started { conversation })
        }
        Action::DeleteWorkspace { workspace, typed } => unmake(deps, ui, ts, workspace, typed),
        Action::DeleteAgent {
            workspace,
            agent,
            typed,
        } => delete_agent(deps, ui, ts, workspace, agent, typed),
        Action::Monitor(verb) => monitor::dispatch(deps, ts, verb),
        // The §4.3 armed loop's family (bl-66fb): arming, which writes one
        // `cadence.yaml` entry. The loop itself is a thread, already running
        // and already finding nothing to do.
        Action::Fleet(verb) => fleet::dispatch(deps, ts, verb),
        // The §8.6 capability family's one writer: the once-answer row, then
        // the releasing `advance`.
        Action::AnswerHold {
            workspace,
            agent,
            ruling,
        } => control::answer_hold(deps, ts, workspace, agent, *ruling),
        // The same family's other writer (VISION §4.9's fifth rung): standing
        // policy for a whole descent, one row, nothing launched.
        Action::Floor {
            workspace,
            agent,
            raised,
        } => control::set_floor(deps, ts, workspace, agent, *raised),
        // The trail's own two operator verbs (§4.2, bl-c417): the same one
        // bodies the frame's ops pane calls ([`crate::opslog::ack`]/[`clear`]).
        Action::Ack => wrote(crate::opslog::ack(root, ts), Reply::Acked),
        Action::MarkSeen { workspace, agent } => acknowledge(deps, ui, ts, workspace, agent),
        Action::ClearTrail => wrote(crate::opslog::clear(root, ts), Reply::TrailCleared),
        // The §9 config family (bl-3f46) — one executor module, because each of
        // the three is a composition of pipelines that already exist.
        Action::ApplyConfig { file, text } => config::apply(deps, ts, file, text),
        Action::SetMarks { workspace, branch } => config::set_marks(deps, ts, workspace, branch),
        Action::PickModel {
            workspace,
            role,
            provider,
            model,
        } => config::pick_model(deps, ts, workspace, &Pick::of(role, provider, model)),
    }
}

/// A **short verb's** answer: its captured run as a reply, or its launch
/// failure as a refusal. A free function beside its twin [`wrote`] rather than
/// a closure inside the table — the two fold the same shape and there is no
/// reason for one of them to be a body and the other a local.
fn outcome(ran: std::io::Result<verbs::Outcome>) -> Result<Reply, String> {
    ran.map(Reply::Outcome).map_err(|e| e.to_string())
}

/// A write-only executor's answer: the reply it earns, or its failure as a
/// refusal. Said once so the trail's two operator verbs stay *rows* in the
/// table above rather than two little bodies inside it.
fn wrote(written: std::io::Result<()>, reply: Reply) -> Result<Reply, String> {
    written.map(|()| reply).map_err(|e| e.to_string())
}

/// The §9.4 exit from the config freeze (bl-2d19): mark this conversation to be
/// re-forked onto the config lineage's head, which its own executor lands at
/// the next step boundary. A body beside [`fork`]'s rather than an arm inside
/// the table, because the table is at §12's per-function budget — the routing
/// is one call, and it belongs to the bound family the arms above it do.
fn retarget(
    deps: &Deps,
    ts: &str,
    workspace: &std::path::Path,
    agent: &str,
) -> std::io::Result<verbs::Outcome> {
    verbs::retarget(&deps.bound(workspace), &deps.state_root, ts, agent)
}

/// One **attempt** (VISION §5 V2): the §4.11 item-8 confinement refusal, then
/// the fork. A body rather than an arm because a birth is gated, and the
/// chokepoint's match is a table — the second door every drone yog births
/// passes through, beside [`prompt`]'s.
fn fork(
    deps: &Deps,
    ts: &str,
    workspace: &std::path::Path,
    parent: &str,
    attempt: &crate::fork::Attempt,
    goal: &str,
) -> Result<Reply, String> {
    control::confinement_gate(workspace)?;
    verbs::fork(
        &deps.bound(workspace),
        &deps.state_root,
        ts,
        &crate::fork::Fire::at(workspace, parent, attempt, goal, &deps.yog_data_root),
    )
    .map(Reply::Outcome)
    .map_err(|e| e.to_string())
}

/// The §6 decision queue's answer (VISION §5 V5.2): write the watermarks the
/// window writes by focusing, then hand back **the queue that remains** —
/// re-derived against the `ui.json` this very call just moved, so one gesture
/// per decision is the whole teleoperator loop. `ts` is the wall clock every
/// boundary caller already mints (§4.2 unix seconds); a clock that states no
/// wall time simply ages every row from zero.
fn acknowledge(
    deps: &Deps,
    ui: &mut UiState,
    ts: &str,
    workspace: &std::path::Path,
    agent: &str,
) -> Result<Reply, String> {
    answer::queue::mark_seen(&deps.snapshot, ui, workspace, agent)?;
    let rows = answer::queue::queue(&deps.snapshot, ui, ts.parse().unwrap_or(0));
    Ok(Reply::Attention(rows))
}
