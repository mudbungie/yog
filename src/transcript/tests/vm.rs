//! Origin-classification and forgiving-parse coverage, through [`build`].
//!
//! This file holds the **delivered** origin — the deposit envelope a message
//! carries and what survives stripping it — and the **raw bucket**, the
//! promise that a name the reader cannot parse is preserved rather than
//! dropped. The two origins with a payload grammar of their own are split off
//! at §12's budget on that seam: [`model`] for an assistant file's content,
//! [`tools`] for a tool result's.

use super::{AGENT, write_msg};
use crate::transcript::{EntryKind, build};
use tempfile::tempdir;

/// Build a one-file transcript and return that entry's classified kind. Every
/// name here opens the counter at `001`, so the listing holds nothing but the
/// file — a higher one also reveals a compaction mark ([`super::compaction`]).
pub(super) fn kind_of(name: &str, bytes: &[u8]) -> EntryKind {
    let dir = tempdir().unwrap();
    write_msg(dir.path(), name, bytes);
    let mut t = build(dir.path(), AGENT);
    assert_eq!(t.entries.len(), 1, "one entry expected for {name}");
    t.entries.remove(0).kind
}

#[test]
fn delivered_md_strips_the_deposit_envelope_off_the_body() {
    // Delivery renames the deposit file into `messages/` with its
    // frontmatter untouched (ARCH §2.11), so the bytes open with the
    // envelope. The parsed body is the message — never the `---` fence.
    assert_eq!(
        kind_of(
            "001-user.md",
            b"---\nfrom: user\ndeposited_at: 2026-08-02T04:00:00Z\n---\nis this thing on?\n"
        ),
        EntryKind::Delivered {
            sender: "user".into(),
            sender_name: None,
            epitaph: None,
            body: "is this thing on?\n".into()
        }
    );
}

#[test]
fn delivered_md_without_an_envelope_is_the_whole_file() {
    // The forgiving read: no frontmatter means no envelope to strip.
    assert_eq!(
        kind_of("001-alice.md", b"hi there\n"),
        EntryKind::Delivered {
            sender: "alice".into(),
            sender_name: None,
            epitaph: None,
            body: "hi there\n".into()
        }
    );
}

#[test]
fn delivered_result_message_with_no_content_has_an_empty_body() {
    // A result message can be envelope-only (ARCH §2.6) — the epitaph is
    // asserted, the child never spoke.
    assert_eq!(
        kind_of(
            "001-kid.md",
            b"---\nfrom: kid\ndeposited_at: t\nepitaph: died\nterminal_ref: sha\n---\n"
        ),
        EntryKind::Delivered {
            sender: "kid".into(),
            sender_name: None,
            epitaph: Some(crate::inboxview::Epitaph::Died),
            body: String::new()
        }
    );
}

#[test]
fn delivered_sender_keeps_internal_hyphens() {
    let EntryKind::Delivered { sender, .. } = kind_of("001-claude-fable.md", b"x") else {
        panic!("expected delivered");
    };
    assert_eq!(sender, "claude-fable");
}

mod model;
mod tools;

#[test]
fn unparseable_names_go_to_raw_bucket_not_dropped() {
    let dir = tempdir().unwrap();
    // no dot / no hyphen / empty counter / non-digit counter / empty origin
    // / unknown extension — all preserved as Raw, never dropped.
    for n in [
        "README",
        "readme.md",
        "-x.md",
        "abc-x.md",
        "003-.md",
        "001-note.txt",
    ] {
        write_msg(dir.path(), n, b"body");
    }
    let t = build(dir.path(), AGENT);
    assert_eq!(t.entries.len(), 6, "none dropped");
    assert!(t.entries.iter().all(|e| e.kind == EntryKind::Raw));
    assert!(t.entries.iter().any(|e| e.name == "001-note.txt"));
}

#[test]
fn raw_entry_keeps_verbatim_bytes() {
    let dir = tempdir().unwrap();
    write_msg(dir.path(), "README", b"\x00\x01raw");
    let t = build(dir.path(), AGENT);
    assert_eq!(t.entries[0].raw, b"\x00\x01raw");
}

/// **The NAME rides beside the id** (bl-6661, litany 0.0.11 upstream bl-a457).
/// The framing sender is the FILENAME's origin token — the addressing key
/// litany's own inbox scan derives from — so a message a named child sent was
/// attributed by sixty characters of timestamped hex and nothing else. The
/// envelope's `from_name:` is the only carrier of the name, and it is kept
/// beside the id for `epitaph:`'s own reason: every other asserted field is
/// re-asserted elsewhere and this one is not.
#[test]
fn a_delivered_row_carries_the_senders_name_beside_its_id() {
    let EntryKind::Delivered {
        sender,
        sender_name,
        ..
    } = kind_of(
        "001-20260814T000000Z-ab12.md",
        b"---\nfrom: 20260814T000000Z-ab12\nfrom_name: DulcetMongoose\n---\ndone\n",
    )
    else {
        panic!("expected delivered");
    };
    assert_eq!(sender, "20260814T000000Z-ab12");
    assert_eq!(sender_name.as_deref(), Some("DulcetMongoose"));

    // `user` never wears one, and neither does an unnamed agent: the field is
    // absent rather than empty, so a reader can tell "no name" from "".
    let EntryKind::Delivered { sender_name, .. } =
        kind_of("001-user.md", b"---\nfrom: user\n---\nhi\n")
    else {
        panic!("expected delivered");
    };
    assert_eq!(sender_name, None);
}
