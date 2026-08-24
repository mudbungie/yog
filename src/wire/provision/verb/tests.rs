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
    let plan = plan(&world, None, None, None, None, None);
    assert_eq!(plan.dir, super::super::super::material::dir(&world));
    assert_eq!(
        plan.act,
        Act::Mint {
            address: format!("{LOOPBACK}:{PORT}"),
            force: false,
        },
        "nothing stated is the mint it has always been"
    );
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
        None,
    );
    assert_eq!(stated, stated.clone(), "a plan is a value, and says so");
    assert_eq!(stated.dir, PathBuf::from("/elsewhere/wire"));
    assert_eq!(
        stated.act,
        Act::Mint {
            address: "engine.example.com:7000".to_owned(),
            force: true,
        }
    );

    let blank = plan(
        &world,
        Some(String::new()),
        Some(String::new()),
        Some(String::new()),
        Some(String::new()),
        Some(String::new()),
    );
    assert_eq!(blank.dir, super::super::super::material::dir(&world));
    assert_eq!(
        blank.act,
        Act::Mint {
            address: format!("{LOOPBACK}:{PORT}"),
            force: false,
        },
        "an empty FORCE is not a rotation, and an empty WIRE_LEAF is not a leaf"
    );
}

/// `WIRE_LEAF` selects the other act outright (REMOTE §8.2): the address and
/// the rotation flag are a mint's facts, and a leaf is issued over the trust
/// root already there — so there is no state in which both are folded.
#[test]
fn a_stated_leaf_is_the_other_act_and_takes_no_mint_facts() {
    let tmp = TempDir::new().expect("tmp");
    let world = world_under(tmp.path());
    let stated = plan(
        &world,
        None,
        Some("engine.example.com".to_owned()),
        Some("7000".to_owned()),
        Some("1".to_owned()),
        Some("phone".to_owned()),
    );
    assert_eq!(stated.dir, super::super::super::material::dir(&world));
    assert_eq!(stated.act, Act::Leaf("phone".to_owned()));
}

/// The verb mints, then refuses to mint again, then rotates when told to —
/// which is the rotation guard's whole contract.
#[test]
fn it_mints_then_refuses_then_rotates() {
    let tmp = TempDir::new().expect("tmp");
    let dir = tmp.path().join("wire");
    let mut plan = Plan {
        dir: dir.clone(),
        act: Act::Mint {
            address: "127.0.0.1:0".to_owned(),
            force: false,
        },
    };
    assert_eq!(perform(&plan), 0, "minted");
    let first = std::fs::read(dir.join(ANCHORS)).expect("ca");
    assert_eq!(perform(&plan), 1, "refused: material is already here");
    assert_eq!(
        std::fs::read(dir.join(ANCHORS)).expect("ca"),
        first,
        "and refused without touching it"
    );
    plan.act = Act::Mint {
        address: "127.0.0.1:0".to_owned(),
        force: true,
    };
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
            act: Act::Mint {
                address: "127.0.0.1:0".to_owned(),
                force: false,
            },
        }),
        1
    );
}

/// The leaf act end to end: the mint first, then one extra leaf over it, then
/// the refusal to issue a second under the same name. Nothing but that pair is
/// written — no CA is founded, no address is touched, no other leaf appears.
#[test]
fn the_leaf_act_issues_once_and_writes_nothing_else() {
    let tmp = TempDir::new().expect("tmp");
    let dir = tmp.path().join("wire");
    let leaf = Plan {
        dir: dir.clone(),
        act: Act::Leaf("phone".to_owned()),
    };
    assert_eq!(perform(&leaf), 1, "nothing to issue under yet");

    assert_eq!(
        perform(&Plan {
            dir: dir.clone(),
            act: Act::Mint {
                address: "127.0.0.1:0".to_owned(),
                force: false,
            },
        }),
        0,
        "minted"
    );
    let ca = std::fs::read(dir.join(ANCHORS)).expect("ca");
    let address = std::fs::read(dir.join(super::super::ADDRESS)).expect("address");

    assert_eq!(perform(&leaf), 0, "issued");
    assert_eq!(perform(&leaf), 1, "and refuses a second under one name");

    let mut left: Vec<String> = std::fs::read_dir(&dir)
        .expect("dir")
        .filter_map(|e| Some(e.ok()?.file_name().to_string_lossy().into_owned()))
        .collect();
    left.sort();
    let mut named = super::super::artifacts();
    named.push("phone.key".to_owned());
    named.push("phone.pem".to_owned());
    named.sort();
    assert_eq!(
        left, named,
        "the mint's artifacts and exactly one more pair"
    );
    assert_eq!(std::fs::read(dir.join(ANCHORS)).expect("ca"), ca);
    assert_eq!(
        std::fs::read(dir.join(super::super::ADDRESS)).expect("address"),
        address
    );
}

/// A refusal the operator can act on: an unusable common name is the
/// registry's own sentence, and nothing is written for it.
#[test]
fn an_unusable_common_name_refuses() {
    let tmp = TempDir::new().expect("tmp");
    assert_eq!(
        perform(&Plan {
            dir: tmp.path().to_owned(),
            act: Act::Leaf("a/b".to_owned()),
        }),
        1
    );
    assert_eq!(std::fs::read_dir(tmp.path()).expect("dir").count(), 0);
}

/// The environment readings the process edge performs, in the order the plan
/// takes them — one list, so a rename cannot silently stop being read.
#[test]
fn the_reads_are_named_once() {
    assert_eq!(
        READS,
        ["WIRE_DIR", "WIRE_HOST", "WIRE_PORT", "FORCE", "WIRE_LEAF"]
    );
}
