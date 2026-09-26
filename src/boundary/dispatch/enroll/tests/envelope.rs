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
use super::{deps, enrolled, provisioned, request};
use crate::registry::Grade;
use crate::wire::rendezvous::material::{PUBLIC, SALT};
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
    let mut value: Value = json!({
        "yog-enroll": 1,
        "grade": enrolled.grade.word(),
        "name": enrolled.name,
        "address": enrolled.address,
        "ca": enrolled.ca,
        "cert": enrolled.cert,
        "key": enrolled.key,
    });
    if let (Some(object), Some(handoff)) = (value.as_object_mut(), &enrolled.rendezvous) {
        object.insert("rendezvous_pub".into(), json!(handoff.public));
        object.insert("pairing_salt".into(), json!(handoff.salt));
    }
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
    // what REMOTE states. A stated host carries the rendezvous pair too
    // (bl-9043), which is what takes it past Q.
    assert!(
        (1600..1850).contains(&size),
        "the envelope measured {size} bytes; REMOTE §8.4 records ~1.7 kB, so the encoding \
         ruling needs re-taking"
    );
    for (level, capacity) in CAPACITY {
        assert_eq!(
            size <= capacity,
            matches!(level, "L" | "M"),
            "level {level} carries {capacity} bytes and the envelope is {size}"
        );
    }
}

/// Every field is present and every one of them is needed: a device handed five
/// of the six cannot dial, cannot verify, or cannot prove who it is. A stated
/// host adds the rendezvous pair, exactly as its two files hold it (bl-9043).
#[test]
fn the_envelope_carries_exactly_the_six_facts_and_a_version() {
    let tmp = tempdir().expect("tmp");
    let (deps, dir) = provisioned(&tmp);
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
            "pairing_salt",
            "rendezvous_pub",
            "yog-enroll"
        ]
    );
    let file = |name: &str| read(&dir.join(name)).expect("minted").trim().to_owned();
    assert_eq!(object.get("rendezvous_pub"), Some(&json!(file(PUBLIC))));
    assert_eq!(object.get("pairing_salt"), Some(&json!(file(SALT))));
    // PEM verbatim: the newlines survive, which is the one thing a naive
    // scanner-side decoder gets wrong.
    assert!(answer.ca.contains("-----BEGIN CERTIFICATE-----\n"));
    assert!(envelope(&answer).contains("-----BEGIN CERTIFICATE-----\\n"));
}

/// **A loopback-only box hands off no rendezvous material** (bl-9043): it
/// minted none — nothing another machine dials it by — so the envelope omits
/// both keys, and still fits every level but H, as it always did.
#[test]
fn a_loopback_box_omits_the_rendezvous_pair() {
    let tmp = tempdir().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    provision::mint(&dir, "127.0.0.1:7737", &[], false).expect("mint");
    assert!(!dir.join(PUBLIC).exists() && !dir.join(SALT).exists());
    let deps = deps(&world, &tmp.path().join("state-root"));
    let answer = enrolled(enroll(&deps, "7", &request("phone-1", Grade::Foot)).expect("act"));
    assert_eq!(answer.rendezvous, None);
    let text = envelope(&answer);
    assert!(!text.contains("rendezvous_pub") && !text.contains("pairing_salt"));
    assert!(text.len() <= CAPACITY[2].1, "{} bytes fit Q", text.len());
}

/// Half a pairing is not a pairing — the reply refuses rather than read one
/// key without the other, and a box holding half refuses before it mints.
#[test]
fn half_the_pair_refuses_on_the_wire_and_on_disk() {
    let mut reply = serde_json::from_str::<Value>(
        r#"{"ok":true,"kind":"enrolled","grade":"foot","name":"n","address":"h:1","ca":"","cert":"","key":"","rendezvous_pub":"00"}"#,
    )
    .expect("json");
    let refusal = crate::boundary::reply::decode(&reply).expect_err("half");
    assert!(refusal.contains("travel together"), "{refusal}");
    reply["pairing_salt"] = json!("11");
    assert!(crate::boundary::reply::decode(&reply).is_ok());

    let tmp = tempdir().expect("tmp");
    let (deps, dir) = provisioned(&tmp);
    std::fs::remove_file(dir.join(SALT)).expect("rm");
    let refusal = enroll(&deps, "7", &request("phone-1", Grade::Foot)).expect_err("half");
    assert!(refusal.contains(SALT), "{refusal}");
    assert!(!dir.join("phone-1.pem").exists(), "nothing minted");
}
