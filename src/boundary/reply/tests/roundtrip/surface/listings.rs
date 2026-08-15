//! The listings (§8.5): what a populating read answered. Each listing carries
//! one row per arm of its row type — the loaded case, the bare case, and every
//! classification token — because a listing whose rows are all the easy case
//! proves only that the easy case survives.

use std::path::PathBuf;

use super::super::super::super::{Reply, WsRow};
use super::board::board;
use crate::binding::WorkspaceKind;
use crate::board::Board;
use crate::config_edit::branch::{ConfigBranch, Lineage};
use crate::config_edit::brazen::ProviderRowView;
use crate::git_tree::AgentState;
use crate::monitor::{Check, Verdict};
use crate::nav::convs::{ConvBall, ConvRow, Flight};
use crate::opslog::{OpRow, Origin};
use crate::projects::join::{JoinRow, JoinState};
use crate::search::{Address, Field, Found, Hit};
use crate::transcript::Tone;

/// The §11 conversation rows: the fully-loaded one, the bare one, and the
/// display-only rung whose `name` the wire withholds and the decode recovers
/// off `display` (bl-7067).
fn conv_rows() -> Vec<ConvRow> {
    let full = ConvRow {
        root_id: "c-1".into(),
        state: AgentState::InFlight,
        uncertain: true,
        preview: "first line".into(),
        age_secs: 42,
        flight: Some(Flight::Inference),
        attention: 1,
        members: 3,
        depth: 2,
        direct: 2,
        stoppable: false,
        stop_children: false,
        ball: Some(ConvBall {
            id: "bl-7".into(),
            state: Some(JoinState::Bound),
            title: Some("t".into()),
            badge: Some("closed".into()),
        }),
        name: Some("brave-fox".into()),
        name_display_only: false,
        verdict: Some(Check {
            workspace: "/ws".into(),
            agent: "c-1".into(),
            verdict: Verdict::Drifting,
            sha: "deadbeef".into(),
            reason: "wandered".into(),
            model: "m".into(),
            input_tokens: Some(7),
            output_tokens: None,
        }),
        tone: Tone::Weak,
    };
    let bare = ConvRow {
        root_id: "c-2".into(),
        state: AgentState::Quiescent,
        uncertain: false,
        preview: String::new(),
        age_secs: 0,
        flight: None,
        attention: 0,
        members: 1,
        depth: 0,
        direct: 0,
        stoppable: false,
        stop_children: false,
        ball: Some(ConvBall {
            id: "stray".into(),
            state: None,
            title: None,
            badge: None,
        }),
        name: None,
        name_display_only: false,
        verdict: None,
        tone: Tone::Plain,
    };
    let legacy = ConvRow {
        root_id: "c-3".into(),
        name: Some("goal-stamped".into()),
        name_display_only: true,
        ball: None,
        ..bare.clone()
    };
    vec![full, bare, legacy]
}

/// The §6 decision queue: a parked row with every signal it can carry, and a
/// quiet one carrying none.
fn queue() -> Vec<crate::boundary::answer::queue::QueueRow> {
    use crate::attention::AttentionKind;
    use crate::boundary::answer::queue::QueueRow;
    vec![
        QueueRow {
            workspace: PathBuf::from("/ws"),
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
            workspace: PathBuf::from("/ws"),
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
                    project: PathBuf::from("/p"),
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

fn workspaces() -> Vec<WsRow> {
    let row = |name: &str, kind, attention, agents, running, pinned| WsRow {
        workspace: name.to_owned(),
        kind,
        attention,
        agents,
        running,
        pinned,
    };
    vec![
        row(
            "alba",
            WorkspaceKind::Named {
                name: "alba".into(),
            },
            2,
            5,
            true,
            // Both arms of the §4.1 pin rank ride the round trip: an absent one
            // must not read back as rank 0, which is the first hoisted tab.
            Some(1),
        ),
        row("f", WorkspaceKind::Foreign, 0, 0, false, None),
        row("r", WorkspaceKind::Replay, 0, 1, false, Some(0)),
    ]
}

pub(super) fn listings() -> Vec<Reply> {
    vec![
        Reply::Workspaces(workspaces()),
        Reply::Conversations(conv_rows()),
        Reply::Balls(vec![
            JoinRow {
                project: PathBuf::from("/p"),
                ball_id: "bl-1".into(),
                state: JoinState::Delivered,
                workspace: Some(PathBuf::from("/ws")),
                claimant: Some("alba".into()),
                title: Some("t".into()),
            },
            JoinRow {
                project: PathBuf::from("/p"),
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
    ]
}
