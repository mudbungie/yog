//! The certificate → identity walk, over one **real** certificate and over
//! hand-built DER for every shape a real one cannot exhibit.
//!
//! The real certificate is the load-bearing test: it is minted by the same
//! `openssl` the operator runs (`make wire-certs`), signed by a CA whose own
//! common name is different, so it proves both that the walk finds the subject
//! and that it does not find the **issuer** — the one failure a byte search for
//! the CN object identifier would have every time.

use super::*;
use crate::test_support::wire::mint;
use rustls::pki_types::CertificateDer;
use rustls::pki_types::pem::PemObject;
use tempfile::TempDir;

/// SEQUENCE.
const SEQ: u8 = 0x30;
/// SET.
const SET: u8 = 0x31;
/// UTF8String.
const UTF8: u8 = 0x0C;

/// One DER type-length-value, short or long form as the body demands.
fn tlv_of(tag: u8, body: &[u8]) -> Vec<u8> {
    let mut out = vec![tag];
    if body.len() < 0x80 {
        out.push(u8::try_from(body.len()).unwrap_or(0));
    } else {
        let len = u32::try_from(body.len()).unwrap_or(0).to_be_bytes();
        out.push(0x84);
        out.extend_from_slice(&len);
    }
    out.extend_from_slice(body);
    out
}

/// Several encoded values concatenated — a constructed value's body.
fn cat(parts: &[Vec<u8>]) -> Vec<u8> {
    parts.concat()
}

/// One `AttributeTypeAndValue` inside its own RDN SET.
fn rdn(oid: &[u8], tag: u8, value: &[u8]) -> Vec<u8> {
    tlv_of(
        SET,
        &tlv_of(SEQ, &cat(&[tlv_of(OID, oid), tlv_of(tag, value)])),
    )
}

/// A certificate skeleton carrying `subject` where a real one carries the
/// subject `Name`: `[0] version`, serial, signature, issuer, validity, subject.
fn certificate(subject: Vec<u8>) -> Vec<u8> {
    let tbs = tlv_of(
        SEQ,
        &cat(&[
            tlv_of(0xA0, &tlv_of(INTEGER, &[2])),
            tlv_of(INTEGER, &[1]),
            tlv_of(SEQ, &[]),
            tlv_of(SEQ, &rdn(&COMMON_NAME, UTF8, b"the-issuer")),
            tlv_of(SEQ, &[]),
            subject,
        ]),
    );
    tlv_of(SEQ, &cat(&[tbs, tlv_of(SEQ, &[]), tlv_of(0x03, &[0])]))
}

/// The whole point: a genuine `openssl`-minted leaf names its own subject, and
/// the CA that signed it — whose common name comes FIRST in the bytes — is not
/// what comes back.
#[test]
fn a_real_leaf_names_its_own_subject_and_not_its_issuer() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let der = CertificateDer::from_pem_file(tmp.path().join("client.pem")).expect("pem");
    assert_eq!(common_name(&der).as_deref(), Some("yog-client"));
    let ca = CertificateDer::from_pem_file(tmp.path().join("ca.pem")).expect("pem");
    assert_eq!(common_name(&ca).as_deref(), Some("yog-ca"));
    let server = CertificateDer::from_pem_file(tmp.path().join("server.pem")).expect("pem");
    assert_eq!(common_name(&server).as_deref(), Some("yog-server"));
    // The window's leaf is the identity the registry seats (REMOTE §4.1 as
    // narrowed, bl-ae05): one const, spent by the mint and by the seating.
    let window = CertificateDer::from_pem_file(tmp.path().join("window.pem")).expect("pem");
    assert_eq!(
        common_name(&window).as_deref(),
        Some(crate::registry::WINDOW)
    );
}

/// A certificate long enough to need the long-form length — every real one is
/// — reads exactly as a short-form one does.
#[test]
fn a_long_form_length_reads_like_a_short_one() {
    let mut name = b"x".repeat(200);
    name.extend_from_slice(b"-tail");
    let der = certificate(tlv_of(SEQ, &rdn(&COMMON_NAME, UTF8, &name)));
    assert_eq!(
        common_name(&der),
        Some(String::from_utf8(name).expect("utf8"))
    );
}

/// A multi-valued subject: the LAST common name is the leaf's own, and an
/// attribute that is not a common name is passed over.
#[test]
fn the_last_common_name_wins_and_other_attributes_are_passed_over() {
    let organisation = [0x55, 0x04, 0x0A];
    let der = certificate(tlv_of(
        SEQ,
        &cat(&[
            rdn(&organisation, UTF8, b"an-org"),
            rdn(&COMMON_NAME, UTF8, b"first"),
            rdn(&COMMON_NAME, UTF8, b"last"),
        ]),
    ));
    assert_eq!(common_name(&der).as_deref(), Some("last"));
}

/// A subject with no common name at all names nobody.
#[test]
fn a_subject_without_a_common_name_names_nobody() {
    let organisation = [0x55, 0x04, 0x0A];
    let der = certificate(tlv_of(SEQ, &rdn(&organisation, UTF8, b"an-org")));
    assert_eq!(common_name(&der), None);
}

