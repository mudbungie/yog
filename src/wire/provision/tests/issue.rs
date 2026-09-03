//! **One extra client leaf under a stated common name** (REMOTE §8.2,
//! bl-64a7): what the host mints for a visiting box, what it refuses, and that
//! the pair reads back as an entry's material once it has been carried there.
//!
//! Real `openssl` at test runtime, like every other test of the mint — a
//! certificate fixture is never committed (REMOTE §8).

use super::super::{CA_KEY, ensure, issue};
use super::described;
use crate::git_env;
use crate::registry::{Grade, LOCAL};
use crate::wire::material::{ADDRESS, ANCHORS, Role, read_dir};
use std::path::Path;
use tempfile::TempDir;

/// The name the host's operator states for the visiting box.
const VISITOR: &str = "phone";

/// A box that founded its own trust root can issue for a visitor, and what it
/// issues is `Role::Client`'s recipe under the operator's name: signed by the
/// CA already here, a client EKU, and a SAN naming itself rather than any host
/// — so the pair is usable from whichever box it is carried to.
#[test]
fn a_host_issues_a_client_leaf_under_the_name_it_states() {
    let tmp = TempDir::new().expect("tmp");
    let dir = tmp.path().join("wire");
    ensure(&dir).expect("mint");
    issue(&dir, VISITOR, Grade::Operator).expect("issue");

    let pem = dir.join(format!("{VISITOR}.pem"));
    assert!(dir.join(format!("{VISITOR}.key")).is_file(), "the key too");
    assert!(
        described(&pem, &["-subject"]).contains(&format!("CN={VISITOR}")),
        "the stated name is the subject: {}",
        described(&pem, &["-subject"])
    );
    let text = described(&pem, &["-text"]);
    assert!(
        text.contains("TLS Web Client Authentication"),
        "a client leaf authenticates as a client: {text}"
    );
    assert!(
        text.contains(&format!("DNS:{VISITOR}")),
        "and its SAN names itself: {text}"
    );
    assert!(
        !text.contains("TLS Web Server Authentication") && !text.contains("IP Address"),
        "no server EKU and no host anywhere in it: {text}"
    );

    let ca = dir.join(ANCHORS).to_string_lossy().into_owned();
    let out = git_env::output(git_env::command(Path::new("openssl")).args([
        "verify",
        "-CAfile",
        &ca,
        &pem.to_string_lossy(),
    ]))
    .expect("openssl");
    assert!(out.status.success(), "the CA already here signed it");
}

/// The one act after the mint (§8.2): the pair is carried to the visiting box
/// by hand and filed as an entry, where it reads back as that box's client
/// material — the basename is a filing convenience and the name inside is the
/// identity, so the rename costs nothing.
#[test]
fn the_pair_reads_back_as_an_entry_on_the_box_it_is_carried_to() {
    let host = TempDir::new().expect("tmp");
    ensure(host.path()).expect("mint");
    issue(host.path(), VISITOR, Grade::Operator).expect("issue");

    let visitor = TempDir::new().expect("tmp");
    let entry = visitor.path().join("workspaces").join(VISITOR);
    std::fs::create_dir_all(&entry).expect("entry");
    for (from, to) in [
        (format!("{VISITOR}.pem"), "client.pem"),
        (format!("{VISITOR}.key"), "client.key"),
        (ANCHORS.to_owned(), ANCHORS),
    ] {
        std::fs::copy(host.path().join(from), entry.join(to)).expect("carried by hand");
    }
    std::fs::write(entry.join(ADDRESS), "engine.example.com:7737\n").expect("stated address");

    let material = read_dir(&entry, Role::Client)
        .expect("readable")
        .expect("provisioned");
    assert_eq!(material.address, "engine.example.com:7737");
    assert_eq!(material.chain, entry.join("client.pem"));
    assert!(
        described(&material.chain, &["-subject"]).contains(&format!("CN={VISITOR}")),
        "the common name inside, not the basename, is the identity"
    );
}

/// A client box cannot issue. Its `ca.pem` is an operator's trust root with no
/// key beside it, and the mint never replaces one — so the refusal names the
/// missing key rather than founding a CA that verifies nothing.
#[test]
fn a_box_with_no_ca_key_refuses_and_says_which_file() {
    let tmp = TempDir::new().expect("tmp");
    let bare = tmp.path().join("bare");
    std::fs::create_dir_all(&bare).expect("dir");
    let refusal = issue(&bare, VISITOR, Grade::Operator).expect_err("nothing to sign with");
    assert!(refusal.contains(CA_KEY), "{refusal}");

    let client = tmp.path().join("client-box");
    std::fs::create_dir_all(&client).expect("dir");
    std::fs::write(client.join(ANCHORS), b"operator's own").expect("anchor");
    let refusal =
        issue(&client, VISITOR, Grade::Operator).expect_err("a client machine cannot issue");
    assert!(refusal.contains(CA_KEY), "{refusal}");
    assert_eq!(
        std::fs::read(client.join(ANCHORS)).expect("anchor"),
        b"operator's own",
        "and the operator's trust root is untouched"
    );
    assert!(!client.join(format!("{VISITOR}.pem")).is_file());
}

/// Re-issuing distrusts nothing, so a second leaf under one name would be two
/// live certificates under one identity. Half a pair is enough to refuse: the
/// remedy is another name, never a silent second issue.
#[test]
fn an_existing_pair_refuses_and_names_its_remedy() {
    let tmp = TempDir::new().expect("tmp");
    ensure(tmp.path()).expect("mint");
    issue(tmp.path(), VISITOR, Grade::Operator).expect("issue");
    let first = std::fs::read(tmp.path().join(format!("{VISITOR}.pem"))).expect("leaf");

    let refusal = issue(tmp.path(), VISITOR, Grade::Operator).expect_err("already issued");
    assert!(refusal.contains("FORCE=1"), "{refusal}");
    assert_eq!(
        std::fs::read(tmp.path().join(format!("{VISITOR}.pem"))).expect("leaf"),
        first,
        "and refused without touching it"
    );

    // Either half is a pair as far as the guard is concerned — an issue over a
    // key still on disk would leave a certificate no key opens.
    std::fs::remove_file(tmp.path().join(format!("{VISITOR}.pem"))).expect("rm");
    assert!(
        issue(tmp.path(), VISITOR, Grade::Operator).is_err(),
        "the key is still here"
    );
    std::fs::remove_file(tmp.path().join(format!("{VISITOR}.key"))).expect("rm");
    issue(tmp.path(), VISITOR, Grade::Operator).expect("a name nothing holds");
}

/// The common name becomes a directory name on both boxes and a client
/// identity in the registry, so it is validated by the registry's own rule —
/// one rule, not three cases (REMOTE §4.1): a plain path component, and never
/// the reserved in-world identity.
#[test]
fn an_identity_the_registry_would_refuse_is_refused_here() {
    let tmp = TempDir::new().expect("tmp");
    ensure(tmp.path()).expect("mint");
    for name in ["", ".", "..", "a/b", "a\\b", "a\0b", LOCAL] {
        let refusal = issue(tmp.path(), name, Grade::Operator).expect_err("unusable");
        assert!(refusal.contains(LOCAL), "{name:?} → {refusal}");
        assert_eq!(
            std::fs::read_dir(tmp.path()).expect("dir").count(),
            super::super::artifacts().len(),
            "{name:?} wrote something"
        );
    }
}
