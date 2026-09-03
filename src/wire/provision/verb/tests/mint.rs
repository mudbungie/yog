//! **The mint act**: the guard in front of it, the rotation it names, and the
//! re-issue a stated host asks for on a directory that already holds material.

use super::super::super::ANCHORS;
use super::super::{Act, Plan, hosts, perform};
use crate::wire::material::{ADDRESS, Role};
use tempfile::TempDir;

/// The verb mints, then refuses to mint again, then rotates when told to —
/// which is the rotation guard's whole contract.
#[test]
fn it_mints_then_refuses_then_rotates() {
    let tmp = TempDir::new().expect("tmp");
    let dir = tmp.path().join("wire");
    let mut plan = Plan {
        dir: dir.clone(),
        act: Act::Mint {
            hosts: Vec::new(),
            port: "0".to_owned(),
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
        hosts: Vec::new(),
        port: "0".to_owned(),
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
                hosts: Vec::new(),
                port: "0".to_owned(),
                force: false,
            },
        }),
        1
    );
}

/// **`WIRE_HOST` is a list** (bl-52f4): a box reachable three ways says so
/// once. The reading it replaces is a list of one, an empty statement is still
/// no host at all, and the debris a `make` variable leaves — a trailing comma,
/// space around an entry — is dropped rather than becoming an empty host.
#[test]
fn a_stated_host_is_a_list_of_them() {
    assert_eq!(hosts(None), Vec::<String>::new());
    assert_eq!(hosts(Some("")), Vec::<String>::new());
    assert_eq!(hosts(Some(", ,")), Vec::<String>::new());
    assert_eq!(hosts(Some("engine.example.com")), ["engine.example.com"]);
    assert_eq!(
        hosts(Some(" engine.example.com , 192.0.2.7 ,")),
        ["engine.example.com", "192.0.2.7"]
    );
}

/// **A stated host on standing material re-issues the server leaf** (bl-52f4),
/// which is the act the refusal above cannot perform: the CA stands, every leaf
/// already issued still verifies, and the address file is left naming the one
/// endpoint the engine binds. The signal is `WIRE_HOST` itself — there is no
/// new reading and no new verb.
#[test]
fn a_stated_host_over_standing_material_re_issues_the_server_leaf() {
    let tmp = TempDir::new().expect("tmp");
    let dir = tmp.path().join("wire");
    let stated = |hosts: &[&str]| Plan {
        dir: dir.clone(),
        act: Act::Mint {
            hosts: hosts.iter().map(|h| (*h).to_owned()).collect(),
            port: "7737".to_owned(),
            force: false,
        },
    };
    assert_eq!(perform(&stated(&[])), 0, "minted");
    let ca = std::fs::read(dir.join(ANCHORS)).expect("ca");
    let address = std::fs::read(dir.join(ADDRESS)).expect("address");
    let client = std::fs::read(dir.join("client.pem")).expect("client leaf");
    let server = std::fs::read(dir.join("server.pem")).expect("server leaf");

    assert_eq!(
        perform(&stated(&["engine.example.com", "192.0.2.7"])),
        0,
        "re-issued rather than refused"
    );
    assert_ne!(
        std::fs::read(dir.join("server.pem")).expect("server leaf"),
        server,
        "the one artifact the act replaces"
    );
    assert_eq!(std::fs::read(dir.join(ANCHORS)).expect("ca"), ca);
    assert_eq!(
        std::fs::read(dir.join("client.pem")).expect("client leaf"),
        client
    );
    assert_eq!(std::fs::read(dir.join(ADDRESS)).expect("address"), address);
    assert!(Role::Server.leaf() == "server");
}

/// A re-issue that cannot run exits non-zero rather than reporting a leaf it
/// did not write: a box holding an operator's anchors and no CA key issues
/// nothing, which is the one guard both acts over a standing root share.
#[test]
fn a_re_issue_a_client_box_cannot_perform_exits_one() {
    let tmp = TempDir::new().expect("tmp");
    std::fs::write(tmp.path().join(ANCHORS), b"an operator's anchors").expect("anchors");
    assert_eq!(
        perform(&Plan {
            dir: tmp.path().to_owned(),
            act: Act::Mint {
                hosts: vec!["engine.example.com".to_owned()],
                port: "7737".to_owned(),
                force: false,
            },
        }),
        1
    );
}
