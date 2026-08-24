//! The reply envelope (§8.5): the outcome, prepared, refusal and help
//! spellings, and the [`cleared`] predicate. The row encoders' own tables
//! live next door in [`rows`].

mod board;
mod config;
mod inspector;
mod queue;
mod receipts;
mod roundtrip;
mod rows;
mod science;
mod search;
mod workdiff;

use super::*;
use crate::opslog::Origin;
use std::path::PathBuf;

fn outcome(exit: i32) -> Outcome {
    Outcome {
        exit,
        stdout: "out".into(),
        stderr: "err".into(),
    }
}

#[test]
fn an_outcome_reply_carries_the_captured_run_and_its_verdict() {
    let ok = encode(&Reply::Outcome(outcome(0)));
    assert_eq!(ok["ok"], true);
    assert_eq!(ok["kind"], "outcome");
    assert_eq!(ok["exit"], 0);
    assert_eq!(ok["stdout"], "out");
    assert_eq!(ok["stderr"], "err");
    let failed = encode(&Reply::Outcome(outcome(3)));
    assert_eq!(failed["ok"], false);
}

#[test]
fn the_prepared_reply_is_the_prompt_gestures_own_spelling() {
    let prepared = Prepared {
        workspace: crate::naming::leaf(&(PathBuf::from("/ws"))),
        binding: Some(PathBuf::from("/target")),
        goal: "g".into(),
        origin: Origin::Balls,
        lineage: None,
    };
    let v = encode(&Reply::Prepared(prepared.clone()));
    assert_eq!(v["ok"], true);
    assert_eq!(v["kind"], "prepared");
    // The round-trip promise: the reply body re-enters as the next gesture.
    let back = serde_json::json!({ "op": "prompt", "prepared": v["prepared"], "goal": "g2" });
    assert_eq!(
        super::super::codec::decode(&back),
        Ok(super::super::Gesture::Act(super::super::Action::Prompt {
            prepared,
            goal: "g2".into(),
            seed: None,
        }))
    );
}

#[test]
fn cleared_is_the_draft_clearing_predicate() {
    assert!(cleared(&Ok(Reply::Outcome(outcome(0)))));
    assert!(!cleared(&Ok(Reply::Outcome(outcome(2)))));
    assert!(cleared(&Ok(Reply::Started {
        conversation: "x".into()
    })));
    assert!(!cleared(&Err("refused".into())));
}

#[test]
fn a_refusal_names_its_reason() {
    let v = refusal("unknown op");
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"], "unknown op");
}

/// A help reply is rows like any other query's: the four facts a page is made
/// of, so a headless reader renders what the window renders.
#[test]
fn a_help_reply_carries_each_page_as_data() {
    let rows = crate::boundary::help::rows(Some("scan"));
    let encoded = encode(&Reply::Help(rows));
    assert_eq!(encoded["ok"], true);
    assert_eq!(encoded["kind"], "help");
    let row = &encoded["rows"][0];
    assert_eq!(row["verb"], "scan");
    assert_eq!(row["usage"], "/scan");
    assert!(row["summary"].as_str().is_some_and(|s| !s.is_empty()));
    assert!(row["detail"].as_str().is_some_and(|s| s.len() > 40));
}
