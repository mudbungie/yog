//! **What the queue carries, read off a laid world** (bl-778e).
//!
//! A turn that merely came to rest is not an ask: the queue holds a parked tool
//! call, a flag, a failure the loop cannot mend, a question put to the
//! operator — and nothing else. Proved through the production derivation and
//! the production queue answer (`Query::Attention`'s own rows), over the trees
//! `yog fixture` lays, so a seat dialling the same state sees the same answer.

use super::*;
use crate::boundary::answer::queue::{QueueRow, queue};
use crate::ui_state::UiState;

const ORIGIN: i64 = 2_000_000_000;

/// The queue a server booted on the laid `state` answers, with nothing seen.
/// `then` runs against the workspace before the derivation reads it.
fn rows(state: &str, then: impl Fn(&Path)) -> Vec<QueueRow> {
    let (_tmp, places, _) = laid(state, ORIGIN);
    let ws = places.workspace(roster::WORKSPACE);
    then(&ws);
    let tree = crate::git_tree::GitTree::from_repo(&ws).expect("derive");
    let snap = crate::boundary::tests::snapshot(&ws, roster::WORKSPACE, tree.agents, vec![]);
    queue(
        &snap,
        &UiState::open(PathBuf::from("/nonexistent/ui.json")),
        ORIGIN,
    )
}

/// Park a tool call on `agent` the way litany's seam does: a blob of one JSON
/// line under `refs/litany/held/<id>`.
fn park(ws: &Path, agent: &str) {
    let blob = ws.join("held.json");
    std::fs::write(
        &blob,
        r#"{"tool_use_id":"toolu_1","tool":"Bash","reason":"writes"}"#,
    )
    .expect("blob");
    let repo = ws.join("repo.git");
    let out = crate::git_env::output(
        crate::git_env::git()
            .arg("--git-dir")
            .arg(&repo)
            .args(["hash-object", "-w"])
            .arg(&blob),
    )
    .expect("hash-object");
    let oid = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    let refname = format!("{}{agent}", crate::control::hold::HELD_PREFIX);
    crate::fixture::disk::git(&repo, &["update-ref", &refname, &oid], None).expect("park");
}

/// After a plain turn ends the queue is empty: the rest is the row's
/// `quiescent` state, never an ask.
#[test]
fn a_turn_that_came_to_rest_is_not_on_the_queue() {
    assert_eq!(rows("settings", |_| ()), vec![]);
}

/// The same conversation, parked on a tool call, is.
#[test]
fn a_parked_call_is_on_the_queue() {
    let got = rows("settings", |ws| park(ws, "c-401"));
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(got[0].agent, "c-401");
    assert_eq!(
        got[0].signals,
        vec![crate::attention::AttentionKind::Held],
        "the park asks, the clean rest beneath it does not"
    );
}

/// And a failure the loop cannot mend is, both arms of the wound.
#[test]
fn a_failure_is_on_the_queue() {
    let got = rows("wound", |_| ());
    let agents: Vec<&str> = got.iter().map(|r| r.agent.as_str()).collect();
    assert_eq!(agents.len(), 2, "{got:?}");
    assert!(agents.contains(&"c-101") && agents.contains(&"c-102"));
    for row in &got {
        assert_eq!(row.state, crate::git_tree::AgentState::Stopped);
        assert_eq!(row.signals, vec![crate::attention::AttentionKind::Stopped]);
    }
}
