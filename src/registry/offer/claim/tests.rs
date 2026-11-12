//! The claim: one code, one public key, one signed leaf — and one answer for
//! every way it can be refused.

use super::*;
use crate::registry::offer::store::Record;
use crate::registry::{Client, Grade, leaf};
use crate::test_support::wire::Device;
use tempfile::{TempDir, tempdir};

const CODE: &str = "ABCDEFGH";

/// A minted wire directory stated at a host and a LAN address, a state root,
/// and one open offer under `grade`.
fn offered(grade: Grade) -> (TempDir, std::path::PathBuf, std::path::PathBuf) {
    let tmp = tempdir().expect("tmp");
    let (wire, root) = (tmp.path().join("wire"), tmp.path().join("state"));
    provision::mint(
        &wire,
        "engine.invalid:7737",
        &["192.0.2.7".to_owned()],
        false,
    )
    .expect("the operator's own mint");
    store::mint(
        &root,
        &Record {
            name: "phone".to_owned(),
            workspace: "home".to_owned(),
            grade,
            code: CODE.to_owned(),
            expires: 600,
            word: None,
            claimed: None,
            declined: false,
        },
    )
    .expect("an open offer");
    (tmp, wire, root)
}

/// The claim, whole: the answer's six fields, a leaf that binds the device's
/// own key under the offered name and grade, the word recorded, the hold
/// taken, and no private key anywhere on this box.
#[test]
fn a_claim_answers_a_leaf_signed_for_the_devices_own_key() {
    for grade in [Grade::Operator, Grade::Foot] {
        let (_tmp, wire, root) = offered(grade);
        let device = Device::new();
        let answer = claim(&root, &wire, 5, CODE, &device.body()).expect("claimed");

        assert_eq!(answer["workspace"], "home");
        assert_eq!(answer["name"], "phone");
        assert_eq!(answer["grade"], grade.word());
        assert_eq!(
            answer["address"],
            json!(["engine.invalid:7737", "192.0.2.7:7737"]),
            "the whole list a client dials, in order (REMOTE §15.3)"
        );
        let ca = answer["ca"].as_str().expect("ca");
        let cert = answer["cert"].as_str().expect("cert");
        let der = CertificateDer::from_pem_slice(cert.as_bytes()).expect("a certificate");
        assert_eq!(leaf::common_name(&der).as_deref(), Some("phone"));
        assert_eq!(leaf::grade(&der), grade);
        assert!(
            der.windows(device.public.len()).any(|w| w == device.public),
            "the leaf binds the key the device made"
        );
        assert_eq!(answer.as_object().expect("object").len(), 6, "and no key");
        assert!(
            !wire.join("phone.key").exists(),
            "no private key was ever here"
        );
        assert!(!wire.join("phone.pub").exists(), "and the scratch is gone");

        let anchor = CertificateDer::from_pem_slice(ca.as_bytes()).expect("ca");
        let held = store::pending(&root, 5);
        assert_eq!(
            held[0].word.as_deref(),
            Some(crate::registry::offer::word(&device.public, &anchor).as_str())
        );
        assert_eq!(held[0].claimed, Some(5));
        assert!(store::held(
            &root,
            &Client::parse("phone").expect("identity")
        ));
        assert_eq!(
            claim(&root, &wire, 6, CODE, &Device::new().body()),
            None,
            "a code is single-use"
        );
    }
}

/// Every body that is not `{"public_key": "<P-256 PEM>"}` is the same refusal,
/// and none of them spends the code.
#[test]
fn a_body_that_is_not_a_p256_public_key_claims_nothing() {
    let (_tmp, wire, root) = offered(Grade::Operator);
    let pem = |der: &[u8]| json!({ "public_key": provision::pem::public_key(der) }).to_string();
    let short = pem(&P256);
    let other_curve = pem(&[0x30u8; P256_LEN]);
    for body in [
        "not json",
        "{}",
        r#"{"public_key": 7}"#,
        r#"{"public_key": "not pem"}"#,
        short.as_str(),
        other_curve.as_str(),
    ] {
        assert_eq!(
            claim(&root, &wire, 5, CODE, body.as_bytes()),
            None,
            "{body}"
        );
    }
    assert!(
        claim(&root, &wire, 5, CODE, &Device::new().body()).is_some(),
        "the code was never spent"
    );
}

/// A key `openssl` will not sign for — the right shape, a point on no curve —
/// puts the offer back, so the device's next request can still claim it.
#[test]
fn a_key_that_cannot_be_signed_for_restores_the_offer() {
    let (_tmp, wire, root) = offered(Grade::Operator);
    let mut off_curve = P256.to_vec();
    off_curve.extend([0xffu8; 64]);
    let body = json!({ "public_key": provision::pem::public_key(&off_curve) }).to_string();
    assert_eq!(claim(&root, &wire, 5, CODE, body.as_bytes()), None);
    assert!(!store::held(
        &root,
        &Client::parse("phone").expect("identity")
    ));
    assert!(!wire.join("phone.pem").exists(), "nothing was issued");
    assert!(
        !wire.join("phone.key").exists(),
        "and no scratch key is left"
    );
    assert!(claim(&root, &wire, 5, CODE, &Device::new().body()).is_some());
}

/// An unknown code and an expired one answer what a spent one answers.
#[test]
fn an_unknown_or_expired_code_claims_nothing() {
    let (_tmp, wire, root) = offered(Grade::Operator);
    let body = Device::new().body();
    assert_eq!(claim(&root, &wire, 5, "ZZZZZZZZ", &body), None);
    assert_eq!(claim(&root, &wire, 600, CODE, &body), None);
}
