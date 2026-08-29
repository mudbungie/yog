//! **Which engine a gesture reaches, and what name crosses to it** (REMOTE
//! §8.2): the entries this box holds are resolved first, and everything else —
//! a name no entry holds, a gesture naming no workspace, a box with no
//! `workspaces/` directory at all — goes where it always went.

use super::*;
/// An engine that says yes only to the workspace name it expects — so one exit
/// code proves both **which** engine was reached and **what name** crossed to
/// it, which is the whole of what §8.2 resolution has to get right.
struct Expects(String);

impl Answerer for Expects {
    fn answer(
        &self,
        _peer: &crate::registry::Peer,
        request: Value,
    ) -> Box<dyn Iterator<Item = Value>> {
        let named = request
            .get("workspace")
            .and_then(Value::as_str)
            .unwrap_or_default();
        Box::new(std::iter::once(
            json!({"ok": named == self.0, "kind": "echo"}),
        ))
    }
}

/// One provisioned engine at `dir`: material minted the way an operator mints
/// it, a listener bound on it, and the `address` file naming the port the
/// kernel actually gave. An entry directory is the flat shape one level down,
/// so the same helper builds either.
fn listening(dir: &std::path::Path, expects: &str) -> Listener {
    mint(dir);
    let listener = Listener::bind(
        &fixture(dir, Role::Server, crate::test_support::wire::EPHEMERAL),
        Arc::new(Expects(expects.to_owned())),
        Presence::default(),
    )
    .expect("bind");
    std::fs::write(dir.join(material::ADDRESS), listener.address()).expect("address");
    listener
}

/// The entry directory for `leaf` under a world's wire material.
fn entry_dir(world: &crate::xdg::Env, leaf: &str) -> std::path::PathBuf {
    material::dir(world).join(entries::ENTRIES).join(leaf)
}

/// **A gesture naming an entry goes down that entry's channel** (§8.2): a
/// different engine, on different material, reached by the client's own name
/// for the workspace. The flat engine expects a name nothing will carry, so an
/// exit of 0 can only have come from the entry's.
#[test]
fn a_gesture_naming_an_entry_goes_down_that_entrys_channel() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let _home = listening(&material::dir(&world), "never");
    let _far = listening(&entry_dir(&world, "work"), "work");
    assert_eq!(
        run(
            &world,
            &[r#"{"op":"conversations","workspace":"work"}"#.to_owned()]
        ),
        0
    );
}

/// **The rename is spent here, and only here** (§8.2). The entry's `workspace`
/// file says this workspace answers to `home` on its host; the seat types
/// `work`, the box's own name for it, and what crosses the wire is `home`.
#[test]
fn an_entry_that_renames_its_workspace_carries_the_hosts_name() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let _home = listening(&material::dir(&world), "never");
    let dir = entry_dir(&world, "work");
    let _far = listening(&dir, "home");
    std::fs::write(dir.join(entries::WORKSPACE), "home\n").expect("write");
    assert_eq!(
        run(
            &world,
            &[r#"{"op":"conversations","workspace":"work"}"#.to_owned()]
        ),
        0
    );
}

/// **A name no entry holds, and a gesture naming no workspace, go where they
/// always went** — the flat directory's client material, the box's own root.
#[test]
fn a_name_no_entry_holds_and_no_name_at_all_go_to_the_flat_root() {
    for (expects, typed) in [
        ("other", r#"{"op":"conversations","workspace":"other"}"#),
        ("", r#"{"op":"workspaces"}"#),
    ] {
        let tmp = TempDir::new().expect("tmp");
        let world = crate::test_support::world_under(tmp.path());
        let _home = listening(&material::dir(&world), expects);
        let _far = listening(&entry_dir(&world, "work"), "never");
        assert_eq!(run(&world, &[typed.to_owned()]), 0, "{typed}");
    }
}

/// **An entry that exists is the answer to its name**, even when it cannot be
/// dialled: a half-provisioned entry refuses with its own sentence, naming
/// itself, rather than falling through to the flat root — which would send a
/// gesture to the wrong engine on the strength of a missing file.
#[test]
fn a_half_provisioned_entry_refuses_in_its_own_words() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = entry_dir(&world, "work");
    std::fs::create_dir_all(&dir).expect("mkdir");
    std::fs::write(dir.join(material::ANCHORS), "-----PEM-----\n").expect("write");

    let typed = json!({"op": "conversations", "workspace": "work"});
    let gesture = Gesture::Ask(Query::Conversations {
        workspace: "work".to_owned(),
    });
    let refused = channel(&world, &gesture, &typed).err().unwrap_or_default();
    assert!(
        refused.contains("half-provisioned") && refused.contains("work"),
        "{refused}"
    );
    assert_eq!(run(&world, &[typed.to_string()]), USAGE_EXIT);
}

/// **Migration: none.** A box with no `workspaces/` directory dials the flat
/// root whatever the gesture names, and the operator's own envelope crosses
/// byte for byte — which is exactly what a seat did before §8.2 existed. An
/// entry that renames nothing is the same: the rewrite is spent only when the
/// two names differ.
#[test]
fn nothing_is_rewritten_where_no_entry_renames_anything() {
    let (_tmp, world, _listener) = engine(true);
    let typed = json!({"op": "conversations", "workspace": "anything"});
    let gesture = Gesture::Ask(Query::Conversations {
        workspace: "anything".to_owned(),
    });
    let flat = open(&world).expect("flat seat").address();

    let (seat, carried) = channel(&world, &gesture, &typed).expect("the flat root");
    assert_eq!(seat.address(), flat);
    assert_eq!(carried, typed, "the envelope crosses as written");

    let dir = entry_dir(&world, "anything");
    mint(&dir);
    std::fs::write(dir.join(material::ADDRESS), "127.0.0.1:7737").expect("address");
    let (seat, carried) = channel(&world, &gesture, &typed).expect("the entry");
    assert_ne!(seat.address(), flat, "the entry's own address");
    assert_eq!(carried, typed, "and its own name, so nothing is rewritten");
}
