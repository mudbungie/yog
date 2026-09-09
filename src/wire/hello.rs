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
//! **Two keys since bl-e598 made the number a MAJOR, and only one of them is
//! adjudicated** (REMOTE §3.2). `protocol` is the major, and the refusal above
//! is still strict equality on it. `edition` beside it is the corpus this
//! build was generated from — the newest stamp in `corpus/shapes.json`,
//! compiled in by `build.rs` as [`EDITION`] — and it decides nothing: an
//! addition ships without a bump, so two ends of one major routinely differ
//! here and are meant to. What it buys is that the difference is *sayable*. A
//! seat that vendors the ledger reads the engine's edition and knows which of
//! two things an absent field is: a fact the engine chose not to state, or a
//! field this engine cannot spell at all — which it then renders as exactly
//! that, rather than as the reassuring default. A peer that states no edition
//! is read as speaking the [`FLOOR`], because every engine of this major
//! writes every path stamped at or below it; that is the right reading of
//! every build that predates the key.
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

use serde_json::{Value, json};

use super::frame;

/// The wire version this build speaks, and the changelog of every bump —
/// its own file at §12's cap (bl-94a5), because the number is one line and
/// the reasoning behind each move of it is the rest: `hello` is the preface
/// EXCHANGE, `version` is what the preface states and why it ever changed.
/// Since bl-9ced it also holds the newest version yog has PUBLISHED, which no
/// preface ever states — the corpus ledger's gate is its one reader.
mod version;
pub use version::{EDITION, FLOOR, PROTOCOL, PROTOCOL_PUBLISHED};

/// The preface's first key: the MAJOR, and the whole of what is decided on.
const KEY: &str = "protocol";

/// Its second: the corpus **edition** this build was generated from (REMOTE
/// §3.2, bl-1be7). It is stated and never adjudicated — an edition can only
/// ever differ *within* an agreed major, which is the point of the split, so a
/// peer is admitted or refused on [`KEY`] alone and the edition is what it is
/// then allowed to say about itself.
const EDITION_KEY: &str = "edition";

/// What a peer that stated no version is called in the sentence. An
/// unversioned build, a peer that hung up mid-preface and noise are one case
/// on purpose: none of them can be served, and telling them apart would be
/// three sentences for one outcome.
const UNSTATED: &str = "no version";

/// Write this build's preface. Called before either end reads, which is what
/// makes the exchange deadlock-free without an ordering rule to remember.
pub(crate) fn state(w: &mut dyn Write) -> io::Result<()> {
    frame::write_value(w, &json!({ KEY: PROTOCOL, EDITION_KEY: EDITION }))
}

/// What the peer stated about itself: the major, and the edition beside it.
struct Preface {
    /// The major, or `None` when it stated none — a frame that never arrived,
    /// a frame that is not an object, and an object without the key collapsing
    /// to the one answer a reader can act on.
    protocol: Option<u64>,
    /// The corpus edition. **Absent reads as [`FLOOR`]**, and so does anything
    /// that is not a stamp: every engine of this major writes every path
    /// stamped at or below the floor, so the floor is what a peer that says
    /// nothing has already promised. It is the reading that is right for a
    /// build older than bl-1be7, which is every build there is.
    edition: u32,
}

/// Read one preface frame.
fn stated(r: &mut dyn Read) -> Preface {
    let frame = frame::read_value(r).ok().flatten().unwrap_or(Value::Null);
    Preface {
        protocol: frame.get(KEY).and_then(Value::as_u64),
        edition: frame
            .get(EDITION_KEY)
            .and_then(Value::as_u64)
            .and_then(|stamp| u32::try_from(stamp).ok())
            .unwrap_or(FLOOR),
    }
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
/// `None` is the whole of the refusal — the caller drops the connection and
/// never decodes a frame of another protocol, so no gesture of a version this
/// build does not speak is ever adjudicated. A refusal that could not be
/// written (a peer already gone) changes nothing: the answer is the same.
///
/// An admitted peer is handed back **its edition** (REMOTE §3.2), which the
/// caller keeps on the connection's presence entry. The decision itself is
/// still [`KEY`] alone: an edition never refuses anybody, because the whole
/// point of splitting it off the major is that a difference in it is one two
/// ends are meant to survive.
pub(crate) fn admit<S: Read + Write>(s: &mut S) -> Option<u32> {
    if state(s).is_err() {
        return None;
    }
    let peer = stated(s);
    if agreed(peer.protocol) {
        return Some(peer.edition);
    }
    // The sentence is bound rather than nested, so this arm is one statement
    // per line: a call rustfmt wraps gets a region per continuation, and the
    // inner one reads uncovered on a path the suite does exercise.
    let said = mismatch(peer.protocol);
    let _ = frame::write_value(s, &crate::boundary::reply::refusal(&said));
    let _ = frame::write_end(s);
    None
}

/// **The seat's half**: read the engine's preface and refuse a mismatch to the
/// caller, as one `Err(String)`.
///
/// `cfg(test)` since bl-7942: no seat ships in this crate any more, and the one
/// client left in the tree is the suite's own
/// ([`test_support::wire::Seat`](crate::test_support::wire)), which has to
/// speak the whole protocol or it would prove the listener against a dialect.
/// The *rule* it implements is still the server's — [`stated`] writes the same
/// preface — so it belongs beside it rather than in the harness.
#[cfg(test)]
pub(crate) fn confirm(r: &mut dyn Read) -> Result<(), String> {
    let peer = stated(r);
    if agreed(peer.protocol) {
        return Ok(());
    }
    Err(mismatch(peer.protocol))
}

#[cfg(test)]
mod tests;
