//! `yog gesture <gesture>` — deposit-and-wait sugar over the [`deposit`] inbox
//! (§8.5): validate the gesture, deposit it create-only, poll for the reply
//! file, print it, exit with its verdict.
//!
//! **Two spellings reach the same deposit** (§8.5): the [`codec`] JSON
//! envelope, and the [`line`] — `yog gesture '/scan' --ws …`, the slash command
//! a human can actually type at a terminal. A line is read into a [`Gesture`]
//! and encoded to the very envelope the JSON spelling would have been, so the
//! transport, the audit and the executor below are one path, not two. The
//! argv verb *is* the deposit path — never a second dispatch implementation
//! (VISION §8) — so a gesture converges identically whether typed here or
//! written by hand into the inbox.
//!
//! stdout carries one product: the reply JSON — or, for `--help`, the answer
//! itself (§8.5: help reads the interface, not the world, so no consumer is
//! involved and the exit is 0).
//!
//! stdout carries one product: the reply JSON. Refusals and the timeout note
//! go to stderr. Exit: `0` reply ok, `1` reply not-ok or a deposit failure,
//! `2` an envelope that never deposited (usage/parse/decode), [`TIMEOUT_EXIT`]
//! when no consumer answered — the deposit **remains**, and the next running
//! yog converges it (I0). A gesture claimed by an engine that then died is
//! answered *in doubt* at the next boot (bl-d1f1, [`deposit`]'s module doc):
//! the reply arrives as a refusal, so a still-polling caller exits `1` with
//! the recovery contract on stdout rather than waiting out the budget.

use serde_json::Value;
use std::path::Path;

use super::{Gesture, Query, deposit, help};

pub(crate) mod argv;

/// The no-consumer exit (the shell's timeout convention).
pub const TIMEOUT_EXIT: i32 = 124;
/// The never-deposited exit: bad usage, bad JSON, an unknown gesture.
pub const USAGE_EXIT: i32 = 2;
/// This seat's own word, for the usage line its refusals carry.
const VERB: &str = "gesture";

/// **What this seat answers `--help` with** (bl-e66f): how to aim a gesture
/// *here*, then the shared gesture list.
///
/// `--help` is a rewrite into `/help` precisely so **one answer serves both
/// seats** (§8.5's higher-order rule), which is exactly why the argv flags may
/// not live inside that answer: they are one seat's. So the seat prints its own
/// line around it. Until bl-e66f it printed nothing of its own, and the flags
/// existed only in refusals — so `yog gesture --help`, the one place an
/// operator looks, named none of them, and the way to learn how to aim a
/// gesture was to type one wrong.
fn help_answer(verb: Option<&str>) -> String {
    format!(
        "{}\n\n{}",
        argv::usage(VERB),
        help::render(&help::rows(verb))
    )
}

/// Run the sugar verb: `args` is the multiplexed tail (exactly one JSON
/// envelope), `seed` the legibility hint the deposit id is minted from
/// ([`deposit::mint`] — the world, not the caller, decides the id), and
/// `waits`×`wait()` the poll budget (injected, so tests never sleep). See the
/// module doc for exits.
pub fn run(
    state_root: &Path,
    args: &[String],
    seed: &str,
    waits: u32,
    wait: &mut dyn FnMut(),
) -> i32 {
    // Read, then validate, then deposit: a gesture either spelling refuses must
    // never enter the inbox — the refusal belongs to the depositor, not the
    // trail.
    let (gesture, value) = match argv::read_gesture(VERB, args) {
        Ok(read) => read,
        Err(e) => {
            eprintln!("yog gesture: {e}");
            return USAGE_EXIT;
        }
    };
    // **Help is answered here** (§8.5): its subject is the interface, not the
    // world, so there is nothing to consume it and nothing to wait for. Asking
    // what a command does must not depend on a yog being up — and must not
    // exit non-zero, because it is an answer, not a refusal.
    if let Gesture::Ask(Query::Help { verb }) = &gesture {
        println!("{}", help_answer(verb.as_deref()));
        return 0;
    }
    // Mint, then deposit: the id is won from the world (an exclusive reply-slot
    // reservation), never guessed from a clock and a pid — two process
    // namespaces share both, and a shared id is a shared reply (bl-aa9f).
    let (id, deposited) = match deposit::mint(state_root, seed)
        .and_then(|id| deposit::deposit(state_root, &id, &value).map(|path| (id, path)))
    {
        Ok(minted) => minted,
        Err(e) => {
            eprintln!("yog gesture: deposit failed: {e}");
            return 1;
        }
    };
    for _ in 0..waits {
        if let Some(reply) = deposit::read_reply(state_root, &id) {
            println!("{reply}");
            let ok = reply.get("ok").and_then(Value::as_bool).unwrap_or(false);
            return i32::from(!ok);
        }
        wait();
    }
    eprintln!(
        "yog gesture: no consumer answered; the deposit remains at {}",
        deposited.display()
    );
    TIMEOUT_EXIT
}

#[cfg(test)]
mod tests;
