//! The mint: what a box with nothing gets, what a box with something keeps,
//! and what a rotation costs.

use super::openssl::{eku, run, san, tool};
use super::*;
use crate::test_support::spawn_guard;
use crate::wire::material::{self, Material};
use tempfile::TempDir;

/// Every artifact present and readable as material for every role — the
/// question `material::read` asks, asked of all three ends at once.
fn provisioned(dir: &Path) -> Vec<Material> {
    material::LEAVES
        .iter()
        .map(|role| {
            material::read_dir(dir, *role)
                .expect("readable")
                .expect("provisioned")
        })
        .collect()
}

/// The unprovisioned box (REMOTE §8 as amended, bl-ae05): boot finds nothing,
/// founds its own loopback trust root, and every end — the server, a terminal
/// seat, the window — is provisioned by it.
#[test]
fn an_unprovisioned_box_founds_its_own_loopback_trust_root() {
    let _guard = spawn_guard();
    let tmp = TempDir::new().expect("tmp");
    let dir = tmp.path().join("wire");
    ensure(&dir).expect("mint");

    let ends = provisioned(&dir);
    assert_eq!(ends.len(), 3, "server, client and window");
    for end in &ends {
        assert_eq!(end.address, format!("{LOOPBACK}:{PORT}"), "loopback only");
    }
    // The window's leaf carries the identity the registry seats (REMOTE §4.1).
    let der = std::fs::read(dir.join("window.pem")).expect("leaf");
    let pem = String::from_utf8_lossy(&der);
    assert!(pem.starts_with("-----BEGIN CERTIFICATE-----"), "{pem}");
    assert_eq!(
        crate::wire::material::Role::Window.common_name(),
        crate::registry::WINDOW
    );
}

/// Idempotence is what makes it safe on every boot: a second call mints
/// nothing, so the certificates a running seat holds keep verifying.
#[test]
fn a_second_ensure_mints_nothing() {
    let _guard = spawn_guard();
    let tmp = TempDir::new().expect("tmp");
    ensure(tmp.path()).expect("mint");
    let before = std::fs::read(tmp.path().join(ANCHORS)).expect("ca");
    ensure(tmp.path()).expect("again");
    assert_eq!(
        std::fs::read(tmp.path().join(ANCHORS)).expect("ca"),
        before,
        "the trust root is untouched"
    );
}

/// A box the operator provisioned by hand — an anchor and a leaf copied on, no
/// CA key — is a **client** machine, and boot must not replace its trust root
/// with one that verifies nothing the operator issued.
#[test]
fn a_box_with_an_anchor_and_no_ca_key_is_left_alone() {
    let _guard = spawn_guard();
    let tmp = TempDir::new().expect("tmp");
    let dir = tmp.path().join("wire");
    std::fs::create_dir_all(&dir).expect("dir");
    std::fs::write(dir.join(ANCHORS), b"operator's own").expect("anchor");
    ensure(&dir).expect("no mint to run");
    assert_eq!(
        std::fs::read(dir.join(ANCHORS)).expect("anchor"),
        b"operator's own",
        "the operator's anchor survives"
    );
    assert!(!dir.join("server.pem").is_file(), "nothing to sign with");
    assert_eq!(
        std::fs::read_to_string(dir.join(ADDRESS))
            .expect("address")
            .trim(),
        format!("{LOOPBACK}:{PORT}"),
        "the address is still written: it is not certificate material"
    );
}

/// The address a box already names is kept, and the leaf is minted for it —
/// which is the whole of "wider listening is the operator's statement of
/// intent" (REMOTE §8). Loopback rides the server leaf regardless, so the
/// window can always reach its own engine.
#[test]
fn an_existing_address_is_kept_and_the_server_leaf_names_it() {
    let _guard = spawn_guard();
    let tmp = TempDir::new().expect("tmp");
    std::fs::write(tmp.path().join(ADDRESS), "engine.example.com:7737\n").expect("address");
    ensure(tmp.path()).expect("mint");
    let m = material::read_dir(tmp.path(), Role::Server)
        .expect("readable")
        .expect("provisioned");
    assert_eq!(m.address, "engine.example.com:7737");
    assert_eq!(
        san(Role::Server, "engine.example.com"),
        format!("DNS:engine.example.com,IP:{LOOPBACK}")
    );
}

