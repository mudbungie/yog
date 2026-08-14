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
use crate::start::{self, StartInputs};
use crate::ui_state::UiState;
use lernie::mint::SplitMix64;

use super::reply::Reply;
use super::{Action, answer, config, control, fleet, monitor};

/// The §3.6 unmaking's two executors — the workspace and the one conversation
/// — split off at §12's budget when the §4.11 capability arm landed (bl-765d).
/// A real seam: everything else here routes, and these two *gate* — each
/// re-derives its confirmation at fire time and refuses fail-closed, whichever
/// frontend fired.
mod delete_exec;
mod deps;
use delete_exec::{delete_agent, unmake};
pub use deps::Deps;

/// Dispatch one action (§8.5). The `Err` is a refusal or executor failure —
/// already a durable ops row wherever an executor ran; `ui` is the durable
/// `ui.json` the §3.6 unmaking prunes (write-through, either frontend's copy).
pub fn dispatch(deps: &Deps, ui: &mut UiState, ts: &str, action: &Action) -> Result<Reply, String> {
    let (bl, root) = (&deps.bl, deps.state_root.as_path());
    let outcome = |r: std::io::Result<verbs::Outcome>| -> Result<Reply, String> {
        r.map(Reply::Outcome).map_err(|e| e.to_string())
    };
    match action {
        // The three §8.2 lernie arms spawn through [`Deps::bound`] and never
        // through `deps.lernie` itself (bl-bf79): a workspace verb's spawn owes
        // its workspace the wall and the name, and that is stated at the one
        // binding rather than three times over here.
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

/// A write-only executor's answer: the reply it earns, or its failure as a
/// refusal. Said once so the trail's two operator verbs stay *rows* in the
/// table above rather than two little bodies inside it.
fn wrote(written: std::io::Result<()>, reply: Reply) -> Result<Reply, String> {
    written.map(|()| reply).map_err(|e| e.to_string())
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

/// The §8.1 mutating half: seed → ensure-workspace → the ball rung's `bl`
/// steps, the composer's [`Prepared`](crate::start::Prepared) back. The
/// occupied names and roots re-derive here from [`Deps`] — the same sources
/// every frontend fills them from, one derivation. `pub` as the chokepoint's
/// typed door — the frame's start glue enters here, and the [`dispatch`]
/// Prepare arm delegates here, so both spellings share this one body.
pub fn prepare(
    deps: &Deps,
    ts: &str,
    workspace: &std::path::Path,
    payload: &crate::start::Payload,
) -> Result<crate::start::Prepared, String> {
    let inputs = StartInputs {
        conversation_names: answer::names_in(&deps.snapshot, workspace),
        workspace: workspace.to_path_buf(),
        payload: payload.clone(),
        home: deps.home.clone(),
        yog_data_root: deps.yog_data_root.clone(),
        balls_state_root: deps.balls_state_root.clone(),
    };
    let start_deps = start::Deps {
        bl: deps.bl.clone(),
        lernie: deps.lernie.clone(),
        state_root: deps.state_root.clone(),
        yog_binary: deps.yog_binary.clone(),
    };
    start::prepare(&start_deps, &inputs, ts).map_err(|e| e.to_string())
}

/// The deferred detached fire (§8.1): mint against the occupied set, spawn
/// with `--name` and the goal verbatim (bl-6920) — the minted conversation
/// name back. `pub`
/// as [`prepare`]'s sibling typed door; the [`dispatch`] Prompt arm delegates.
///
/// **The §3.5 spend ceiling gates here and nowhere else** ([`super::ceiling`]):
/// this is the one door every drone yog births passes through, so one gate
/// covers every spawn path, and a birth is the only thing it can refuse — the
/// ruling forbids touching a drone that is already running. `ui` is the durable
/// `ui.json` the ceiling and the price table are read from.
///
/// The §4.11 item-8 **confinement refusal** rides the same door, and before the
/// ceiling: a workspace that requires a confinement layer this platform does
/// not have fires nothing at all, so there is no spend to judge.
pub fn prompt(
    deps: &Deps,
    ui: &UiState,
    ts: &str,
    prepared: &crate::start::Prepared,
    goal: &str,
) -> Result<String, String> {
    control::confinement_gate(&prepared.workspace)?;
    super::ceiling::gate(ui, &deps.state_root, ts, prepared)?;
    // The fired loop carries the target workspace's wall (§16.2 as amended):
    // lernie hands its own environment to every tool subprocess, and a bare
    // `bz` in an agent's bash is the world's shim re-entering yog — so this one
    // layer is what puts the whole descendant tree inside the sphere's
    // providers, sign-ins and model cache.
    // …and, for a launch that was NOT raised onto a project, its own balls
    // space (§16.3's launch clause): the ball rung is by construction pointed
    // at a project's board — it was offered on that project's balls section and
    // its `bl claim` already landed there — so it carries no `YOG_MARKS` and
    // its `bl` is the board's own, instantly consistent with what yog renders.
    // Every other rung tracks on a space of its own, which is the ruling's
    // default. Nothing new decides this: `Payload::origin` is the rung, already
    // carried on `Prepared` for the §7.3 banner.
    let own_space = prepared.origin != crate::opslog::Origin::Balls;
    let lernie = deps
        .lernie
        .and_env(crate::world::wall::pairs(&deps.world, &prepared.workspace))
        .and_env(crate::world::marks::pairs(
            &deps.world,
            &prepared.workspace,
            own_space,
        ));
    start::execute_prompt(
        &lernie,
        &deps.state_root,
        ts,
        prepared,
        goal,
        &answer::names_in(&deps.snapshot, &prepared.workspace),
        &SplitMix64::from_seed(deps.mint_seed),
    )
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
