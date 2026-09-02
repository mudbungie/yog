//! The §11 inspector family (bl-6233) and the work diff that shares its shape:
//! the six conversation-addressed reads. Every entry kind, every record class,
//! every churn and every preview arm appears once, because each is its own
//! decode arm.

use std::collections::BTreeMap;

mod science;
/// The steps pane's own two reads, its own file at §12's cap.
mod steps;
mod workdiff;

use super::super::super::super::Reply;
use super::preview;
use crate::files_view::{FileEntry, FilesView, Preview};
use crate::git_tree::AgentState;
use crate::inboxview::{Deposit, Epitaph, InboxEntry};
use crate::rail::{ChildCard, Notch, Place, Rail};
use crate::steps_view::{Orphan, StepsView};
use crate::transcript::{Block, Entry, EntryKind, Transcript};
use science::science;
use steps::{step_detail, steps};
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
            entry(
                "«006–008»",
                EntryKind::Compacted {
                    first: 6,
                    last: 8,
                    summary: "what the compactor cut".into(),
                },
            ),
            entry("005-junk.json", EntryKind::Raw),
        ],
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
        // The §3.9 projection over those same attempts (bl-40ab): one row with
        // every optional column populated, one with none of them, so a decoder
        // that dropped either reading would not pass on the other.
        Reply::Science(science()),
        Reply::Science(vec![]),
        Reply::Transcript(transcript()),
        // One follow frame (bl-73e7). Three readings of the fold, because each
        // field is absent until a delta of its kind lands and absence is not
        // the same claim as an empty string: nothing said yet, reasoning only,
        // and both with the last delta being the answer.
        Reply::Follow(crate::git_tree::Stream::default()),
        Reply::Follow(crate::git_tree::Stream {
            thinking: Some("first I".into()),
            text: None,
            last_delta: Some(crate::git_tree::Delta::Thinking),
        }),
        Reply::Follow(crate::git_tree::Stream {
            thinking: Some("first I".into()),
            text: Some("then this".into()),
            last_delta: Some(crate::git_tree::Delta::Text),
        }),
        Reply::Steps(steps()),
        Reply::Steps(StepsView {
            steps: vec![],
            orphan: Orphan::Mute(crate::steps_view::Tail::ToolWindow),
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
            working_dir: Some("/home/u/proj".into()),
        },
        // The torn-down worktree, whose absence is a fact rather than an
        // empty listing — and the opaque preview arm beside it.
        Reply::Files {
            view: FilesView::AbsentWorktree,
            preview: Some(Preview::Binary { size: 4 }),
            working_dir: None,
        },
        Reply::Rail(rail()),
        Reply::Inbox(inbox()),
    ]
}
