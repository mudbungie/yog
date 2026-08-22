//! **The act whose receipt the frame is still owed** (REMOTE §1.2, §9.8;
//! bl-1747) — the last four gestures whose answer the click used to read
//! synchronously, held across the frames between the post and the receipt.
//!
//! [`super::act`] is the general spelling: `fire` for the act nobody holds a
//! receipt for, `Held` for the act whose receipt is a *sentence*. This file is
//! the third and last kind — the act whose receipt gates a **frame-side state
//! change**: `Message` (the draft clear and the §3.4 echo), the §8.1
//! `Prepare`/`Prompt` doors (the workspace adoption, the §3.3 mint-seed spend,
//! the §3.4 start claim) and the §8.5 line's act arm (whether the typed line
//! clears). Those were REMOTE §9.8's whole residual, and `AppModel::dispatch`
//! was exactly their size.
//!
//! **Two orthogonal axes, because a receipt owes two different parties.**
//! [`Owes`] is what the *act* re-derives, which is a fact about the gesture and
//! the same however it was asked for. [`Seat`] is what the *hand that fired*
//! shows for it — a draft to empty, a reply to render, or neither. Every one of
//! the four is one of each, and nothing needs a fifth field: the §3.6 deletes
//! hold their own tickets on their own modals (`super::delete`,
//! `super::delete_agent`), because a dialog's answer is a dialog's.
//!
//! **One act at a time, and the newest wins.** A second fire while one is
//! outstanding replaces the hold, which is [`super::act::Held`]'s rule for its
//! reason exactly: the earlier act still happens — an act is never unsent — and
//! it is the newer one's answer this seat is waiting on.
//!
//! Coverage-excluded glue like the rest of `src/shell/*`: the posting, the
//! ticket and the receipt are covered where they live (`app::acts`,
//! `wire::post`), and what this file wires is driven end to end from
//! `shell::acceptance`.

use super::ShellState;
use crate::AppModel;
use crate::actions::DraftKey;
use crate::boundary::Action;
use crate::wire::post::Ticket;
use std::path::{Path, PathBuf};

/// The §3.8 fan's door and aftermath (bl-77bc) — the start family's N-wide
/// sibling, split on the same seam.
pub(super) mod fan;
/// The §8.1 start family's two aftermaths — split per §12's budget, on the seam
/// this file's own doc draws: everything here is the hold, and that is the one
/// [`Owes`] pair whose landing is itself a step in a longer gesture.
pub(super) mod start;

/// What the **act's** landing re-derives on the frame. A fact about the
/// gesture: the same act owes the same thing whichever hand fired it.
enum Owes {
    /// Nothing but the ops row every act already leaves (INV-2) — the §8.5
    /// line's ordinary verbs, whose durable record is the §7.3 banner's.
    Nothing,
    /// A landed §8.2 `Message`: the §3.4 pending echo. The deposit is piped and
    /// its `NNN-user.md` appears only at the driver's next step boundary, so
    /// without it the operator's own words leave the screen with the draft and
    /// are nowhere in yog until then. **Held on the receipt rather than on the
    /// synchronous `Ok`** (§9.8 ruling 3) — the echo and the receipt stay two
    /// facts at two rates, and only the trigger moved.
    ///
    /// `queued` is the §11 queue seat's reconciliation baseline (§7.2,
    /// bl-78d8): how many deposits that seat's standing `Query::Inbox` showed
    /// when this act was **posted**. It is read here and carried because here
    /// is the last moment it is knowable — the verb runs piped, so by the
    /// receipt the deposit is already on disk and a count taken then would
    /// retire the echo at birth.
    Message {
        agent: String,
        content: String,
        queued: usize,
    },
    /// A landed §8.1 `Prepare`: the §3.4 workspace adoption, then either the
    /// goal box on the prefill or — for the composer's own Enter, which carries
    /// its typed text straight through — the chained `Prompt`.
    Prepared { goal: Option<String> },
    /// A landed §8.1 `Prompt`: the §3.3 seed the prediction spent, the pane the
    /// fire consumed, and the §3.4 start claim on the minted name.
    Started { goal: String },
    /// A landed §3.8 `Fan(Spread)` (bl-77bc): the N rebound starts it answered,
    /// each owed its ordinary `Prompt` — the gesture hands off N times, and the
    /// last of them carries this seat's own aftermath.
    Fanned { goal: String },
}

/// What the **seat** that fired shows for the receipt. A fact about the hand:
/// the same act clears a different draft, or none, depending on which one made
/// it.
#[derive(Clone)]
enum Seat {
    /// Nothing composed it and nothing paints its answer — a button, a rung, a
    /// row menu. A refusal still reaches the operator, as the §7.3 banner reads
    /// the act's own `ops.jsonl` line back.
    Quiet,
    /// The composer's box (bl-a69a): a clean landing empties **this** draft and
    /// no other, the box being one widget over many buffers and the selection
    /// being free to move while the act is in flight.
    Draft(DraftKey),
    /// The §8.5 line: a clean run empties the line **and** shows the reply — the
    /// same JSON a deposited line's answer file carries, because a line typed at
    /// the window and one deposited from a terminal earn the same answer.
    Line(DraftKey),
}

