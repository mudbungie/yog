//! The two inspector replies whose rows are **messages**: the conversation
//! itself and the mail waiting for it. Every entry class the parser can produce
//! has to stay distinguishable on the wire, exactly as it does on screen —
//! §15 Y12's "surface them, never drop them" in its serialized form.

use crate::boundary::reply::{Reply, encode};
use crate::inboxview::{Deposit, Epitaph, InboxEntry};
use crate::transcript::{Block, Entry, EntryKind, Transcript, Usage};

fn entry(name: &str, raw: &str, kind: EntryKind) -> Entry {
    Entry {
        name: name.to_owned(),
        raw: raw.as_bytes().to_vec(),
        kind,
    }
}

/// Every `EntryKind` arm answers its own token and its own fields — a
/// delivered message, a model turn with all three block kinds and the
/// provider's own counters, a tool result, the live tail, and the raw bucket.
#[test]
fn every_transcript_entry_class_says_which_it_is() {
    let usage: Usage = [("input_tokens".to_owned(), 5u64)].into_iter().collect();
    let rows = encode(&Reply::Transcript(Transcript {
        entries: vec![
            entry(
                "001-user.md",
                "---\nfrom: user\n---\ngo\n",
                EntryKind::Delivered {
                    sender: "user".to_owned(),
                    epitaph: None,
                    body: "go\n".to_owned(),
                },
            ),
            entry(
                "002-kid.md",
                "bye",
                EntryKind::Delivered {
                    sender: "kid".to_owned(),
                    epitaph: Some(Epitaph::Died),
                    body: String::new(),
                },
            ),
            entry(
                "003-opus.json",
                "[]",
                EntryKind::Model {
                    model_id: "opus".to_owned(),
                    blocks: vec![
                        Block::Thinking("hmm".to_owned()),
                        Block::Text("done".to_owned()),
                        Block::ToolUse {
                            id: "toolu_1".to_owned(),
                            name: "bash".to_owned(),
                            input_summary: "ls".to_owned(),
                        },
                    ],
                    usage,
                },
            ),
            entry(
                "004-tool.json",
                "{}",
                EntryKind::ToolResult {
                    tool_use_id: "toolu_1".to_owned(),
                    content: "a b".to_owned(),
                    is_error: true,
                },
            ),
            entry(
                "«live»",
                "half",
                EntryKind::Streaming {
                    thinking: String::new(),
                    text: "half".to_owned(),
                },
            ),
            entry("notes.txt", "\u{fffd}", EntryKind::Raw),
        ],
    }));
    assert_eq!(rows["kind"], "transcript");
    let at = |i: usize| rows["rows"][i].clone();
    assert_eq!(at(0)["kind"], "delivered");
    assert_eq!(at(0)["sender"], "user");
    assert_eq!(at(0)["body"], "go\n");
    // The envelope the parsed view drops is still reachable — the headless seat
    // has no Raw toggle to reach it with.
    assert_eq!(at(0)["raw"], "---\nfrom: user\n---\ngo\n");
    assert!(at(0).get("epitaph").is_none(), "not a result deposit");
    assert_eq!(at(1)["epitaph"], "died");
    assert_eq!(at(2)["kind"], "model");
    assert_eq!(at(2)["model_id"], "opus");
    assert_eq!(at(2)["usage"]["input_tokens"], 5);
    assert_eq!(at(2)["blocks"][0]["kind"], "thinking");
    assert_eq!(at(2)["blocks"][1]["text"], "done");
    assert_eq!(at(2)["blocks"][2]["kind"], "tool-use");
    assert_eq!(at(2)["blocks"][2]["input"], "ls");
    assert_eq!(at(3)["kind"], "tool-result");
    assert_eq!(at(3)["tool_use_id"], "toolu_1");
    assert_eq!(at(3)["is_error"], true);
    assert_eq!(at(4)["kind"], "streaming");
    assert_eq!(at(4)["text"], "half");
    assert_eq!(at(5)["kind"], "raw");
}

/// A deposit says what it stated and nothing more: a forgiving parse of a
/// hand-edited file leaves fields absent, and an absent key is a different
/// claim from an empty one.
#[test]
fn a_deposit_carries_what_it_stated_and_omits_what_it_did_not() {
    let rows = encode(&Reply::Inbox(vec![
        InboxEntry {
            name: "user-001.md".to_owned(),
            raw: b"---\nfrom: user\n---\nhi\n".to_vec(),
            deposit: Deposit {
                sender: Some("user".to_owned()),
                deposited_at: Some("2026-08-14T00:00:00Z".to_owned()),
                epitaph: Some(Epitaph::FinalResponse),
                terminal_ref: Some("sha".to_owned()),
                body: "hi\n".to_owned(),
            },
        },
        InboxEntry {
            name: "raw-002.md".to_owned(),
            raw: b"no envelope".to_vec(),
            deposit: Deposit {
                body: "no envelope".to_owned(),
                ..Deposit::default()
            },
        },
    ]));
    assert_eq!(rows["kind"], "inbox");
    let stated = rows["rows"][0].clone();
    assert_eq!(stated["name"], "user-001.md");
    assert_eq!(stated["raw"], "---\nfrom: user\n---\nhi\n");
    assert_eq!(stated["deposit"]["from"], "user");
    assert_eq!(stated["deposit"]["deposited_at"], "2026-08-14T00:00:00Z");
    assert_eq!(stated["deposit"]["epitaph"], "final-response");
    assert_eq!(stated["deposit"]["terminal_ref"], "sha");
    assert_eq!(stated["deposit"]["body"], "hi\n");
    let bare = rows["rows"][1].clone();
    assert_eq!(bare["deposit"]["body"], "no envelope");
    for key in ["from", "deposited_at", "epitaph", "terminal_ref"] {
        assert!(bare["deposit"].get(key).is_none(), "{key} was never stated");
    }
}
