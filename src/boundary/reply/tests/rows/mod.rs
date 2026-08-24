//! The row encoders' tables ([`rows`](super::super::rows)): every field
//! a workspace, join or ops row carries. The conversation row is the widest of
//! the four and carries the two token mappings besides, so it is [`convs`],
//! split off at §12's cap on the row types' own seam.

/// The conversation row and its token mappings.
mod convs;

use super::super::{Reply, WsRow, encode};
use crate::opslog::OpRow;
use crate::opslog::Origin;
use crate::projects::join::JoinRow;
use crate::projects::join::JoinState;

#[test]
fn workspace_rows_carry_the_classification_and_rollups() {
    use crate::binding::WorkspaceKind;
    let rows = vec![
        WsRow {
            workspace: "alba".into(),
            kind: WorkspaceKind::Named {
                name: "alba".into(),
            },
            attention: 2,
            agents: 5,
            running: true,
            pinned: Some(3),
            config_tip: Some(crate::model_pick::ConfigTip {
                oid: "c".repeat(40),
                short_oid: "cccccccc".into(),
            }),
        },
        WsRow {
            workspace: "f".into(),
            kind: WorkspaceKind::Foreign,
            attention: 0,
            agents: 0,
            running: false,
            pinned: None,
            config_tip: None,
        },
        WsRow {
            workspace: "r".into(),
            kind: WorkspaceKind::Replay,
            attention: 0,
            agents: 1,
            running: false,
            pinned: Some(0),
            config_tip: None,
        },
    ];
    let v = encode(&Reply::Workspaces(crate::boundary::reply::Workspaces {
        rows,
        // The §7.2 notes ride the same answer (bl-b4b5): a fresh derivation
        // says neither, and the encoder must then write neither key.
        stale: None,
        growth: None,
    }));
    assert_eq!(v["kind"], "workspaces");
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(rows[0]["kind"], "named");
    // The row's identity is its NAME and there is no second copy of it
    // (REMOTE §8, bl-f5f6) — the path it used to carry is gone with it.
    assert_eq!(rows[0]["workspace"], "alba");
    assert!(rows[0].get("name").is_none(), "the name is the identity");
    assert_eq!(rows[0]["attention"], 2);
    assert_eq!(rows[0]["running"], true);
    assert_eq!(rows[1]["kind"], "foreign");
    assert_eq!(
        rows[1]["workspace"], "f",
        "a foreign leaf names it just as well"
    );
    assert_eq!(rows[2]["kind"], "replay");
    // The §4.1 pin rank (bl-296f): stated where there is one, **absent** where
    // there is not — rank 0 is a real hoist and must not read as "unpinned".
    assert_eq!(rows[0]["pinned"], 3);
    assert!(rows[1].get("pinned").is_none(), "unpinned states nothing");
    assert_eq!(
        rows[2]["pinned"], 0,
        "the first hoist is a rank, not a flag"
    );
    // The §2.2 lineage tip, both oids where there is one and **absent** where
    // there is not (bl-b4b5) — a workspace with no lineage derived yet.
    assert_eq!(rows[0]["config_tip"]["short_oid"], "cccccccc");
    assert!(rows[1].get("config_tip").is_none(), "no lineage, no key");
    // A fresh derivation states neither §7.2 note.
    assert!(v.get("stale").is_none(), "a current answer says nothing");
    assert!(v.get("growth").is_none(), "a quiet world says nothing");
}

#[test]
fn join_rows_encode_the_binding_facts() {
    let full = JoinRow {
        project: "p".into(),
        ball_id: "bl-1".into(),
        state: JoinState::Delivered,
        workspace: Some("ws".into()),
        claimant: Some("alba".into()),
        title: Some("t".into()),
    };
    let bare = JoinRow {
        project: "p".into(),
        ball_id: "bl-2".into(),
        state: JoinState::ReadyStartable,
        workspace: None,
        claimant: None,
        title: None,
    };
    let v = encode(&Reply::Balls(vec![full, bare]));
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(rows[0]["state"], "delivered");
    // Both addresses are §8.1 names now (bl-b4b5), not paths.
    assert_eq!(rows[0]["project"], "p");
    assert_eq!(rows[0]["workspace"], "ws");
    assert_eq!(rows[0]["claimant"], "alba");
    assert_eq!(rows[1]["state"], "ready");
    assert!(rows[1].get("workspace").is_none());
    assert!(rows[1].get("claimant").is_none());
    assert!(rows[1].get("title").is_none());
}

#[test]
fn ops_rows_encode_the_durable_line_fields() {
    let row = OpRow {
        ts: "1700".into(),
        argv: "bl close x".into(),
        cwd: "/p".into(),
        exit: 1,
        stdout: String::new(),
        stderr: "gate".into(),
        origin: Origin::Balls,
    };
    let v = encode(&Reply::Ops(vec![row]));
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(rows[0]["ts"], "1700");
    assert_eq!(rows[0]["exit"], 1);
    assert_eq!(rows[0]["origin"], "balls");
}