/// An attribute whose first element is not an object identifier, and one with
/// no value at all: both are skipped rather than read as a name.
#[test]
fn a_malformed_attribute_is_skipped() {
    let lone = tlv_of(SET, &tlv_of(SEQ, &tlv_of(OID, &COMMON_NAME)));
    assert_eq!(common_name(&certificate(tlv_of(SEQ, &lone))), None);
    let mistyped = tlv_of(
        SET,
        &tlv_of(
            SEQ,
            &cat(&[tlv_of(UTF8, &COMMON_NAME), tlv_of(UTF8, b"nope")]),
        ),
    );
    assert_eq!(common_name(&certificate(tlv_of(SEQ, &mistyped))), None);
}

/// A common name that is not UTF-8 — a `BMPString` is UTF-16 — is skipped
/// rather than mis-read into a mojibake identity.
#[test]
fn a_name_that_is_not_utf8_is_skipped() {
    let der = certificate(tlv_of(SEQ, &rdn(&COMMON_NAME, 0x1E, &[0xFF, 0xFE])));
    assert_eq!(common_name(&der), None);
}

/// Bytes that are not a certificate name nobody, at each depth the walk can
/// fail at: no bytes, a header with no length, a body that is not a TLV, a
/// certificate with no serial number, and one that ends before the subject.
#[test]
fn bytes_that_are_not_a_certificate_name_nobody() {
    assert_eq!(common_name(&[]), None);
    assert_eq!(common_name(&[SEQ]), None);
    assert_eq!(common_name(&tlv_of(SEQ, &[0xFF])), None);
    assert_eq!(common_name(&tlv_of(SEQ, &tlv_of(SEQ, &[]))), None);
    let short = tlv_of(
        SEQ,
        &tlv_of(SEQ, &cat(&[tlv_of(INTEGER, &[1]), tlv_of(SEQ, &[])])),
    );
    assert_eq!(common_name(&short), None);
}

/// The two lengths DER does not permit: the indefinite form BER allows, and one
/// wider than this walk serves. Plus a long form whose length bytes are cut off.
#[test]
fn a_length_der_forbids_ends_the_read() {
    assert_eq!(tlv(&[SEQ, 0x80, 0x00, 0x00]), None);
    assert_eq!(tlv(&[SEQ, 0x85, 0, 0, 0, 0, 0]), None);
    assert_eq!(tlv(&[SEQ, 0x82, 0x01]), None);
    // A length longer than the bytes that follow it.
    assert_eq!(tlv(&[SEQ, 0x40, 0x01]), None);
}

/// A trailing run that is not a whole value ends the element walk, leaving the
/// values read before it — which is what makes every read above total.
#[test]
fn a_malformed_tail_ends_the_element_walk() {
    let body = cat(&[tlv_of(INTEGER, &[7]), vec![SEQ, 0x7F]]);
    let read = elements(&body);
    assert_eq!(read.len(), 1);
    assert_eq!(read.first().map(|(tag, _)| *tag), Some(INTEGER));
}

/// **The grade round trip over a real certificate** (REMOTE §4.2, bl-7ff3):
/// the one recipe mints a foot leaf, and this walk reads the grade back off it
/// without touching the identity. The ordinary leaves beside it stay operator
/// grade, which is default-operator working — the property that keeps every
/// certificate minted before the grade existed exactly as good as it was.
#[test]
fn a_minted_foot_reads_back_as_one_and_every_other_leaf_does_not() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    crate::wire::provision::issue(tmp.path(), "host", crate::registry::Grade::Foot)
        .expect("a foot leaf");
    crate::wire::provision::issue(tmp.path(), "desk", crate::registry::Grade::Operator)
        .expect("an operator leaf");
    let read = |name: &str| {
        let der =
            CertificateDer::from_pem_file(tmp.path().join(format!("{name}.pem"))).expect("pem");
        (common_name(&der), grade(&der))
    };
    assert_eq!(
        read("host"),
        (Some("host".to_owned()), crate::registry::Grade::Foot),
        "the grade rides beside the identity, not instead of it"
    );
    assert_eq!(
        read("desk"),
        (Some("desk".to_owned()), crate::registry::Grade::Operator)
    );
    assert_eq!(read("client").1, crate::registry::Grade::Operator);
    assert_eq!(read("ca").1, crate::registry::Grade::Operator);
}

/// Only the grade's own word is the grade. Another organizational unit is not
/// a foot, and neither are bytes that are no certificate at all — the default
/// answers, rather than the read failing.
#[test]
fn only_the_grades_own_word_demotes_and_unreadable_bytes_do_not() {
    let footed = certificate(tlv_of(
        SEQ,
        &cat(&[
            rdn(&ORG_UNIT, UTF8, crate::registry::peer::FOOT.as_bytes()),
            rdn(&COMMON_NAME, UTF8, b"host"),
        ]),
    ));
    assert_eq!(grade(&footed), crate::registry::Grade::Foot);
    assert_eq!(common_name(&footed).as_deref(), Some("host"));

    let other = certificate(tlv_of(
        SEQ,
        &cat(&[
            rdn(&ORG_UNIT, UTF8, b"engineering"),
            rdn(&COMMON_NAME, UTF8, b"host"),
        ]),
    ));
    assert_eq!(grade(&other), crate::registry::Grade::Operator);
    assert_eq!(grade(&[]), crate::registry::Grade::Operator);
}
