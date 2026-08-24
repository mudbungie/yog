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
//! **Which engine it reaches is the gesture's own workspace name** (REMOTE
//! §8.2). A name one of this box's [`entries`](super::entries) holds goes down
//! that entry's channel, on that entry's material, carrying the name that
//! workspace bears on its host; everything else — a name no entry holds, and a
//! gesture naming no workspace — goes to the flat directory's client material,
//! exactly as it always did. See [`channel`].
//!
//! stdout carries one product: the reply stream, one envelope per line (today
//! always one — see [`frame`](super::frame)). Exit: `0` the last reply is ok,
//! `1` it is not or the channel failed, `2` bad usage or no wire provisioned.

use super::channel::Origin;
use super::client::Seat;
use super::entries;
use super::material::{self, Material, REMEDY, Role};
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
    let (seat, carried) = match channel(world, &gesture, &value) {
        Ok(dialled) => dialled,
        Err(e) => {
            eprintln!("yog {VERB}: {e}");
            return USAGE_EXIT;
        }
    };
    match seat.ask(&carried) {
        Ok(stream) => report(&stream),
        Err(e) => {
            eprintln!("yog {VERB}: {e}");
            1
        }
    }
}

/// **Which channel this gesture goes down, and what it carries there**
/// (REMOTE §8.2). The gesture's workspace name is resolved over the entries
/// this box holds *first*; a name no entry holds — and a gesture naming no
/// workspace — goes where it always went, the flat directory's client
/// material. The flat directory therefore stays what it has always been: the
/// box's own root, and the one client relationship the box holds without
/// naming it.
///
/// **The leaf↔host-name mapping is spent at the channel boundary** (§8.2), and
/// since bl-670c this seat spends it at the same *function* the window's three
/// paths do — [`Origin::carried`]. An entry's leaf is the *client's* name for
/// the workspace and its [`WORKSPACE`](entries::WORKSPACE) file is the name that
/// workspace answers to on its host; when they differ the gesture is re-encoded
/// carrying the host's name, there, never earlier — every seat above this line
/// reasons in the leaf — and never later, because below it is a socket. When
/// they agree the operator's own envelope crosses byte for byte, as it always
/// has. One site, so a gesture cannot cross renamed from a window and unrenamed
/// from a terminal.
///
/// A half-provisioned entry refuses with **its own** sentence (`entries`), and
/// that refusal is one entry's rather than the box's: nothing here reads
/// through to the flat root on a name an entry does hold, because an entry that
/// exists is the answer to that name.
fn channel(world: &Env, gesture: &Gesture, value: &Value) -> Result<(Seat, Value), String> {
    let named = gesture
        .workspace()
        .and_then(|name| entries::entries(world).into_iter().find(|e| e.leaf == name));
    let Some(entry) = named else {
        return Ok((open(world)?, value.clone()));
    };
    Ok((entry.seat()?, Origin::of(&entry).carried(value)))
}

/// This machine's own seat — the flat root's, which is what every caller with
/// no workspace to resolve wants. Shared with the tool-host client mode
/// (bl-024b), which is provisioned by the same out-of-channel act and refuses
/// in the same words.
pub(crate) fn open(world: &Env) -> Result<Seat, String> {
    Seat::open(&flat(world)?)
}

/// The flat directory's client material, or why this box has none. Absent
/// material is a refusal here rather than the silence it is at the engine: a
/// seat with nothing to present has nothing to do, and the remedy is the same
/// out-of-channel act (§1.4).
pub(crate) fn flat(world: &Env) -> Result<Material, String> {
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
        Some(m) => Ok(m),
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
