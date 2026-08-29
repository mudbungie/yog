//! **The version preface** (REMOTE §3, bl-a670): the first frame each end
//! writes on a connection, carrying the one fact it states about itself.
//!
//! Until the four-component split (REMOTE §12) one crate shipped both ends of
//! every connection, so the wire could not skew and needed no version. Four
//! separately installed components can skew, so it carries one.
//!
//! **Three properties, each a refusal of something easier.**
//!
//! - **Both ends state, and either end may refuse.** Each writes its preface
//!   before it reads the peer's, so neither waits on the other and a mismatch
//!   is nameable from whichever side notices first. An engine refusing a seat
//!   and a seat refusing an engine are one rule with one sentence, not two
//!   halves that can disagree.
//! - **No negotiation.** No version list, no capability probe, no compat shim:
//!   a mismatch is fail-closed and the sentence — which names *both* versions —
//!   is the upgrade prompt. Negotiation is the mechanism that makes every later
//!   version carry every earlier one's shape forever, and it buys nothing here
//!   because the operator installs both ends.
//! - **The request frame is untouched.** The preface rides *beside* the gesture
//!   envelope, never inside it, so the frame the wire carries stays byte for
//!   byte the frame the `gestures/` inbox carries (REMOTE §3: the wire adds
//!   nothing to the boundary) and the codec gains no field.
//!
//! **ALPN was the alternative, and it cannot say this.** rustls will refuse a
//! handshake whose application protocol does not match, at no cost and with no
//! frame — but that refusal is a TLS alert, so neither end learns the other's
//! version and an operator reads a transport error where a sentence belongs.
//! Naming both versions is the requirement, so the preface is in band.
//!
//! **It costs no round trip.** Each end writes its preface and only then reads
//! the peer's, and the seat writes its request in the same breath as its
//! preface — so the check travels with bytes that were already going, and the
//! only connection it stops is one that was going to be refused anyway.

use std::io::{self, Read, Write};

use serde_json::json;

use super::frame;

/// The protocol this build speaks.
///
/// **One integer, and a new verb is not a bump.** A `Query`, an `Action` or a
/// reply kind the peer has not heard of already refuses in band, naming it
/// (REMOTE §3's strict decode) — which is the boundary correcting itself, not
/// two protocols meeting. This changes when the *existing* shape changes
/// meaning: the framing, the envelope, or what a spelling already in use is
/// taken to say.
pub const PROTOCOL: u32 = 1;

/// The preface's one key, and the whole of its shape.
const KEY: &str = "protocol";

/// What a peer that stated no version is called in the sentence. An
/// unversioned build, a peer that hung up mid-preface and noise are one case
/// on purpose: none of them can be served, and telling them apart would be
/// three sentences for one outcome.
const UNSTATED: &str = "no version";

/// Write this build's preface. Called before either end reads, which is what
/// makes the exchange deadlock-free without an ordering rule to remember.
pub(crate) fn state(w: &mut dyn Write) -> io::Result<()> {
    frame::write_value(w, &json!({ KEY: PROTOCOL }))
}

/// The version the peer stated, or `None` when it stated none — a frame that
/// never arrived, a frame that is not an object, and an object without the key
/// collapsing to the one answer a reader can act on.
fn stated(r: &mut dyn Read) -> Option<u64> {
    frame::read_value(r).ok().flatten()?.get(KEY)?.as_u64()
}

/// Whether what the peer stated is this build's own protocol.
fn agreed(peer: Option<u64>) -> bool {
    peer == Some(u64::from(PROTOCOL))
}

/// The refusal, said the same way at both ends: both versions, and what to do
/// about it. It is the upgrade prompt, so it names the remedy rather than
/// leaving an operator to infer one from a number.
fn mismatch(peer: Option<u64>) -> String {
    let peer = peer.map_or_else(|| UNSTATED.to_owned(), |v| v.to_string());
    format!(
        "wire protocol mismatch: this end speaks version {PROTOCOL}, \
         the peer speaks {peer}. There is no negotiation — \
         upgrade the older component until both speak one version."
    )
}

/// **The engine's half**: state, read, and either admit the peer or refuse it
/// in band on the connection it opened.
///
/// `false` is the whole of the refusal — the caller drops the connection and
/// never decodes a frame of another protocol, so no gesture of a version this
/// build does not speak is ever adjudicated. A refusal that could not be
/// written (a peer already gone) changes nothing: the answer is the same.
pub(crate) fn admit<S: Read + Write>(s: &mut S) -> bool {
    if state(s).is_err() {
        return false;
    }
    let peer = stated(s);
    if agreed(peer) {
        return true;
    }
    let _ = frame::write_value(s, &crate::boundary::reply::refusal(&mismatch(peer)));
    let _ = frame::write_end(s);
    false
}

/// **The seat's half**: read the engine's preface and refuse a mismatch to the
/// caller, as one `Err(String)` — which is where every other thing that can go
/// wrong with a transport already arrives (REMOTE §9.7), so a surface painting
/// the sentence carries no new case.
pub(crate) fn confirm(r: &mut dyn Read) -> Result<(), String> {
    let peer = stated(r);
    if agreed(peer) {
        return Ok(());
    }
    Err(mismatch(peer))
}

#[cfg(test)]
mod tests;
