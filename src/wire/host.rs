//! `yog tool-host` — **the client-side executor** (REMOTE §5, §9 step 7;
//! bl-024b): the far end of the routing leg, and the wire's second shipped
//! client mode.
//!
//! It is a *client*, not a server, and that is REMOTE §3's whole routing
//! ruling: the ask never inverts. This process dials the engine exactly as
//! [`seat`](super::seat) does, presents what this machine can run, and then
//! **rides a follow-class read** for its next invocation — one ordinary
//! `Query` whose answer takes as long as it takes. It runs what comes back and
//! posts each capture as an ordinary `Action`. Nothing about the framing, the
//! listener or a client's socket posture changes; the engine's work flows down
//! a stream this process asked for.
//!
//! **One loop, three gestures, all of them the boundary's** (REMOTE §3's ban on
//! wire-only verbs): `advertise` once, then `invocations` → run → `complete`,
//! forever. Every one of them is typable at any other seat.
//!
//! **It runs serially and it does not reconnect.** A host executes one
//! invocation at a time, so it is *absent* — holding no connection — for as
//! long as a tool takes, which is why nothing in the engine treats presence as
//! the routing predicate (see [`crate::registry::mailbox`]). And when the
//! channel fails it exits, saying why: a reconnect ladder is a policy about
//! *this machine's* supervision, which the operator who installed the tool host
//! already owns, and inventing one here would be yog deciding how a box it does
//! not administer restarts a program.
//!
//! **It serves every channel this box holds** (REMOTE §8.2, bl-4e31): the flat
//! root and one per [`entry`](crate::wire::entries), each on that entry's own
//! material and therefore under that entry's own client identity. Serial stays
//! serial *per channel*; [`entries`] is the resolution and the fan-out, and its
//! doc is where that seam is stated.

use std::time::Duration;

use serde_json::Value;

use super::client::Seat;
use super::material::Material;
use crate::boundary::codec;
use crate::boundary::reply::{self, Reply};
use crate::boundary::{Action, Gesture, Query};
use crate::registry::mailbox::{Capture, Completion, Invocation, Verb};
use crate::xdg::Env;

/// What this machine can run, and how (REMOTE §5.2).
pub mod config;
/// Every channel this box serves (REMOTE §8.2, bl-4e31) — the flat root beside
/// one per entry, and the fan-out that serves them at once.
pub(crate) mod entries;
/// Running one invocation locally — lernie's own tool contract.
pub mod exec;

/// This mode's own word, for the usage line its refusals carry.
const VERB: &str = super::HOST_SUBCMD;

/// How long one tool may run before the child is terminated and the capture
/// says so. It is the host's own bound, not the caller's: the machine that
/// spawned the process is the one that can stop it, and the driver's patience
/// (`tool_host::remote::patience`) stands behind it as a second, longer bound
/// for the case where this whole process went away.
const DEADLINE: Duration = Duration::from_mins(2);

/// Run the tool-host mode. It does not return while the wire is up; a channel
/// that fails is an exit with the reason on stderr, and a machine with no
/// config or no wire material is the same refusal `yog seat` gives.
pub fn run(world: &Env, args: &[String]) -> i32 {
    if let Some(extra) = args.first() {
        eprintln!("yog {VERB}: takes no arguments, got {extra:?}");
        return super::seat::USAGE_EXIT;
    }
    eprintln!("yog {VERB}: {}", serve(world));
    1
}

/// Present, then wait, run and answer — on **every** channel this box is
/// provisioned for — until each has stopped, and **it answers the sentences
/// that stopped them**.
///
/// There is no success exit, so none is spelled: a channel's only way out is a
/// gesture that failed, and a `Result` here would carry an `Ok` arm no state of
/// the world can reach. Every sentence below is a reason an operator can read.
///
/// The config is read first, because a machine with nothing to offer has
/// nothing to present and no reason to dial anything. Then §8.2's channel set:
/// a box with no channel at all refuses with what its channels said, which for
/// a box holding no entries is the flat root's own sentence and nothing else.
fn serve(world: &Env) -> String {
    let set = match config::read(&config::path(world)) {
        Ok(set) => set,
        Err(reason) => return reason,
    };
    let (held, refused) = entries::channels(world);
    if held.is_empty() {
        return refused.join("\n");
    }
    // A channel this box cannot open is that channel's refusal, said once —
    // never the whole host's, which is reserved for holding no channel at all.
    for reason in &refused {
        eprintln!("yog {VERB}: {reason}");
    }
    entries::fan(&set, held)
}

/// One channel, served: advertise once, then `invocations` → run → `complete`,
/// forever. Serial by construction, which is REMOTE §10's deferred-concurrency
/// row unmoved — a host executes one invocation at a time, per engine it is
/// present at.
fn hold(set: &[config::Local], material: &Material) -> String {
    let seat = match Seat::open(material) {
        Ok(seat) => seat,
        Err(reason) => return reason,
    };
    let presenting = Gesture::Act(Action::Advertise {
        tools: config::advertisement(set),
    });
    if let Err(reason) = tell(&seat, &presenting) {
        return reason;
    }
    loop {
        let work = match waited(&seat) {
            Ok(work) => work,
            Err(reason) => return reason,
        };
        for invocation in work {
            let capture = exec::execute(set, &invocation, DEADLINE);
            if let Err(reason) = answer(&seat, &invocation, capture) {
                return reason;
            }
        }
    }
}

/// The follow-class read: this machine's next work, or the empty answer of a
/// hold that ended quietly. Both are ordinary; only a channel failure is not.
fn waited(seat: &Seat) -> Result<Vec<Invocation>, String> {
    match tell(seat, &Gesture::Ask(Query::Invocations))? {
        Reply::Invocations(rows) => Ok(rows),
        other => Err(format!(
            "engine answered {other:?}, not this machine's work"
        )),
    }
}

/// Post one capture back. The receipt is read rather than discarded, because
/// an engine that refused the completion — an expired handle, a slot addressed
/// elsewhere — is a thing this host must stop rather than keep answering into.
fn answer(seat: &Seat, invocation: &Invocation, capture: Capture) -> Result<(), String> {
    tell(
        seat,
        &Gesture::Act(Action::Route(Verb::Complete(Completion {
            invocation: invocation.id.clone(),
            capture,
        }))),
    )
    .map(|_| ())
}

/// One gesture over the wire, in the one codec, read back with the one reply
/// decoder — so this client speaks exactly what every other seat speaks and can
/// add nothing to it.
fn tell(seat: &Seat, gesture: &Gesture) -> Result<Reply, String> {
    let stream = seat.ask(&codec::encode(gesture))?;
    let last: &Value = stream
        .last()
        .ok_or("the engine closed the stream without answering")?;
    match reply::decode(last) {
        Ok(answered) => answered,
        Err(e) => Err(format!("undecodable engine reply: {e}")),
    }
}

#[cfg(test)]
mod tests;
