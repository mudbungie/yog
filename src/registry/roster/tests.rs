//! The roster: the three reads joined, and what each one contributes
//! (REMOTE §5).

use super::*;
use serde_json::json;
use tempfile::TempDir;

fn client(name: &str) -> Client {
    Client::parse(name).expect("a usable identity")
}

fn tool(name: &str) -> Tool {
    Tool {
        name: name.to_owned(),
        description: "does a thing".to_owned(),
        input_schema: json!({"type": "object"}),
        subject_cwd: false,
    }
}

/// A box with no registry answers nothing — the posture of every box before an
/// operator seats a first client (§4.1).
#[test]
fn a_world_with_no_registry_has_no_clients() {
    let tmp = TempDir::new().expect("tmp");
    assert!(roster(tmp.path(), &Presence::default(), "home").is_empty());
}

/// The registration listing is what makes a client visible in a workspace, and
/// only there: §1.5's trust domain, read straight off the paths.
#[test]
fn only_the_workspaces_registrations_are_answered() {
    let tmp = TempDir::new().expect("tmp");
    super::super::register(tmp.path(), &client("phone"), "home").expect("seated");
    super::super::register(tmp.path(), &client("laptop"), "corp").expect("seated");
    let rows = roster(tmp.path(), &Presence::default(), "home");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].client, "phone");
    assert!(roster(tmp.path(), &Presence::default(), "elsewhere").is_empty());
}

/// **Present or absent, both rendered** (REMOTE §5): a registered client that
/// is not connected is still a fact about the workspace.
#[test]
fn presence_is_answered_and_absence_is_a_row_all_the_same() {
    let tmp = TempDir::new().expect("tmp");
    for name in ["laptop", "phone"] {
        super::super::register(tmp.path(), &client(name), "home").expect("seated");
    }
    let presence = Presence::default();
    let _live = presence.enter(&client("phone"));
    let rows = roster(tmp.path(), &presence, "home");
    assert_eq!(
        rows.iter()
            .map(|r| (r.client.clone(), r.present))
            .collect::<Vec<_>>(),
        vec![("laptop".to_owned(), false), ("phone".to_owned(), true)],
        "sorted by identity, presence read per row"
    );
}

/// The advertised set rides beside the presence, and it stands whether or not
/// the client is connected — the durable half of §5's two facts.
#[test]
fn each_row_carries_what_that_client_advertises() {
    let tmp = TempDir::new().expect("tmp");
    let laptop = client("laptop");
    super::super::register(tmp.path(), &laptop, "home").expect("seated");
    super::super::register(tmp.path(), &client("phone"), "home").expect("seated");
    tools::store(tmp.path(), &laptop, &[tool("Bash")]).expect("stored");
    let rows = roster(tmp.path(), &Presence::default(), "home");
    assert_eq!(rows[0].tools, vec![tool("Bash")]);
    assert!(!rows[0].present, "advertisement outlives the connection");
    assert!(rows[1].tools.is_empty(), "a client that never advertised");
}

/// `local` owns a directory here and is not a client:
/// the reservation that refuses the name is the whole of the filter.
#[test]
fn the_reserved_local_directory_is_never_a_row() {
    let tmp = TempDir::new().expect("tmp");
    let dir = super::super::dir(tmp.path(), &Client::local()).join(super::super::WORKSPACES);
    std::fs::create_dir_all(&dir).expect("mkdir");
    std::fs::write(dir.join("home"), []).expect("touch");
    assert!(roster(tmp.path(), &Presence::default(), "home").is_empty());
}
