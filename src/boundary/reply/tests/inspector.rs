//! The §11 inspector family's wire shapes (bl-6233, REMOTE §9 step 1) — the
//! five surfaces that had no headless spelling at all, so no seat but the
//! window could read a conversation.
//!
//! [`chat`] holds the two whose rows are messages (the transcript and the
//! inbox), [`records`] the three whose rows are machinery (steps, one step's
//! records, the worktree and the spine). Here: the chokepoint drive that proves
//! the query reaches the encoder, over a real workspace on disk.

mod chat;
mod records;

use std::path::PathBuf;
use std::sync::Arc;

use super::super::encode;
use crate::boundary::answer::answer;
use crate::boundary::dispatch::Deps;
use crate::boundary::reply::Reply;
use crate::boundary::{Query, tests::snapshot};
use crate::cli_outbound::Cli;
use crate::ui_state::UiState;

const AGENT: &str = "c-1";

/// A `Deps` wrapping `snap` — the inspector family reads the world's bytes and
/// the snapshot, never a substrate binary.
fn deps(snap: crate::app::Snapshot) -> Deps {
    Deps {
        lernie: Cli::new("/no/such/lernie"),
        bl: Cli::new("/no/such/bl"),
        state_root: PathBuf::from("/nonexistent/state"),
        yog_binary: PathBuf::from("/no/such/yog"),
        world: crate::test_support::no_world(),
        home: PathBuf::from("/home/x"),
        yog_data_root: PathBuf::from("/data"),
        balls_state_root: PathBuf::from("/balls"),
        snapshot: Arc::new(snap),
        mint_seed: 0,
    }
}

fn ui() -> UiState {
    UiState::open(PathBuf::from("/nonexistent/ui.json"))
}

/// **The ball's claim, end to end**: a conversation is readable with no window
/// at all. Every one of the six reads is asked at the chokepoint over a real
/// workspace and answers its own `kind` — which is the whole of what "the chats
/// are unreachable by any face but the window" was missing.
#[test]
fn every_conversation_read_answers_from_the_chokepoint() {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path();
    let work = ws.join("agents").join(AGENT);
    std::fs::create_dir_all(work.join("messages")).unwrap();
    std::fs::write(work.join("messages").join("001-user.md"), b"go").unwrap();
    std::fs::write(work.join("goal.md"), b"ship it").unwrap();
    let inbox = ws.join("inbox").join(AGENT);
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::write(inbox.join("user-001.md"), b"---\nfrom: user\n---\nhi\n").unwrap();
    let deps = deps(snapshot(ws, "alba", vec![], vec![]));
    let at = |query| encode(&answer(&query, &deps, &ui(), 0).expect("a read never refuses"));
    let address = (ws.to_path_buf(), AGENT.to_owned());
    for (query, kind) in [
        (
            Query::Transcript {
                workspace: crate::naming::leaf(&(address.0.clone())),
                agent: address.1.clone(),
            },
            "transcript",
        ),
        (
            Query::Steps {
                workspace: crate::naming::leaf(&(address.0.clone())),
                agent: address.1.clone(),
            },
            "steps",
        ),
        (
            Query::Rail {
                workspace: crate::naming::leaf(&(address.0.clone())),
                agent: address.1.clone(),
            },
            "rail",
        ),
        (
            Query::Inbox {
                workspace: crate::naming::leaf(&(address.0.clone())),
                agent: address.1.clone(),
            },
            "inbox",
        ),
        (
            Query::Step {
                workspace: crate::naming::leaf(&(address.0.clone())),
                agent: address.1.clone(),
                seq: "001".to_owned(),
            },
            "step",
        ),
        (
            Query::Files {
                workspace: crate::naming::leaf(&(address.0.clone())),
                agent: address.1.clone(),
                path: Some("goal.md".to_owned()),
            },
            "files",
        ),
    ] {
        let body = at(query);
        assert_eq!(body["ok"], true);
        assert_eq!(body["kind"], kind);
    }
}

/// The two reads that carry a body answer it: the message that was delivered,
/// and the file that was named. A `kind` alone would prove the wiring and not
/// the reading.
#[test]
fn the_chokepoint_answers_the_bytes_and_not_just_a_kind() {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path();
    let work = ws.join("agents").join(AGENT);
    std::fs::create_dir_all(work.join("messages")).unwrap();
    std::fs::write(work.join("messages").join("001-user.md"), b"go").unwrap();
    std::fs::write(work.join("goal.md"), b"ship it").unwrap();
    let deps = deps(snapshot(ws, "alba", vec![], vec![]));
    let ask = |query| answer(&query, &deps, &ui(), 0).expect("a read never refuses");
    let chat = encode(&ask(Query::Transcript {
        workspace: crate::naming::leaf(ws),
        agent: AGENT.to_owned(),
    }));
    assert_eq!(chat["rows"][0]["body"], "go");
    let files = encode(&ask(Query::Files {
        workspace: crate::naming::leaf(ws),
        agent: AGENT.to_owned(),
        path: Some("goal.md".to_owned()),
    }));
    assert_eq!(files["preview"]["text"], "ship it");
    // And the reply the GUI holds is the same value, not a second derivation.
    assert!(matches!(
        ask(Query::Files {
            workspace: crate::naming::leaf(ws),
            agent: AGENT.to_owned(),
            path: None,
        }),
        Reply::Files { preview: None, .. }
    ));
}
