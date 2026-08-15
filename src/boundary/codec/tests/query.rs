//! The query roster's round trips — cut from the sibling table on the seam
//! production took (`codec/query.rs`): §4.8's taxonomy is a file boundary on
//! both sides.

use super::rt;
use crate::boundary::{Gesture, Query, codec::decode};

#[test]
fn every_query_variant_round_trips() {
    rt(Gesture::Ask(Query::Workspaces));
    rt(Gesture::Ask(Query::Conversations {
        workspace: "ws".into(),
    }));
    rt(Gesture::Ask(Query::Balls));
    rt(Gesture::Ask(Query::Board));
    rt(Gesture::Ask(Query::Attention));
    rt(Gesture::Ask(Query::Ops { max: 32 }));
    rt(Gesture::Ask(Query::Search {
        text: "tekeli-li".into(),
    }));
    for file in [
        None,
        Some(crate::workdiff::WorkFile {
            ball: "bl-1".into(),
            path: "src/a.rs".into(),
        }),
    ] {
        rt(Gesture::Ask(Query::WorkDiff {
            workspace: "ws".into(),
            file,
        }));
    }
    // The §9 browse and roster (bl-dff8), each carrying the sphere it is asked
    // in — providers, sign-ins and lineages all live inside a workspace.
    rt(Gesture::Ask(Query::Lineages {
        workspace: "ws".into(),
    }));
    rt(Gesture::Ask(Query::Models {
        workspace: "ws".into(),
        provider: "acme".into(),
    }));
    // REMOTE §5's roster (bl-4e08).
    rt(Gesture::Ask(Query::Clients {
        workspace: "ws".into(),
    }));
    // The routing leg's two reads (bl-024b): one names a handle, the other
    // names nothing — the queue it drains is the intake's own.
    rt(Gesture::Ask(Query::Invocations));
    rt(Gesture::Ask(Query::Capture {
        invocation: "inv-1".into(),
    }));
    inspector_family();
}

/// The §11 inspector family (bl-6233, bl-13f9): the reads addressed at a
/// conversation rather than a workspace, so each carries both halves of the
/// address and only what no seat could supply beside them.
fn inspector_family() {
    let (workspace, agent) = ("ws".to_owned(), "c-1".to_owned());
    for query in [
        Query::Transcript {
            workspace: workspace.clone(),
            agent: agent.clone(),
        },
        Query::Steps {
            workspace: workspace.clone(),
            agent: agent.clone(),
        },
        Query::Rail {
            workspace: workspace.clone(),
            agent: agent.clone(),
        },
        Query::Inbox {
            workspace: workspace.clone(),
            agent: agent.clone(),
        },
        Query::Step {
            workspace: workspace.clone(),
            agent: agent.clone(),
            seq: "003".to_owned(),
        },
        Query::Agent {
            workspace: workspace.clone(),
            agent: agent.clone(),
        },
    ] {
        rt(Gesture::Ask(query));
    }
    // The listing and one file's bytes are the same query at two depths — the
    // `work-diff` shape, so the path is optional and both sides round-trip. The
    // tree is the third selection on it (bl-44e9): live, or as of one commit.
    for path in [None, Some("src/a.rs".to_owned())] {
        for at in [None, Some("abcdef1234".to_owned())] {
            rt(Gesture::Ask(Query::Files {
                workspace: workspace.clone(),
                agent: agent.clone(),
                path: path.clone(),
                at,
            }));
        }
    }
    // Config-frozen-at (bl-13f9): the same optional commit, at the family's
    // other tree-subject read — bare is the conversation's own tip.
    for at in [None, Some("abcdef1234".to_owned())] {
        rt(Gesture::Ask(Query::Governing {
            workspace: workspace.clone(),
            agent: agent.clone(),
            at,
        }));
    }
}

/// A conversation read names its conversation, always: half an address would
/// answer about a different chat, so the envelope refuses rather than guess.
#[test]
fn an_inspector_envelope_missing_half_its_address_is_refused() {
    for op in [
        "transcript",
        "steps",
        "step",
        "files",
        "governing",
        "rail",
        "inbox",
    ] {
        assert!(
            decode(&serde_json::json!({ "op": op, "workspace": "/ws" })).is_err(),
            "{op} without an agent"
        );
        assert!(
            decode(&serde_json::json!({ "op": op, "agent": "c-1" })).is_err(),
            "{op} without a workspace"
        );
    }
    // And a step names the step: "some step" is not a question.
    assert!(
        decode(&serde_json::json!({ "op": "step", "workspace": "/ws", "agent": "c-1" })).is_err()
    );
}

/// A roster names its row, always: the envelope refuses rather than list some
/// other provider's models (and, like every wall-scoped read, it names its
/// workspace).
#[test]
fn a_roster_envelope_without_its_provider_is_refused() {
    assert!(decode(&serde_json::json!({ "op": "models", "workspace": "/ws" })).is_err());
    assert!(decode(&serde_json::json!({ "op": "models", "provider": "acme" })).is_err());
    assert!(decode(&serde_json::json!({ "op": "lineages" })).is_err());
}

/// The work-diff's `file` is all-or-nothing: half of it is a patch read that
/// would open the wrong file, so the envelope refuses rather than guessing.
#[test]
fn a_half_named_work_file_is_refused() {
    let envelope = |file: serde_json::Value| serde_json::json!({ "op": "work-diff", "workspace": "/ws", "file": file });
    assert!(decode(&envelope(serde_json::json!({ "ball": "bl-1" }))).is_err());
    assert!(decode(&envelope(serde_json::json!({ "path": "a.rs" }))).is_err());
    assert_eq!(
        decode(&envelope(serde_json::json!("src/a.rs"))),
        Err("file: not a JSON object".to_owned())
    );
}
