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
            port: Some("0".to_owned()),
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
        port: Some("0".to_owned()),
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
                port: Some("0".to_owned()),
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

/// **A stated host on standing material re-issues the server leaf** (bl-52f4)
/// **and states the address** (bl-98ef), which is the act the refusal above
/// cannot perform: the CA stands and every leaf already issued still verifies.
/// The signal is `WIRE_HOST` itself — there is no new reading and no new verb.
#[test]
fn a_stated_host_over_standing_material_re_issues_the_server_leaf() {
    let tmp = TempDir::new().expect("tmp");
    let dir = tmp.path().join("wire");
    let stated = |hosts: &[&str]| Plan {
        dir: dir.clone(),
        act: Act::Mint {
            hosts: hosts.iter().map(|h| (*h).to_owned()).collect(),
            port: Some("7737".to_owned()),
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
    assert_ne!(
        std::fs::read(dir.join(ADDRESS)).expect("address"),
        address,
        "the endpoint is stated, not left behind"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join(ADDRESS)).expect("address"),
        "engine.example.com:7737\n",
        "the first host stated, on the port stated"
    );
    assert!(Role::Server.leaf() == "server");
}

/// **The box a self-provisioned boot left behind is the one this act is for**
/// (bl-98ef). Its `address` reads `127.0.0.1:0` — a request only the listener
/// ever learns the answer to, which no seat can dial and no enrollment can put
/// in a QR — and the only act that could replace it was the `FORCE=1` that
/// distrusts every leaf already carried away. A stated endpoint writes it, over
/// the CA already here, and nothing stops verifying.
#[test]
fn a_stated_endpoint_replaces_a_self_provisioned_request() {
    let tmp = TempDir::new().expect("tmp");
    let dir = tmp.path().join("wire");
    super::super::super::ensure(&dir).expect("the boot's own mint");
    assert_eq!(
        std::fs::read_to_string(dir.join(ADDRESS)).expect("address"),
        "127.0.0.1:0\n",
        "what a boot with nothing to read writes"
    );
    let ca = std::fs::read(dir.join(ANCHORS)).expect("ca");

    assert_eq!(
        perform(&Plan {
            dir: dir.clone(),
            act: Act::Mint {
                hosts: vec!["127.0.0.1".to_owned()],
                port: Some("7752".to_owned()),
                force: false,
            },
        }),
        0
    );
    assert_eq!(
        std::fs::read_to_string(dir.join(ADDRESS)).expect("address"),
        "127.0.0.1:7752\n"
    );
    assert_eq!(
        std::fs::read(dir.join(ANCHORS)).expect("ca"),
        ca,
        "and the trust root is untouched, which is what makes it not a rotation"
    );
}

/// **An unstated port keeps the standing one, and a `:0` is not one** (bl-98ef).
/// An operator widening the SAN of a box that binds 7752 states no port, and
/// moving its endpoint to the default underneath them would be this act
/// breaking the box it was asked to describe — while a `:0` names no endpoint
/// to keep, so a statement over it lands on the default a machine can be told
/// to dial.
#[test]
fn an_unstated_port_keeps_the_standing_endpoint() {
    let tmp = TempDir::new().expect("tmp");
    let dir = tmp.path().join("wire");
    let widen = |hosts: &[&str]| Plan {
        dir: dir.clone(),
        act: Act::Mint {
            hosts: hosts.iter().map(|h| (*h).to_owned()).collect(),
            port: None,
            force: false,
        },
    };
    assert_eq!(perform(&widen(&[])), 0, "minted at the default port");
    super::super::super::state(&dir, "127.0.0.1:7752").expect("an operator's own endpoint");

    assert_eq!(perform(&widen(&["127.0.0.1", "192.0.2.7"])), 0);
    assert_eq!(
        std::fs::read_to_string(dir.join(ADDRESS)).expect("address"),
        "127.0.0.1:7752\n",
        "the port the operator stated, not the default"
    );

    super::super::super::state(&dir, "127.0.0.1:0").expect("a self-provisioned request");
    assert_eq!(perform(&widen(&["127.0.0.1"])), 0);
    assert_eq!(
        std::fs::read_to_string(dir.join(ADDRESS)).expect("address"),
        "127.0.0.1:7737\n",
        "a `:0` names no endpoint to keep"
    );
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
                port: Some("7737".to_owned()),
                force: false,
            },
        }),
        1
    );
}
