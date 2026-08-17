//! The projection's spelling (§8.5). The **round trip** over a populated row of
//! every arm lives with the rest of the surface
//! (`boundary::reply::tests::roundtrip`); what belongs here is the half a
//! fixture cannot reach — the refusals a malformed body earns, which must name
//! their token rather than guessing a row.

use serde_json::{Value, json};

use crate::science::wire::{reply, rows_of};
use crate::science::{Attempt, Outcome, Verdict};

/// The narrowest row that encodes: no conversation, no goal, no freeze.
fn bare() -> Attempt {
    Attempt {
        diff: crate::workdiff::Attempt {
            project: "p".to_owned(),
            ball_id: "bl-1".to_owned(),
            handle: None,
            delivered: None,
            change: crate::workdiff::Change::Unreadable,
        },
        base: None,
        conversation: None,
        goal: None,
        pins: Vec::new(),
        governing: None,
        usage: crate::budgets::BudgetSpend::default(),
        wall_secs: 0,
        steps: 0,
        response: None,
        verdicts: Vec::new(),
        outcome: Outcome::Pending,
    }
}

/// The five absent columns are **absent**, not null: a reader must not have to
/// tell "no conversation" from "a conversation called nothing".
#[test]
fn an_unbound_row_omits_what_it_cannot_say() {
    let body = reply(&[bare()]);
    let row = &body["rows"][0];
    for key in ["base", "conversation", "goal", "governing", "response"] {
        assert!(row.get(key).is_none(), "{key} rides only when there is one");
    }
    assert_eq!(row["outcome"], json!({ "state": "pending" }));
    assert_eq!(body["kind"], "science");
}

/// A rejection with nobody named omits `by` for the same reason — and the
/// round trip still lands on the same value.
#[test]
fn a_rejection_by_nobody_round_trips() {
    let mut row = bare();
    row.outcome = Outcome::Rejected { by: None };
    row.verdicts = vec![Verdict {
        sender: "judge".to_owned(),
        body: "no".to_owned(),
    }];
    let body = reply(&[row.clone()]);
    assert!(body["rows"][0]["outcome"].get("by").is_none());
    assert_eq!(read(&body).unwrap(), vec![row]);
}

/// Every refusal names the token it choked on. Each of these is a body a peer
/// could send and a guessed row would be worse than none.
#[test]
fn a_malformed_body_refuses_by_name() {
    let cases = [
        (json!({ "rows": [1] }), "not an object"),
        (json!({ "rows": [{}] }), "missing diff"),
        (
            json!({ "rows": [{ "diff": diff(), "pins": [], "usage": 1 }] }),
            "not an object",
        ),
        (
            json!({ "rows": [{ "diff": diff(), "pins": [] }] }),
            "missing usage",
        ),
        (
            json!({ "rows": [{ "diff": diff(), "pins": [], "usage": usage(),
                               "wall_secs": 0, "steps": 0, "verdicts": [{}] }] }),
            "sender",
        ),
        (
            json!({ "rows": [{ "diff": diff(), "pins": [], "usage": usage(),
                               "wall_secs": 0, "steps": 0, "verdicts": [] }] }),
            "missing outcome",
        ),
        (
            json!({ "rows": [row_with(json!({ "state": "adopted" }))] }),
            "unknown state \"adopted\"",
        ),
        (
            json!({ "rows": [row_with(json!(7))] }),
            "outcome: not an object",
        ),
        (
            json!({ "rows": [row_with(json!({ "state": "accepted" }))] }),
            "commit",
        ),
        (
            json!({ "rows": [{ "diff": diff(), "pins": [], "usage": usage(),
                               "wall_secs": 0, "steps": 0, "verdicts": [1] }] }),
            "verdict: not an object",
        ),
    ];
    for (body, needle) in cases {
        let err = read(&body).expect_err("a malformed row refuses");
        assert!(err.contains(needle), "{err:?} should name {needle:?}");
    }
}

/// The accepted and reworked arms, and a populated diff column, complete the
/// round trip over every outcome the enum has.
#[test]
fn every_outcome_arm_round_trips() {
    for outcome in [
        Outcome::Accepted {
            commit: "abc".to_owned(),
        },
        Outcome::Rejected {
            by: Some("at-1".to_owned()),
        },
        Outcome::Reworked,
        Outcome::Pending,
    ] {
        let mut row = bare();
        row.outcome = outcome;
        row.base = Some("bbb".to_owned());
        row.conversation = Some("a-1".to_owned());
        row.goal = Some("do it".to_owned());
        row.governing = Some("cfg".to_owned());
        row.response = Some("done".to_owned());
        row.pins = vec!["instructions/00-A.md=/p/A.md".to_owned()];
        row.wall_secs = 12;
        row.steps = 3;
        assert_eq!(read(&reply(&[row.clone()])).unwrap(), vec![row]);
    }
}

fn read(body: &Value) -> Result<Vec<Attempt>, String> {
    rows_of(body.as_object().unwrap())
}

fn diff() -> Value {
    crate::workdiff::wire::attempt_row(&bare().diff)
}

fn usage() -> Value {
    json!({ "input_tokens": 0, "output_tokens": 0,
            "cache_read_tokens": 0, "cache_write_tokens": 0 })
}

fn row_with(outcome: Value) -> Value {
    json!({ "diff": diff(), "pins": [], "usage": usage(),
            "wall_secs": 0, "steps": 0, "verdicts": [], "outcome": outcome })
}
