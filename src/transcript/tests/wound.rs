//! The **settled-failure notice** (bl-015b): the §7.3 wound folded onto a
//! committed transcript as a virtual trailing entry, so a conversation that
//! stopped says so where the operator is reading it.
//!
//! The defect it closes is the empty pane — a conversation refused at its
//! first model call painted its user message and then nothing at all, while
//! the fact and its remedy sat on surfaces nobody had open.

use tempfile::tempdir;

use super::{AGENT, write_msg};
use crate::git_tree::{Delta, Stream};
use crate::login::auth::AuthFailure;
use crate::steps_view::Wound;
use crate::transcript::{EntryKind, build};

/// The ball's own shape: one committed user message, refused, nothing else.
#[test]
fn a_refused_conversation_says_so_instead_of_ending_on_the_user_message() {
    let dir = tempdir().unwrap();
    write_msg(dir.path(), "001-user.md", b"go");
    let committed = build(dir.path(), AGENT);
    assert_eq!(committed.entries.len(), 1, "the pane that said nothing");

    let wound = Wound::Refused(AuthFailure::Row("anthropic".into()));
    let t = committed.with_wound(&wound);
    assert_eq!(t.entries.len(), 2);
    let notice = &t.entries[1];
    assert!(matches!(
        &notice.kind,
        EntryKind::Wounded { wound } if matches!(wound, Wound::Refused(_))
    ));
    // The row carries the wound; the sentence is the projection of it, and the
    // Raw view is that sentence because no file backs this entry.
    assert_eq!(notice.raw, wound.banner().into_bytes());
    let said = String::from_utf8(notice.raw.clone()).unwrap();
    assert!(
        said.contains("anthropic"),
        "it names the row to sign in: {said}"
    );
}

/// Every arm of the vocabulary reaches the notice, which is the whole reason
/// the entry carries the wound rather than the refusal alone: a driver that
/// produced nothing and an output limit that ended the turn leave the same
/// silent conversation an operator cannot read.
#[test]
fn each_wound_class_reaches_the_notice_in_its_own_words() {
    let dir = tempdir().unwrap();
    write_msg(dir.path(), "001-user.md", b"go");
    let committed = build(dir.path(), AGENT);
    for wound in [
        Wound::Mute,
        Wound::Spoke("the adapter's last words".into()),
        Wound::OutputLimit,
        Wound::Refused(AuthFailure::Unrouted),
    ] {
        let t = committed.with_wound(&wound);
        assert_eq!(t.entries.len(), 2, "{wound:?} seats a notice");
        assert_eq!(t.entries[1].name, "«wound»");
        assert_eq!(t.entries[1].raw, wound.banner().into_bytes());
        assert!(!wound.banner().is_empty(), "{wound:?} says something");
    }
}

/// A healthy conversation says nothing about its health — the same rule the
/// live tail follows for a stream that has said nothing.
#[test]
fn an_unwounded_conversation_gains_no_notice() {
    let dir = tempdir().unwrap();
    write_msg(dir.path(), "001-user.md", b"go");
    let committed = build(dir.path(), AGENT);
    assert_eq!(committed.with_wound(&Wound::None), committed);
}

/// The fold replaces its own kind of tail rather than appending beside one, so
/// a caller may fold a fresher derivation on without stripping the older one —
/// `with_live`'s rule, stated once per row.
#[test]
fn a_second_wound_fold_replaces_the_first() {
    let dir = tempdir().unwrap();
    write_msg(dir.path(), "001-user.md", b"go");
    let committed = build(dir.path(), AGENT);
    let once = committed.with_wound(&Wound::OutputLimit);
    let twice = committed
        .with_wound(&Wound::Mute)
        .with_wound(&Wound::OutputLimit);
    assert_eq!(once, twice);
    assert_eq!(twice.entries.len(), 2, "the committed half plus ONE notice");
    // And a healed conversation takes the notice away with it.
    assert_eq!(
        committed.with_wound(&Wound::Mute).with_wound(&Wound::None),
        committed
    );
}

/// The two folds are exclusive by derivation, never by a rule in either of
/// them: each strips only its own kind, so neither can eat the other's row.
#[test]
fn the_two_trailing_folds_do_not_strip_each_other() {
    let dir = tempdir().unwrap();
    write_msg(dir.path(), "001-user.md", b"go");
    let stream = Stream {
        text: Some("half".into()),
        thinking: None,
        last_delta: Some(Delta::Text),
    };
    let both = build(dir.path(), AGENT)
        .with_live(&stream)
        .with_wound(&Wound::Mute);
    assert_eq!(both.entries.len(), 3);
    assert!(matches!(both.entries[1].kind, EntryKind::Streaming { .. }));
    assert!(matches!(both.entries[2].kind, EntryKind::Wounded { .. }));
}
