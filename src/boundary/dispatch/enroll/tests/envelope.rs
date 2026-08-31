//! **The QR envelope, measured** (REMOTE §1.4 as amended, bl-f4e3): the
//! payload contract a seat draws and a device scans, sized against what a QR
//! code actually carries — with the certificates this box's own recipe mints,
//! not with an estimate.
//!
//! This is the test that decided the encoding. PEM rides **verbatim** because
//! the measurement below says it fits with room to spare; DER-plus-base64 was
//! the fallback and is not needed, and re-encoding a field the operator can
//! also read with `openssl x509 -text` would have bought ~13% for a legibility
//! nobody should have to pay for. It is a regression guard as much as a
//! record: a mint that moved to RSA would fail here rather than in a
//! photograph.

use super::super::*;
use super::{enrolled, provisioned, request};
use crate::registry::Grade;
use serde_json::{Value, json};
use tempfile::tempdir;

/// **Byte-mode capacity of a version-40 QR code, at each error-correction
/// level** (ISO/IEC 18004). L is the largest a scanner will read and H the most
/// damage-tolerant; the envelope is measured against all four, because which
/// one a seat picks is the seat's decision and this is the fact it needs.
const CAPACITY: [(&str, usize); 4] = [("L", 2953), ("M", 2331), ("Q", 1663), ("H", 1273)];

/// The envelope, per REMOTE §1.4: the reply's six fields under a version
/// marker, compact — no spaces, because every one of them is a byte a scanner
/// has to carry. `ok` and `kind` do not travel: they say what a *wire answer*
/// is, and a photograph is not one.
fn envelope(enrolled: &Enrolled) -> String {
    let value: Value = json!({
        "yog-enroll": 1,
        "grade": enrolled.grade.word(),
        "name": enrolled.name,
        "address": enrolled.address,
        "ca": enrolled.ca,
        "cert": enrolled.cert,
        "key": enrolled.key,
    });
    value.to_string()
}

/// The measurement REMOTE records. A foot leaf is the larger of the two — its
/// subject carries the extra organizational unit — so it is the one measured.
#[test]
fn the_envelope_fits_a_version_40_qr_code() {
    let tmp = tempdir().expect("tmp");
    let (deps, _) = provisioned(&tmp);
    let answer = enrolled(enroll(&deps, "7", &request("phone-1", Grade::Foot)).expect("act"));
    let size = envelope(&answer).len();

    // The recorded figure. P-256 keys and 825-day leaves are what
    // `wire::provision` mints, so this is stable to within the few bytes a
    // DER serial and a name length move it — the bound, not the equality, is
    // what REMOTE states.
    assert!(
        (1400..1700).contains(&size),
        "the envelope measured {size} bytes; REMOTE §1.4 records ~1.6 kB, so the encoding \
         ruling needs re-taking"
    );
    for (level, capacity) in CAPACITY {
        assert_eq!(
            size <= capacity,
            level != "H",
            "level {level} carries {capacity} bytes and the envelope is {size}"
        );
    }
}

/// Every field is present and every one of them is needed: a device handed five
/// of the six cannot dial, cannot verify, or cannot prove who it is.
#[test]
fn the_envelope_carries_exactly_the_six_facts_and_a_version() {
    let tmp = tempdir().expect("tmp");
    let (deps, _) = provisioned(&tmp);
    let answer = enrolled(enroll(&deps, "7", &request("phone-1", Grade::Operator)).expect("act"));
    let value: Value = serde_json::from_str(&envelope(&answer)).expect("compact JSON");
    let object = value.as_object().expect("an object");

    assert_eq!(object.get("yog-enroll"), Some(&json!(1)), "the marker");
    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "address",
            "ca",
            "cert",
            "grade",
            "key",
            "name",
            "yog-enroll"
        ]
    );
    // PEM verbatim: the newlines survive, which is the one thing a naive
    // scanner-side decoder gets wrong.
    assert!(answer.ca.contains("-----BEGIN CERTIFICATE-----\n"));
    assert!(envelope(&answer).contains("-----BEGIN CERTIFICATE-----\\n"));
}
