//! **The wire** (REMOTE §9 step 5, bl-b6fa): the mTLS channel a seat reaches
//! the control boundary over, and the transport a seat is made of.
//!
//! REMOTE §1.2 rules that the UI operates entirely via RPC and that even the
//! local, single-machine arrangement does the hard split. This module is the
//! channel that makes that possible: [`server`] is the engine's listener,
//! [`client`] is a seat's transport, [`frame`] is what they say to each other,
//! [`material`] is the operator-provisioned trust the whole thing rests on, and
//! [`seat`] is the first shipped consumer — the terminal seat, `yog seat`.
//!
//! **The wire is a transport for the boundary, not a vocabulary** (REMOTE §3).
//! [`intake`] hands a request straight to the deposit consumer's own context,
//! which decodes with the one codec and runs the one `dispatch`/`answer`. So
//! there is no wire verb, no wire-only capability and no second dispatch
//! implementation (VISION §8) — the listener is a **second intake to the same
//! chokepoints**, exactly as the gestures inbox is, and the inbox remains for
//! the world's own residents (REMOTE §3).
//!
//! **The listener rides the engine, not one face** (bl-b6fa). Both faces run
//! the deposit consumer so a deposit converges whichever face is up (§8.5, I0);
//! a seat wants the identical guarantee, so the listener boots in
//! [`Engine::boot`](crate::engine::Engine::boot) beside it. That is also what
//! keeps **one engine per world**: a windowed yog serves the wire it would
//! otherwise have to start a second engine to reach, and `yog serve` is the
//! same engine with no window.
//!
//! **Absence WAS the off switch, and is not since bl-ae05.** REMOTE §1.2's
//! window is a client of this listener now, so a box with no material would be
//! a window that paints nothing — and REMOTE §8 had already rejected both ways
//! around that. A boot therefore founds its own loopback trust root
//! ([`provision`]), which is the operator's own out-of-channel act performed on
//! the operator's own box. A *half*-provisioned wire the mint cannot heal is
//! still warned about and still refuses, because silently degrading to no
//! encryption is the one failure this design exists to exclude.

use crate::xdg::Env;
use std::sync::Arc;

/// The window's off-frame asker (REMOTE §1.2, bl-ae05) — the thread that makes
/// the local window a wire client of its own engine.
pub mod asker;
pub mod client;
pub mod frame;
/// The tool-host client mode (REMOTE §5, bl-024b) — the wire's second shipped
/// client: it advertises what this machine can run, rides a follow-class read
/// for its next invocation, and posts each capture back.
pub mod host;
pub mod intake;
/// The frame's half of that read path: the standing questions and what landed.
pub mod link;
pub mod material;
/// The mint (REMOTE §1.4, §8; bl-ae05) — the one `openssl` recipe, spent by the
/// engine's boot and by `yog wire-certs` alike.
pub mod provision;
pub mod seat;
pub mod server;
pub mod tls;

/// The argv seat's leading word: `yog seat`. Named once, here, because the arm
/// that routes it and the help that advertises it would otherwise be two facts.
pub const SEAT_SUBCMD: &str = "seat";

/// The tool-host client mode's leading word: `yog tool-host` (REMOTE §5,
/// bl-024b). Named here beside [`SEAT_SUBCMD`] and for its reason exactly.
pub const HOST_SUBCMD: &str = "tool-host";

/// **The address the local window dials** (REMOTE §1.2, §8; bl-ae05): loopback
/// at the port the listener actually bound.
///
/// The window is a client of `127.0.0.1` and of nothing else, whatever
/// `address` names the engine to the rest of the world — which is why
/// [`provision`] always puts loopback on the server leaf. The **bound** port
/// rather than the requested one, for the reason
/// [`Listener::address`](server::Listener::address) exists: a `:0` in the file
/// is a request, and only the listener knows what it became.
pub fn loopback(bound: &str) -> String {
    let port = bound.rsplit_once(':').map_or("", |(_, port)| port);
    format!("{}:{port}", provision::LOOPBACK)
}

/// Bring the engine's listener up, or explain why there is none.
///
/// **It founds its own trust root first** (REMOTE §8 as amended, bl-ae05).
/// Absence of material used to be the off switch; it cannot be any more,
/// because REMOTE §1.2 makes the window a client of this listener and a window
/// with no listener paints nothing. So [`provision::ensure`] performs the
/// out-of-channel mint — on this box, before anything is dialled — and what it
/// writes is aimed at loopback. Wider listening is still the operator's own
/// act: `address` is one fact with one home, and only an operator ever writes
/// a host that is not loopback into it.
///
/// A refusal is written to stderr and is never fatal — a box with no `openssl`
/// gets the engine yog has always been, and a seat that cannot reach it says so
/// at the seat.
pub fn listen(
    world: &Env,
    answerer: Arc<dyn server::Answerer>,
    presence: crate::registry::presence::Presence,
) -> Option<server::Listener> {
    if let Err(e) = provision::ensure(&material::dir(world)) {
        eprintln!("yog: wire: {e}");
    }
    let material = match material::read(world, material::Role::Server) {
        Ok(Some(m)) => m,
        Ok(None) => return None,
        Err(e) => {
            eprintln!("yog: wire: {e}");
            return None;
        }
    };
    match server::Listener::bind(&material, answerer, presence) {
        Ok(listener) => Some(listener),
        Err(e) => {
            eprintln!("yog: wire: {e}");
            None
        }
    }
}

#[cfg(test)]
mod tests;