/// One posted act, and the two folds its receipt owes.
pub(super) struct Acting {
    ticket: Ticket,
    /// The workspace **path** the aftermath is about. A reply spells a
    /// workspace only as a §3.1 name, and a start that raises one names a
    /// workspace no snapshot can resolve a path for yet — the raise is what
    /// founds it — so the path is the fire's own knowledge, carried rather than
    /// re-derived.
    ws: PathBuf,
    seat: Seat,
    owes: Owes,
}

/// Deposit the composer's box into an inbox — **either depositing gesture**
/// (§8.2's resume send, and bl-a33d's send-and-interrupt behind it): a clean
/// landing clears the draft and raises the §3.4 echo, and a refusal leaves the
/// operator's words exactly where they can be fixed and re-sent (§5.3 — a draft
/// is RAM until *sent*, and over the wire "sent" is not knowable at the click).
///
/// One body for the two, because the aftermath is a fact about depositing and
/// not about which verb ran ahead of it: the caller constructs the variant, and
/// which one it is decides what the substrate does, never what the box shows.
pub(super) fn deposit(
    model: &mut AppModel,
    state: &mut ShellState,
    key: &DraftKey,
    ws: &Path,
    action: &Action,
) {
    let owes = match action {
        Action::Message { agent, content, .. } | Action::Interrupt { agent, content, .. } => {
            Owes::Message {
                agent: agent.clone(),
                content: content.clone(),
                queued: shown(model, ws, agent),
            }
        }
        _ => Owes::Nothing,
    };
    hold(model, state, ws, action, Seat::Draft(key.clone()), owes);
}

/// Fire one §8.5 line's **act** arm: the note under the box and whether the
/// typed line clears are both its receipt's, and the start family's own
/// aftermath rides it too — a `/prepare` seats the composer and a `/prompt`
/// spends the seed exactly as the buttons' fires do, because that is the act's
/// consequence and not the seat's.
pub(super) fn line(
    model: &mut AppModel,
    state: &mut ShellState,
    key: &DraftKey,
    ws: &Path,
    action: &Action,
) {
    let owes = match action {
        Action::Prepare { .. } => Owes::Prepared { goal: None },
        Action::Prompt { goal, .. } => Owes::Started { goal: goal.clone() },
        _ => Owes::Nothing,
    };
    hold(model, state, ws, action, Seat::Line(key.clone()), owes);
}

/// How many deposits the §11 queue is showing for `agent` **right now**
/// (bl-78d8) — the echo's queue-seat baseline, read off the very ask the region
/// above the box paints from (`inspector::inbox`, one memoized standing
/// question, so this costs nothing the frame was not already asking). An
/// unanswered ask showed nothing, which is the same count at zero rather than a
/// case of its own.
fn shown(model: &mut AppModel, ws: &Path, agent: &str) -> usize {
    super::inspector::inbox(model, ws, agent)
        .value
        .map_or(0, |entries| entries.len())
}

/// Post and hold — the one place this seat mints a ticket.
fn hold(
    model: &mut AppModel,
    state: &mut ShellState,
    ws: &Path,
    action: &Action,
    seat: Seat,
    owes: Owes,
) {
    let ticket = model.post_act(action);
    state.acting = Some(Acting {
        ticket,
        ws: ws.to_path_buf(),
        seat,
        owes,
    });
}

/// One frame's fold of the outstanding act. Nothing in flight is nothing to do,
/// which is every frame but the handful after a gesture.
pub(super) fn settle(model: &mut AppModel, state: &mut ShellState) {
    let Some(ticket) = state.acting.as_ref().map(|a| a.ticket) else {
        return;
    };
    let Some(landed) = model.act_receipt(ticket) else {
        return;
    };
    let Some(acting) = state.acting.take() else {
        return;
    };
    let trouble = super::act::trouble(&landed);
    // The act's own fold first, and only on a clean landing: a refusal changed
    // nothing in the world, so it may change nothing on the frame. It answers
    // whether the gesture **handed off** — the composer's Enter chains a
    // `Prompt` behind its `Prepared` — because the seat's fold belongs to the
    // act that finished the gesture, never to the one that got half way.
    if trouble.is_none() && acted(model, state, &acting, &landed) {
        return;
    }
    match acting.seat {
        Seat::Quiet => quietly(state, trouble),
        Seat::Draft(key) => match trouble {
            None => state.actions.drafts.set(key, String::new()),
            Some(reason) => state.slash = Some(reason),
        },
        // The line's own rendering of the answer, in full — help as help, a
        // search as the §11 tab, anything else as its JSON — which also says
        // whether the line landed.
        Seat::Line(key) => {
            if super::slash::note(state, landed) {
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
            model.await_message(&acting.ws, agent, content, *queued);
            false
        }
        (Owes::Prepared { goal }, Ok(Reply::Prepared(prepared))) => {
            start::staged(model, state, acting, prepared, goal.clone())
        }
        (Owes::Started { goal }, Ok(Reply::Started { conversation })) => {
            start::fired(model, state, &acting.ws, conversation, goal);
            false
        }
        (Owes::Fanned { goal }, Ok(Reply::Fanned(candidates))) => {
            fan::fanned(model, state, &acting.ws, candidates, goal)
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
