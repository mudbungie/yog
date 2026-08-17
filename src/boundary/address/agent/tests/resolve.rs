//! The needle → id ladder (bl-49bc): an id untouched, a stored name resolved off
//! the derivation or off disk, and everything else refused naming the token.

use super::super::resolve_agent;
use crate::git_tree::tests::fixture::Fixture;

/// The workspace name every fixture snapshot here publishes.
const WS: &str = "alba";

/// A workspace path the fixtures address, and the snapshot around it.
fn snap(agents: Vec<crate::git_tree::Agent>) -> (std::path::PathBuf, crate::app::Snapshot) {
    let ws = std::path::PathBuf::from("/names/alba");
    let snapshot = crate::boundary::tests::snapshot(&ws, WS, agents, vec![]);
    (ws, snapshot)
}

/// One derived agent wearing `name` as its **stored** fact.
fn named(id: &str, name: Option<&str>) -> crate::git_tree::Agent {
    crate::git_tree::Agent {
        name: name.map(str::to_owned),
        ..crate::boundary::tests::agent(id, crate::git_tree::AgentState::Quiescent, 1)
    }
}

/// A gesture that names no conversation resolves to nothing — the general path
/// with no input, which is what lets both chokepoints spell the resolution
/// unconditionally.
#[test]
fn no_needle_resolves_to_nothing() {
    let (ws, snapshot) = snap(vec![]);
    assert_eq!(resolve_agent(&snapshot, &ws, None), Ok(String::new()));
}

/// Rung one: an **id-shaped** needle is an id, returned untouched and with no
/// enumeration at all — so a path that never existed is never even read, and
/// `delete-agent` keeps admitting the id no ref answers to (lernie §9.2).
#[test]
fn an_id_shaped_needle_passes_through_unread() {
    let id = "20260101T000000Z-aaaa-20260102T000000Z-bbbb";
    let (_, snapshot) = snap(vec![]);
    assert_eq!(
        resolve_agent(
            &snapshot,
            std::path::Path::new("/nonexistent"),
            Some(id.to_owned())
        ),
        Ok(id.to_owned())
    );
}

/// Rung two, first reading: an id the derivation holds that lernie's stamp
/// grammar does not recognize — a foreign or hand-made branch — still addresses
/// itself. Refusing it would have made every such conversation unreachable.
#[test]
fn a_foreign_id_the_derivation_holds_addresses_itself() {
    let (ws, snapshot) = snap(vec![named("hand-made", None)]);
    assert_eq!(
        resolve_agent(&snapshot, &ws, Some("hand-made".to_owned())),
        Ok("hand-made".to_owned())
    );
}

/// Rung two, second reading, and the receipt this ball is about: the minted §3.3
/// name a `Started` reply hands back resolves to the root's own id, so the
/// handle composes with every agent-addressed gesture rather than with `message`
/// alone.
#[test]
fn a_stored_name_resolves_to_the_root_it_names() {
    let (ws, snapshot) = snap(vec![named("20260101T000000Z-aaaa", Some("pale-otter"))]);
    assert_eq!(
        resolve_agent(&snapshot, &ws, Some("pale-otter".to_owned())),
        Ok("20260101T000000Z-aaaa".to_owned())
    );
}

/// The §3.3 ladder's **legacy display-only** rung (bl-8068) is a title, never an
/// address: a `You are <x>.` goal stamp with no stored `name` blob behind it
/// refuses exactly as an unknown name does.
#[test]
fn a_legacy_display_only_name_refuses() {
    let stamped = crate::git_tree::Agent {
        goal_name: Some("pale-fox".to_owned()),
        ..named("20260101T000000Z-aaaa", None)
    };
    let (ws, snapshot) = snap(vec![stamped]);
    let why = resolve_agent(&snapshot, &ws, Some("pale-fox".to_owned())).expect_err("refused");
    assert!(why.contains("unknown conversation"), "{why}");
}

/// One name worn by two living agents refuses rather than guessing, exactly as
/// two workspace roots sharing a leaf do — a guess would act on the wrong
/// conversation.
#[test]
fn an_ambiguous_name_refuses() {
    let (ws, snapshot) = snap(vec![
        named("20260101T000000Z-aaaa", Some("pale-otter")),
        named("20260101T000000Z-bbbb", Some("pale-otter")),
    ]);
    let why = resolve_agent(&snapshot, &ws, Some("pale-otter".to_owned())).expect_err("refused");
    assert!(why.contains("ambiguous conversation"), "{why}");
    assert!(why.contains("pale-otter"), "{why}");
}

/// A needle no reading answers to **refuses**, naming the token — never a
/// pass-through that would let a policy row land on a string no conversation
/// wears.
#[test]
fn an_unknown_name_refuses_naming_the_token() {
    let (ws, snapshot) = snap(vec![named("20260101T000000Z-aaaa", Some("pale-otter"))]);
    let why = resolve_agent(&snapshot, &ws, Some("grey-heron".to_owned())).expect_err("refused");
    assert!(why.contains("unknown conversation"), "{why}");
    assert!(why.contains("grey-heron"), "{why}");
}

/// **Rung three — the barrier** (bl-6c9e one noun down): a conversation on disk
/// that no derivation has swept yet still addresses. The snapshot here holds a
/// tree for a *different* path, exactly as a published derivation taken before
/// the fire does, and the name resolves off the workspace's own refs.
#[test]
fn a_conversation_the_derivation_has_not_swept_resolves_off_disk() {
    let fx = Fixture::new();
    fx.build_agent("20260101T000000Z-aaaa", "one");
    fx.name_agent("20260101T000000Z-aaaa", "pale-otter");
    let (_, snapshot) = snap(vec![]);
    assert_eq!(
        resolve_agent(&snapshot, &fx.path, Some("pale-otter".to_owned())),
        Ok("20260101T000000Z-aaaa".to_owned())
    );
}
