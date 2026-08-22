//! The draft store's rules (bl-a69a): a draft is its target's, absence is
//! emptiness, and a send clears one key.

use super::{DraftKey, Drafts};
use std::path::PathBuf;

/// The new-conversation key for a workspace path.
fn start_in(path: &str) -> DraftKey {
    DraftKey::NewConversation(Some(PathBuf::from(path)))
}

/// The `(workspace, selection)` a composer derives its key from.
fn composer(path: &str, selected: Option<&str>) -> DraftKey {
    DraftKey::composer(Some(PathBuf::from(path)), selected.map(str::to_owned))
}

#[test]
fn an_untyped_target_reads_empty() {
    let drafts = Drafts::default();
    assert!(drafts.is_empty());
    assert_eq!(drafts.text(&start_in("/w")), "");
    assert_eq!(drafts.text(&DraftKey::Message("c-1".into())), "");
}

#[test]
fn each_target_holds_its_own_draft() {
    // The bug: a goal typed for a new conversation followed the selection into
    // a message box. Three targets, three drafts, no bleed.
    let mut drafts = Drafts::default();
    let start = start_in("/w");
    let other = start_in("/other");
    let msg = DraftKey::Message("c-1".into());
    drafts.set(start.clone(), "ship the goal".into());
    assert_eq!(
        drafts.text(&msg),
        "",
        "the message box is its own, and empty"
    );
    assert_eq!(drafts.text(&other), "", "so is another workspace's");
    drafts.set(msg.clone(), "ping".into());
    assert_eq!(
        drafts.text(&start),
        "ship the goal",
        "switching back restores the goal"
    );
    assert_eq!(drafts.text(&msg), "ping");
}

#[test]
fn a_send_clears_only_its_own_key() {
    let mut drafts = Drafts::default();
    let start = start_in("/w");
    let msg = DraftKey::Message("c-1".into());
    drafts.set(start.clone(), "ship the goal".into());
    drafts.set(msg.clone(), "ping".into());
    // A clean deposit clears the draft it deposited (§5.3) — and nothing else.
    drafts.set(msg.clone(), String::new());
    assert_eq!(drafts.text(&msg), "");
    assert_eq!(drafts.text(&start), "ship the goal");
    assert!(!drafts.is_empty(), "the other target still holds its draft");
    // Emptying is removal: absence and "" are one representation, not two.
    drafts.set(start, String::new());
    assert!(drafts.is_empty());
}

#[test]
fn the_composer_key_follows_the_selection() {
    // A selected agent is a message; no selection is a new conversation in the
    // focused workspace; the empty world's bootstrap box is that same case with
    // the workspace absent.
    assert_eq!(
        composer("/w", Some("c-1")),
        DraftKey::Message("c-1".into()),
        "a selection is a message target"
    );
    assert_eq!(composer("/w", None), start_in("/w"));
    assert_eq!(
        DraftKey::composer(None, None),
        DraftKey::NewConversation(None),
        "the empty world is the general case with no workspace"
    );
    // Two workspaces are two new-conversation targets, not one.
    assert_ne!(composer("/w", None), composer("/other", None));
}

#[test]
fn drafts_clone_eq_and_debug() {
    let mut drafts = Drafts::default();
    let key = DraftKey::Message("c-1".into());
    drafts.set(key.clone(), "ping".into());
    assert_eq!(drafts.clone(), drafts);
    assert!(format!("{drafts:?}").contains("ping"));
    assert!(format!("{key:?}").contains("c-1"));
}

/// **A clean send takes out what it sent, and nothing else** (§5.3, bl-56c6).
/// The box stays live across the frames between a post and its receipt, so what
/// is typed there is a draft like any other — and a buffer edited under the send
/// is left exactly as it is rather than half-cut.
#[test]
fn a_send_removes_the_words_it_deposited_and_never_the_ones_after_them() {
    let mut drafts = Drafts::default();
    let key = DraftKey::Message("c-1".into());

    drafts.set(key.clone(), "ping".into());
    drafts.sent(&key, "ping");
    assert_eq!(drafts.text(&key), "", "the ordinary case: nothing is left");

    drafts.set(key.clone(), "ping and hurry".into());
    drafts.sent(&key, "ping");
    assert_eq!(
        drafts.text(&key),
        " and hurry",
        "what was typed after Enter is still theirs, verbatim"
    );

    // Edited under the send: the fired words are no longer a prefix, so nothing
    // is cut — a draft is never destroyed to keep a rule tidy.
    drafts.set(key.clone(), "something else".into());
    drafts.sent(&key, "ping");
    assert_eq!(drafts.text(&key), "something else");

    // A send from a key that holds nothing is the general path at zero.
    let empty = DraftKey::Message("c-2".into());
    drafts.sent(&empty, "ping");
    assert_eq!(drafts.text(&empty), "");
}

/// **One buffer follows its conversation through every spelling of its
/// identity** (§3.4, bl-56c6): the workspace's new-conversation draft, then the
/// minted §3.3 name's, then the id's. Idempotent, so the seat that does it may
/// do it every frame — and never destructive, so a destination already holding
/// a draft keeps it.
#[test]
fn a_draft_is_carried_to_the_key_its_conversation_acquired() {
    let mut drafts = Drafts::default();
    let bare = start_in("/w");
    let named = DraftKey::Message("CourtyardRooftop".into());
    let id = DraftKey::Message("c-2".into());

    drafts.set(bare.clone(), "half a thought".into());
    drafts.carry(&bare, &named);
    assert_eq!(drafts.text(&named), "half a thought");
    assert_eq!(drafts.text(&bare), "", "and it is not in two places");

    // Idempotent: a second pass moves nothing, because the source is empty now.
    drafts.carry(&bare, &named);
    assert_eq!(drafts.text(&named), "half a thought");
    drafts.carry(&named, &id);
    assert_eq!(drafts.text(&id), "half a thought");

    // One spelling to itself is a no-op, and an occupied destination is left
    // alone rather than overwritten.
    drafts.carry(&id, &id);
    assert_eq!(drafts.text(&id), "half a thought");
    drafts.set(named.clone(), "typed since".into());
    drafts.carry(&named, &id);
    assert_eq!(drafts.text(&id), "half a thought");
    assert_eq!(drafts.text(&named), "typed since");
}
