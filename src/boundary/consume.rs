//! One consumption pass over the gestures inbox (§8.5): claim each pending
//! deposit, run it through the same two chokepoints the GUI uses
//! ([`dispatch`](super::dispatch::dispatch) / [`answer`](super::answer::answer)),
//! and write its reply file. Pure over its inputs — the thread that drives it
//! is [`super::consumer`], and a test drives this directly.
//!
//! No dirty-marking here: an action's effects land in watched roots, and I4's
//! rule stands — watches are latency, the sweeps are correctness. A reply that
//! cannot be written is a `["yog-step","gesture-reply"]` failure row (§4.2),
//! so no error class is dropped (INV-2).

use crate::actions::verbs::log_step_failure;
use crate::opslog::Origin;
use crate::ui_state::UiState;
use serde_json::Value;
use std::fs;

use super::dispatch::{Deps, dispatch};
use super::{Gesture, answer, codec, deposit, reply};

/// Claim and answer every pending deposit. Returns how many were consumed.
/// `ts` stamps the ops rows the executors write; `now_unix` is the query
/// families' wall clock (both minted by the caller, one boundary).
pub fn consume(deps: &Deps, ui: &mut UiState, ts: &str, now_unix: i64) -> usize {
    let root = deps.state_root.clone();
    let mut consumed = 0;
    for (id, _) in deposit::pending(&root) {
        // The rename is the claim: losing the race to another consumer is the
        // benign outcome, not an error — the winner answers.
        let Ok(claimed) = deposit::claim(&root, &id) else {
            continue;
        };
        let answered = run(
            deps,
            ui,
            ts,
            now_unix,
            &fs::read(&claimed).unwrap_or_default(),
        );
        if deposit::write_reply(&root, &id, &answered).is_err() {
            let _ = log_step_failure(
                &root,
                ts,
                &deposit::gestures_dir(&root),
                "gesture-reply",
                &format!("reply for {id:?} could not be written"),
                Origin::World,
            );
        }
        consumed += 1;
    }
    consumed
}

/// Decode and run one gesture's bytes to its reply value. Every failure mode
/// is a refusal envelope naming its reason — a torn or hand-mangled deposit
/// answers, it does not wedge the inbox.
fn run(deps: &Deps, ui: &mut UiState, ts: &str, now_unix: i64, bytes: &[u8]) -> Value {
    let parsed: Value = match serde_json::from_slice(bytes) {
        Ok(v) => v,
        Err(e) => return reply::refusal(&format!("deposit is not JSON: {e}")),
    };
    run_value(deps, ui, ts, now_unix, &parsed)
}

/// One already-parsed gesture envelope, run to its reply value. **This is the
/// one room both intakes open onto** (REMOTE §3, bl-b6fa): the deposit above
/// reaches it after reading a file, and a wire connection
/// ([`crate::wire::intake`]) reaches it after reading a frame — same codec,
/// same chokepoints, so the wire can add no verb.
pub(crate) fn run_value(
    deps: &Deps,
    ui: &mut UiState,
    ts: &str,
    now_unix: i64,
    parsed: &Value,
) -> Value {
    match codec::decode(parsed) {
        Ok(Gesture::Act(action)) => match dispatch(deps, ui, ts, &action) {
            Ok(r) => reply::encode(&r),
            Err(e) => reply::refusal(&e),
        },
        Ok(Gesture::Ask(query)) => match answer::answer(&query, deps, ui, now_unix) {
            Ok(r) => reply::encode(&r),
            Err(e) => reply::refusal(&e),
        },
        Err(e) => reply::refusal(&e),
    }
}

#[cfg(test)]
mod tests;
