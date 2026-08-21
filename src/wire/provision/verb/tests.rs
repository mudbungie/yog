//! `yog wire-certs`: what the environment says, and what the verb does with it.

use super::*;
use crate::test_support::world_under;
use tempfile::TempDir;

/// Nothing stated is loopback at the default port, in the composed world's own
/// material directory — the same place a boot mints into, because there is one.
#[test]
fn an_unstated_plan_is_the_worlds_own_loopback() {
    let tmp = TempDir::new().expect("tmp");
    let world = world_under(tmp.path());
    let plan = plan(&world, None, None, None, None);
    assert_eq!(plan.dir, super::super::super::material::dir(&world));
    assert_eq!(plan.address, format!("{LOOPBACK}:{PORT}"));
    assert!(!plan.force);
}

/// Every stated field is taken, and an empty one is the same as unstated — a
/// `make` variable that expanded to nothing must not become an empty host.
#[test]
fn a_stated_plan_is_taken_and_an_empty_statement_is_not() {
    let tmp = TempDir::new().expect("tmp");
    let world = world_under(tmp.path());
    let stated = plan(
        &world,
        Some("/elsewhere/wire".to_owned()),
        Some("engine.example.com".to_owned()),
        Some("7000".to_owned()),
        Some("1".to_owned()),
    );
    assert_eq!(stated, stated.clone(), "a plan is a value, and says so");
    assert_eq!(stated.dir, PathBuf::from("/elsewhere/wire"));
    assert_eq!(stated.address, "engine.example.com:7000");
    assert!(stated.force);

    let blank = plan(
        &world,
        Some(String::new()),
        Some(String::new()),
        Some(String::new()),
        Some(String::new()),
    );
    assert_eq!(blank.dir, super::super::super::material::dir(&world));
    assert_eq!(blank.address, format!("{LOOPBACK}:{PORT}"));
    assert!(!blank.force, "an empty FORCE is not a rotation");
}

/// The verb mints, then refuses to mint again, then rotates when told to —
/// which is the rotation guard's whole contract.
#[test]
fn it_mints_then_refuses_then_rotates() {
    let tmp = TempDir::new().expect("tmp");
    let dir = tmp.path().join("wire");
    let mut plan = Plan {
        dir: dir.clone(),
        address: "127.0.0.1:0".to_owned(),
        force: false,
    };
    assert_eq!(perform(&plan), 0, "minted");
    let first = std::fs::read(dir.join(ANCHORS)).expect("ca");
    assert_eq!(perform(&plan), 1, "refused: material is already here");
    assert_eq!(
        std::fs::read(dir.join(ANCHORS)).expect("ca"),
        first,
        "and refused without touching it"
    );
    plan.force = true;
    assert_eq!(perform(&plan), 0, "rotated");
    assert_ne!(std::fs::read(dir.join(ANCHORS)).expect("ca"), first);
}

/// A mint that cannot run exits non-zero rather than reporting a directory it
/// did not write.
#[test]
fn a_mint_that_cannot_run_exits_one() {
    let tmp = TempDir::new().expect("tmp");
    let blocked = tmp.path().join("file");
    std::fs::write(&blocked, b"not a directory").expect("file");
    assert_eq!(
        perform(&Plan {
            dir: blocked,
            address: "127.0.0.1:0".to_owned(),
            force: false,
        }),
        1
    );
}

/// The environment readings the process edge performs, in the order the plan
/// takes them — one list, so a rename cannot silently stop being read.
#[test]
fn the_reads_are_named_once() {
    assert_eq!(READS, ["WIRE_DIR", "WIRE_HOST", "WIRE_PORT", "FORCE"]);
}
