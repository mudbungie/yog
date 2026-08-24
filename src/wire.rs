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

/// The window's off-frame asker (REMOTE §1.2, bl-ae05) — the thread that makes
/// the local window a wire client of its own engine.
pub mod asker;
/// **One channel the window holds** (REMOTE §8.2, bl-028a): the roster slice it
/// feeds, and the one place its leaf↔host-name mapping is spent on the read
/// path — [`seat::channel`]'s discipline, read from the frame.
pub mod channel;
/// **The window's channel set and the union over it** (REMOTE §8.2, bl-028a):
/// the roster composed across the channels, and the name resolution that
/// refuses a collision.
pub mod channels;
pub mod client;
/// **Every channel a window's off-frame thread dials** (REMOTE §8.2, bl-670c):
/// [`channels`] seen from the other end of the same links — one seat per
/// channel, the routing that picks which, and the mapping spent on the way.
pub mod dial;
/// The client-side workspaces this box holds elsewhere (REMOTE §8.2, bl-aaec)
/// — [`material`]'s shape one level down and named, one directory per
/// workspace hosted on another box.
pub mod entries;
pub mod frame;
/// The tool-host client mode (REMOTE §5, bl-024b) — the wire's second shipped
/// client: it advertises what this machine can run, rides a follow-class read
/// for its next invocation, and posts each capture back.
pub mod host;
pub mod intake;
/// The window's **second** asker lane (REMOTE §3, §10; bl-73e7): one held
/// connection on the focused conversation's live tail, so the serial pass below
/// is never stalled by a read that deliberately never finishes.
pub mod lane;
/// The frame's half of that read path: the standing questions and what landed.
pub mod link;
pub mod material;
/// The frame's half of the **act** path (REMOTE §9.8, bl-4841): what the window
/// has sent, and the minted ticket its receipt lands under.
pub mod post;
/// The window's off-frame poster (bl-4841) — the asker's twin on the write
/// side, pushed rather than polled and on a thread of its own.
pub mod poster;
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
