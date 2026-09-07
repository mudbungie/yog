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
/// 11 → 12 (bl-09ef): every queue row — `reply/attention` and the
/// `reply/acknowledged` remainder that spells rows the same way — gained
/// `says`, the firing rules **in words**. The escalation those words existed
/// for was ruled a **seat's** act (DESIGN §6: a desktop notification belongs on
/// the box the operator is looking at), and `AttentionKind::says` stays this
/// engine's one home for the sentence, so the sentence has to cross rather than
/// be re-worded per seat. A gain on two shapes, which the ledger sees and §3
/// bumps for regardless.
/// 12 → 13 (bl-dc3f): `reply/config` gained `settings` — the file's own schema
/// applied to the text answered beside it (§9.5), so a config destination is
/// read as the typed thing it is and not as bytes a seat must parse. A field
/// gained on a shape in use, which §3's rule bumps outright; **the ledger could
/// not have caught it**, since `reply/config`'s signature had stood at 1 and
/// its one free move would have covered exactly this — REMOTE §9.15's
/// correction, applied again. It does **not** batch onto 12 the way §9.15
/// batched two reads: 12 landed on `main` ahead of this, and whether a release
/// captures it before this lands is a race no reader could resolve later —
/// batching is the exception, and an exception taken against a running release
/// train is how a version comes to mean two things.
/// 13 → 14 (bl-fec6, bl-d542): two additive fields on two shapes in use, which
/// §3's rule bumps for outright. `request/enroll` gained an optional
/// **`address`** — the endpoint the DEVICE being enrolled will dial, which need
/// not be the one this engine wrote for itself (an emulator reaches its host
/// through the emulator's own alias, a phone on the LAN, an overlay peer by
/// name), so the envelope stopped being correct only for a device sharing this
/// box's loopback view. And `reply/clients` gained an optional **`last_seen`**,
/// the unix second a client last spoke: `present` alone could not tell a
/// machine that spoke ten seconds ago from one that never once connected, and
/// on a terminal seat it reads `false` for everything. Both are ABSENT rather
/// than null when unstated — the absence is the fact in each case ("this
/// engine's own address", "never") — and both land at ONE version, because two
/// bumps a minute apart would make every client re-pin twice for one wave.
/// 14 → 15 (bl-5305): `reply/follow` gained `tools`, the **tool window** — an
/// entry as a call is posted (its name, which for a routed call carries the
/// machine, and its bounded input) and another when the capture lands (its exit
/// code). The lane carried the model's prose alone, so an operator watching an
/// agent administer their boxes saw a thinking marker and two sentences while
/// eight commands ran on two machines. A field gained on a shape in use, which
/// §3's rule bumps outright; and the field is **required rather than
/// optional-absent-reads-empty**, on `reply/advertised`'s own precedent at 8 —
/// absent would read as *nothing ran*, the reassuring answer, on exactly the
/// build that cannot tell.
/// 15 → 16 (bl-e59e): `reply/ops` rows gained `client` — the identity that made
/// the act: a connection's certificate common name, or `local` for the window,
/// the `gestures/` inbox, `yog gesture` and yog's own loops. Round-1 ruling 5
/// (*identity is the leaf*) landing on the durable half: §4.1 already makes the
/// leaf's common name the identity, §4 narrows the published derivation by it
/// and §5's roster renders it, and then the trail dropped it at the one place a
/// person later has to reconstruct what happened. It is the one fact on the row
/// that **cannot** be recovered afterwards — presence is a point-in-time
/// observation by design, so nothing later can say who a row belonged to. A
/// field gained on a shape in use, which §3's rule bumps outright.
/// 16 → 17 (bl-ebef, bl-6661): **two shapes, one version**, because both are
/// the same litany 0.0.11 pin landing and two bumps a minute apart would make
/// every client re-pin twice for one wave (14's own reasoning).
///
/// The §6 signal vocabulary gained **`truncated`** — rule
/// 2's rest said in the word that is true of it when the turn was cut off at
/// the request's output cap, the second refinement beside `refused` and
/// standing where `stopped` would, never beside it. litany 0.0.11 makes that
/// cut a named failure (`Error::OutputTruncated`, upstream bl-155f / bl-ecf9):
/// the staging sink is never sealed, so nothing is committed and no tool call
/// runs — while the seat read *"came to rest — your turn"*, the sentence the
/// upstream ball measured against nine `apply_patch` calls arriving as
/// `input: {}`. **No field moved and the ledger cannot see this one**: a
/// signature is field shapes, and this is a new VALUE in `signals`, which a
/// strict decoder built against 16 refuses by name. That is §3's rule reaching
/// past the mechanism for the third time (REMOTE §9.9, §9.15), and it is why
/// the number and not the ledger is the authority. It does **not** batch onto
/// 16: that landed on `main` ahead of this, and whether a release captures it
/// first is a race no reader could resolve later.
///
/// And a delivered row — on `reply/transcript` and on `reply/inbox`'s deposit
/// envelope alike — gained an optional **`sender_name`** / **`from_name`**
/// (bl-6661, litany 0.0.11 upstream bl-a457): the sender's display name, present
/// exactly when the sender is an agent wearing one. The framing sender is the
/// FILENAME's origin token — the addressing key litany's own inbox scan derives
/// from, and always will be — so every message a child sent was attributed by
/// sixty characters of timestamped hex, which on a phone is the whole row
/// header. ABSENT rather than null when there is no name (`user` never wears
/// one, an unnamed agent never does), the absence being the fact, on
/// `reply/enroll`'s `address` precedent at 14. The id keeps riding beside it and
/// is not replaced: it is the durable handle once the agent is deleted and the
/// name recycled.
/// 17 → 18: **the shapes below, one version.** Three lanes raised the wire in
/// one night, and one number carries all of them on 14's and 17's own
/// reasoning said a third time — two bumps a minute apart make every client
/// re-pin twice for one wave — with a second argument this release now has:
/// under bl-bca2's gate a raise HOLDS the release until three consumer mains
/// vendor it, so each extra number is another window in which no published
/// suite composes. **A later lane landing on this version adds its shape to
/// this entry rather than taking 19.**
///
/// **`reply/follow`'s tool-window entry gained `held`** (bl-58bb) —
/// the capability control's reason for parking the call, beside the `tool_use`
/// id and the tool name the entry already carried. The window's two entries
/// come off the pair of files litany lands, and a held invocation is parked
/// *before* the executor is entered, so it lands neither: the one lane an
/// operator has open while a command they did not expect is about to run on
/// their server reported the conversation as at rest and ended the stream, at
/// the exact moment the operator was the thing it was waiting for. On a foot
/// lane every call to a non-shell tool is held, so this was most of the
/// conversation. A field gained on a shape in use, which §3's rule bumps
/// outright; its presence is the status, the discipline `exit_code` already
/// carries on the same entry.
pub const PROTOCOL: u32 = 18;

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
