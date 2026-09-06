//! The needle → id ladder (bl-49bc): an id untouched, a stored name resolved off
//! the derivation or off disk, and everything else refused naming the token.

use super::super::{resolve_agent, resolving};
use crate::git_tree::tests::fixture::Fixture;
use std::time::Duration;

/// The hold a test that is not about the hold names: look once, wait never
/// ([`settle`](super::super::settle) — the production pair is
/// [`resolve_agent`]'s).
const NO_HOLD: (u32, Duration) = (0, Duration::ZERO);

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
/// `delete-agent` keeps admitting the id no ref answers to (litany §9.2).
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

/// Rung two, first reading: an id the derivation holds that litany's stamp
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
    let why = resolving(
        &snapshot,
        &ws,
        Some("pale-fox".to_owned()),
        NO_HOLD.0,
        NO_HOLD.1,
    )
    .expect_err("refused");
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
    let why = resolving(
        &snapshot,
        &ws,
        Some("grey-heron".to_owned()),
        NO_HOLD.0,
        NO_HOLD.1,
    )
    .expect_err("refused");
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

/// **The name a start just handed back is addressable** (bl-802a): the disk
/// rung holds while the detached driver writes its branch instead of answering
/// with the sentence a name that never existed earns.
///
/// The shape is the sighting's exactly — `start` returns a minted name, a
/// `follow` naming it fires at once, and the `agents/<id>` ref lands a moment
/// later. On the old tree rung three read an empty enumeration and refused; the
/// hold looks again until the branch is there.
#[test]
fn a_name_the_fire_just_minted_resolves_once_the_driver_writes_its_branch() {
    let fx = Fixture::new();
    let ws = fx.path.clone();
    let (_, snapshot) = snap(vec![]);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            std::thread::sleep(Duration::from_millis(100));
            fx.build_agent("20260101T000000Z-aaaa", "one");
            fx.name_agent("20260101T000000Z-aaaa", "pale-otter");
        });
        assert_eq!(
            resolving(
                &snapshot,
                &ws,
                Some("pale-otter".to_owned()),
                80,
                Duration::from_millis(25),
            ),
            Ok("20260101T000000Z-aaaa".to_owned()),
            "the disk rung waits out the driver's launch",
        );
    });
}

/// The hold ends, and what it ends in is the refusal it always was: a name that
/// never appears is still an unknown conversation, named in the refusal.
#[test]
fn a_name_that_never_appears_still_refuses_after_the_hold() {
    let fx = Fixture::new();
    let (_, snapshot) = snap(vec![]);
    let why = resolving(
        &snapshot,
        &fx.path,
        Some("grey-heron".to_owned()),
        2,
        Duration::from_millis(1),
    )
    .expect_err("refused");
    assert!(why.contains("unknown conversation"), "{why}");
    assert!(why.contains("grey-heron"), "{why}");
}
