//! The two minted files, their three states, and the derivations both ends
//! must agree on.

use super::*;
use tempfile::TempDir;

#[test]
fn nothing_minted_is_none_and_a_mint_is_two_private_files() {
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
            assert_eq!(mode & 0o777, 0o600, "{name} is private");
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
