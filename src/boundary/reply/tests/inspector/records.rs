//! The three inspector replies whose rows are **machinery**: the steps list,
//! one step's records, the agent worktree and the step spine. What is pinned
//! here is that every absence stays an absence — a step that recorded no
//! `meta.json`, a notch with no seat, a worktree that is gone — because on the
//! wire, as on screen, a zero standing in for an unrecorded fact is a lie.

use crate::boundary::reply::{Reply, encode};
use crate::budgets::BudgetSpend;
use crate::files_view::{FileEntry, FilesView, Preview};
use crate::git_tree::{AgentState, Framing};
use crate::login::auth::AuthFailure;
use crate::rail::{ChildCard, Notch, Place, Rail};
use crate::steps_view::{Doc, StepDetail, StepSummary, StepsView, ToolIo, UNPARSED, Wound};

fn step(seq: &str, framing: Framing) -> StepSummary {
    StepSummary {
        seq: seq.to_owned(),
        framing,
        attempts: 1,
        tokens: BudgetSpend {
            input_tokens: 3,
            output_tokens: 4,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        },
        commit: None,
        started_at: None,
        ended_at: None,
        auth_failed: AuthFailure::No,
        wound: Wound::None,
    }
}

/// A settled step states everything `meta.json` recorded; an unsettled one
/// omits exactly what it did not record, and says its own two verdicts — the
/// §8.3 login affordance and the §7.3 wound — rather than leaving a reader to
/// infer them from a framing.
#[test]
fn a_step_row_states_what_was_recorded_and_omits_what_was_not() {
    let mut settled = step("001", Framing::Complete);
    settled.commit = Some("abc".to_owned());
    settled.started_at = Some("t0".to_owned());
    settled.ended_at = Some("t1".to_owned());
    let mut failed = step("002", Framing::Failed);
    failed.auth_failed = AuthFailure::Row("acme".to_owned());
    let mut killed = step("003", Framing::Killed);
    killed.auth_failed = AuthFailure::Unrouted;
    killed.wound = Wound::Spoke("no credentials".to_owned());
    let rows = encode(&Reply::Steps(StepsView {
        steps: vec![settled, failed, killed],
    }));
    assert_eq!(rows["kind"], "steps");
    let at = |i: usize| rows["rows"][i].clone();
    assert_eq!(at(0)["framing"], "complete");
    assert_eq!(at(0)["attempts"], 1);
    assert_eq!(at(0)["commit"], "abc");
    assert_eq!(at(0)["started_at"], "t0");
    assert_eq!(at(0)["ended_at"], "t1");
    assert_eq!(at(0)["tokens"]["input"], 3);
    assert_eq!(at(0)["tokens"]["total"], 7);
    assert_eq!(at(0)["auth_failed"], false);
    assert_eq!(at(0)["wounded"], false);
    assert_eq!(at(1)["framing"], "failed");
    assert_eq!(at(1)["auth_failed"], true);
    assert_eq!(at(1)["auth_row"], "acme");
    assert_eq!(at(2)["framing"], "killed");
    // Unrouted: the affordance is offered and there is nothing to pick for you.
    assert_eq!(at(2)["auth_failed"], true);
    assert!(at(2).get("auth_row").is_none());
    assert_eq!(at(2)["wounded"], true);
    assert_eq!(at(2)["wound_reason"], "no credentials");
    for key in ["commit", "started_at", "ended_at"] {
        assert!(at(2).get(key).is_none(), "{key} was never recorded");
    }
}

