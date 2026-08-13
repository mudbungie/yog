//! Unit tests for prompt recall (§11 inbox-composer, bl-f908): what counts as
//! something *you* said, the caret gate, and the two-field walk with its
//! derived exit.

use super::{Caret, Recall, Step, prompts};
use crate::inboxview::{Deposit, Epitaph, InboxEntry};
use crate::transcript::{Entry, EntryKind, Transcript};

fn delivered(sender: &str, body: &str) -> Entry {
    Entry {
        name: format!("001-{sender}.md"),
        raw: body.as_bytes().to_vec(),
        kind: EntryKind::Delivered {
            sender: sender.into(),
            epitaph: None,
            body: body.into(),
        },
    }
}

fn ended(sender: &str, body: &str) -> Entry {
    Entry {
        name: format!("002-{sender}.md"),
        raw: body.as_bytes().to_vec(),
        kind: EntryKind::Delivered {
            sender: sender.into(),
            epitaph: Some(Epitaph::FinalResponse),
            body: body.into(),
        },
    }
}

fn model(text: &str) -> Entry {
    Entry {
        name: "003-claude.json".into(),
        raw: Vec::new(),
        kind: EntryKind::Model {
            model_id: "claude".into(),
            blocks: vec![crate::transcript::Block::Text(text.into())],
            usage: crate::transcript::Usage::default(),
        },
    }
}

fn deposit(sender: Option<&str>, body: &str) -> InboxEntry {
    InboxEntry {
        name: format!("{}-001.md", sender.unwrap_or("anon")),
        raw: body.as_bytes().to_vec(),
        deposit: Deposit {
            sender: sender.map(Into::into),
            body: body.into(),
            ..Deposit::default()
        },
    }
}

/// The history is the operator's own turns, newest first — pending ahead of
/// delivered, since pending is by construction the newer end.
#[test]
fn prompts_are_the_operators_own_messages_newest_first() {
    let tx = Transcript {
        entries: vec![
            delivered("user", "first"),
            model("some answer"),
            delivered("user", "second"),
        ],
    };
    let pending = vec![deposit(Some("user"), "third")];
    assert_eq!(prompts(&pending, &tx), ["third", "second", "first"]);
}

/// Everything said *to* you is skipped, at both seats: a peer's mail, a
/// child's result deposit, a sender-less deposit (the §11 peer catch-all),
/// and every machinery entry that has no speaker at all.
#[test]
fn nothing_said_to_you_is_offered_back() {
    let tx = Transcript {
        entries: vec![
            delivered("peer-1", "a peer's mail"),
            ended("user", "a result deposit"),
            model("the agent"),
            Entry {
                name: "004-tool.json".into(),
                raw: Vec::new(),
                kind: EntryKind::ToolResult {
                    tool_use_id: "t1".into(),
                    content: "out".into(),
                    is_error: false,
                },
            },
        ],
    };
    let pending = vec![deposit(Some("peer-2"), "more mail"), deposit(None, "anon")];
    assert!(prompts(&pending, &tx).is_empty());
}

/// A conversation with nothing in it — a new conversation's whole state, and
/// the general path rather than a case.
#[test]
fn an_empty_conversation_offers_nothing() {
    assert!(prompts(&[], &Transcript::default()).is_empty());
}

/// The gate: ↑ belongs to the box only from the top row, ↓ only from the
/// bottom one. An empty box is both.
#[test]
fn the_caret_gate_opens_only_at_the_matching_edge() {
    let empty = Caret { row: 0, rows: 0 };
    assert!(empty.open(Step::Back) && empty.open(Step::Forward));
    let top = Caret { row: 0, rows: 3 };
    assert!(top.open(Step::Back) && !top.open(Step::Forward));
    let middle = Caret { row: 1, rows: 3 };
    assert!(!middle.open(Step::Back) && !middle.open(Step::Forward));
    let bottom = Caret { row: 2, rows: 3 };
    assert!(!bottom.open(Step::Back) && bottom.open(Step::Forward));
}

/// The walk: back through the prompts, then forward past the newest to the
/// draft that was displaced — verbatim.
#[test]
fn back_walks_the_prompts_and_forward_restores_the_draft() {
    let history = ["newest".to_string(), "older".to_string()];
    let flat = Caret::default();
    let mut recall = Recall::default();
    assert_eq!(
        recall
            .step(Step::Back, flat, "half-typed", &history)
            .unwrap(),
        "newest"
    );
    assert_eq!(
        recall.step(Step::Back, flat, "newest", &history).unwrap(),
        "older"
    );
    assert_eq!(
        recall.step(Step::Back, flat, "older", &history),
        None,
        "the oldest prompt is the end of the walk"
    );
    assert_eq!(
        recall.step(Step::Forward, flat, "older", &history).unwrap(),
        "newest"
    );
    assert_eq!(
        recall
            .step(Step::Forward, flat, "newest", &history)
            .unwrap(),
        "half-typed",
        "forward past the newest hands the draft back"
    );
    assert_eq!(
        recall.step(Step::Forward, flat, "half-typed", &history),
        None,
        "and there is nothing past the draft"
    );
}

/// A caret that is not at the gesture's edge keeps its key: the arrows still
/// move the caret inside a recalled prompt.
#[test]
fn a_caret_off_the_edge_keeps_its_key() {
    let history = ["newest".to_string()];
    let mut recall = Recall::default();
    let middle = Caret { row: 1, rows: 3 };
    assert_eq!(recall.step(Step::Back, middle, "a\nb\nc", &history), None);
    assert_eq!(
        recall.step(Step::Forward, middle, "a\nb\nc", &history),
        None
    );
}

/// Editing a recalled prompt makes it yours: the recall is left, and the
/// stashed draft is forgotten rather than able to overwrite the edit.
#[test]
fn editing_a_recalled_prompt_leaves_the_recall_and_forgets_the_stash() {
    let history = ["newest".to_string()];
    let flat = Caret::default();
    let mut recall = Recall::default();
    recall.step(Step::Back, flat, "half-typed", &history);
    recall.settle("newest, amended", &history);
    assert_eq!(
        recall.step(Step::Forward, flat, "newest, amended", &history),
        None,
        "depth is back to the draft, so forward has nowhere to go"
    );
    assert_eq!(
        recall
            .step(Step::Back, flat, "newest, amended", &history)
            .unwrap(),
        "newest",
        "and back starts the walk over from the edited draft"
    );
}

/// The same one check covers a landed send: the draft clears and the history
/// grows under it, so the recall is simply not where it was.
#[test]
fn a_send_settles_the_recall_out() {
    let mut recall = Recall::default();
    recall.step(Step::Back, Caret::default(), "", &["newest".to_string()]);
    recall.settle("", &["sent".to_string(), "newest".to_string()]);
    assert_eq!(
        recall.step(Step::Forward, Caret::default(), "", &[]),
        None,
        "nothing is held open across the send"
    );
}

/// An unrecalled box settles to no-op, however the history moves under it.
#[test]
fn a_live_draft_is_never_settled_away() {
    let mut recall = Recall::default();
    recall.settle("half-typed", &[]);
    assert_eq!(
        recall
            .step(Step::Back, Caret::default(), "half-typed", &["a".into()])
            .unwrap(),
        "a"
    );
}
