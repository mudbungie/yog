//! The action chokepoint (§8.5): one exhaustive match from [`Action`] to the
//! §8 executors. Every frontend lands here — the deposit consumer decodes a
//! gesture and calls [`dispatch`]; the wire's listener decodes one and calls the
//! same function; the window's click-glue constructs a variant and, since
//! bl-4841, mostly **posts** it over that wire (REMOTE §9.8) rather than calling
//! in process. So every mutating gesture has exactly one implementation however
//! it was asked for, and every attempt leaves its `ops.jsonl` line through the
//! executors' own §4.2 logging (nothing is logged twice here).
//!
//! [`Deps`] — the environment a gesture executes in — lives in its own file
//! beside this one (§12's budget); everything here is the match and the two
//! `pub` typed doors the frame's start glue enters through.

use crate::actions::verbs;
use crate::model_pick::Pick;
use crate::ui_state::UiState;

use super::reply::Reply;
use super::{Action, config, control, fan, fleet, interrupt, monitor, routing};

/// The §3.6 unmaking's two executors — the workspace and the one conversation
/// — split off at §12's budget when the §4.11 capability arm landed (bl-765d).
/// A real seam: everything else here routes, and these two *gate* — each
/// re-derives its confirmation at fire time and refuses fail-closed, whichever
/// frontend fired.
mod advertise;
/// The bodies the table's arms call — split off at §12's budget (bl-c088) on
/// the seam this module's own doc draws: everything here is the match, and
/// each of those is a call that had to be a body rather than a row.
mod arms;
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
/// REMOTE §1.4's enrollment (bl-f4e3) — the third gating arm, beside
/// [`advertise`] and [`delete_exec`]: it mints a certificate and seats a
/// registration, so it re-derives every precondition at fire time and refuses
/// fail-closed.
mod enroll;
/// The one address resolution, and the §4.1 raise it carries — split off at
/// §12's cap (bl-4e08); it stands ahead of the table rather than inside it.
mod resolve;
use arms::{acknowledge, fork, outcome, retarget, spend, staged, wrote};
use delete_exec::{delete_agent, unmake};
pub use deps::{Caller, Deps};
pub use doors::{prepare, prompt};
use resolve::resolve_workspace;