/// The drill-in's three record classes stay apart: parsed (with the bytes it
/// parsed from beside it), absent, and bytes that are not JSON — framed as
/// malformed, because rendered bare that is indistinguishable from a file whose
/// content happens to be that text.
#[test]
fn a_records_reply_keeps_parsed_absent_and_malformed_apart() {
    let body = encode(&Reply::Step(StepDetail {
        seq: "001".to_owned(),
        meta: Doc::Json {
            value: serde_json::json!({ "commit": "abc" }),
            raw: br#"{"commit":"abc"}"#.to_vec(),
        },
        request: Doc::Absent,
        staging: Doc::Unparsed(b"not json".to_vec()),
        response: vec![Doc::Json {
            value: serde_json::json!({ "type": "end" }),
            raw: br#"{"type":"end"}"#.to_vec(),
        }],
        tools: vec![ToolIo {
            tool_id: "t-1".to_owned(),
            input: Doc::Absent,
            output: Doc::Unparsed(b"boom".to_vec()),
            is_error: true,
        }],
    }));
    assert_eq!(body["kind"], "step");
    assert_eq!(body["seq"], "001");
    assert_eq!(body["meta"]["kind"], "json");
    assert_eq!(body["meta"]["value"]["commit"], "abc");
    assert_eq!(body["meta"]["raw"], r#"{"commit":"abc"}"#);
    assert_eq!(body["request"]["kind"], "absent");
    assert_eq!(body["staging"]["kind"], "unparsed");
    assert_eq!(body["staging"]["note"], UNPARSED);
    assert_eq!(body["staging"]["raw"], "not json");
    assert_eq!(body["response"][0]["value"]["type"], "end");
    assert_eq!(body["tools"][0]["tool_id"], "t-1");
    assert_eq!(body["tools"][0]["is_error"], true);
    assert_eq!(body["tools"][0]["output"]["raw"], "boom");
}

/// A torn-down worktree is a fact, not an empty listing (§3.5) — so `rows` is
/// present exactly when there is a worktree to list, and a reader never has to
/// tell "gone" from "nothing in it".
#[test]
fn a_files_reply_tells_an_absent_worktree_from_an_empty_one() {
    let present = encode(&Reply::Files {
        view: FilesView::Present {
            entries: vec![
                FileEntry {
                    rel_path: "src".to_owned(),
                    size: 0,
                    is_dir: true,
                },
                FileEntry {
                    rel_path: "src/a.rs".to_owned(),
                    size: 12,
                    is_dir: false,
                },
            ],
            truncated: true,
        },
        preview: Some(Preview::Text("fn main() {}".to_owned())),
    });
    assert_eq!(present["kind"], "files");
    assert_eq!(present["worktree"], true);
    assert_eq!(present["truncated"], true);
    assert_eq!(present["rows"][0]["dir"], true);
    assert_eq!(present["rows"][1]["path"], "src/a.rs");
    assert_eq!(present["rows"][1]["size"], 12);
    assert_eq!(present["preview"]["kind"], "text");
    let gone = encode(&Reply::Files {
        view: FilesView::AbsentWorktree,
        preview: None,
    });
    assert_eq!(gone["worktree"], false);
    assert!(gone.get("rows").is_none(), "there is nothing to list");
    assert!(gone.get("preview").is_none());
}

/// The spine: a notch that can be pinned states its commit and its seat, one
/// that cannot omits both — which is exactly what makes it unreachable — and a
/// card names the notch it was born at.
#[test]
fn a_rail_reply_omits_what_makes_a_notch_unpinnable() {
    let body = encode(&Reply::Rail(Rail {
        notches: vec![
            Notch {
                seq: "001".to_owned(),
                commit: Some("abcdef1234".to_owned()),
                budget: 7,
                place: Some(Place {
                    row: "001-user.md".to_owned(),
                    cut: 1,
                }),
            },
            Notch {
                seq: "002".to_owned(),
                commit: None,
                budget: 7,
                place: None,
            },
        ],
        cards: vec![ChildCard {
            agent_id: "c-1-w-1".to_owned(),
            name: "koi".to_owned(),
            fork: "from here".to_owned(),
            state: AgentState::InFlight,
            tokens: 12,
            tail: Some("still going".to_owned()),
            provenance_notch: 0,
        }],
    }));
    assert_eq!(body["kind"], "rail");
    assert_eq!(body["rows"][0]["commit"], "abcdef1234");
    assert_eq!(body["rows"][0]["short"], "abcdef1");
    assert_eq!(body["rows"][0]["row"], "001-user.md");
    assert_eq!(body["rows"][0]["cut"], 1);
    assert_eq!(body["rows"][0]["budget"], 7);
    for key in ["commit", "short", "row", "cut"] {
        assert!(body["rows"][1].get(key).is_none(), "{key} unpinnable");
    }
    assert_eq!(body["cards"][0]["agent"], "c-1-w-1");
    assert_eq!(body["cards"][0]["fork"], "from here");
    assert_eq!(body["cards"][0]["state"], "in-flight");
    assert_eq!(body["cards"][0]["notch"], 0);
    assert_eq!(body["cards"][0]["tail"], "still going");
    // A child that has produced no inference text says nothing, rather than
    // saying the empty string — two different claims.
    let quiet = encode(&Reply::Rail(Rail {
        notches: Vec::new(),
        cards: vec![ChildCard {
            agent_id: "c-1-w-2".to_owned(),
            name: "eel".to_owned(),
            fork: "from config/default".to_owned(),
            state: AgentState::Quiescent,
            tokens: 0,
            tail: None,
            provenance_notch: 0,
        }],
    }));
    assert!(quiet["cards"][0].get("tail").is_none());
}
