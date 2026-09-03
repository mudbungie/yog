//! The registry: what a client identity may be, where its files land, and the
//! three acts §4 defines over them — seat, read, revoke.

use super::*;
use tempfile::TempDir;

fn client(name: &str) -> Client {
    Client::parse(name).expect("a usable identity")
}

/// The identity is a path component, and the check is the one place that says
/// so: a certificate is an untrusted peer's text, and a name carrying a
/// separator is a name that addresses the filesystem.
#[test]
fn an_identity_that_is_not_a_plain_component_is_refused() {
    for name in ["", ".", "..", "a/b", "..\\..\\etc", "nul\0byte"] {
        let refusal = Client::parse(name).expect_err("refused");
        assert!(refusal.contains("client identity"), "{refusal}");
    }
    assert_eq!(client("yog-client").name(), "yog-client");
}

/// `local` is spent by the layout, so no certificate may claim it — the same
/// rule that refuses `.` and `..`, not a case of its own.
#[test]
fn the_reserved_local_identity_is_not_parseable() {
    assert!(Client::parse(LOCAL).is_err());
    assert!(Client::local().is_local());
    assert_eq!(Client::local().name(), LOCAL);
    assert!(!client("yog-client").is_local());
}

/// The layout, stated as paths: a client's directory holds one file per
/// registration.
#[test]
fn the_layout_is_the_client_directory_and_its_registrations() {
    let root = std::path::Path::new("/home/u/state/yog");
    let c = client("phone");
    assert_eq!(dir(root, &c), root.join(CLIENTS).join("phone"));
    assert_eq!(registrations(root, &c), dir(root, &c).join(WORKSPACES));
}

/// A fresh server has registered nobody: a certificate the operator has not
/// seated reads as the empty set, which is the general path with no input and
/// not a bootstrap branch.
#[test]
fn a_client_with_no_directory_is_registered_nowhere() {
    let tmp = TempDir::new().expect("tmp");
    assert!(registered(tmp.path(), &client("phone")).is_empty());
    assert!(registered(tmp.path(), &Client::local()).is_empty());
}

/// Seat, read, revoke — and the first registration on a fresh server is the
/// operator's own `mkdir`+`touch`, which is exactly what `register` writes.
#[test]
fn registering_seats_a_client_and_deleting_the_file_revokes_it() {
    let tmp = TempDir::new().expect("tmp");
    let c = client("phone");
    register(tmp.path(), &c, "home").expect("seated");
    register(tmp.path(), &c, "corp").expect("seated");
    // Idempotent: seating an existing registration rewrites the same empty file.
    register(tmp.path(), &c, "home").expect("seated");
    assert_eq!(
        registered(tmp.path(), &c),
        BTreeSet::from(["home".to_owned(), "corp".to_owned()])
    );
    // A registration has no content — it IS the pair, and the pair is the path.
    assert_eq!(
        std::fs::read(registrations(tmp.path(), &c).join("home")).expect("read"),
        Vec::<u8>::new()
    );
    // Revocation is deletion (§4).
    std::fs::remove_file(registrations(tmp.path(), &c).join("corp")).expect("revoked");
    assert_eq!(
        registered(tmp.path(), &c),
        BTreeSet::from(["home".to_owned()])
    );
    // Another client's registrations are its own.
    assert!(registered(tmp.path(), &client("laptop")).is_empty());
}

/// The workspace name becomes a file name, so it is checked on the same terms
/// the identity is.
#[test]
fn a_workspace_name_that_is_not_a_plain_component_is_refused() {
    let tmp = TempDir::new().expect("tmp");
    let refused = register(tmp.path(), &client("phone"), "../elsewhere").expect_err("refused");
    assert!(refused.to_string().contains("workspace name"), "{refused}");
}

/// A registrations path that cannot be made is a refusal, not a panic: the
/// directory's own parent is a file here.
#[test]
fn an_unmakeable_registration_directory_refuses() {
    let tmp = TempDir::new().expect("tmp");
    std::fs::write(tmp.path().join(CLIENTS), b"not a directory").expect("write");
    assert!(register(tmp.path(), &client("phone"), "home").is_err());
    assert!(registered(tmp.path(), &client("phone")).is_empty());
}
