//! The offer's vocabulary: the code, the word, the states — and, beside them,
//! the store.

use super::*;

mod store;

/// The public key and the CA certificate the vector below is over. Both are
/// throwaway public material minted for this test; neither key exists.
const DEVICE: &str = "-----BEGIN PUBLIC KEY-----\n\
    MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAENflj+nLA7lsFuxVfqaryK84n9wP4\n\
    3JqG0mf0zqZecVFyPJ6ZX6m9IVK9g0IKk4hdMBbVTWXZvaIRcR4UDpgzCg==\n\
    -----END PUBLIC KEY-----\n";
const CA: &str = "-----BEGIN CERTIFICATE-----\n\
    MIIBdzCCAR2gAwIBAgIUS6jy88xKuMgm0widLw6ejqRjV2EwCgYIKoZIzj0EAwIw\n\
    ETEPMA0GA1UEAwwGeW9nLWNhMB4XDTI2MTExMTE5MTgyOVoXDTI5MDIxMzE5MTgy\n\
    OVowETEPMA0GA1UEAwwGeW9nLWNhMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE\n\
    a2lFtf+Bdi55ENgocMU/VFmRRnrTvKFmGo1Jc/jkpoLCcUiuY+oHUyxbLu9bjGPN\n\
    KTzzSgWlmGrNv8lTlWTamKNTMFEwHQYDVR0OBBYEFDuf4kFm3Ug1PoxXdM4G1sUD\n\
    nSeYMB8GA1UdIwQYMBaAFDuf4kFm3Ug1PoxXdM4G1sUDnSeYMA8GA1UdEwEB/wQF\n\
    MAMBAf8wCgYIKoZIzj0EAwIDSAAwRQIhAIYwai5c3FwmIffCooBMLoSYb8A+40Id\n\
    n1/Fr3EXpRUyAiB/98dHbS9USqSz9WaFPMWG6lVZgiWI3ZGtfzdB30JEmA==\n\
    -----END CERTIFICATE-----\n";

/// **The confirmation word's vector** (REMOTE §15.1 step 3). The seat, the
/// foot and the phone each reproduce this from the same two PEM blocks: decode
/// each to DER, SHA-256 the public key's bytes followed by the certificate's,
/// and read the digest's first thirty bits five at a time as six symbols.
/// A change here is a change to all four implementations.
#[test]
fn the_confirmation_word_is_pinned_by_a_vector() {
    use rustls::pki_types::pem::PemObject;
    use rustls::pki_types::{CertificateDer, SubjectPublicKeyInfoDer};
    let public = SubjectPublicKeyInfoDer::from_pem_slice(DEVICE.as_bytes()).expect("public key");
    let ca = CertificateDer::from_pem_slice(CA.as_bytes()).expect("certificate");
    assert_eq!((public.len(), ca.len()), (91, 379));
    let digest = ring::digest::digest(&ring::digest::SHA256, &[&public[..], &ca[..]].concat());
    assert_eq!(
        digest.as_ref().get(..4),
        Some(&[0xc2, 0x8b, 0x19, 0x3c][..]),
        "the digest's head, which is all the word reads"
    );
    assert_eq!(word(&public, &ca), "2LFTUR");
    // The order is part of the contract: the key first, then the certificate.
    assert_ne!(word(&ca, &public), "2LFTUR");
}

/// The alphabet is the thirty-two symbols a person can read off a screen
/// without mistaking one for another, and a five-bit value indexes it.
#[test]
fn the_alphabet_has_no_symbol_a_reader_confuses() {
    assert_eq!(ALPHABET.len(), 32);
    for confusable in b"IO01" {
        assert!(!ALPHABET.contains(confusable));
    }
    assert_eq!(symbol(0), 'A');
    assert_eq!(symbol(31), '9');
    assert_eq!(symbol(32), 'A', "only the low five bits are read");
}

/// A code is eight symbols of the alphabet, and two are not the same code.
#[test]
fn a_code_is_eight_symbols_from_the_random_source() {
    let (one, two) = (code().expect("code"), code().expect("code"));
    assert_eq!(one.len(), CODE_LEN);
    assert!(one.bytes().all(|b| ALPHABET.contains(&b)), "{one}");
    assert_ne!(one, two, "forty bits do not repeat by accident");
}

/// The three state words round-trip, and an unknown one is nobody's.
#[test]
fn a_state_is_its_word_both_ways() {
    for state in [State::Open, State::Claimed, State::Proved] {
        assert_eq!(State::of(state.word()), Some(state));
    }
    assert_eq!(State::of("pending"), None);
}

/// A row whose state no build has heard of, and a row that is not an object,
/// each refuse naming what was wrong.
#[test]
fn a_malformed_offers_row_refuses_by_name() {
    use serde_json::json;
    let read = |rows: serde_json::Value| {
        let reply = json!({ "ok": true, "kind": "offers", "rows": rows });
        wire::answer_of("offers", reply.as_object().expect("object")).expect("this family's")
    };
    let unknown = read(json!([{ "name": "phone", "grade": "operator",
                                "state": "pending", "expires": 1 }]));
    assert!(unknown.expect_err("refused").contains("pending"));
    assert!(read(json!(["phone"])).expect_err("refused").contains("row"));
    assert!(wire::answer_of("enrolled", &serde_json::Map::new()).is_none());
}