/// A rotation deletes the lot and starts over, which is why it is never
/// implicit: every certificate already issued stops verifying.
#[test]
fn a_rotation_replaces_the_trust_root() {
    let _guard = spawn_guard();
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path(), "127.0.0.1:0", false).expect("mint");
    let before = std::fs::read(tmp.path().join(ANCHORS)).expect("ca");
    mint(tmp.path(), "127.0.0.1:9", true).expect("rotate");
    assert_ne!(
        std::fs::read(tmp.path().join(ANCHORS)).expect("ca"),
        before,
        "a new trust root"
    );
    assert_eq!(
        std::fs::read_to_string(tmp.path().join(ADDRESS))
            .expect("address")
            .trim(),
        "127.0.0.1:9",
        "and the address it was rotated onto"
    );
}

/// Half a leaf is no leaf: the pair is minted together and is useless apart, so
/// a truncated key is re-minted rather than left to fail a handshake.
#[test]
fn half_a_leaf_is_re_minted() {
    let _guard = spawn_guard();
    let tmp = TempDir::new().expect("tmp");
    ensure(tmp.path()).expect("mint");
    assert!(leaf_present(tmp.path(), Role::Window));
    std::fs::remove_file(tmp.path().join("window.key")).expect("rm");
    assert!(!leaf_present(tmp.path(), Role::Window));
    ensure(tmp.path()).expect("again");
    assert!(leaf_present(tmp.path(), Role::Window), "re-minted");
}

/// The address file's three readings — absent, blank, and named — are the same
/// three [`material::read_dir`] makes, because a blank file names no address.
#[test]
fn a_blank_address_names_nothing() {
    let tmp = TempDir::new().expect("tmp");
    assert_eq!(address_at(tmp.path()), None, "absent");
    std::fs::write(tmp.path().join(ADDRESS), "  \n").expect("blank");
    assert_eq!(address_at(tmp.path()), None, "blank");
    std::fs::write(tmp.path().join(ADDRESS), " host:1 \n").expect("named");
    assert_eq!(address_at(tmp.path()), Some("host:1".to_owned()));
}

/// The host a SAN is derived from, off every address spelling the file may
/// carry.
#[test]
fn a_host_is_the_address_without_its_port_or_brackets() {
    assert_eq!(host_of("127.0.0.1:7737"), "127.0.0.1");
    assert_eq!(host_of("[::1]:7737"), "::1");
    assert_eq!(host_of("engine.example.com"), "engine.example.com");
}

/// A SAN says which **kind** of name a seat verifies against, and the loopback
/// entry is unconditional — except where it would be said twice.
#[test]
fn a_server_leaf_always_names_loopback_and_never_twice() {
    assert_eq!(san(Role::Server, LOOPBACK), format!("IP:{LOOPBACK}"));
    assert_eq!(
        san(Role::Server, "192.0.2.7"),
        format!("IP:192.0.2.7,IP:{LOOPBACK}")
    );
    assert_eq!(
        san(Role::Client, "ignored"),
        format!("DNS:{}", Role::Client.common_name())
    );
    assert_eq!(eku(Role::Server), "serverAuth");
    assert_eq!(eku(Role::Window), "clientAuth");
}

/// Every file the mint writes is named once, and the list is what a rotation
/// deletes and a report prints.
#[test]
fn the_artifact_list_is_the_whole_of_what_is_written() {
    let _guard = spawn_guard();
    let tmp = TempDir::new().expect("tmp");
    ensure(tmp.path()).expect("mint");
    for name in artifacts() {
        assert!(tmp.path().join(&name).is_file(), "{name} was not written");
    }
    assert_eq!(artifacts().len(), 9, "a CA pair, an address, three leaves");
}

/// A directory that cannot be made, and an address that cannot be written, are
/// both the mint saying so rather than half-provisioning a box.
#[test]
fn an_unwritable_target_refuses() {
    let _guard = spawn_guard();
    let tmp = TempDir::new().expect("tmp");
    let blocked = tmp.path().join("file");
    std::fs::write(&blocked, b"not a directory").expect("file");
    assert!(mint(&blocked, "127.0.0.1:0", false).is_err(), "no dir");

    let dir = tmp.path().join("wire");
    std::fs::create_dir_all(dir.join(ADDRESS)).expect("a directory where the file goes");
    let refusal = mint(&dir, "127.0.0.1:0", false).expect_err("cannot write the address");
    assert!(refusal.contains(ADDRESS), "{refusal}");
}

/// The tool's own failures: one it cannot start, and one it started and
/// refused. Both are the operator's sentence, never a silent half-mint.
#[test]
fn the_tool_speaks_for_itself() {
    let _guard = spawn_guard();
    let missing = run(Path::new("yog-no-such-tool"), &[]).expect_err("cannot start");
    assert!(missing.contains("yog-no-such-tool"), "{missing}");
    let refused =
        tool(&["x509", "-in", "/nonexistent/yog-not-a-certificate"]).expect_err("openssl refuses");
    assert!(refused.starts_with("openssl x509:"), "{refused}");
}
