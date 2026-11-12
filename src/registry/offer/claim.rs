//! **The claim** (REMOTE §15.1 step 2; bl-9ea6): what the registration door
//! does with one request — a code and a public key in, a signed leaf out.
//!
//! **This is not a gesture and is reached by none.** The caller holds no
//! certificate, so it has no identity, no grade and no scope; it never meets
//! the codec, `dispatch` or a [`Reply`](crate::boundary::reply::Reply). What it can
//! do is spend one code, once, and what that buys is a certificate that is
//! **held** ([`store::held`]) until an operator approves it.
//!
//! **Every refusal is `None`, and they are one answer.** An unknown code, an
//! expired one, one already claimed, a body that is not the one JSON object, a
//! key that is not P-256: the door says the same bare `404` to all of them,
//! because which it was is not said to a stranger.

use super::store::{self, Record};
use crate::wire::{material, provision};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, SubjectPublicKeyInfoDer};
use serde_json::{Value, json};
use std::path::Path;

/// The trail's word for a claim (§4.2).
const STEP: &str = "claim";

/// A P-256 public key's `SubjectPublicKeyInfo`, up to the point: the
/// `id-ecPublicKey` algorithm with the `prime256v1` curve, a 66-byte bit
/// string, and the `04` that says the point is uncompressed. The 64 bytes
/// after it are the point; whether they are ON the curve is `openssl`'s to
/// refuse at the signature.
pub(crate) const P256: [u8; 27] = [
    0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, 0x06, 0x08, 0x2a,
    0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, 0x03, 0x42, 0x00, 0x04,
];
/// The whole key's length: the prefix and the 64-byte point.
const P256_LEN: usize = 91;

/// Spend `code` for the key in `body`, at `now`. `Some` is the JSON object the
/// device is answered with — the workspace its entry is filed under, the name
/// and grade as offered, the address list, the CA and the leaf just signed.
pub fn claim(state_root: &Path, wire: &Path, now: i64, code: &str, body: &[u8]) -> Option<Value> {
    let public = public_key(body)?;
    let record = store::take(state_root, code, now)?;
    let answer = issue(state_root, wire, now, &record, &public);
    if answer.is_none() {
        store::restore(state_root, &record);
    }
    answer
}

/// The device's public key out of the request body, as DER — `None` unless the
/// body is `{"public_key": "<PEM>"}` and the key is P-256. The DER read here
/// is the DER signed and the DER the word is derived from: one parse.
fn public_key(body: &[u8]) -> Option<Vec<u8>> {
    let body: Value = serde_json::from_slice(body).ok()?;
    let pem = body.get("public_key")?.as_str()?;
    let der = SubjectPublicKeyInfoDer::from_pem_slice(pem.as_bytes()).ok()?;
    (der.len() == P256_LEN && der.starts_with(&P256)).then(|| der.to_vec())
}

/// Sign, read back, record the claim, and assemble the answer.
fn issue(
    state_root: &Path,
    wire: &Path,
    now: i64,
    record: &Record,
    public: &[u8],
) -> Option<Value> {
    provision::sign(wire, &record.name, record.grade, public).ok()?;
    let ca = std::fs::read_to_string(wire.join(material::ANCHORS)).ok()?;
    let cert = std::fs::read_to_string(wire.join(format!("{}.pem", record.name))).ok()?;
    let anchor = CertificateDer::from_pem_slice(ca.as_bytes()).ok()?;
    let claimed = Record {
        word: Some(super::word(public, &anchor)),
        claimed: Some(now),
        ..record.clone()
    };
    store::settle(state_root, &claimed).ok()?;
    let _ = crate::actions::verbs::log_step_done(
        state_root,
        &now.to_string(),
        wire,
        STEP,
        crate::opslog::Origin::World,
        crate::registry::Client::parse(&record.name).unwrap_or_default(),
    );
    Some(json!({
        "workspace": record.workspace, "name": record.name, "grade": record.grade.word(),
        "address": material::addresses(wire), "ca": ca, "cert": cert,
    }))
}

#[cfg(test)]
mod tests;
