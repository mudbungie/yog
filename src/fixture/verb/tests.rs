//! The verb's decision, which is pure, and the act, which is not.

use super::*;
use crate::fixture::roster;
use std::time::Duration;
use tempfile::TempDir;

/// A world anchored somewhere no fixture will ever be laid.
fn ambient(root: &str) -> Env {
    Env::from_pairs([("XDG_DATA_HOME", root.to_owned())])
}

/// No subject is the roster, and an empty one is no subject — a `make`
/// variable that expanded to nothing must not be a state name.
#[test]
fn a_nameless_invocation_lists_the_roster() {
    let env = ambient("/live");
    assert_eq!(plan(&env, None, None, None, None), Plan::List);
    assert_eq!(
        plan(&env, Some(String::new()), None, None, None),
        Plan::List
    );
    assert_eq!(perform(&Plan::List), 0);
}

/// An unknown name is refused **with the roster**, because a refusal that does
/// not say what would have worked costs a second run.
#[test]
fn an_unknown_state_is_refused_naming_every_known_one() {
    let plan = plan(&ambient("/live"), Some("nope".to_owned()), None, None, None);
    let Plan::Refuse(sentence) = &plan else {
        panic!("expected a refusal, got {plan:?}");
    };
    assert!(sentence.contains("\"nope\""));
    for name in roster::names() {
        assert!(sentence.contains(&name), "{name} unnamed in the refusal");
    }
    assert_eq!(perform(&plan), 1);
}

/// The address is stated before anything binds: the host and port readings when
/// they are given, and a free port asked of the kernel when they are not.
#[test]
fn the_address_is_stated_from_the_readings_or_a_free_port() {
    let env = ambient("/live");
    let stated = plan(
        &env,
        Some("busy".to_owned()),
        Some("/tmp/fx".to_owned()),
        Some("engine.example.com".to_owned()),
        Some("7737".to_owned()),
    );
    assert_eq!(
        stated,
        Plan::Lay {
            state: "busy".to_owned(),
            root: PathBuf::from("/tmp/fx"),
            address: "engine.example.com:7737".to_owned(),
        }
    );
    let Plan::Lay { address, root, .. } = plan(&env, Some("busy".to_owned()), None, None, None)
    else {
        panic!("expected a lay");
    };
    let port = address.strip_prefix("127.0.0.1:").expect("loopback");
    assert!(port.parse::<u16>().expect("a port") > 0, "not :0");
    assert!(root.ends_with("yog/fixture/busy"), "{}", root.display());
}

/// A bind the kernel would not answer falls back to the one stated default,
/// and a clock outside `i64` reads as its edge rather than panicking.
#[test]
fn the_impure_edges_answer_as_values() {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 4242));
    assert_eq!(port_of(Ok(addr)), "4242");
    assert_eq!(
        port_of(Err(std::io::Error::other("no"))),
        crate::wire::provision::PORT
    );
    assert_eq!(unix_of(Ok(Duration::from_secs(7))), 7);
    assert_eq!(unix_of(Ok(Duration::from_secs(u64::MAX))), i64::MAX);
    let before = std::time::UNIX_EPOCH - Duration::from_secs(1);
    assert_eq!(unix_of(before.duration_since(std::time::UNIX_EPOCH)), 0);
    assert!(now_unix() > 1_700_000_000, "a real clock");
    assert!(free_port().parse::<u16>().expect("a port") > 0);
}

