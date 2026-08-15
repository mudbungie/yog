//! The two configurations, and every way PEM can fail to be one.

use super::*;
use crate::test_support::wire::{material, mint};
use crate::wire::material::Role;
use tempfile::TempDir;

/// Provisioned material builds both ends.
#[test]
fn provisioned_material_builds_both_ends() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    server_config(&material(tmp.path(), Role::Server, "127.0.0.1:0")).expect("server");
    client_config(&material(tmp.path(), Role::Client, "127.0.0.1:0")).expect("client");
}

/// An anchors file that holds no certificate is refused by name — not
/// quietly accepted as an empty trust store, which would trust nothing while
/// looking configured.
#[test]
fn empty_anchors_refuse_by_name() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    std::fs::write(tmp.path().join("ca.pem"), "").expect("write");
    let m = material(tmp.path(), Role::Server, "127.0.0.1:0");
    let err = server_config(&m).expect_err("refused");
    assert!(err.contains("ca.pem"), "{err}");
    let err =
        client_config(&material(tmp.path(), Role::Client, "127.0.0.1:0")).expect_err("refused");
    assert!(err.contains("ca.pem"), "{err}");
}

/// Anchors that are not PEM at all are refused where they are read.
#[test]
fn unreadable_anchors_refuse() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    std::fs::remove_file(tmp.path().join("ca.pem")).expect("rm");
    let err =
        server_config(&material(tmp.path(), Role::Server, "127.0.0.1:0")).expect_err("refused");
    assert!(err.contains("ca.pem"), "{err}");
}

/// An anchor that is syntactically a certificate and semantically not one is
/// refused by the store rather than added.
#[test]
fn a_malformed_anchor_refuses() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    std::fs::write(
        tmp.path().join("ca.pem"),
        "-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n",
    )
    .expect("write");
    let err =
        server_config(&material(tmp.path(), Role::Server, "127.0.0.1:0")).expect_err("refused");
    assert!(err.contains("ca.pem"), "{err}");
}

/// A chain file with no certificate in it is refused by name.
#[test]
fn an_empty_chain_refuses() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    std::fs::write(tmp.path().join("server.pem"), "").expect("write");
    let err =
        server_config(&material(tmp.path(), Role::Server, "127.0.0.1:0")).expect_err("refused");
    assert!(err.contains("server.pem"), "{err}");
}

/// A chain that will not read is refused where it is read.
#[test]
fn an_unreadable_chain_refuses() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    std::fs::remove_file(tmp.path().join("client.pem")).expect("rm");
    let err =
        client_config(&material(tmp.path(), Role::Client, "127.0.0.1:0")).expect_err("refused");
    assert!(err.contains("client.pem"), "{err}");
}

/// A chain whose PEM is malformed refuses on the entry, not on the file.
#[test]
fn a_malformed_chain_entry_refuses() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let good = std::fs::read_to_string(tmp.path().join("server.pem")).expect("read");
    std::fs::write(
        tmp.path().join("server.pem"),
        format!("{good}-----BEGIN CERTIFICATE-----\nnot base64!!\n"),
    )
    .expect("write");
    let err =
        server_config(&material(tmp.path(), Role::Server, "127.0.0.1:0")).expect_err("refused");
    assert!(err.contains("server.pem"), "{err}");
}

/// A missing key is refused by name.
#[test]
fn a_missing_key_refuses() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    std::fs::remove_file(tmp.path().join("server.key")).expect("rm");
    let err =
        server_config(&material(tmp.path(), Role::Server, "127.0.0.1:0")).expect_err("refused");
    assert!(err.contains("server.key"), "{err}");
    std::fs::remove_file(tmp.path().join("client.key")).expect("rm");
    let err =
        client_config(&material(tmp.path(), Role::Client, "127.0.0.1:0")).expect_err("refused");
    assert!(err.contains("client.key"), "{err}");
}

/// A key that does not match its certificate is refused by rustls, and the
/// refusal names the chain it disagreed with.
#[test]
fn a_key_that_does_not_match_its_chain_refuses() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let other = TempDir::new().expect("tmp");
    mint(other.path());
    std::fs::copy(
        other.path().join("server.key"),
        tmp.path().join("server.key"),
    )
    .expect("copy");
    std::fs::copy(
        other.path().join("client.key"),
        tmp.path().join("client.key"),
    )
    .expect("copy");
    let err =
        server_config(&material(tmp.path(), Role::Server, "127.0.0.1:0")).expect_err("refused");
    assert!(err.contains("server.pem"), "{err}");
    let err =
        client_config(&material(tmp.path(), Role::Client, "127.0.0.1:0")).expect_err("refused");
    assert!(err.contains("client.pem"), "{err}");
}
