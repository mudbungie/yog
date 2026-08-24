//! The listings (§8.5): what a populating read answered. Each listing carries
//! one row per arm of its row type — the loaded case, the bare case, and every
//! classification token — because a listing whose rows are all the easy case
//! proves only that the easy case survives.

use std::path::PathBuf;

use super::super::super::super::Reply;

/// The §11 altitude-0 answers — the enumeration with its §7.2 notes, and one
/// workspace's ball listing — cut off this file at §12's per-file budget
/// (bl-b4b5) on the seam the surface itself draws: those two are what the
/// chrome asks, and everything left here is what a pane asks.
mod chrome;
/// The conversation rows, the widest row type here.
mod convs;
use super::board::board;
use crate::board::Board;
use crate::config_edit::branch::{ConfigBranch, Lineage};
use crate::config_edit::brazen::ProviderRowView;
use crate::git_tree::AgentState;
use crate::opslog::{OpRow, Origin};
use crate::projects::join::{JoinRow, JoinState};
use crate::search::{Address, Field, Found, Hit};
use convs::conv_rows;

/// The §6 decision queue: a parked row with every signal it can carry, and a
/// quiet one carrying none.
fn queue() -> Vec<crate::boundary::answer::queue::QueueRow> {
    use crate::attention::AttentionKind;
    use crate::boundary::answer::queue::QueueRow;
    vec![
        QueueRow {
            workspace: "ws".into(),
            agent: "c-1".into(),
            display: "Cobalt".into(),
            state: AgentState::Stopped,
            uncertain: false,
            signals: vec![AttentionKind::Held, AttentionKind::Mail],
            preview: "p".into(),
            age_secs: 5,
            pending: 2,
            held: Some(crate::control::hold::Held {
                tool_use_id: "toolu_1".into(),
                tool: "Bash".into(),
                reason: "writes".into(),
            }),
        },
        QueueRow {
            workspace: "ws".into(),
            agent: "c-2".into(),
            display: "Dun".into(),
            state: AgentState::Live,
            uncertain: true,
            signals: vec![],
            preview: String::new(),
            age_secs: 0,
            pending: 0,
            held: None,
        },
    ]
}

/// All three §8.5 address shapes, so the flattened keys are read back under
/// the token that named them rather than by luck of which keys are present.
fn found() -> Found {
    Found {
        needle: "gate".into(),
        hits: vec![
            Hit {
                at: Address::Ball {
                    project: "p".into(),
                    id: "bl-1".into(),
                },
                field: Field::Name,
                offset: 0,
                excerpt: "bl-1".into(),
            },
            Hit {
                at: Address::Workspace {
                    path: PathBuf::from("/ws"),
                },
                field: Field::Summary,
                offset: 3,
                excerpt: "ws".into(),
            },
            Hit {
                at: Address::Conversation {
                    workspace: PathBuf::from("/ws"),
                    agent: "c-1".into(),
                },
                field: Field::Text,
                offset: 12,
                excerpt: "the gate".into(),
            },
        ],
        unreadable: vec!["/p: not a repo".into()],
    }
}

pub(super) fn listings() -> Vec<Reply> {
    let mut out = chrome::chrome();
    out.extend([
        // The §11 altitude-0 answers, in their own file at the budget.
        Reply::Conversations(conv_rows()),
        Reply::Balls(vec![
            JoinRow {
                project: "p".into(),
                ball_id: "bl-1".into(),
                state: JoinState::Delivered,
                workspace: Some("ws".into()),
                claimant: Some("alba".into()),
                title: Some("t".into()),
            },
            JoinRow {
                project: "p".into(),
                ball_id: "bl-2".into(),
                state: JoinState::ReadyStartable,
                workspace: None,
                claimant: None,
                title: None,
            },
        ]),
        Reply::Board(board()),
        // The unarmed world, whose `fleet` key is absent rather than empty.
        Reply::Board(Board::default()),
        Reply::Attention(queue()),
        Reply::Ops(vec![OpRow {
            ts: "1700".into(),
            argv: "bl close x".into(),
            cwd: "/p".into(),
            exit: 1,
            stdout: String::new(),
            stderr: "gate".into(),
            origin: Origin::Balls,
        }]),
        Reply::Help(crate::boundary::help::rows(None)),
        Reply::Search(found()),
        // A search that matched nothing still knows its own question.
        Reply::Search(Found::default()),
        Reply::Providers(vec![
            ProviderRowView {
                name: "anthropic".into(),
                fact: "credential present".into(),
                blocked: None,
            },
            ProviderRowView {
                name: "openai".into(),
                fact: "no credential".into(),
                blocked: Some("no login flow".into()),
            },
        ]),
        Reply::Lineages(vec![Lineage {
            branch: ConfigBranch {
                name: "main".into(),
                tip_oid: "abcdef1234".into(),
                tip_short_oid: "abcdef1".into(),
                tip_timestamp_unix: 1_700_000_000,
            },
            files: vec!["workflow.yaml".into()],
        }]),
        Reply::Models(vec!["opus".into(), "sonnet".into()]),
        // The follow-class read's answer at both of its arms (bl-024b): a hold
        // that ended with nothing, and one carrying work whose input must
        // survive the trip verbatim.
        Reply::Invocations(Vec::new()),
        Reply::Invocations(vec![crate::registry::mailbox::Invocation {
            id: "inv-1".into(),
            tool: "Bash".into(),
            input: serde_json::json!({"command": "ls -l", "timeout": 30}),
        }]),
        // REMOTE §5's roster (bl-4e08): both presence arms, a client that
        // advertises and one that has not, and a schema deep enough that a
        // codec rebuilding it rather than carrying it verbatim would show.
        Reply::Clients(vec![
            crate::registry::roster::ClientRow {
                client: "laptop".into(),
                present: true,
                tools: vec![crate::registry::tools::Tool {
                    name: "Bash".into(),
                    description: "run a command".into(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {"command": {"type": "string", "minLength": 1}},
                        "required": ["command"],
                    }),
                }],
            },
            crate::registry::roster::ClientRow {
                client: "phone".into(),
                present: false,
                tools: Vec::new(),
            },
        ]),
    ]);
    out
}
