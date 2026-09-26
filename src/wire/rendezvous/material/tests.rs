//! The two minted files, their three states, and the derivations both ends
//! must agree on.

use super::*;
use tempfile::TempDir;

#[test]
fn nothing_minted_is_none_and_a_mint_is_two_private_files_and_a_public_one() {
    let tmp = TempDir::new().expect("tmp");
    assert_eq!(read_dir(tmp.path()).expect("read"), None);
    mint(tmp.path()).expect("mint");
    let pairing = read_dir(tmp.path()).expect("read").expect("minted");
    assert_ne!(pairing.seed, pairing.salt, "two draws, not one");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for name in artifacts() {
            let mode = std::fs::metadata(tmp.path().join(&name))
                .expect("meta")
                .permissions()
                .mode();
            let want = if name == PUBLIC { 0o644 } else { 0o600 };
            assert_eq!(mode & 0o777, want, "{name}'s mode");
        }
    }
}

#[test]
fn a_second_mint_rewrites_nothing() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path()).expect("mint");
    let before = read_dir(tmp.path()).expect("read");
    mint(tmp.path()).expect("again");
    assert_eq!(read_dir(tmp.path()).expect("read"), before);
}

#[test]
fn half_the_material_is_a_refusal_naming_the_remedy() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path()).expect("mint");
    std::fs::remove_file(tmp.path().join(SALT)).expect("rm");
    let refusal = read_dir(tmp.path()).expect_err("half");
    assert!(
        refusal.contains(SALT) && refusal.contains("yog wire-certs"),
        "{refusal}"
    );
}

#[test]
fn a_file_that_is_not_32_bytes_of_hex_is_no_material() {
    let tmp = TempDir::new().expect("tmp");
    std::fs::write(tmp.path().join(KEY), "zz\n").expect("write");
    std::fs::write(tmp.path().join(SALT), "abc\n").expect("write");
    assert_eq!(read_dir(tmp.path()).expect("read"), None);
    std::fs::write(tmp.path().join(SALT), format!("{}\n", hex(&[7u8; 31]))).expect("write");
    assert_eq!(read_dir(tmp.path()).expect("read"), None);
}

#[test]
fn an_unwritable_directory_refuses() {
    let refusal = mint(std::path::Path::new("/nonexistent/yog-rendezvous")).expect_err("no dir");
    assert!(refusal.contains(KEY), "{refusal}");
}

#[test]
fn hex_round_trips_and_refuses_what_is_not_hex() {
    assert_eq!(hex(&[0, 15, 255]), "000fff");
    assert_eq!(unhex("000fff"), Some(vec![0, 15, 255]));
    assert_eq!(unhex("0"), None, "odd length");
    assert_eq!(unhex("0g"), None, "not a digit");
    assert_eq!(unhex("é0"), None, "not even ASCII");
}

#[test]
fn every_derivation_is_deterministic_and_distinct() {
    let pairing = Pairing {
        seed: [1u8; 32],
        salt: [2u8; 32],
    };
    let again = pairing.clone();
    assert_eq!(pairing.seal_key(), again.seal_key());
    assert_eq!(pairing.presence_salt(), again.presence_salt());
    assert_eq!(pairing.inbox_salt(), again.inbox_salt());
    assert_eq!(
        pairing.inbox_keypair().expect("k").public(),
        again.inbox_keypair().expect("k").public()
    );
    assert_ne!(pairing.presence_salt(), pairing.inbox_salt());
    assert_ne!(pairing.seal_key().to_vec(), pairing.inbox_salt());
    assert_ne!(
        pairing.keypair().expect("k").public(),
        pairing.inbox_keypair().expect("k").public(),
        "the inbox key is derived from the salt, never the engine's own"
    );
    let other = Pairing {
        seed: [1u8; 32],
        salt: [3u8; 32],
    };
    assert_ne!(
        other.seal_key(),
        pairing.seal_key(),
        "another salt, another key"
    );
}

/// RFC 8032 §7.1 test 1: the public key a known seed determines, pinned, so
/// the file a client carries is checked against the standard and not against
/// the same code that wrote it.
#[test]
fn the_public_file_is_the_seeds_public_key_and_heals_when_absent() {
    let seed = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
    let public = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
    let tmp = TempDir::new().expect("tmp");
    std::fs::write(tmp.path().join(KEY), format!("{seed}\n")).expect("seed");
    mint(tmp.path()).expect("mint");
    let read = || std::fs::read_to_string(tmp.path().join(PUBLIC)).expect("pub");
    assert_eq!(read(), format!("{public}\n"));
    let pairing = read_dir(tmp.path()).expect("read").expect("minted");
    assert_eq!(pairing.handoff().expect("handoff").public, public);
    assert_eq!(pairing.handoff().expect("handoff").salt, hex(&pairing.salt));
    // A box minted before the file existed: the next mint derives it again.
    std::fs::remove_file(tmp.path().join(PUBLIC)).expect("rm");
    mint(tmp.path()).expect("again");
    assert_eq!(read(), format!("{public}\n"));
}

#[test]
fn a_seed_that_does_not_decode_refuses_to_publish() {
    let tmp = TempDir::new().expect("tmp");
    std::fs::write(tmp.path().join(KEY), "zz\n").expect("write");
    let refusal = mint(tmp.path()).expect_err("no seed");
    assert!(refusal.contains(KEY), "{refusal}");
}

#[test]
fn an_unwritable_public_file_refuses() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path()).expect("mint");
    std::fs::remove_file(tmp.path().join(PUBLIC)).expect("rm");
    std::fs::create_dir(tmp.path().join(PUBLIC)).expect("a directory where the file goes");
    let refusal = bundle(tmp.path()).expect_err("a directory in the way");
    assert!(refusal.contains(PUBLIC), "{refusal}");
}

#[test]
fn the_bundle_is_both_files_or_neither() {
    let tmp = TempDir::new().expect("tmp");
    assert_eq!(bundle(tmp.path()).expect("none"), Vec::<String>::new());
    mint(tmp.path()).expect("mint");
    std::fs::remove_file(tmp.path().join(PUBLIC)).expect("rm");
    assert_eq!(bundle(tmp.path()).expect("both"), [PUBLIC, SALT]);
    assert!(tmp.path().join(PUBLIC).is_file(), "derived on the way");
}
