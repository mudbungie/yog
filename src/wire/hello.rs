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
///
/// 4 → 5 (bl-e654): the `governing` reply's `branch` key — *the lineage whose
/// tip the frozen commit still is* — is gone, replaced by `follows` and
/// `diverged_lineages`, and the `oid` beside them stopped meaning the fork
/// commit and started meaning the commit control resolves from. Same verb,
/// same question, a different thing said: exactly the case above.
///
/// 5 → 6 (bl-23bd): `reply/providers` rows gained `effort` and `priority`, the
/// two per-row tuning capabilities a seat decides a control's existence by. The
/// **two new ops beside them cost nothing** — `/effort` and `/priority` are new
/// spellings in an existing vocabulary, and §3's rule is that a peer which has
/// not heard of one already refuses it in band by name. One bump for the row,
/// and nothing else shape-changing is batched behind it: clients re-vendor per
/// bump, and this number walked 2 → 5 inside one unreleased cycle already.
/// 6 → 7 (bl-8758): every `reply/help` row gained `surface`, the word saying
/// whether a seat-class client owes that op a control (`docs/PARITY.md` §2).
/// The ledger would have let it through — help's signature last moved at 1, so
/// its one free move at 6 was unspent — but §3's rule is the authority and is
/// stricter than the mechanism: any wire-visible shape change, *gained
/// included*, bumps the version, and the ledger cannot see what has shipped
/// (REMOTE §9.9's correction). It is also the bump that pays for itself: the
/// classification is the artifact clients vendor and judge themselves against,
/// so a client must re-vendor to read it, and a bump is exactly what makes it.
/// 7 → 8 (bl-66d4): `reply/advertised` gained `wrote`, the word saying whether
/// this engine WROTE the advertised set or found it identical and compared. It
/// is required rather than optional-absent-reads-false, because absent would
/// read as *"nothing was restored"* — the reassuring answer — on exactly the
/// build that cannot tell, and the field exists to make one event audible.
/// 8 → 9 (bl-015b): `reply/transcript` gained the `wounded` entry — the §8.5
/// settled-failure notice, the third virtual entry — and `reply/steps` LOST
/// `auth_failed`, the §8.3 affordance now being the `refused` arm of the wound
/// vocabulary both shapes spell. A gain and a loss on two shapes, which is
/// two of the four things §3 says bump the version; the ledger's one free move
/// would have covered neither, since it cannot see what has shipped.
/// 9 → 10 (bl-09aa): **no field moved, and that is why this bump is the rule
/// rather than an exception to it.** `attention` became follow-class (REMOTE
/// §14.1): the same ask, the same reply shape, but a *sequence* — the first
/// frame at connect, a further frame whenever the answer changes, the
/// terminator when the hold ends. That is precisely "what a spelling already in
/// use is taken to say", and it is the one class of change the corpus ledger
/// cannot see, since frame count is not a field signature. A seat built against
/// 9 would read the first frame and then wait on a terminator up to a hold
/// away; strict equality here is what turns that into an upgrade sentence.
/// 10 → 11 (bl-4d81): `reply/ops` gained the three readings a §7.3 failure
/// banner is made of — `failed`, `exit_label` and `standing` — so the row
/// answers what it *is* and not only what was logged. A field gained on a shape
/// already in use, which §3's rule bumps outright; and the bump is the point
/// rather than a tax, since the whole gain is a classification that reaches a
/// client only through a re-vendor.
pub const PROTOCOL: u32 = 11;

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
    if agreed(peer) {
        return Ok(());
    }
    Err(mismatch(peer))
}

#[cfg(test)]
mod tests;
