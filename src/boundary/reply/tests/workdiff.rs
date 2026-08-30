//! **S11-T4 headless work-diff**: the query answers off the same derivation
//! the Work tab renders, and its wire shape keeps every state distinguishable
//! — an unreadable project, an absent ref and a real diff are three different
//! answers on the wire exactly as they are three different sentences on screen.

use std::path::PathBuf;
use std::sync::Arc;

use super::super::encode;
use crate::boundary::answer::answer;
use crate::boundary::dispatch::Deps;
use crate::boundary::reply::Reply;
use crate::boundary::{Query, tests::snapshot};
use crate::cli_outbound::Cli;
use crate::files_view::Preview;
use crate::ui_state::UiState;
use crate::workdiff::{Attempt, Change, Churn, FileChurn, WorkFile};

/// A `Deps` wrapping `snap` — this query never touches the rest of it.
/// `pub(super)` since bl-40ab: the §3.9 projection's own arm is answered from
/// the same chokepoint over the same environment, and a second copy of this
/// would be a second answer to what a boundary read runs in.
pub(super) fn deps(snap: crate::app::Snapshot) -> Deps {
    Deps {
        litany: Cli::new("/no/such/litany"),
        bl: Cli::new("/no/such/bl"),
        state_root: PathBuf::from("/nonexistent/state"),
        yog_binary: PathBuf::from("/no/such/yog"),
        world: crate::test_support::no_world(),
        home: PathBuf::from("/home/x"),
        yog_data_root: PathBuf::from("/data"),
        balls_state_root: PathBuf::from("/balls"),
        snapshot: Arc::new(snap),
        caller: crate::boundary::dispatch::Caller::default(),
    }
}

fn attempt(change: Change) -> Attempt {
    Attempt {
        project: "proj".to_owned(),
        ball_id: "bl-1".to_owned(),
        handle: None,
        delivered: None,
        change,
    }
}

fn diff() -> Change {
    Change::Diff {
        target: "main".to_owned(),
        source: "work/bl-1".to_owned(),
        target_oid: "aaaa".to_owned(),
        source_oid: "bbbb".to_owned(),
        files: vec![
            FileChurn {
                path: "src/a.rs".to_owned(),
                churn: Churn::Text {
                    added: 2,
                    removed: 1,
                },
            },
            FileChurn {
                path: "logo.png".to_owned(),
                churn: Churn::Binary,
            },
        ],
        truncated: false,
    }
}

/// The diff row carries both refs, both commits, and one entry per changed
/// file — binary said as itself, never as zero lines.
#[test]
fn a_diff_row_carries_the_range_the_commits_and_the_files() {
    let v = encode(&Reply::WorkDiff {
        attempts: vec![attempt(diff())],
        patch: None,
    });
    assert_eq!(v["ok"], true);
    assert_eq!(v["kind"], "work-diff");
    let row = &v["rows"][0];
    assert_eq!(row["project"], "proj");
    assert_eq!(row["ball_id"], "bl-1");
    assert_eq!(row["state"], "diff");
    assert_eq!(row["target"], "main");
    assert_eq!(row["source"], "work/bl-1");
    assert_eq!(row["target_oid"], "aaaa");
    assert_eq!(row["source_oid"], "bbbb");
    assert_eq!(row["truncated"], false);
    assert_eq!(row["files"][0]["added"], 2);
    assert_eq!(row["files"][0]["removed"], 1);
    assert_eq!(row["files"][1]["binary"], true);
    assert!(v.get("patch").is_none(), "no file was asked for");
}

/// The two declines keep their own tokens: an answer that flattened them into
/// "no rows" would be the silent empty listing the ruling forbids.
#[test]
fn every_decline_keeps_its_own_token() {
    let v = encode(&Reply::WorkDiff {
        attempts: vec![
            attempt(Change::Unreadable),
            attempt(Change::Absent {
                target: "main".to_owned(),
                source: "work/bl-1".to_owned(),
                missing: vec!["work/bl-1".to_owned()],
            }),
        ],
        patch: None,
    });
    assert_eq!(v["rows"][0]["state"], "unreadable");
    assert!(v["rows"][0].get("target").is_none(), "nothing to name");
    assert_eq!(v["rows"][1]["state"], "absent");
    assert_eq!(v["rows"][1]["missing"][0], "work/bl-1");
}

/// The patch rides the same three classes the seat paints.
#[test]
fn the_patch_rides_the_previews_own_classes() {
    let with = |patch| {
        encode(&Reply::WorkDiff {
            attempts: vec![attempt(diff())],
            patch: Some(patch),
        })["patch"]
            .clone()
    };
    assert_eq!(with(Preview::Text("@@".to_owned()))["kind"], "text");
    let truncated = with(Preview::Truncated {
        text: "@@".to_owned(),
        size: 99,
    });
    assert_eq!(truncated["kind"], "truncated");
    assert_eq!(truncated["size"], 99);
    assert_eq!(with(Preview::Binary { size: 7 })["kind"], "binary");
}

/// The query is answered by the same read the tab builds — a workspace with no
/// claim answers with no rows, and a named file with no attempt behind it adds
/// no patch. One derivation, two serializations.
#[test]
fn the_query_answers_off_the_tabs_own_derivation() {
    let ws = PathBuf::from("/names/alba");
    let snap = snapshot(&ws, "alba", vec![], vec![]);
    let ui = UiState::open(PathBuf::from("/nonexistent/ui.json"));
    let d = deps(snap);
    let Ok(Reply::WorkDiff { attempts, patch }) = answer(
        &Query::WorkDiff {
            workspace: crate::naming::leaf(&(ws)),
            file: Some(WorkFile {
                ball: "bl-1".to_owned(),
                handle: None,
                path: "src/a.rs".to_owned(),
            }),
        },
        &d,
        &ui,
        200,
    ) else {
        panic!("the work-diff query answers a work-diff");
    };
    assert!(attempts.is_empty(), "this workspace claims nothing");
    assert!(patch.is_none(), "no attempt, no patch");
}
