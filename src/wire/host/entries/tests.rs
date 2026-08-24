//! What a tool host serves, and what it says about what it cannot — including
//! the box that holds no entry at all, which must read exactly as it read
//! before §8.2 existed.

use super::*;
use crate::test_support::wire::{NO_LISTENER, mint};
use crate::test_support::world_under;
use crate::wire::entries::ENTRIES;
use crate::wire::material::{ADDRESS, ANCHORS};
use std::path::Path;
use tempfile::TempDir;

/// A stated address, so the flat root is dialable at all: a mint writes `:0`,
/// which is a request only the engine that bound it can answer (bl-dc14).
fn stated(dir: &Path) {
    std::fs::write(dir.join(ADDRESS), NO_LISTENER).expect("address");
}

/// An entry whose four files exist but whose certificates are text. Enough for
/// the channel set, which reads a directory and dials nothing.
fn shaped(root: &Path, leaf: &str) -> std::path::PathBuf {
    let dir = root.join(ENTRIES).join(leaf);
    std::fs::create_dir_all(&dir).expect("mkdir");
    for name in [ANCHORS, "client.pem", "client.key"] {
        std::fs::write(dir.join(name), "-----PEM-----\n").expect("write");
    }
    std::fs::write(dir.join(ADDRESS), NO_LISTENER).expect("write");
    dir
}

/// **Migration: none.** A box with a flat `wire/` and no `workspaces/`
/// directory serves one channel — its own root, unnamed — which is the whole
/// of what a tool host ever served.
#[test]
fn a_box_with_no_entries_serves_exactly_its_flat_root() {
    let tmp = TempDir::new().expect("tmp");
    let world = world_under(tmp.path());
    let root = crate::wire::material::dir(&world);
    mint(&root);
    stated(&root);
    let (held, refused) = channels(&world);
    assert!(refused.is_empty(), "{refused:?}");
    assert_eq!(held.len(), 1);
    assert_eq!(held.first().and_then(|c| c.name.clone()), None);
    assert_eq!(
        held.first().map(|c| c.said("stopped")),
        Some("stopped".to_owned()),
        "the box's own root is not one relationship among others, so it is \
         not labelled like one"
    );
}

/// A box with no wire at all holds no channel, and what it refuses with is the
/// flat root's own sentence — verbatim and alone, the one `yog seat` gives.
#[test]
fn a_box_with_no_channel_refuses_in_the_flat_roots_own_words() {
    let tmp = TempDir::new().expect("tmp");
    let world = world_under(tmp.path());
    let (held, refused) = channels(&world);
    assert!(held.is_empty());
    assert_eq!(refused, vec![seat::flat(&world).expect_err("no wire")]);
}

/// **A refusal is one entry's, never the set's.** A half-provisioned entry is
/// said once, naming itself, while its neighbours are served.
#[test]
fn a_half_provisioned_entry_is_said_once_and_its_neighbours_are_served() {
    let tmp = TempDir::new().expect("tmp");
    let world = world_under(tmp.path());
    let root = crate::wire::material::dir(&world);
    mint(&root);
    stated(&root);
    shaped(&root, "good");
    let bad = root.join(ENTRIES).join("bad");
    std::fs::create_dir_all(&bad).expect("mkdir");
    std::fs::write(bad.join(ANCHORS), "-----PEM-----\n").expect("write");

    let (held, refused) = channels(&world);
    let names: Vec<Option<String>> = held.iter().map(|c| c.name.clone()).collect();
    assert_eq!(names, vec![None, Some("good".to_owned())]);
    assert_eq!(refused.len(), 1, "{refused:?}");
    let said = refused.first().cloned().unwrap_or_default();
    assert!(
        said.contains("half-provisioned") && said.contains("bad"),
        "{said}"
    );
}

/// The fan-out: every channel served at once, each answering the sentence that
/// stopped it, and each labelled by the entry it is — so a box holding several
/// can tell which one spoke. Nothing is listening at any of the three
/// addresses, which is how each loop ends promptly; the third's certificates
/// are text, so it never opens a seat at all.
#[test]
fn every_channel_is_served_and_each_answers_for_itself() {
    let tmp = TempDir::new().expect("tmp");
    let world = world_under(tmp.path());
    let root = crate::wire::material::dir(&world);
    mint(&root);
    stated(&root);
    let dialable = root.join(ENTRIES).join("dialable");
    std::fs::create_dir_all(&dialable).expect("mkdir");
    for name in [ANCHORS, "client.pem", "client.key"] {
        std::fs::copy(root.join(name), dialable.join(name)).expect("copy");
    }
    stated(&dialable);
    shaped(&root, "unreadable");

    let (held, refused) = channels(&world);
    assert!(refused.is_empty(), "{refused:?}");
    assert_eq!(held.len(), 3);
    let said = fan(&[], held);
    let lines: Vec<&str> = said.lines().collect();
    assert_eq!(lines.len(), 3, "{said}");
    assert!(
        lines.iter().any(|l| l.starts_with("dialable: connect")),
        "an entry's sentence names the entry: {said}"
    );
    assert!(
        lines.iter().any(|l| l.starts_with("unreadable: ")),
        "a seat that will not open is that channel's refusal: {said}"
    );
    assert!(
        lines.iter().any(|l| l.starts_with("connect ")),
        "and the flat root's is bare: {said}"
    );
}

/// A box whose flat root is only self-provisioned — `:0`, the request only its
/// own engine can answer — still serves every entry it holds. §8.2's flat
/// directory is the box's own root and nothing more: a relationship it cannot
/// dial subtracts one channel, never the set.
#[test]
fn a_self_provisioned_flat_root_costs_only_its_own_channel() {
    let tmp = TempDir::new().expect("tmp");
    let world = world_under(tmp.path());
    let root = crate::wire::material::dir(&world);
    mint(&root);
    shaped(&root, "elsewhere");

    let (held, refused) = channels(&world);
    let names: Vec<Option<String>> = held.iter().map(|c| c.name.clone()).collect();
    assert_eq!(names, vec![Some("elsewhere".to_owned())]);
    assert_eq!(refused, vec![seat::flat(&world).expect_err(":0")]);
}
