//! `yog seat <gesture>` — **the wire's first shipped seat** (REMOTE §2, §8;
//! bl-b6fa): the same gesture surface `yog gesture` types, sent over the mTLS
//! channel instead of deposited into the world's inbox.
//!
//! **Two intakes, one boundary** (REMOTE §3). `yog gesture` is the *world's own
//! resident's* door — same machine, same disk, disk is the bus — and it stays.
//! This is the door for a caller across a trust domain, which is every caller
//! that holds a certificate and no world. The argv, the flags, the `--help`
//! rewrite and the refusals are literally the same reader
//! ([`argv::read_gesture`](crate::boundary::sugar::argv::read_gesture)), so the
//! two seats cannot drift; only the transport below them differs.
//!
//! It is a *seat*, not a verb: REMOTE §3's ban is on a capability that exists
//! on the wire and nowhere else, and this adds none — a gesture typed here is
//! the same envelope, answered by the same `dispatch`/`answer`. A TUI or a
//! phone is the next consumer of exactly this transport and needs nothing new
//! from the engine (REMOTE §8).
//!
//! stdout carries one product: the reply stream, one envelope per line (today
//! always one — see [`frame`](super::frame)). Exit: `0` the last reply is ok,
//! `1` it is not or the channel failed, `2` bad usage or no wire provisioned.

use super::client::Seat;
use super::material::{self, REMEDY, Role};
use crate::boundary::sugar::argv;
use crate::boundary::{Gesture, Query, help};
use crate::xdg::Env;
use serde_json::Value;

/// This seat's own word, for the usage line its refusals carry.
const VERB: &str = "seat";
/// Bad usage, an undecodable gesture, or a machine with no wire.
pub const USAGE_EXIT: i32 = 2;

/// Run the seat verb: `args` is the multiplexed tail. See the module doc for
/// the exits.
pub fn run(world: &Env, args: &[String]) -> i32 {
    let (gesture, value) = match argv::read_gesture(VERB, args) {
        Ok(read) => read,
        Err(e) => {
            eprintln!("yog {VERB}: {e}");
            return USAGE_EXIT;
        }
    };
    // **Help is answered here** (§8.5), for the reason it is answered at every
    // other seat: its subject is the interface, not the world, so asking what a
    // verb does must not depend on an engine being up — or, here, on this
    // machine having been provisioned at all.
    if let Gesture::Ask(Query::Help { verb }) = &gesture {
        println!("{}", help::render(&help::rows(verb.as_deref())));
        return 0;
    }
    let seat = match open(world) {
        Ok(seat) => seat,
        Err(e) => {
            eprintln!("yog {VERB}: {e}");
            return USAGE_EXIT;
        }
    };
    match seat.ask(&value) {
        Ok(stream) => report(&stream),
        Err(e) => {
            eprintln!("yog {VERB}: {e}");
            1
        }
    }
}

/// This machine's seat, or why it has none — shared with the tool-host client
/// mode (bl-024b), which is provisioned by the same out-of-channel act and
/// refuses in the same words. Absent material is a refusal here
/// rather than the silence it is at the engine: a seat with nothing to present
/// has nothing to do, and the remedy is the same out-of-channel act (§1.4).
pub(crate) fn open(world: &Env) -> Result<Seat, String> {
    match material::read(world, Role::Client)? {
        // A `:0` is self-provisioning's request for a kernel-chosen port
        // (bl-dc14): only the engine that bound it knows what it became, and
        // it tells its own window in RAM — so there is nothing here to dial,
        // and saying so beats the raw connect error a port 0 earns.
        Some(m) if m.address.ends_with(":0") => Err(format!(
            "{} names {} — a kernel-chosen port only that engine's own window \
             is told; a seat wants a stated address: run `{REMEDY}`",
            material::dir(world).join(material::ADDRESS).display(),
            m.address
        )),
        Some(m) => Seat::open(&m),
        None => Err(format!(
            "no wire provisioned at {} — run `{REMEDY}`",
            material::dir(world).display()
        )),
    }
}

/// Print the reply stream and exit on its last envelope's verdict. An empty
/// stream is an engine that terminated without answering — not ok.
fn report(stream: &[Value]) -> i32 {
    for chunk in stream {
        println!("{chunk}");
    }
    let ok = stream
        .last()
        .and_then(|r| r.get("ok"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    i32::from(!ok)
}

#[cfg(test)]
mod tests;
