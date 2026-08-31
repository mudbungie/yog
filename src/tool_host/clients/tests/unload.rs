//! **The `unload` op** (REMOTE §5.2; bl-3455): load's mirror, and the second
//! `clients` op that writes.
//!
//! Its authority is not the roster but this agent's own document, which is what
//! makes its failures its own — a tool the conversation does not hold, a client
//! it loaded nothing from, a document that cannot be written — and each of them
//! refuses **whole**, because a partial unload leaves the model believing it
//! dropped a tool it still declares.
//!
//! Every act here is driven through [`answer`] on an **impatient** site, whose
//! engine ask gives up at once. That is the assertion, not a shortcut: an
//! unload that asked the engine would fail on every one of these.

use std::os::unix::fs::PermissionsExt;

use super::*;

/// The site every test here spends: impatient, so any engine ask would refuse.
fn offline(root: &Path) -> Site {
    site(root, impatient())
}

/// Put `take` into the agent's set through the real load act, against a
/// stand-in engine advertising `advertised`.
fn load_from(root: &Path, client: &str, advertised: Vec<Tool>, take: &[&str]) {
    against(
        root,
        vec![row(client, true, advertised)],
        &json!({"op": "load", "client": client, "tools": take}),
    )
    .expect("loaded");
}

/// What the agent's document holds, by presented name.
fn held(root: &Path) -> Vec<String> {
    crate::tool_host::loaded::read(root, "home", "dulcet-mongoose")
        .iter()
        .map(crate::tool_host::loaded::Entry::presented)
        .collect()
}

/// One named tool stops being declared and every other one stands.
#[test]
fn unload_stops_declaring_the_named_tool_and_leaves_the_rest() {
    let root = TempDir::new().expect("tmp");
    load_from(
        root.path(),
        "laptop",
        vec![tool("Bash"), tool("Read")],
        &["Bash", "Read"],
    );
    let said = answer(
        &offline(root.path()),
        &json!({"op": "unload", "client": "laptop", "tools": ["Bash"]}),
        &quiet(),
    )
    .expect("unloaded");
    assert!(said.contains("unloaded, observed 1970-01-01"), "{said}");
    assert!(said.contains("  laptop_Bash"), "{said}");
    assert!(said.contains("now holds 1 loaded tool."), "{said}");
    assert!(
        said.contains("op=load makes any of them callable again"),
        "{said}"
    );
    assert_eq!(held(root.path()), vec!["laptop_Read".to_owned()]);
}

/// **`tools` omitted is that client's whole loaded set**, and only that
/// client's: an agent finished with one machine has not finished with another.
#[test]
fn unload_without_tools_drops_that_clients_whole_set_and_no_others() {
    let root = TempDir::new().expect("tmp");
    load_from(
        root.path(),
        "laptop",
        vec![tool("Bash"), tool("Read")],
        &["Bash", "Read"],
    );
    load_from(root.path(), "desk", vec![tool("Bash")], &["Bash"]);
    let said = answer(
        &offline(root.path()),
        &json!({"op": "unload", "client": "laptop"}),
        &quiet(),
    )
    .expect("unloaded");
    assert!(said.contains("  laptop_Bash"), "{said}");
    assert!(said.contains("  laptop_Read"), "{said}");
    assert!(said.contains("now holds 1 loaded tool."), "{said}");
    assert_eq!(held(root.path()), vec!["desk_Bash".to_owned()]);
}

/// **The client is half of every name.** Two machines advertising `Bash` are
/// two loaded tools, and an unload names one of them.
#[test]
fn an_unload_drops_only_the_named_clients_copy_of_a_shared_name() {
    let root = TempDir::new().expect("tmp");
    load_from(root.path(), "laptop", vec![tool("Bash")], &["Bash"]);
    load_from(root.path(), "desk", vec![tool("Bash")], &["Bash"]);
    answer(
        &offline(root.path()),
        &json!({"op": "unload", "client": "desk", "tools": ["Bash"]}),
        &quiet(),
    )
    .expect("unloaded");
    assert_eq!(held(root.path()), vec!["laptop_Bash".to_owned()]);
}

