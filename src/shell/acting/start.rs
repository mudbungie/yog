//! **The §8.1 start family's two doors, posted and held** (DESIGN §3.3, §3.4,
//! §8.1; REMOTE §9.8, bl-1747) — split from [`super`] per §12's budget, on the
//! seam that file's doc draws: everything there is the hold, and this is the one
//! pair whose landing is itself a step in a longer gesture.
//!
//! A start is two gestures — `Prepare` (seed/create/claim/ensure, answering the
//! composer's [`Prepared`]) and the deferred `Prompt` that fires it — and every
//! rung's aftermath used to ride the answer *in the same frame*. That is
//! precisely why the pair was §9.8's residual: the §3.4 workspace adoption, the
//! §3.3 mint-seed spend and the §3.4 start claim are frame-side facts gated on
//! a reply.
//!
//! **Fire-and-hold was already the shape** (§9.8 answer 4): the start claim and
//! the pending echo have always held state across frames awaiting a derivation.
//! What is new is only what retires the hold — an answer arriving rather than
//! the world catching up — and the general handle for that is the ticket. The
//! world's catch-up still retires the claim itself
//! ([`AppModel::adopt_started`]).

use super::{Acting, Owes, Seat};
use crate::AppModel;
use crate::actions::DraftKey;
use crate::boundary::Action;
use crate::shell::ShellState;
use crate::start::{Prepared, StartInputs};
use std::path::Path;

/// Post the §8.1 prepare for `inputs` — §3.4's two axes and nothing else, the
/// same pair the boundary's `Prepare` variant carries, because the rest (the
/// roots, the occupied names) re-derives inside the chokepoint from the sources
/// every frontend fills it from.
///
/// This is the rung that **stops at the goal box**: ▶ Start, ▶ Continue, the
/// new-ball form, the §11 raise. Nothing composed it, so nothing clears.
pub(in crate::shell) fn prepare(
    model: &mut AppModel,
    state: &mut ShellState,
    inputs: &StartInputs,
) {
    staging(model, state, inputs, Seat::Quiet, None);
}

/// The composer's own Enter (§3.4's bare and path rungs): the typed text **is**
/// the goal, so the `Prepared` that comes back is chained straight into a
/// `Prompt` and the box empties when *that* lands — the whole gesture, judged
/// once, exactly as the synchronous pair was.
pub(in crate::shell) fn fire(
    model: &mut AppModel,
    state: &mut ShellState,
    inputs: &StartInputs,
    key: &DraftKey,
    text: &str,
) {
    staging(
        model,
        state,
        inputs,
        Seat::Draft(key.clone()),
        Some(text.to_owned()),
    );
}

/// The post both rungs make.
fn staging(
    model: &mut AppModel,
    state: &mut ShellState,
    inputs: &StartInputs,
    seat: Seat,
    goal: Option<String>,
) {
    let action = Action::Prepare {
        workspace: model.snap.ws_name(&inputs.workspace),
        payload: inputs.payload.clone(),
    };
    super::hold(
        model,
        state,
        &inputs.workspace,
        &action,
        seat,
        Owes::Prepared { goal },
    );
}

/// Post the deferred prompt (§8.1) with **this seat's own §3.3 seed** — the one
/// the greyed prediction was drawn off, so the preview and the fired `--name`
/// are one draw (bl-28ba). The seed is spent when the fire *lands*, not here: a
/// launch that failed minted no name, so its prediction still stands.
pub(in crate::shell) fn prompt(
    model: &mut AppModel,
    state: &mut ShellState,
    ws: &Path,
    prepared: &Prepared,
    goal: &str,
) {
    post(model, state, ws, prepared, goal, Seat::Quiet);
}

/// The post itself, with the seat the caller is carrying — the chain below
/// hands the composer's own draft forward, so the box empties when the *prompt*
/// lands rather than when the prepare does.
fn post(
    model: &mut AppModel,
    state: &mut ShellState,
    ws: &Path,
    prepared: &Prepared,
    goal: &str,
    seat: Seat,
) {
    let action = Action::Prompt {
        prepared: prepared.clone(),
        goal: goal.to_owned(),
        seed: Some(state.start.mint_seed),
    };
    super::hold(
        model,
        state,
        ws,
        &action,
        seat,
        Owes::Started {
            goal: goal.to_owned(),
        },
    );
}

/// A landed `Prepare`: **adopt the workspace it resolved** (§3.4 — a start
/// focuses what it started), then either chain the prompt the composer's Enter
/// carried or open the goal box on the prefill. `true` when it chained.
///
/// The adoption is unconditional because it is never a second decision:
/// `prepare` resolves exactly one target workspace, and the rungs that name one
/// deliberately other than the focus (▶ Continue, the raise) are exactly where
/// it is the correction (bl-2826).
pub(super) fn staged(
    model: &mut AppModel,
    state: &mut ShellState,
    acting: &Acting,
    prepared: &Prepared,
    goal: Option<String>,
) -> bool {
    model.adopt_workspace(&acting.ws);
    let Some(text) = goal else {
        // A draft opens only when the rung composed one (bl-9acf): one
        // predicate, not a rung match — the prefill's blankness is the fact,
        // and the raise that arrives here with a blank one hands the keyboard
        // to the composer it just re-aimed.
        if crate::actions::goal_present(&prepared.goal) {
            state.start.pending = Some(prepared.clone());
        } else {
            crate::shell::focus::request(state);
        }
        return false;
    };
    post(
        model,
        state,
        &acting.ws,
        prepared,
        &text,
        acting.seat.clone(),
    );
    true
}

/// A landed `Prompt`: retire the seed the prediction spent (bl-28ba), drop the
/// pane the fire consumed, and hold the §3.4 start claim on the minted name.
///
/// The claim is held by the **minted conversation name**, which is all the fire
/// knows — a root has no agent id until the detached driver writes one — and it
/// carries the goal with it, so the operator's text has a row from the moment
/// the engine says it launched (§7.2, bl-915e).
pub(super) fn fired(
    model: &mut AppModel,
    state: &mut ShellState,
    ws: &Path,
    conversation: &str,
    goal: &str,
) {
    state.start.spend_mint();
    // Only a pane that was open hands the keyboard back (§11 focus
    // discipline); the bare rung's Enter never left the composer.
    if state.start.pending.take().is_some() {
        crate::shell::focus::request(state);
    }
    model.await_conversation(ws, conversation, goal);
}
