//! The §11 inspector family (bl-6233) and the work diff that shares its shape:
//! the six conversation-addressed reads. Every entry kind, every record class,
//! every churn and every preview arm appears once, because each is its own
//! decode arm.

use std::collections::BTreeMap;

mod workdiff;

use super::super::super::super::Reply;
use super::{preview, spend};
use crate::files_view::{FileEntry, FilesView, Preview};
use crate::git_tree::{AgentState, Framing};
use crate::inboxview::{Deposit, Epitaph, InboxEntry};
use crate::login::auth::AuthFailure;
use crate::rail::{ChildCard, Notch, Place, Rail};
use crate::steps_view::{Doc, Orphan, StepDetail, StepSummary, StepsView, ToolIo, Wound};
use crate::transcript::{Block, Entry, EntryKind, Transcript};
use workdiff::attempts;

/// One entry per [`EntryKind`] arm, the epitaph present and absent.
fn transcript() -> Transcript {
    let entry = |name: &str, kind| Entry {
        name: name.into(),
        raw: b"{}".to_vec(),
        kind,
    };
    Transcript {
        entries: vec![
            entry(
                "001-user.md",
                EntryKind::Delivered {
                    sender: "user".into(),
                    epitaph: Some(Epitaph::BudgetExhausted),
                    body: "hello".into(),
                },
            ),
            entry(
                "002-user.md",
                EntryKind::Delivered {
                    sender: "user".into(),
                    epitaph: None,
                    body: String::new(),
                },
            ),
            entry(
                "003-claude.json",
                EntryKind::Model {
                    model_id: "opus".into(),
                    blocks: vec![
                        Block::Text("said".into()),
                        Block::Thinking("thought".into()),
                        Block::ToolUse {
                            id: "toolu_1".into(),
                            name: "Read".into(),
                            input_summary: "file".into(),
                        },
                    ],
                    usage: BTreeMap::from([("input_tokens".to_owned(), 5_u64)]),
                },
            ),
            entry(
                "004-tool.json",
                EntryKind::ToolResult {
                    tool_use_id: "toolu_1".into(),
                    content: "out".into(),
                    is_error: true,
                },
            ),
            entry(
                "streaming",
                EntryKind::Streaming {
                    thinking: "...".into(),
                    text: "partial".into(),
                },
            ),
            entry("005-junk.json", EntryKind::Raw),
        ],
    }
}

/// One step per [`Framing`], per [`AuthFailure`] and per [`Wound`] arm — the
/// two pair-encoded shapes are the ones a widening would have been needed for
/// had either not been a bijection.
fn steps() -> StepsView {
    let base = StepSummary {
        seq: "001".into(),
        framing: Framing::Complete,
        attempts: 1,
        tokens: spend(),
        commit: Some("abc".into()),
        started_at: Some("t0".into()),
        ended_at: Some("t1".into()),
        auth_failed: AuthFailure::No,
        wound: Wound::None,
    };
    StepsView {
        steps: vec![
            base.clone(),
            StepSummary {
                seq: "002".into(),
                framing: Framing::Failed,
                auth_failed: AuthFailure::Row("anthropic".into()),
                wound: Wound::Spoke("no bytes".into()),
                commit: None,
                started_at: None,
                ended_at: None,
                ..base.clone()
            },
            StepSummary {
                seq: "003".into(),
                framing: Framing::Killed,
                auth_failed: AuthFailure::Unrouted,
                wound: Wound::Mute,
                ..base
            },
        ],
        // The view-level orphaned-mail pair (bl-ace6): the Spoke arm here,
        // the Mute and None arms as their own replies below.
        orphan: Orphan::Spoke("driver died".into()),
    }
}

/// One record per [`Doc`] arm: parsed with its bytes, absent, and bytes that
/// are not JSON — plus one capture log present and one absent (bl-83d6), the
/// pair the picker's row set is derived from.
fn step_detail() -> StepDetail {
    StepDetail {
        seq: "001".into(),
        meta: Doc::Json {
            value: serde_json::json!({ "commit": "abc" }),
            raw: br#"{"commit":"abc"}"#.to_vec(),
        },
        request: Doc::Absent,
        staging: Doc::Unparsed(b"not json".to_vec()),
        response: vec![Doc::Absent],
        tools: vec![ToolIo {
            tool_id: "toolu_1".into(),
            input: Doc::Absent,
            output: Doc::Unparsed(b"raw".to_vec()),
            is_error: false,
        }],
        stderr: Some(crate::files_view::Preview::Truncated {
            text: "the adapter's last words".into(),
            size: 999_999,
        }),
        driver: None,
    }
}

/// A pinnable notch and an unreachable one; a card mid-sentence and a silent
/// one — the four absences the encoder spells as absent keys.
fn rail() -> Rail {
    Rail {
        notches: vec![
            Notch {
                seq: "001".into(),
                commit: Some("abcdef1234567890".into()),
                budget: 120,
                place: Some(Place {
                    row: "003-claude.json".into(),
                    cut: 2,
                }),
            },
            Notch {
                seq: "002".into(),
                commit: None,
                budget: 120,
                place: None,
            },
        ],
        cards: vec![
            ChildCard {
                agent_id: "c-1-a".into(),
                name: "Cobalt".into(),
                fork: "from here".into(),
                state: AgentState::Live,
                tokens: 9,
                tail: Some("working".into()),
                provenance_notch: 0,
            },
            ChildCard {
                agent_id: "c-1-b".into(),
                name: "Dun".into(),
                fork: "from config/main".into(),
                state: AgentState::Stopped,
                tokens: 0,
                tail: None,
                provenance_notch: 1,
            },
        ],
    }
}

/// A parsed deposit with every frontmatter field — the epitaph an unrecognized
/// one, which rides through verbatim — and a bare one with none.
fn inbox() -> Vec<InboxEntry> {
    vec![
        InboxEntry {
            name: "user-001.md".into(),
            raw: b"---\nfrom: user\n---\nhi".to_vec(),
            deposit: Deposit {
                sender: Some("user".into()),
                deposited_at: Some("t0".into()),
                epitaph: Some(Epitaph::Unknown("sideways".into())),
                terminal_ref: Some("refs/x".into()),
                body: "hi".into(),
            },
        },
        InboxEntry {
            name: "raw.md".into(),
            raw: b"bare".to_vec(),
            deposit: Deposit::default(),
        },
    ]
}

pub(super) fn inspector() -> Vec<Reply> {
    vec![
        Reply::WorkDiff {
            attempts: attempts(),
            patch: Some(preview()),
        },
        Reply::WorkDiff {
            attempts: vec![],
            patch: None,
        },
        Reply::Transcript(transcript()),
        Reply::Steps(steps()),
        Reply::Steps(StepsView {
            steps: vec![],
            orphan: Orphan::Mute,
        }),
        Reply::Steps(StepsView::default()),
        Reply::Step(step_detail()),
        Reply::Files {
            view: FilesView::Present {
                entries: vec![FileEntry {
                    rel_path: "src/a.rs".into(),
                    size: 12,
                    is_dir: false,
                }],
                truncated: true,
            },
            preview: Some(Preview::Text("body".into())),
        },
        // The torn-down worktree, whose absence is a fact rather than an
        // empty listing — and the opaque preview arm beside it.
        Reply::Files {
            view: FilesView::AbsentWorktree,
            preview: Some(Preview::Binary { size: 4 }),
        },
        Reply::Rail(rail()),
        Reply::Inbox(inbox()),
    ]
}