/// A name the conversation does not hold refuses the whole act and writes
/// nothing — load's rule, read backwards.
#[test]
fn a_tool_the_conversation_does_not_hold_refuses_the_whole_unload() {
    let root = TempDir::new().expect("tmp");
    load_from(root.path(), "laptop", vec![tool("Bash")], &["Bash"]);
    let e = answer(
        &offline(root.path()),
        &json!({"op": "unload", "client": "laptop", "tools": ["Bash", "Read"]}),
        &quiet(),
    )
    .expect_err("refused");
    assert!(
        e.contains("no tool \"Read\" loaded from client \"laptop\""),
        "{e}"
    );
    assert_eq!(
        held(root.path()),
        vec!["laptop_Bash".to_owned()],
        "the whole act was refused"
    );
}

/// A client this conversation loaded nothing from refuses rather than answering
/// an empty success — in both spellings, because both are the model saying
/// something about a set it does not have.
#[test]
fn a_client_this_conversation_loaded_nothing_from_refuses() {
    let root = TempDir::new().expect("tmp");
    load_from(root.path(), "laptop", vec![tool("Bash")], &["Bash"]);
    let whole = answer(
        &offline(root.path()),
        &json!({"op": "unload", "client": "desk"}),
        &quiet(),
    )
    .expect_err("refused");
    assert!(
        whole.contains("no tool loaded from client \"desk\""),
        "{whole}"
    );
    let named = answer(
        &offline(root.path()),
        &json!({"op": "unload", "client": "desk", "tools": ["Bash"]}),
        &quiet(),
    )
    .expect_err("refused");
    assert!(
        named.contains("no tool \"Bash\" loaded from client \"desk\""),
        "{named}"
    );
    assert_eq!(held(root.path()), vec!["laptop_Bash".to_owned()]);
}

/// **The document survives being emptied**, and a load after an unload declares
/// the tool again with its definition whole. An emptied set reads as the same
/// nothing a fresh agent reads, which is why no reader carries a second case.
#[test]
fn a_load_after_an_unload_declares_it_again() {
    let root = TempDir::new().expect("tmp");
    load_from(root.path(), "laptop", vec![tool("Bash")], &["Bash"]);
    answer(
        &offline(root.path()),
        &json!({"op": "unload", "client": "laptop"}),
        &quiet(),
    )
    .expect("unloaded");
    assert!(held(root.path()).is_empty());
    load_from(root.path(), "laptop", vec![tool("Bash")], &["Bash"]);
    assert_eq!(held(root.path()), vec!["laptop_Bash".to_owned()]);
    let back = crate::tool_host::loaded::read(root.path(), "home", "dulcet-mongoose");
    assert_eq!(back[0].tool, tool("Bash"), "the definition is frozen whole");
}

/// An unload never asks the engine: its subject is a file on this box, so a
/// finished host can be dropped with the engine down and no gesture is
/// deposited.
#[test]
fn an_unload_never_reaches_the_engine() {
    let root = TempDir::new().expect("tmp");
    crate::tool_host::loaded::add(
        root.path(),
        "home",
        "dulcet-mongoose",
        &[crate::tool_host::loaded::Entry {
            client: "laptop".to_owned(),
            tool: tool("Bash"),
        }],
    )
    .expect("seeded");
    answer(
        &offline(root.path()),
        &json!({"op": "unload", "client": "laptop"}),
        &quiet(),
    )
    .expect("unloaded");
    assert!(
        crate::boundary::deposit::pending(root.path()).is_empty(),
        "nothing was deposited"
    );
}

/// An unload the agent's document cannot be rewritten with refuses, naming the
/// act — the write half, mirroring the load's own.
#[test]
fn an_unload_that_cannot_be_recorded_says_so() {
    let root = TempDir::new().expect("tmp");
    load_from(root.path(), "laptop", vec![tool("Bash")], &["Bash"]);
    let file = crate::tool_host::loaded::path(root.path(), "home", "dulcet-mongoose")
        .expect("addressable");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o444)).expect("chmod");
    let e = answer(
        &offline(root.path()),
        &json!({"op": "unload", "client": "laptop"}),
        &quiet(),
    )
    .expect_err("unrecordable");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).expect("chmod");
    assert!(e.contains("recording the unload"), "{e}");
}
