//! **The receipt, folded** (REMOTE §9.8, bl-1747) — the other half of
//! [`super`], split off at §12's per-file budget on the seam that file's own doc
//! draws: everything there is the **hold** (what a gesture posts and what it
//! writes down about itself), and this is what happens when the engine answers.
//!
//! Two folds per receipt and no more, because a receipt owes two parties: the
//! act's own ([`acted`], which is `Owes`) and the firing seat's (the `match`
//! below, which is `Seat`). They run in that order and the first may end the
//! frame's work outright — a gesture that **handed off** to a second act has not
//! finished, so the seat that composed it must not be told it has.

use super::{Acting, Owes, Seat, fan, start};
use crate::AppModel;
use crate::actions::DraftKey;
use crate::shell::ShellState;

/// One frame's fold of the outstanding act. Nothing in flight is nothing to do,
/// which is every frame but the handful after a gesture.
pub(in crate::shell) fn settle(model: &mut AppModel, state: &mut ShellState) {
    let Some(ticket) = state.acting.as_ref().map(|a| a.ticket) else {
        return;
    };
    let Some(landed) = model.act_receipt(ticket) else {
        return;
    };
    let Some(acting) = state.acting.take() else {
        return;
    };
    let trouble = crate::shell::act::trouble(&landed);
    // The act's own fold first, and only on a clean landing: a refusal changed
    // nothing in the world, so it may change nothing on the frame. It answers
    // whether the gesture **handed off** — the composer's Enter chains a
    // `Prompt` behind its `Prepared` — because the seat's fold belongs to the
    // act that finished the gesture, never to the one that got half way.
    if trouble.is_none() && acted(model, state, &acting, &landed) {
        return;
    }
    // **Which conversation this receipt started**, if it started one — read off
    // the reply itself, which is the only thing that knows the minted §3.3 name,
    // and read here because both `acting` and `landed` are about to be spent.
    let started = match (&acting.owes, &landed) {
        (Owes::Started { .. }, Ok(crate::boundary::reply::Reply::Started { conversation })) => {
            Some(conversation.clone())
        }
        _ => None,
    };
    match acting.seat {
        Seat::Quiet => quietly(state, trouble),
        Seat::Draft(key, fired) => match trouble {
            None => {
                state.actions.drafts.sent(&key, &fired);
                // **What is left in the box follows the box** (bl-56c6). A
                // start is fired from the *new conversation* composer, and the
                // instant it lands that same box is the started conversation's
                // — a different [`DraftKey`], because the key is the target
                // (bl-a69a). Text typed while the pair of acts was in flight is
                // the operator's, so it travels rather than being stranded
                // under a spelling nothing points at any more. Only a landed
                // start re-keys its own box; every other send's two spellings
                // are the same one, and [`crate::actions::Drafts::carry`] is a
                // no-op for those.
                if let Some(name) = started {
                    // The key the composer computes on its very next frame: the
                    // minted §3.3 name the reply carried, which is the §3.4
                    // claim the fire has just made.
                    let now = DraftKey::composer(model.focused_workspace(), Some(name));
                    state.actions.drafts.carry(&key, &now);
                }
            }
            Some(reason) => state.slash = Some(reason),
        },
        // The line's own rendering of the answer, in full — help as help, a
        // search as the §11 tab, anything else as its JSON — which also says
        // whether the line landed.
        Seat::Line(key) => {
            if crate::shell::slash::note(state, landed) {
                state.actions.drafts.set(key, String::new());
            }
        }
    }
}

/// The act's own aftermath. `true` when this receipt handed the gesture on to a
/// second act rather than finishing it.
fn acted(
    model: &mut AppModel,
    state: &mut ShellState,
    acting: &Acting,
    landed: &Result<crate::boundary::reply::Reply, String>,
) -> bool {
    use crate::boundary::reply::Reply;
    match (&acting.owes, landed) {
        (
            Owes::Message {
                agent,
                content,
                queued,
            },
            _,
        ) => {
            // The §3.4 echo is keyed by path like every other frame-side
            // optimism, so it is raised only where this box has one (bl-e349).
            if let Some(ws) = acting.ws.as_deref() {
                model.await_message(ws, agent, content, *queued);
            }
            false
        }
        (Owes::Prepared { goal }, Ok(Reply::Prepared(prepared))) => {
            start::staged(model, state, acting, prepared, goal.clone())
        }
        (Owes::Started { goal }, Ok(Reply::Started { conversation })) => {
            start::fired(model, state, acting.ws.as_deref(), conversation, goal);
            false
        }
        (Owes::Fanned { goal }, Ok(Reply::Fanned(candidates))) => {
            fan::fanned(model, state, acting.ws.as_deref(), candidates, goal)
        }
        // A clean reply of a kind this fire cannot read — a codec defect rather
        // than a state, and the seat below still says what came back.
        (_, _) => false,
    }
}

/// A quiet seat's refusal: the durable record is the act's own `ops.jsonl` line
/// the §7.3 banner reads back (INV-2), and this is the sentence beside it,
/// under the box where the operator is already looking.
fn quietly(state: &mut ShellState, trouble: Option<String>) {
    if let Some(reason) = trouble {
        state.slash = Some(reason);
    }
}
