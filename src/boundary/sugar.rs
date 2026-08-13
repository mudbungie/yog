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
//! yog converges it (I0).

use serde_json::Value;
use std::path::Path;

use super::{Gesture, Query, codec, deposit, help, line};

mod argv;

/// The no-consumer exit (the shell's timeout convention).
pub const TIMEOUT_EXIT: i32 = 124;
/// The never-deposited exit: bad usage, bad JSON, an unknown gesture.
pub const USAGE_EXIT: i32 = 2;

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
    let (gesture, value) = match argv::read(args).and_then(|it| envelope(&it)) {
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
        println!("{}", help::render(&help::rows(verb.as_deref())));
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

/// The deposit envelope this invocation means: a line read at the seat its
/// flags describe, or the JSON envelope validated as written. Either way what
/// is deposited is the codec's own encoding — the line is a serialization of
/// the boundary, never a second inbox format.
fn envelope(invocation: &argv::Invocation) -> Result<(Gesture, Value), String> {
    if line::is_command(&invocation.payload) {
        let gesture = line::parse(&invocation.payload, &invocation.context)?;
        let value = codec::encode(&gesture);
        return Ok((gesture, value));
    }
    let value: Value =
        serde_json::from_str(&invocation.payload).map_err(|e| format!("not JSON: {e}"))?;
    // The envelope is deposited **as written**, not as re-encoded: the audit
    // keeps the operator's own bytes. Decoding is the validation and the read
    // the help short-circuit above needs.
    Ok((codec::decode(&value)?, value))
}

#[cfg(test)]
mod tests;
