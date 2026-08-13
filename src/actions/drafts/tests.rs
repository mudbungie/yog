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
