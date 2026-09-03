//! **A box is reachable more than one way** (bl-52f4): the host list a server
//! leaf covers, and the act that widens it without founding anything.
//!
//! Real `openssl` at test runtime, like every other test of the mint — what a
//! certificate asserts is read back off the certificate, never off the string
//! that went in.

use super::super::openssl::san;
use super::super::{ANCHORS, CA_KEY, LOOPBACK, ensure, hosts_of, issue, mint, reissue};
use super::{described, provisioned};
use crate::wire::material::{self, ADDRESS, Role};
use std::path::Path;
use tempfile::TempDir;

/// What the leaf `role` was minted under actually asserts. The bare host is
/// what is looked for rather than a rendered SAN entry: OpenSSL 3 prints an
/// address as `IP Address:…` and the exact spelling is the toolset's, while
/// [`san`]'s own tests pin the string yog hands it. Nothing else in a leaf's
/// text carries a host, so the substring can only have come from the SAN.
fn names(dir: &Path, role: Role) -> String {
    described(&dir.join(format!("{}.pem", role.leaf())), &["-text"])
}

/// **A box is reachable more than one way** (bl-52f4): every host stated is one
/// SAN entry, read as an address or a name entry by entry, with loopback still
/// appended exactly once and a repeat — of loopback or of anything else — said
/// once. An unstated list is the loopback-only leaf a self-provisioned box has
/// always had.
#[test]
fn every_host_stated_is_one_entry_and_none_is_said_twice() {
    assert_eq!(
        san(
            Role::Server,
            &[
                "engine.example.com".to_owned(),
                "198.51.100.9".to_owned(),
                "192.0.2.7".to_owned(),
            ]
        ),
        format!("DNS:engine.example.com,IP:198.51.100.9,IP:192.0.2.7,IP:{LOOPBACK}")
    );
    assert_eq!(
        san(
            Role::Server,
            &[
                "engine.example.com".to_owned(),
                LOOPBACK.to_owned(),
                "engine.example.com".to_owned(),
            ]
        ),
        format!("DNS:engine.example.com,IP:{LOOPBACK}"),
        "a repeat is said once, wherever it falls"
    );
    assert_eq!(san(Role::Server, &[]), format!("IP:{LOOPBACK}"));
}

/// The mint's own list: the address's host leads it and every further host
/// stated follows, so one certificate covers a box reachable three ways
/// (bl-52f4).
#[test]
fn the_mint_covers_the_address_and_every_further_host() {
    assert_eq!(
        hosts_of("engine.example.com:7737", &["192.0.2.7".to_owned()]),
        vec!["engine.example.com".to_owned(), "192.0.2.7".to_owned()]
    );
    assert_eq!(hosts_of("[::1]:7737", &[]), vec!["::1".to_owned()]);

    let tmp = TempDir::new().expect("tmp");
    mint(
        tmp.path(),
        "engine.example.com:7737",
        &["192.0.2.7".to_owned()],
        false,
    )
    .expect("mint");
    assert!(
        names(tmp.path(), Role::Server).contains("192.0.2.7"),
        "the further host rode the server leaf"
    );
    assert!(
        !names(tmp.path(), Role::Client).contains("192.0.2.7"),
        "and no client leaf names a host at all"
    );
}

/// **Re-issuing the server leaf is not a rotation** (bl-52f4): the CA and every
/// other leaf are untouched, so a client leaf already carried to another box
/// still verifies, and the address file — a different fact — is not rewritten.
#[test]
fn the_server_leaf_re_issues_over_the_standing_ca() {
    let tmp = TempDir::new().expect("tmp");
    ensure(tmp.path()).expect("mint");
    issue(tmp.path(), "phone", crate::registry::Grade::Operator).expect("a visiting box");
    let ca = std::fs::read(tmp.path().join(ANCHORS)).expect("ca");
    let carried = std::fs::read(tmp.path().join("phone.pem")).expect("the carried leaf");
    let address = std::fs::read(tmp.path().join(ADDRESS)).expect("address");

    reissue(tmp.path(), &["engine.example.com".to_owned()]).expect("re-issue");
    assert!(names(tmp.path(), Role::Server).contains("engine.example.com"));
    assert_eq!(std::fs::read(tmp.path().join(ANCHORS)).expect("ca"), ca);
    assert_eq!(
        std::fs::read(tmp.path().join("phone.pem")).expect("leaf"),
        carried,
        "nothing already issued was touched"
    );
    assert_eq!(
        std::fs::read(tmp.path().join(ADDRESS)).expect("address"),
        address
    );
    assert!(provisioned(tmp.path()).len() == material::LEAVES.len());
}

/// A box holding an operator's anchors and no CA key cannot issue under them —
/// the one guard both acts over a standing trust root share.
#[test]
fn a_box_that_founded_nothing_cannot_re_issue() {
    let tmp = TempDir::new().expect("tmp");
    ensure(tmp.path()).expect("mint");
    std::fs::remove_file(tmp.path().join(CA_KEY)).expect("a client box");
    let refusal = reissue(tmp.path(), &["engine.example.com".to_owned()])
        .expect_err("only the box that founded it can issue");
    assert!(refusal.contains(CA_KEY), "{refusal}");
}
