//! The three answers a material directory can give.

use super::*;
use tempfile::TempDir;

/// Nothing provisioned is an answer, not an error: the wire is off, and
/// removing the directory deleted config rather than editing code.
#[test]
fn an_unprovisioned_box_has_no_wire() {
    let dir = TempDir::new().expect("tmp");
    assert_eq!(read_dir(&dir.path().join("wire"), Role::Server), Ok(None));
}

/// A whole provisioning reads back as itself, per role.
#[test]
fn a_provisioned_box_reads_its_material() {
    let dir = TempDir::new().expect("tmp");
    crate::test_support::wire::mint(dir.path());
    let server = read_dir(dir.path(), Role::Server)
        .expect("read")
        .expect("provisioned");
    assert_eq!(server.chain, dir.path().join("server.pem"));
    assert_eq!(server.key, dir.path().join("server.key"));
    assert_eq!(server.anchors, dir.path().join(ANCHORS));
    assert_eq!(server.address, crate::test_support::wire::EPHEMERAL);
    let client = read_dir(dir.path(), Role::Client)
        .expect("read")
        .expect("provisioned");
    assert_eq!(client.chain, dir.path().join("client.pem"));
    // One CA, both directions — the anchors are the same file.
    assert_eq!(client.anchors, server.anchors);
}

/// Half a trust store is a misconfiguration, and it names every gap at once
/// plus the remedy — a remedy that reveals one gap per run is run four times.
#[test]
fn a_half_provisioned_box_refuses_naming_every_gap() {
    let dir = TempDir::new().expect("tmp");
    crate::test_support::wire::mint(dir.path());
    std::fs::remove_file(dir.path().join("server.key")).expect("rm");
    std::fs::remove_file(dir.path().join(ANCHORS)).expect("rm");
    let err = read_dir(dir.path(), Role::Server).expect_err("refused");
    assert!(err.contains("server.key"), "{err}");
    assert!(err.contains(ANCHORS), "{err}");
    assert!(err.contains(REMEDY), "{err}");
}

/// An address file with nothing in it is a gap the file's existence hides, so
/// it is refused by content rather than by presence.
#[test]
fn an_empty_address_refuses() {
    let dir = TempDir::new().expect("tmp");
    crate::test_support::wire::mint(dir.path());
    std::fs::write(dir.path().join(ADDRESS), "  \n").expect("write");
    let err = read_dir(dir.path(), Role::Client).expect_err("refused");
    assert!(err.contains(REMEDY), "{err}");
}

/// The material directory hangs off the yog data root, beside the world and
/// never inside it: a reseed of the world must not be a revocation.
#[test]
fn the_material_sits_beside_the_world_not_inside_it() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let material = super::dir(&world);
    assert_eq!(material.file_name().and_then(|n| n.to_str()), Some(DIR));
    assert_eq!(material.parent(), Some(world.yog_data_root().as_path()));
    assert!(!material.starts_with(crate::world::layout(&world).root));
}

/// A role's leaf names its own certificate and nothing else does.
#[test]
fn a_role_names_its_leaf() {
    assert_eq!(Role::Server.leaf(), "server");
    assert_eq!(Role::Client.leaf(), "client");
}

/// `read` is `read_dir` at the world's own material directory.
#[test]
fn the_world_read_is_the_directory_read() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    assert_eq!(read(&world, Role::Server), Ok(None));
    crate::test_support::wire::mint(&super::dir(&world));
    assert_eq!(
        read(&world, Role::Server),
        read_dir(&super::dir(&world), Role::Server)
    );
}
