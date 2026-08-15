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
//! **Absence is the off switch.** With no material provisioned there is no
//! listener and nothing said about it: removing the directory deletes config,
//! not code. A *half*-provisioned wire is warned about, because silently
//! degrading to no encryption is the one failure this design exists to
//! exclude.

use crate::xdg::Env;
use std::sync::Arc;

pub mod client;
pub mod frame;
/// The tool-host client mode (REMOTE §5, bl-024b) — the wire's second shipped
/// client: it advertises what this machine can run, rides a follow-class read
/// for its next invocation, and posts each capture back.
pub mod host;
pub mod intake;
pub mod material;
pub mod seat;
pub mod server;
pub mod tls;

/// The argv seat's leading word: `yog seat`. Named once, here, because the arm
/// that routes it and the help that advertises it would otherwise be two facts.
pub const SEAT_SUBCMD: &str = "seat";

/// The tool-host client mode's leading word: `yog tool-host` (REMOTE §5,
/// bl-024b). Named here beside [`SEAT_SUBCMD`] and for its reason exactly.
pub const HOST_SUBCMD: &str = "tool-host";

/// Bring the engine's listener up, or explain why there is none. `None` is the
/// ordinary answer on a box with no wire provisioned; a refusal is written to
/// stderr and is never fatal — an engine with no wire is the engine yog has
/// always been, and a seat that cannot reach it says so at the seat.
pub fn listen(
    world: &Env,
    answerer: Arc<dyn server::Answerer>,
    presence: crate::registry::presence::Presence,
) -> Option<server::Listener> {
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
