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
//! the operator's own box — aimed at `127.0.0.1:0`, so no two engines contend
//! for a process-global port (I0, bl-dc14). A *half*-provisioned wire the mint
//! cannot heal still refuses, because silently degrading to no encryption is
//! the one failure this design exists to exclude — and the refusal is a
//! *returned sentence* now, painted by the window it starves
//! (`shell::refusal`) rather than lost on a desktop launch's stderr.

use crate::xdg::Env;
use std::sync::Arc;

pub mod frame;
/// The version preface (REMOTE §3, bl-a670): what each end states about itself
/// before it says anything, and the fail-closed refusal a skew earns.
pub mod hello;
pub mod intake;
pub mod material;
/// The mint (REMOTE §1.4, §8; bl-ae05) — the one `openssl` recipe, spent by the
/// engine's boot and by `yog wire-certs` alike.
pub mod provision;
pub mod server;
pub mod tls;

/// **How often a seat re-asks its standing set** — human cadence (REMOTE §3),
/// and the number REMOTE §10's ask-rate criterion is measured against.
///
/// It is a **protocol** period rather than a client's private knob, which is
/// why it survived the seat's departure (bl-7942): the §7.3 wound grace is the
/// sum of every leg a fact crosses before a seat can see it
/// ([`Cadence::wound_grace`](crate::app::Cadence::wound_grace)), and the last
/// of those legs is the ask. A server that did not name the period could not
/// state the grace, and would raise the alarm the grace exists to prevent.
pub const ASK_PERIOD: std::time::Duration = std::time::Duration::from_millis(500);

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
/// A refusal is *returned*, never fatal — a box with no `openssl` gets the
/// engine yog has always been. The caller owns the saying (bl-dc14): the
/// engine's boot writes it to stderr for the windowless face and hands it to
/// the model for the windowed one, because a window whose every read and act
/// crosses this wire must paint the refusal, not open inert with one line on a
/// stream a desktop launch has nowhere to show.
pub fn listen(
    world: &Env,
    answerer: Arc<dyn server::Answerer>,
    presence: crate::registry::presence::Presence,
) -> Result<server::Listener, String> {
    let minted = provision::ensure(&material::dir(world));
    let material = match material::read(world, material::Role::Server) {
        Ok(Some(m)) => m,
        // Nothing readable at all is exactly a box whose mint failed —
        // `ensure` writes `address` on every success — so the mint's own words
        // are the refusal. The half-provisioned read (`Err`) speaks for itself.
        Ok(None) => return Err(minted.err().unwrap_or_default()),
        Err(e) => return Err(e),
    };
    // A mint failure that still left readable material is a warning, not a
    // refusal: the wire the box already had is the wire it keeps.
    if let Err(e) = minted {
        eprintln!("yog: wire: {e}");
    }
    server::Listener::bind(&material, answerer, presence)
}

#[cfg(test)]
mod tests;
