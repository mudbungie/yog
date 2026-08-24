//! What a box holds when it participates in workspaces elsewhere — and what it
//! holds when it does not, which must be exactly what it held before §8.2.

use super::*;
use crate::wire::material::{ADDRESS, ANCHORS};
use std::path::PathBuf;
use tempfile::TempDir;

/// A host an entry names. Loopback, because a fixture never dials it and a
/// routable address in the tree is a disclosure.
const HOST: &str = "127.0.0.1:7737";

/// Write a whole entry under `root`'s entries directory: the four files
/// `Role::Client` wants, and nothing else.
fn provision(root: &Path, leaf: &str) -> PathBuf {
    let dir = root.join(ENTRIES).join(leaf);
    std::fs::create_dir_all(&dir).expect("mkdir");
    for name in [ANCHORS, "client.pem", "client.key"] {
        std::fs::write(dir.join(name), "-----PEM-----\n").expect("write");
    }
    std::fs::write(dir.join(ADDRESS), format!("{HOST}\n")).expect("write");
    dir
}

/// **Migration: none.** A box with a flat `wire/` and no `workspaces/`
/// directory is the general path with zero entries: the flat read answers
/// exactly what it answered before this module existed, and the entry set is
/// empty rather than absent, erroring or defaulted.
#[test]
fn a_box_with_no_entries_directory_behaves_as_it_always_did() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let flat = material::dir(&world);
    // Before anything is provisioned at all.
    assert_eq!(entries(&world), Vec::new());
    assert_eq!(material::read(&world, Role::Client), Ok(None));
    // And with the box's own root minted, which is the deployed shape.
    crate::test_support::wire::mint(&flat);
    let before = material::read(&world, Role::Client);
    assert!(before.as_ref().is_ok_and(Option::is_some), "{before:?}");
    assert_eq!(entries(&world), Vec::new());
    assert_eq!(material::read(&world, Role::Client), before);
    assert!(!flat.join(ENTRIES).exists());
}

/// An entry directory that will not read at all — a regular file where the
/// directory would be — is zero entries too. Absent and unreadable are one
/// fact: this box holds no workspace elsewhere.
#[test]
fn an_unreadable_entries_directory_is_zero_entries() {
    let tmp = TempDir::new().expect("tmp");
    std::fs::write(tmp.path().join(ENTRIES), "not a directory").expect("write");
    assert_eq!(read_dir(&tmp.path().join(ENTRIES)), Vec::new());
}

/// §8.2's own migration sentence, executed: a flat client set aimed at another
/// machine becomes the entry it always was with one `mkdir` and one `mv`, and
/// `material::read_dir` reads it unchanged.
#[test]
fn an_entry_is_the_flat_directory_one_level_down() {
    let tmp = TempDir::new().expect("tmp");
    let flat = tmp.path().join("carried");
    crate::test_support::wire::mint(&flat);
    let root = tmp.path().join(material::DIR);
    std::fs::create_dir_all(root.join(ENTRIES)).expect("mkdir");
    std::fs::rename(&flat, root.join(ENTRIES).join("attic")).expect("mv");

    let held = read_dir(&root.join(ENTRIES));
    let [entry] = held.as_slice() else {
        panic!("one entry, got {held:?}")
    };
    assert_eq!(entry.leaf, "attic");
    let direct = material::read_dir(&root.join(ENTRIES).join("attic"), Role::Client).expect("read");
    assert_eq!(entry.channel.clone().ok(), direct);
}

/// Several entries come back sorted by leaf, so a roster's order is the
/// directory's names and not the filesystem's mood. A stray file beside them
/// names no intent and is not an entry.
#[test]
fn entries_are_sorted_by_leaf_and_files_are_not_entries() {
    let tmp = TempDir::new().expect("tmp");
    for leaf in ["oxide", "attic", "cobalt"] {
        provision(tmp.path(), leaf);
    }
    std::fs::write(tmp.path().join(ENTRIES).join(".stray"), "").expect("write");
    let held = read_dir(&tmp.path().join(ENTRIES));
    let leaves: Vec<&str> = held.iter().map(|e| e.leaf.as_str()).collect();
    assert_eq!(leaves, ["attic", "cobalt", "oxide"]);
    for entry in &held {
        assert_eq!(
            entry.channel.as_ref().map(|m| m.address.clone()),
            Ok(HOST.to_owned())
        );
    }
}

/// Half an entry refuses as its own channel — naming every gap at once, and
/// leaving every other entry standing. The refusal discipline is per entry;
/// the whole shell is reserved for the wire the window cannot exist without.
#[test]
fn a_half_provisioned_entry_refuses_alone() {
    let tmp = TempDir::new().expect("tmp");
    let broken = provision(tmp.path(), "attic");
    provision(tmp.path(), "cobalt");
    std::fs::remove_file(broken.join("client.key")).expect("rm");

    let held = read_dir(&tmp.path().join(ENTRIES));
    let [attic, cobalt] = held.as_slice() else {
        panic!("two entries, got {held:?}")
    };
    let refusal = attic.channel.as_ref().expect_err("refused");
    assert!(refusal.contains("client.key"), "{refusal}");
    assert!(refusal.contains(REMEDY), "{refusal}");
    assert!(cobalt.channel.is_ok(), "{cobalt:?}");
}

/// An entry directory with nothing in it is a refusal, not silence: absence is
/// the off switch at the flat root, but a directory somebody made names an
/// intent no material stands behind.
#[test]
fn an_empty_entry_refuses_rather_than_going_quiet() {
    let tmp = TempDir::new().expect("tmp");
    let dir = tmp.path().join(ENTRIES).join("attic");
    std::fs::create_dir_all(&dir).expect("mkdir");
    let held = read_dir(&tmp.path().join(ENTRIES));
    let [entry] = held.as_slice() else {
        panic!("one entry, got {held:?}")
    };
    let refusal = entry.channel.as_ref().expect_err("refused");
    assert!(refusal.contains("attic"), "{refusal}");
    assert!(refusal.contains(REMEDY), "{refusal}");
    // Its name still reads: a channel that cannot be dialled is still a
    // workspace the roster can paint unreachable.
    assert_eq!(entry.workspace, "attic");
}

/// The `workspace` file is optional and states the host's own name for the
/// workspace. Absent, empty and whitespace are one answer — the leaf — because
/// they are one fact: the entry states no host-side name.
#[test]
fn the_optional_workspace_file_names_the_host_side_workspace() {
    let tmp = TempDir::new().expect("tmp");
    let dir = provision(tmp.path(), "attic");
    let read = |dir: &Path| entry(dir).workspace;
    assert_eq!(read(&dir), "attic");
    std::fs::write(dir.join(WORKSPACE), "  \n").expect("write");
    assert_eq!(read(&dir), "attic");
    std::fs::write(dir.join(WORKSPACE), " home\n").expect("write");
    assert_eq!(read(&dir), "home");
}

/// `entries` is `read_dir` at the world's own entries directory, which sits
/// inside `wire/` — an entry is the same operator-provisioned, irreplaceable
/// class of fact the anchors are, so a reseed of the world never touches it.
#[test]
fn the_world_read_is_the_directory_read() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world).join(ENTRIES);
    provision(&material::dir(&world), "attic");
    assert!(dir.starts_with(material::dir(&world)));
    assert!(!dir.starts_with(crate::world::layout(&world).root));
    assert_eq!(entries(&world), read_dir(&dir));
    assert_eq!(entries(&world).len(), 1);
}