/// Dispatch one action (§8.5). The `Err` is a refusal or executor failure —
/// already a durable ops row wherever an executor ran; `ui` is the durable
/// `ui.json` the §3.6 unmaking prunes (write-through, either frontend's copy).
pub fn dispatch(deps: &Deps, ui: &mut UiState, ts: &str, action: &Action) -> Result<Reply, String> {
    let (bl, root) = (&deps.bl, deps.state_root.as_path());
    // **The one resolution** (REMOTE §8, bl-f5f6): the wire spells names, the
    // world is addressed by path. The two `Action` tables ([`super::address`])
    // say which name this gesture carries, and it is turned into a path here —
    // once, ahead of the table — so no arm below re-derives an address and an
    // unresolvable name refuses naming the token before anything runs. A
    // gesture that names neither resolves to nothing and no arm reads it: the
    // general path with no input, not a case of its own.
    let ws: &std::path::Path = &match action.workspace() {
        Some(name) => resolve_workspace(deps, action, &name)?,
        None => std::path::PathBuf::new(),
    };
    let project: &std::path::Path = &match action.project() {
        Some(name) => deps.snapshot.project_path(&name)?,
        None => std::path::PathBuf::new(),
    };
    // …and the third noun, on the same terms (bl-49bc): the conversation table
    // ([`super::address`]) says which agent this gesture names and the resolver
    // turns it into the **id** every executor keys on — an id untouched, a
    // stored name resolved, anything else refused before an arm runs. Its own
    // doc holds the ruling; a gesture naming no conversation resolves to
    // nothing and no arm reads it.
    let agent: &str = &super::address::resolve_agent(&deps.snapshot, ws, action.agent())?;
    match action {
        // The §8.2 litany arms spawn through [`Deps::bound`] and never through
        // `deps.litany` itself (bl-bf79): a workspace verb's spawn owes its
        // workspace the wall and the name, stated at that one binding rather
        // than once per arm here — `Retarget` is the §9.4 exit (bl-2d19).
        Action::Message { content, .. } => {
            outcome(verbs::message(&deps.bound(ws), root, ts, agent, content))
        }
        Action::Stop { children, .. } => {
            outcome(verbs::stop(&deps.bound(ws), root, ts, agent, *children))
        }
        // Send-and-interrupt (bl-a33d): the one arm that composes two acts, so
        // it has a body of its own ([`interrupt`]) and leaves the two rows those
        // acts each leave. The deposit's driver-start is the trigger — litany's
        // standing law (ARCH §2.9), not a verb yog adds.
        Action::Interrupt { content, .. } => interrupt::interrupt(deps, ts, ws, agent, content),
        Action::Scan { .. } => outcome(verbs::scan(&deps.bound(ws), root, ts)),
        // The §8.2 nudge (bl-9bef): a detached `litany advance`, which is the
        // §8.6 release's own launch — one body in [`control`], because "start a
        // driver on this conversation" is one act however it was asked for.
        // Detached and never piped: an advance runs the conversation until it
        // goes quiet, and no gesture may block a frame on that.
        Action::Nudge { .. } => control::advance(deps, ts, ws, agent).map(|()| Reply::Nudged),
        Action::Retarget { .. } => outcome(retarget(deps, ts, ws, agent)),
        Action::Fork { attempt, goal, .. } => fork(deps, ts, ws, agent, attempt, goal),
        // The §8.2 `bl` family (bl-92d3): one row here, five members one level
        // down, exactly as the monitor's and the fan's route.
        Action::Ball(verb) => match verb {
            verbs::Verb::Close { id, name, .. } => spend(verbs::close, deps, ts, project, id, name),
            verbs::Verb::Assign { id, name, .. } => {
                spend(verbs::assign, deps, ts, project, id, name)
            }
            verbs::Verb::Release { id, name, .. } => {
                spend(verbs::unclaim, deps, ts, project, id, name)
            }
            verbs::Verb::Create { name, fields, .. } => {
                outcome(verbs::create(bl, root, ts, project, name, fields))
            }
            verbs::Verb::Update {
                id, name, fields, ..
            } => outcome(verbs::update(bl, root, ts, project, id, name, fields)),
        },
        Action::Prepare { payload, .. } => staged(deps, ts, ws, project, payload),
        Action::Prompt {
            prepared,
            goal,
            seed,
        } => prompt(deps, ui, ts, ws, prepared, goal, *seed)
            .map(|conversation| Reply::Started { conversation }),
        // The §3.8 mutating fan's family (bl-8746; V3's delivery, bl-c2bd),
        // one variant since bl-a33d: spread N candidates off one pinned
        // target, retire one, or deliver one — routed as the monitor's and
        // the fleet's families are.
        Action::Fan(verb) => fan::dispatch(deps, ts, verb),
        Action::DeleteWorkspace { typed, .. } => unmake(deps, ui, ts, ws, typed),
        Action::DeleteAgent { typed, .. } => delete_agent(deps, ui, ts, ws, agent, typed),
        Action::Monitor(verb) => monitor::dispatch(deps, ts, ws, agent, verb),
        // The §4.3 armed loop's family (bl-66fb): arming, which writes one
        // `cadence.yaml` entry. The loop itself is a thread, already running
        // and already finding nothing to do.
        Action::Fleet(verb) => fleet::dispatch(deps, ts, ws, verb),
        // The §8.6 capability family's one writer: the once-answer row, then
        // the releasing `advance`.
        Action::AnswerHold { ruling, .. } => control::answer_hold(deps, ts, ws, agent, *ruling),
        // The same family's other writer (VISION §4.9's fifth rung): standing
        // policy for a whole descent, one row, nothing launched.
        Action::Floor { raised, .. } => control::set_floor(deps, ts, ws, agent, *raised),
        // The trail's own two operator verbs (§4.2, bl-c417): the same one
        // bodies the frame's ops pane calls ([`crate::opslog::ack`]/[`clear`]).
        Action::Ack => wrote(crate::opslog::ack(root, ts), Reply::Acked),
        Action::MarkSeen { .. } => acknowledge(deps, ui, ts, ws, agent),
        Action::ClearTrail => wrote(crate::opslog::clear(root, ts), Reply::TrailCleared),
        // The §9 config family (bl-3f46) — one executor module, because each of
        // the three is a composition of pipelines that already exist.
        Action::ApplyConfig { file, text } => config::apply(deps, ts, ws, file, text),
        Action::SetMarks { branch, .. } => config::set_marks(deps, ts, ws, branch),
        Action::PickModel {
            role,
            provider,
            model,
            ..
        } => config::pick_model(deps, ts, ws, &Pick::of(role, provider, model)),
        // REMOTE §5's tool-host presentation (bl-4e08): the set lands under the
        // identity the INTAKE carries, so the gate is who is asking rather than
        // what was named — an in-world caller has no client and is refused.
        Action::Advertise { tools } => advertise::advertise(deps, tools),
        // REMOTE §1.4's enrollment (bl-f4e3): mint a device's leaf on this
        // box's CA, seat its registration, answer the material and shred the
        // key. Operator grade only, and no gate here says so — an act is
        // outside the foot set, so `Grade::admits` refuses it one layer up.
        Action::Enroll(request) => enroll::enroll(deps, ts, request),
        // REMOTE §5's routing leg (bl-024b): queue a call for the machine that
        // advertised it, and take a tool host's answer to one. Neither waits —
        // the intake here is one thread for the whole world.
        Action::Route(verb) => routing::route(deps, ts, verb),
    }
}