/// **The live-world guard**, both directions and the near miss. A lay wipes its
/// root, so a root that contains the operator's world — or sits inside it — is
/// refused before anything is removed.
#[test]
fn a_root_overlapping_the_live_world_is_refused_both_ways() {
    let refuses = |root: &str, live: &str| {
        matches!(
            plan(
                &ambient(live),
                Some("busy".to_owned()),
                Some(root.to_owned()),
                None,
                None
            ),
            Plan::Refuse(_)
        )
    };
    // The fixture root IS the data root's parent, and the data root is
    // `<XDG_DATA_HOME>/yog`, so this contains the live world.
    assert!(refuses("/live", "/live"));
    assert!(refuses("/live/yog/inside", "/live"));
    assert!(refuses("/", "/live"));
    // A sibling whose name merely starts the same is not an overlap — the
    // trailing separator is what tells them apart.
    assert!(!refuses("/live-elsewhere", "/live"));
    assert!(!refuses("/tmp/fx", "/live"));
    let Plan::Refuse(sentence) = plan(
        &ambient("/live"),
        Some("busy".to_owned()),
        Some("/live".to_owned()),
        None,
        None,
    ) else {
        panic!("expected a refusal");
    };
    assert!(sentence.contains("LIVE world"));
    assert!(sentence.contains(READS[0]));
}

/// One state per lay: a second word is a setting in the wrong place, and it
/// refuses rather than vanishing into a default.
#[test]
fn a_second_word_is_refused_naming_the_prefix_spelling() {
    assert!(stray(&[]).is_none());
    assert!(stray(&["busy".to_owned()]).is_none());
    let sentence = stray(&["busy".to_owned(), "/tmp/x".to_owned()]).expect("refusal");
    assert!(sentence.contains("\"/tmp/x\""));
    assert!(sentence.contains(READS[0]));
}

/// The act, end to end: it lays, it mints, and every path it answers with is
/// really there — the whole of what a consumer is promised.
#[test]
fn a_lay_answers_paths_that_exist() {
    let tmp = TempDir::new().expect("tmp");
    let root = tmp.path().join("state");
    let laid = perform_lay("wound", &root, "127.0.0.1:7737").expect("lay");
    assert_eq!(laid.state, "wound");
    assert_eq!(laid.address, "127.0.0.1:7737");
    assert!(laid.anchors.is_file(), "the CA");
    assert!(laid.chain.is_file(), "the client leaf");
    assert!(laid.key.is_file(), "its key");
    assert!(laid.origin > 1_700_000_000);
    // The material is readable as the client's, aimed where it says.
    let material = crate::wire::material::read_dir(
        &laid.root.join("yog").join(crate::wire::material::DIR),
        Role::Client,
    )
    .expect("read")
    .expect("provisioned");
    assert_eq!(material.address, "127.0.0.1:7737");
    assert_eq!(
        perform(&Plan::Lay {
            state: "wound".to_owned(),
            root,
            address: "127.0.0.1:7737".to_owned(),
        }),
        0
    );
}

/// **A lay starts from nothing.** The root is wiped first, so a second lay of a
/// different state into one root cannot leave the first one's conversations
/// standing beside it.
#[test]
fn a_second_lay_replaces_the_first() {
    let tmp = TempDir::new().expect("tmp");
    let root = tmp.path().join("state");
    perform_lay("busy", &root, "127.0.0.1:1").expect("first");
    let laid = perform_lay("wound", &root, "127.0.0.1:1").expect("second");
    let places = Places::under(&laid.root);
    let ws = places.workspace(roster::WORKSPACE);
    assert!(ws.join("agents/c-101").is_dir(), "the wound state");
    assert!(!ws.join("agents/c-001").exists(), "the busy state is gone");
}

/// Every refusal the act can reach is a sentence, and none of them panics.
#[test]
fn the_act_refuses_in_words() {
    let tmp = TempDir::new().expect("tmp");
    assert!(
        perform_lay("nope", &tmp.path().join("x"), "127.0.0.1:1")
            .expect_err("unknown")
            .contains("not a state")
    );
    // A root that is a file cannot be cleared as a directory.
    let file = tmp.path().join("f");
    std::fs::write(&file, "x").expect("write");
    assert!(
        perform_lay("busy", &file, "127.0.0.1:1")
            .expect_err("clear")
            .contains("clear")
    );
    // A root that cannot be written refuses out of the writer.
    let under = file.join("under");
    assert!(perform_lay("busy", &under, "127.0.0.1:1").is_err());
    assert_eq!(
        perform(&Plan::Lay {
            state: "busy".to_owned(),
            root: under,
            address: "127.0.0.1:1".to_owned(),
        }),
        1
    );
}
