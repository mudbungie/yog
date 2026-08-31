//! **Enrollment** (REMOTE §1.4 as amended, §4.2, §8.2; bl-f4e3): what an
//! operator asks for when a new device joins, and the whole of what the engine
//! answers with.
//!
//! **§1.4 is not lifted and this is not a pairing flow.** The device performs
//! no channel act — it has no certificate, so it cannot open a connection at
//! all. An **operator-grade seat** performs the act, over its own already
//! authenticated channel, and the material the answer carries then travels to
//! the device **out of channel**: a QR on a screen is a scp shaped like a
//! photograph, and the operator is standing in front of both machines. That is
//! the same class as the engine-boot mint (`wire::provision`, bl-ae05) — the
//! operator's own tooling, reached through the boundary because REMOTE §3
//! forbids a capability that exists on the wire and nowhere else.
//!
//! **The two values here are the vocabulary, not the act.** The executor is
//! [`boundary::dispatch::enroll`](crate::boundary::dispatch), beside
//! `advertise` and for its reason: everything else in the chokepoint routes,
//! and these gate. They live in the registry because the identity they mint
//! and the registration they seat are the registry's own two facts, and a
//! payload type with its own module is the fold [`mailbox::Verb`](super::mailbox::Verb)
//! and [`monitor::Verb`](crate::monitor::Verb) already take — one variant at
//! the boundary, one home for the doc.

use super::Grade;

/// What an enrollment asks for: a workspace to seat the new client in, the
/// common name its certificate will carry, and what that certificate may say.
///
/// **It addresses a workspace like any other gesture** (REMOTE §8). The act
/// creates the registration, and a registration is the pair
/// `(client, workspace)` — so an enrollment that named no workspace would mint
/// a certificate that authenticates and sees nothing, and the operator would
/// have to finish the job with a `touch`. One act, one pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The workspace the new client is registered in (§4.1) —
    /// `clients/<name>/workspaces/<workspace>`.
    pub workspace: String,
    /// The certificate's subject common name, which **is** the client identity
    /// (§2). Refused by [`Client::parse`](super::Client::parse) exactly as
    /// every other identity is: one path component, never [`LOCAL`](super::LOCAL).
    pub name: String,
    /// What that certificate may say (§4.2). Minted into the subject by the
    /// operator's own CA, which is the only thing entitled to write it.
    pub grade: Grade,
}

/// What one enrollment answers with — the whole of what a new device needs and
/// nothing else.
///
/// **The private key is here and nowhere else.** The engine mints the pair,
/// reads it, hands it over and **shreds the key** before the answer leaves;
/// what stays on disk is the certificate, which is public material and whose
/// presence is what refuses a second enrollment under the same name
/// ([`provision::issue`](crate::wire::provision)). Custody after that is the
/// transport's: over the wire the answer is TLS bytes and a seat's RAM (§6),
/// while a deposit through the `gestures/` inbox lands it in a reply file
/// inside the world — on the operator's own box, beside the CA that can mint
/// the same leaf again at will, so it discloses nothing to anyone who could
/// not already mint. It does *persist*, and the remedy is `rm`.
///
/// **The payload contract is REMOTE §1.4's**: the QR envelope is these six
/// fields under a `"yog-enroll": 1` marker, compact JSON, PEM verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enrolled {
    /// The grade minted into the subject (§4.2).
    pub grade: Grade,
    /// The common name the certificate carries — the device's identity.
    pub name: String,
    /// **The engine's own wire address, as clients dial it** (§8): the one the
    /// boot recorded in `wire/address`, never the port a `:0` request became.
    /// A runtime port changes on the next boot, so a QR carrying one would be
    /// stale before it was scanned.
    pub address: String,
    /// The operator CA both ends verify against, PEM.
    pub ca: String,
    /// This device's own certificate, PEM.
    pub cert: String,
    /// Its private key, PEM — held by nothing on this box once this value is
    /// built.
    pub key: String,
}
